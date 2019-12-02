#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate dotenv;

use log::LevelFilter::{Debug, Info};
use server::api::server::{DatabaseConfig, OauthConfig, Server};
use std::env;
use tokio::runtime::current_thread;
use dotenv::dotenv;

/// The notedly command-line interface.
#[derive(Clap)]
#[clap(version = "1.0", author = "Dowland A.")]
struct Opts {
    /// Print debug info
    #[clap(short = "d", long = "debug")]
    debug: bool,

    /// Prevent any non-critical information from being printed to the console
    #[clap(short = "s", long = "silent")]
    silent: bool,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// A subcommand of the notedly CLI.
#[derive(Clap)]
enum SubCommand {
    /// Starts the notedly API web server
    #[clap(name = "serve", version = "1.0", author = "Dowland A.")]
    Serve(Serve),
}

/// Starts the notedly API web server. Please note that `serve` assumes the following variables
/// have been set, and can be found in your OS env: GITHUB_OAUTH_CLIENT_ID,
/// GITHUB_OAUTH_CLIENT_SECRET, GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_CLIENT_SECRET.
#[derive(Clap)]
#[clap(name = "serve", version = "1.0", author = "Dowland A.")]
struct Serve {
    /// The port the API will be served on
    #[clap(short = "p", default_value = "8080")]
    port: u16,

    /// The remote db connection endpoint
    #[clap(short = "e", default_value = "couchbase://0.0.0.0")]
    database_endpoint: String,
}

/// The entry point for the notedly CLI.
fn main() {
    // Load any configuration vars from dotfiles
    dotenv().ok();

    let opts: Opts = Opts::parse(); // Parse any arguments issued by the user

    // Configure the logger
    if !opts.silent {
        if opts.debug {
            env_logger::builder().filter_level(Debug).init(); // Include debug statements in logger output
        } else {
            env_logger::builder().filter_level(Info).init(); // Include info statements
        }
    }

    // Check if the user is trying to start the web server or just use the CLI
    match opts.subcmd {
        // Start serving
        SubCommand::Serve(cfg) => serve(cfg),
    }
}

/// Starts serving the notedly API.
///
/// # Arguments
///
/// * `serve` - A config for the serve command
fn serve(serve: Serve) {
    // The names of the environment variables where we expect that the oauth config & couchbase
    // credentials have been stored
    let required_vars = [
        "GITHUB_OAUTH_CLIENT_ID",
        "GITHUB_OAUTH_CLIENT_SECRET",
        "GOOGLE_OAUTH_CLIENT_ID",
        "GOOGLE_OAUTH_CLIENT_SECRET",
    ];
    let mut var_values: Vec<String> = Vec::new();

    // Iterate through each of the desired ENV variables
    for required_var in required_vars.iter() {
        match env::var(required_var) {
            // If the var exists, add it to the var values vec
            Ok(var) => var_values.push(var),

            // If the var doesn't exist, panic! We need each of the vars to be set in order to
            // work.
            Err(_) => error!("Expected env var {} to be set.", required_var),
        };
    }

    // If the user hasn't provided the required variables, return
    if var_values.len() == required_vars.len() {
        // Make a new oauth config from the collected env variables
        let (oauth_config, mut var_values) = OauthConfig::new(var_values);

        // Make a newe database config from the collected env variables, as well as the inputted
        // command arguments
        let db_config = DatabaseConfig::new(serve.database_endpoint, var_values.remove(0), var_values.remove(0));

        // Make a new server from the generated oauth config
        let s = Server::new(oauth_config, db_config, serve.port);

        // Start the server
        s.start();
    }
}
