use actix_web::{App, HttpServer};
use oauth2::basic::BasicClient;

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
        (OauthConfig {
            github_api_credentials: (oauth_credentials.remove(0), oauth_credentials.remove(0)),
            google_api_credentials: (oauth_credentials.remove(0), oauth_credentials.remove(0)),
        }, oauth_credentials)
    }
}

/// A configuration for the server's database connection.
pub struct DatabaseConfig {
    /// The remote database connection endpoint
    endpoint: String,

    /// The username of the administrator database account that will be used for user data
    /// storage and retrieval
    admin_username: String,

    /// The password of the database administrator account
    admin_password: String,
}

impl DatabaseConfig {
    /// Initializes and returns a new PostgresConfig with the given endpoint, admin username, and
    /// admin password.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The database endpoint to which the server will connect
    /// * `admin_username` - The username of the administrator db account
    /// * `admin_password` - The password of the administrator db account
    pub fn new(endpoint: String, admin_username: String, admin_password: String) -> Self {
        // Restructure the inputted data as a database configuration
        Self {
            endpoint,
            admin_username,
            admin_password,
        }
    }
}

/// An HTTP web server conforming to the REST standard.
pub struct Server {
    /// The configuration for the server's oauth integrations
    oauth_config: OauthConfig,

    /// The configuration for the server' database connection
    database_config: DatabaseConfig,

    /// The port the API should be served on
    port: u16,
}

impl Server {
    /// Initializes a new Server with the given oauth configuration.
    ///
    /// # Arguments
    ///
    /// * `oauth_config` - The active Oauth API access configuration
    /// * `database_config` - The active database configuration
    /// * `port` - The port that the API will be served on
    pub fn new(oauth_config: OauthConfig, database_config: DatabaseConfig, port: u16) -> Self {
        Self {
            oauth_config,
            database_config,
            port,
        } // Return the initialized server
    }

    /// Starts the API web server.
    pub fn start(&self) {
        HttpServer::new(|| {
            App::new()
        }); // Setup the HTTP server
    }
}
