use std::{collections::HashMap, error::Error};

/// A provider of some user credentials or information.
pub enum AccessProvider {
    GitHub,
    Google,
}

impl AccessProvider {
    /// Derives an API request URI for a user's basic account information from the access provider
    /// type.
    ///
    /// # Example
    ///
    /// ```
    /// use server::user::AccessProvider;
    ///
    /// // => "https://www.googleapis.com/userinfo/v2/me"
    /// println!(AccessProvider::Google.base_profile_request_uri());
    /// ```
    pub fn base_profile_request_uri(&self) -> &str {
        match self {
            AccessProvider::GitHub => "https://api.github.com/user",
            AccessProvider::Google => "https://www.googleapis.com/userinfo/v2/me",
        } // The URI that we can use to get the user's personal information
    }
}

/// Any user of the notedly app.
pub struct User {
    /// The user's GitHub/Google access token
    pub access_token: String,

    /// The organization providing access to the user's profile
    pub provider: AccessProvider,
}

impl User {
    /// Gets the email of the user from the resource provider.
    pub async fn email(&self) -> Result<String, Box<dyn Error>> {
        // Get the user's overall profile information
        let resp: HashMap<String, String> = reqwest::get(self.provider.base_profile_request_uri())
            .await?
            .json()
            .await?;

        match resp.get("email") {
            Some(v) => Ok(*v),
            None => 
        }
        Ok(resp.get("email")?) // Return the email from the response
    }
}
