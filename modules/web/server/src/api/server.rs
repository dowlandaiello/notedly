use actix_web::{App, HttpServer};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use std::io;

/// A configuration for the server's oauth capabilities.
pub struct OauthConfig {
    /// The client ID and client secret for the GitHub API
    github_api_credentials: (String, String),

    /// The client ID and client secret for the Google API
    google_api_credentials: (String, String),
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
        // Return the initialized OauthConfig, as well as the remaining env vars
        (
            OauthConfig {
                github_api_credentials: (oauth_credentials.remove(0), oauth_credentials.remove(0)),
                google_api_credentials: (oauth_credentials.remove(0), oauth_credentials.remove(0)),
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
    pub fn new(oauth_config: OauthConfig, database_endpoint: String, port: u16) -> Self {
        Self {
            oauth_config,
            database_endpoint,
            port,
        } // Return the initialized server
    }

    /// Starts the API web server.
    pub fn start(&self) -> io::Result<()> {
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
            // Star the HTTP server
            {
                match HttpServer::new(move || {
                    // Register all of the API's routes, and attach the db connection handler
                    App::new().data(pool.clone())
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
