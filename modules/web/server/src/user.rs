use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use snafu::Snafu;
use std::{
    collections::HashMap,
    convert::{From, TryInto},
};

/// Any error that might occur whilst dealing with a user profile.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("No records could be found for the user"))]
    NoUserRecords,
    #[snafu(display("Request failed: {}", request_error))]
    Request { request_error: reqwest::Error },
}

impl From<reqwest::Error> for Error {
    /// Converts a reqwest error to a notedly-native user Error.
    ///
    /// # Arguments
    ///
    /// * `error` - The reqwest error
    fn from(error: reqwest::Error) -> Self {
        Self::Request {
            request_error: error,
        } // Return the error
    }
}

/// A provider of some user credentials or information.
#[derive(Serialize, Deserialize)]
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
    /// println!("{}", AccessProvider::Google.base_profile_request_uri());
    /// ```
    pub fn base_profile_request_uri(&self) -> &str {
        match self {
            AccessProvider::GitHub => "https://api.github.com/user",
            AccessProvider::Google => "https://www.googleapis.com/userinfo/v2/me",
        } // The URI that we can use to get the user's personal information
    }
}

/// Any user of the notedly app.
#[derive(Serialize, Deserialize)]
pub struct User {
    /// The user's GitHub/Google access token
    pub access_token: String,

    /// The organization providing access to the user's profile
    pub provider: AccessProvider,
}

impl User {
    /// Generates a unique identifier for the user based on their access token.
    ///
    /// # Example
    ///
    /// ```
    /// use server::user::{User, AccessProvider::Google};
    ///
    /// let u = User {
    ///     access_token: "MY_ACCESS_TOKEN".to_owned(),
    ///     provider: Google
    /// }; // Make an empty user
    ///
    /// // The user should have a 32-byte user ID
    /// assert_eq!(u.user_id().len(), 32);
    /// ```
    pub fn user_id(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new(); // Make a new sha3_256 hasher

        // Put the user's access token into the hasher. This is guaranteed to be unique.
        hasher.input(self.access_token.as_bytes());

        hasher.result()[..].try_into().expect("malformed hash") // Return the result of hashing the input
    }

    /// Gets the email of the user from the resource provider.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use tokio::runtime::current_thread::Runtime;
    /// use server::user::{User, AccessProvider::Google};
    ///
    /// # fn main() {
    /// #     Runtime::new().unwrap().block_on(test()); // Run the test
    /// # }
    ///
    /// # async fn test() {
    /// let u = User {
    ///     access_token: "MY_ACCESS_TOKEN".to_owned(),
    ///     provider: Google,
    /// }; // Make a new user
    ///
    /// // Print the user's email
    /// println!("{}", u.email().await.unwrap());
    /// # }
    /// ```
    pub async fn email(&self) -> Result<String, Error> {
        // Get the user's overall profile information
        let resp: HashMap<String, String> = reqwest::get(self.provider.base_profile_request_uri())
            .await?
            .json()
            .await?;

        // If the response doesn't contain an email, return an error
        match resp.get("email") {
            Some(v) => Ok(v.clone()),
            None => Err(Error::NoUserRecords),
        }
    }
}
