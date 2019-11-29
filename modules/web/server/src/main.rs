#[macro_use]
extern crate clap;

use std::env;
use tokio::runtime::current_thread;

/// The notedly command-line interface.
#[derive(Clap)]
#[clap(version = crate_version!(), author = "Dowland A.")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// A subcommand of the notedly CLI.
#[derive(Clap)]
enum SubCommand {
    /// Starts the notedly API web server.
    Serve(Serve),
}

/// Starts the notedly API web server. Please note that `serve` assumes the following variables
/// have been set, and can be found in your OS env: GITHUB_OAUTH_CLIENT_ID,
/// GITHUB_OAUTH_CLIENT_SECRET, GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_CLIENT_SECRET.
#[derive(Clap)]
#[clap(name = "serve", version = crate_version!(), author = "Dowland A.")]
struct Serve {
    /// Print debug info
    #[clap(short = "d")]
    debug: bool,
    /// The port the API will be served on
    #[clap(short = "p")]
    port: u16,
}

/// The entry point for the notedly CLI.
fn main() {
    let opts: Opts = Opts::parse(); // Parse any arguments issued by the user

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
    // The names of the environment variables where we expect that the oauth config has been stored
    let required_vars = [
        "GITHUB_OAUTH_CLIENT_ID",
        "GITHUB_OAUTH_CLIENT_SECRET",
        "GOOGLE_OAUTH_CLIENT_ID",
        "GOOGLE_OAUTH_CLIENT_SECRET",
    ];
    let mut var_values: Vec<String>;

    // Iterate through each of the desired ENV variables
    for required_var in required_vars.iter() {
        match env::var(required_var) {
            // If the var exists, add it to the var values vec
            Ok(var) => var_values.push(var),
            // If the var doesn't exist, panic! We need each of the vars to be set in order to
            // work.
            Err(e) => panic!("Expected env var {} to be set.", required_var),
        }
    }
}
