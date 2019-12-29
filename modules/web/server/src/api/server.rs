use super::{boards, oauth, users};
use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::{middleware::Logger, App, HttpServer};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use rand::Rng;
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
        let callback_url = format!("http://localhost:{}/api/oauth/cb", port); // Get the oauth callback url

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
    pub async fn start(&mut self) -> io::Result<()> {
        // Log the pending connection
        info!(
            "Connecting to postgres database (postgres://{}:****@{})",
            self.database_endpoint.split("://").collect::<Vec<&str>>()[1]
                .split(':')
                .collect::<Vec<&str>>()[0],
            self.database_endpoint.split('@').collect::<Vec<&str>>()[1]
        );

        let manager = ConnectionManager::<PgConnection>::new(&self.database_endpoint); // Make a new connection manager from the config's db endpoint

        // Make a connection pool from the r2d2 manager
        match Pool::builder().build(manager) {
            // Setup the HTTP server
            Ok(pool) =>
            // Start the HTTP server
            {
                let cfg = self.oauth_config.clone(); // Clone the server's oauth configuration, so we can move it into the server logic closure
                HttpServer::new(move || {
                    let encryption_key: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>(); // Generate an encryption key

                    // Register all of the API's routes, and attach the db connection handler
                    App::new()
                        .wrap(Logger::default())
                        .wrap(CookieSession::private(&encryption_key).secure(false)) // Use secure session storage to store state vars, pkce challenges
                        .wrap(Cors::new().allowed_origin("*").finish()) // TODO: Better CORS policy?
                        .data(pool.clone()) // Allow usage of the db connector from API routes
                        .data(cfg.clone()) // Allow access to the oauth configuration from request handlers
                        .service(oauth::authenticate) // Register the authentication oauth service
                        .service(oauth::callback) // Register the oauth callback service
                        .service(users::all_user_ids) // Register the all users service
                        .service(users::user_with_id) // Register the GET service for a particular user
                        .service(users::user) // Register the GET service for a user matching a bearer token
                        .service(users::boards_from_user_with_id) // Register the GET service for a user's boards (IDs only)
                        .service(users::notes_from_user_with_id) // The same GET, but for notes
                        .service(users::permissions_for_user_with_id) // Register the GET service all of a user's assignments
                        .service(users::permission_for_user_with_board) // Register the GET service for a specific assignmen
                        .service(boards::viewable_boards) // Register the GET service for all viewable boards (user has perms to see)
                        .service(boards::specific_board) // Register the GET service for a specific board (only viewable given certain permissions)
                        .service(boards::new_board) // Register the POST service for a new board
                        .service(boards::update_specific_board) // Register the PATCH service for a specific board
                        .service(boards::delete_specific_board) // Reigster the DELETE service for a specific board
                        .service(boards::all_permissions) // Register the GET service for a specific board's permissions
                })
                .bind(format!("0.0.0.0:{}", self.port))?
                .start()
                .await
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
