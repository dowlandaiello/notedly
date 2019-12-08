use super::server::OauthConfig;
use actix_session::Session;
use actix_web::{
    client::Client,
    http,
    web::{Data, Path, Query},
    Error, HttpResponse,
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use oauth2::{
    reqwest::http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::default::Default;

/// Generates a pkce challenge, and forwards the user to the respective authentication portal.
#[get("/api/oauth/login/{provider}")]
pub async fn authenticate(
    info: Path<String>,
    data: Data<OauthConfig>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let provider = &*info; // Get the provider from the path

    // If the provider is invalid, respond with a bad request code
    if provider != "google" && provider != "github" {
        Ok(HttpResponse::BadRequest().finish()) // Respond with a 400
    } else {
        let (client, scopes) = match provider.as_ref() {
            // If the request asked for a google auth, use the Google API client
            "google" => (&data.google_api_client, "profile"),
            // If the request asked for a GitHub auth, use the GitHub API client
            _ => (&data.github_api_client, "user:email"),
        }; // Get an appropriate client based on the given OauthConfig

        // Generate a key exchange challenge that the client must solve
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Get an auth URL
        let (auth_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(scopes.to_owned()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        // Put all of the required verification state vars into the redis session
        // storage db
        session.set::<String>("state", csrf_state.secret().clone())?;
        session.set::<PkceCodeVerifier>("verifier", pkce_verifier)?;
        session.set::<String>("provider", provider.to_owned())?;

        // Redirect the user to the auth url
        Ok(HttpResponse::TemporaryRedirect()
            .header(http::header::LOCATION, auth_url.as_str())
            .finish())
    }
}

/// A request to the /cb route
#[derive(Serialize, Deserialize)]
pub struct CallbackRequest {
    /// The code provided by the caller of the /cb route
    pub code: String,

    /// The state provided by the caller of the /cb route
    pub state: String,
}

/// A response from the GitHub API for the user's emails.
#[derive(Serialize, Deserialize, Debug)]
struct GitHubEmailResponse {
    pub emails: Vec<GitHubEmail>,
}

impl GitHubEmailResponse {
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

/// Authenticates the user with a given authorization code.
#[get("/api/oauth/cb")]
pub async fn callback(
    info: Query<CallbackRequest>,
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    data: Data<OauthConfig>,
    session: Session,
) -> Result<HttpResponse, Error> {
    // Abort the request if the state has been corrupted
    if info.state != session.get::<String>("state")?.unwrap_or("".to_owned()) {
        Ok(HttpResponse::Conflict().finish()) // Respond with a 409
    } else {
        let client = match session
            .get::<String>("provider")?
            .unwrap_or("".to_owned())
            .as_ref()
        {
            "google" => &data.google_api_client,
            _ => &data.github_api_client,
        }; // Get an appropriate client based on the session provider

        // Get the pkce code verifier from session storage
        if let Some(verifier) = session.get::<PkceCodeVerifier>("verifier")? {
            // Exchange the authorization code for an access token
            match client
                .exchange_code(AuthorizationCode::new(info.code.clone()))
                .set_pkce_verifier(verifier)
                .request(http_client)
            {
                Ok(response) => {
                    // Get an access token from the response
                    let access_token = response.access_token();

                    // Get a HTTP client so we can obtain the email of the user
                    let client = Client::default();

                    // Get a request URL based on the given provider cookie
                    let request_url =
                        if &session.get::<String>("provider")?.unwrap_or("".to_owned()) == "google"
                        {
                            "https://openidconnect.googleapis.com/v1/userinfo"
                        } else {
                            "https://api.github.com/user/emails"
                        };

                    // Get the user's email from the access token
                    let email = client
                        .get(request_url) // Make a request to the respective user API
                        .set_header(
                            "Authorization",
                            format!("Bearer {}", access_token.secret().as_str()),
                        ) // Use our new access token
                        .set_header("User-Agent", "Notedly") // And make sure the request will be made on behalf of the user from actix
                        .send() // Send the request
                        .await?
                        .json::<GitHubEmailResponse>()
                        .await?.best_email().email;

                    println!("{}", email);

                    Ok(HttpResponse::Conflict().finish())
                }

                // Handle any errors by returning a 500
                Err(_) => Ok(HttpResponse::InternalServerError().finish()),
            }
        } else {
            // Return a 500 error, since we can't continue with out a pkce verifier
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
