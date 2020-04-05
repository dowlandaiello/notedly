use super::{
    super::{models, schema::users::dsl::*},
    server::OauthConfig,
    wrapper,
};
use actix_session::Session;
use actix_web::{
    error, http,
    web::{Data, Json, Path, Query},
    Error, HttpResponse,
    Scope as ActixScope
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    RunQueryDsl,
};
use oauth2::{
    reqwest::http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Constructs an actix service group for the oauth endpoint.     
pub fn build_service_group() -> ActixScope { 
    ActixScope::new("/oauth/").service(authenticate).service(callback)
}

/// Generates a pkce challenge, and forwards the user to the respective authentication portal.
#[get("/login/{provider}")]
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

/// Authenticates the user with a given authorization code.
#[get("/cb")]
pub async fn callback(
    info: Query<CallbackRequest>,
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    data: Data<OauthConfig>,
    session: Session,
) -> Result<Json<models::OwnedUser>, Error> {
    // Abort the request if the state has been corrupted
    if info.state
        != session
            .get::<String>("state")?
            .unwrap_or_else(|| "".to_owned())
    {
        // Respond with a 409
        Err(error::ErrorConflict(
            "The state challenge was not completed successfully.",
        ))
    } else {
        let client = match session
            .get::<String>("provider")?
            .unwrap_or_else(|| "".to_owned())
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
                    // Get a connection from the pool
                    match pool.get() {
                        Ok(conn) => {
                            // Get an access token from the response
                            let access_token = response.access_token();

                            // Hash the user's access token
                            let mut token_hasher = Sha3_256::new();
                            token_hasher.input(access_token.secret());

                            let mut user = wrapper::User::new(
                                access_token.secret().to_owned(),
                                session
                                    .get::<String>("provider")?
                                    .unwrap_or_else(|| "".to_owned()),
                            ); // Generate a new wrapper for the user API from the acess token and provider

                            // Get the user's oauth ID
                            let id_oauth = user.oauth_id().await?;

                            // Generate a user with an empty UID (postgres will figure this out)
                            let schema_user = models::NewUser {
                                oauth_id: id_oauth,
                                oauth_token: &hex::encode(token_hasher.result()),
                                email: user.email().await?,
                            };

                            // Put the new user in the DB
                            match diesel::insert_into(users)
                                .values(&schema_user)
                                .on_conflict(oauth_id)
                                .do_update()
                                .set(&models::UpdateUser {
                                    oauth_token: &schema_user.oauth_token,
                                    email: &schema_user.email,
                                })
                                .execute(&conn)
                            {
                                // The operation was completed successfully, 200
                                Ok(_) => {
                                    // Save the token in a session cookie
                                    session
                                        .set::<String>("token", access_token.secret().to_owned())?;

                                    // Respond with the user's details
                                    Ok(Json(models::OwnedUser {
                                        oauth_id: id_oauth.to_owned(),
                                        oauth_token: access_token.secret().to_owned(),
                                        email: schema_user.email.to_owned(),
                                    }))
                                }

                                // Since an error was thrown, return a 300
                                Err(e) => Err(error::ErrorInternalServerError(e)),
                            }
                        }

                        // Return the error in a response
                        Err(e) => Err(error::ErrorInternalServerError(e)),
                    }
                }

                // Handle any errors by returning a 500
                Err(e) => Err(error::ErrorInternalServerError(e)),
            }
        } else {
            // Return a 500 error, since we can't continue with out a pkce verifier
            Err(error::ErrorInternalServerError(
                "No PKCE challenge verifier exists.",
            ))
        }
    }
}
