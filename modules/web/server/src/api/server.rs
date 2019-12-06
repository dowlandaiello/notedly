use actix_session::CookieSession;
use actix_web::{middleware::Logger, App, HttpServer};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use rand::Rng;
use super::oauth;
use std::io;

/// A configuration for the server's oauth capabilities.
#[derive(Clone)]
pub struct OauthConfig {
    /// A basic oauth2 client for the GitHub authentication API
    pub github_api_client: BasicClient,

    /// A basic oauth2 client for the Google authentication API
    pub google_api_client: BasicClient,
}

impl OauthConfig {
    /// Initializes a new OauthConfig from the provided vector of environment variables.
    /// Note: This method consumes the original vector of environment variables.
    ///
    /// # Arguments
    ///
    /// * `oauth_credentials` - A vector of environment variables representing the following oauth
    /// API credentials: github client ID, github client secret, google client ID, google client
    /// secret
    pub fn new(mut oauth_credentials: Vec<String>) -> (Self, Vec<String>) {
        let mut clients: Vec<BasicClient> = Vec::new(); // A tuple to store the constructed clients in

        // Construct an oauth client for each of the given credentials
        for i in 0..2 {
            let auth_url = AuthUrl::new(if i == 0 {
                // Use the GitHub oauth authorization URL, instead of the google URL
                "https://github.com/login/oauth/authorize".to_owned()
            } else {
                // use the Google oauth authorization URL, instead of the github URL
                "https://accounts.google.com/o/oauth2/v2/auth".to_owned()
            }); // Get the authorization URL for both providers

            let token_url = TokenUrl::new(if i == 0 {
                // Use the GitHub oauth token URL, instead of the google URL
                "https://github.com/login/oauth/access_token".to_owned()
            } else {
                // use the Google oauth token URL, instead of the github URL
                "https://accounts.google.com/o/oauth2/v3/token".to_owned()
            }); // Get the authorization URL for both providers

            let client_id = ClientId::new(oauth_credentials.remove(0)); // Make a client ID from the provided credential
            let client_secret = ClientSecret::new(oauth_credentials.remove(0)); // Make a client secret from the provided credential

            // Make the oauth client, and store it in the clients tuple
            clients.push(BasicClient::new(
                client_id,
                Some(client_secret),
                auth_url.unwrap(),
                token_url.ok(),
            ));
        }

        // Return the initialized OauthConfig, as well as the remaining env vars
        (
            OauthConfig {
                github_api_client: clients.remove(0),
                google_api_client: clients.remove(0),
            },
            oauth_credentials,
        )
    }
}

/// An HTTP web server conforming to the REST standard.
pub struct Server {
    /// The configuration for the server's oauth integrations
    oauth_config: OauthConfig,

    /// The database URL to which the server will connect
    database_endpoint: String,

    /// The port the API should be served on
    port: u16,
}

impl Server {
    /// Initializes a new Server with the given oauth configuration.
    ///
    /// # Arguments
    ///
    /// * `oauth_config` - The active Oauth API access configuration
    /// * `database_endpoint` - The active database connection URI
    /// * `port` - The port that the API will be served on
    pub fn new(mut oauth_config: OauthConfig, database_endpoint: String, port: u16) -> Self {
        let callback_url = format!("http://localhost:{}/oauth/cb", port); // Get the oauth callback url

        // Set the redirect URL for both clients
        oauth_config.google_api_client = oauth_config
            .google_api_client
            .set_redirect_url(RedirectUrl::new(callback_url.clone()).unwrap());
        oauth_config.github_api_client = oauth_config
            .github_api_client
            .set_redirect_url(RedirectUrl::new(callback_url).unwrap());

        Self {
            oauth_config,
            database_endpoint,
            port,
        } // Return the initialized server
    }

    /// Starts the API web server.
    pub fn start(&mut self) -> io::Result<()> {
        // Log the pending connection
        info!(
            "Connecting to postgres database (postgres://{}:****@{})",
            self.database_endpoint.split("://").collect::<Vec<&str>>()[1]
                .split(":")
                .collect::<Vec<&str>>()[0],
            self.database_endpoint.split("@").collect::<Vec<&str>>()[1]
        );

        let manager = ConnectionManager::<PgConnection>::new(&self.database_endpoint); // Make a new connection manager from the config's db endpoint

        // Make a connection pool from the r2d2 manager
        match Pool::builder().build(manager) {
            // Setup the HTTP server
            Ok(pool) =>
            // Start the HTTP server
            {
                let cfg = self.oauth_config.clone(); // Clone the server's oauth configuration, so we can move it into the server logic closure
                match HttpServer::new(move || {
                    let encryption_key: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>(); // Generate an encryption key

                    // Register all of the API's routes, and attach the db connection handler
                    App::new()
                        .wrap(Logger::default())
                        .wrap(CookieSession::signed(&encryption_key).secure(true)) // Use secure session storage to store state vars, pkce challenges
                        .data(pool.clone()) // Allow usage of the db connector from API routes
                        .data(cfg.clone()) // Allow access to the oauth configuration from request handlers
                        .data(encryption_key) // Allow access to the encryption key from request handlers
                        .service(oauth::authenticate) // Register the authentication oauth service
                })
                .bind(format!("localhost:{}", self.port))
                {
                    // Got a listener on the given port, start it
                    Ok(ln) => ln.run(),

                    // Log an error
                    Err(e) => {
                        error!("Failed to start the API server: {}", e);

                        Ok(())
                    }
                }
            }

            // Log an error
            Err(e) => {
                error!(
                    "Failed to construct a connection pool for the database: {}",
                    e
                ); // Log the error

                Ok(()) // Stop the main thread
            }
        }
    }
}
