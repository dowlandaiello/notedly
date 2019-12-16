use super::super::schema::users::dsl::*;
use actix_web::{client::Client, Error, error};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
    QueryDsl,
};
use serde::{Deserialize, Serialize};
use std::{default::Default, io};

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
        let mut best: &GitHubEmail = &self.emails[0];

        // Iterate through the emails
        for i in 0..self.emails.len() {
            let gh_email: &GitHubEmail = &self.emails[i]; // Get a reference to the email

            // Check if the email is at least verified
            if gh_email.verified {
                best = gh_email; // Set the new best email

                if gh_email.primary {
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

/// A GitHub user ID.
#[derive(Serialize, Deserialize)]
struct GitHubIDResponse {
    /// The ID of the user
    id: i32,
}

impl User {
    /// Initializes a new user from the given access token and provider.
    pub fn new(access_token: String, provider: String) -> Self {
        Self {
            email: "".to_owned(),
            access_token,
            provider,
            client: Client::default(),
        } // Return the new instance
    }

    /// Gets the URL of the account's oauth provider.
    ///
    /// # Arguments
    ///
    /// * `postfix` - A string appended to the end of the provider url
    ///
    /// # Example
    ///
    /// ```
    /// use server::api::wrapper::User;
    //
    /// // A new GitHub user
    /// let u = User::new("SOME_ACCESS_TOKEN", "github");
    ///
    /// println!("{}", u.provider_url("emails")); // => https://api.github.com/user/emails
    /// ```
    pub fn provider_url(&self, postfix: &str) -> String {
        // Return the respective authentication URL
        if self.provider == "google" {
            // Idk some google stuff
            format!(
                "https://openidconnect.googleapis.com/v1/userinfo{}",
                if postfix != "" {
                    format!("/{}", postfix)
                } else {
                    "".to_owned()
                }
            )
        } else {
            // A URL for all of the information known about the user
            format!(
                "https://api.github.com/user{}",
                if postfix != "" {
                    format!("/{}", postfix)
                } else {
                    "".to_owned()
                }
            )
        }
    }

    /// Gets the oauth ID of the user from the known provider.
    pub async fn oauth_id(&self) -> Result<i32, Error> {
        // Send a request asking for the oauth ID of the user with the matching oauth token, and
        // await the response from the service
        let mut response = self
            .client
            .get(self.provider_url("")) // Start the request
            .set_header("Authorization", format!("Bearer {}", self.access_token)) // Tell GitHub which user we would like to get the ID of
            .set_header("User-Agent", "Notedly") // Make sure GitHub sees this as a valid request
            .send() // Send the request
            .await?; // Await the response

        if self.provider == "github" {
            // Convert the general response to a GitHub response
            let github_resp: GitHubIDResponse = response.json::<GitHubIDResponse>().await?;

            // Return the identifier of the user as an owned string
            Ok(github_resp.id)
        } else {
            Err(error::ErrorBadRequest(io::Error::new(
                io::ErrorKind::Other,
                "the provider does not exit",
            ))) // User did something bad
        }
    }

    /// Gets the email of the user from the known provider.
    pub async fn email(&mut self) -> Result<&str, Error> {
        // Send a request asking for the email of the user with the matching oauth token, and await
        // the response from the service
        let mut response = self
            .client
            .get(self.provider_url("emails")) // Start the request
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
