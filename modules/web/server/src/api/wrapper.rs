use super::super::schema::users::dls::*;
use actix_web::{client::Client, Error};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::default::Default;

/// A wrapper for each of the respective oauth provider APIs (Google, GitHub).
pub struct User {
    /// The access token associated with the user
    access_token: String,

    /// Cached results for the /email GET
    email: String,

    /// The provider of the authorization service
    provider: String,

    /// The HTTP client used to make requests
    client: Client,
}

/// A response from the GitHub API for the user's emails.
#[derive(Serialize, Deserialize, Debug)]
struct GitHubEmailResponse {
    pub emails: Vec<GitHubEmail>,
}

impl GitHubEmailResponse {
    /// Initializes a new GitHubEmailResponse with the given GitHub emails.
    ///
    /// # Arguments
    ///
    /// * `emails` - The emails contained in the response
    pub fn new(emails: Vec<GitHubEmail>) -> Self {
        Self { emails } // Return the new instance
    }

    /// Gets the most suitable email of the available GitHub emails.
    pub fn best_email(&self) -> &GitHubEmail {
        // We'll use the first email until we find a better one
        let mut best = &self.emails[0];

        // Iterate through the emails
        for i in 0..self.emails.len() {
            let email = &self.emails[i]; // Get a reference to the email

            // Check if the email is at least verified
            if email.verified {
                best = email; // Set the new best email

                if email.primary {
                    break; // Stop, use the best possible email
                }
            }
        }

        best // Return the best email
    }
}

/// A GitHub preference regarding a user's email.
#[derive(Serialize, Deserialize, Debug)]
struct GitHubEmail {
    /// The email from the response
    pub email: String,

    /// Whether or not the email has been verified
    pub verified: bool,

    /// Whether or not the email is a primary email
    pub primary: bool,

    /// The visibility of the email
    #[serde(skip)]
    visibility: String,
}

impl User {
    /// Initializes a new user from the given access token and provider.
    pub fn new(access_token: String, provider: String) -> Self {
        Self {
            access_token,
            provider,
            client: Client::default(),
        } // Return the new instance
    }

    /// Checks if a user with the email exists in the database.
    ///
    /// # Arguments
    ///
    /// * `conn` - The connection to the database that will be used to check for the user's
    /// existence
    pub fn exists(&self, conn: PooledConnection<ConnectionManager<PgConnection>>) -> bool {
        // Run a query that will check if the email is contained in the database
        select(exists(users.find(self.email)))
            .get_result(&conn)
            .unwrap_or(false)
    }

    /// Gets the ID for the user if the user already exists.
    ///
    /// # Arguments
    ///
    /// * `conn` - The connection to the database that will be used to find the user's ID
    pub fn id(&self, conn: PooledConnection<ConnectionManager<PgConnection>>) -> Option<i32> {
        // If the user doesn't already exist, return a None
        if !self.exists(conn) {
            None
        } else {
            diesel::select(users.find(self.email)).get_result(&conn) // Return the user's ID
        }
    }

    /// Gets the email of the user from the known provider.
    pub async fn email(&self) -> Result<&str, Error> {
        let request_url = if self.provider == "google" {
            "https://openidconnect.googleapis.com/v1/userinfo"
        } else {
            "https://api.github.com/user/emails"
        }; // Get the URL of the prospective request from the known oauth provider

        // Send a request asking for the email of the user with the matching oauth token, and await
        // the response from the service
        let mut response = self
            .client
            .get(request_url) // Start the request
            .set_header("Authorization", format!("Bearer {}", self.access_token)) // Tell GitHub which user we would like to get the email of by sending our token
            .set_header("User-Agent", "Notedly") // Make sure GitHub sees this as a valid request
            .send() // Send the request
            .await?; // Yay async

        Ok(if self.provider == "github" {
            // Get the email from the GitHub response
            self.email = GitHubEmailResponse::new(response.json::<Vec<GitHubEmail>>().await?)
                .best_email()
                .email
                .clone();

            &self.email // Return the user's email
        } else {
            ""
            // TODO: Add Google openID connect support
        })
    }
}
