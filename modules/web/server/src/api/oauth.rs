use super::server::OauthConfig;
use actix_session::Session;
use actix_web::{
    http,
    web::{Data, Path},
    Error, HttpResponse,
};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

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
            _ => (&data.github_api_client, ""),
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
        session.set("state", csrf_state)?;
        session.set("verifier", pkce_verifier)?;

        println!("{}", auth_url.as_str());

        // Redirect the user to the auth url
        Ok(HttpResponse::TemporaryRedirect()
            .header(http::header::LOCATION, auth_url.as_str())
            .finish())
    }
}
