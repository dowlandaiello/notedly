use super::super::{
    models::{Board, Note, User},
    schema::{self, users::dsl::*},
};
use actix_web::{
    error,
    web::{Data, HttpRequest, Json, Path},
    Error as ActixError,
};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use sha3::{Digest, Sha3_256};

/// Represents an extended actix webserver error.
pub struct Error(pub ActixError);

impl Into<ActixError> for Error {
    /// Converts the general notedly error into an Actix error.
    fn into(self) -> ActixError {
        // Return the error contained inside the error
        self.0
    }
}

impl<E: std::error::Error + 'static> From<E> for Error {
    /// Converts the provided error into a geral notedly server error by simply constructing
    /// a new notedly error describing an internal server error.
    fn from(e: E) -> Self {
        Self(error::ErrorInternalServerError(e)) // Return the error as an internal server error
    }
}

/// Gets the user's oauth bearer token from the given HTTP request. If the token isn't found, an
/// error is returned.
///
/// # Arguments
///
/// * `req` - The user's request
pub(crate) fn extract_bearer(req: &HttpRequest) -> Result<&'_ str, Error> {
    // First, check that the key even exists in the request's headers
    if let Some(bearer_token) = req.headers().get("Authorization") {
        // Remove the "Bearer " prefix from the header value
        if let Some(split_token) = bearer_token.to_str()?.split(' ').last() {
            Ok(split_token) // Return the token as a string
        } else {
            // The token value doesn't exist, so just return an empty string
            Ok("")
        }
    } else {
        // Return error describing this discrepancy
        Err(Error(error::ErrorUnauthorized(
            "No applicable bearer token was provided.",
        )))
    }
}

/// Hashes the inputted string via sha3 256.
///
/// # Arguments
///
/// * `token` - The token to be hashed
pub(crate) fn hash_token(token: &str) -> String {
    // Make a new hasher for the token
    let mut token_hasher = Sha3_256::new();
    token_hasher.input(token);

    // Return the hashed input as a string (hex-encoded)
    hex::encode(token_hasher.result())
}

/// Gets the user with the given oauth token (much, much slower than with_id, since this involves
/// filters).
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method
#[get("/api/user")]
pub async fn user(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
) -> Result<Json<User>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get and return the matching user
    Ok(Json(
        users
            .filter(oauth_token.eq(hash_token(token)))
            .first(&conn)?,
    ))
}

/// Gets a list of boards belonging to a user with the given ID.
///
/// # Arguments
///
/// * `pool` -The connection pool that will be used to connect to the postgres database
/// * `user_id` - A parameter contained in the path, as such: /api/users/my_id, where my_id is the
/// unique identifier assigned to a particular user. The ID assigned to each user is a 32-bit
/// integer
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/api/users/{user_id}/boards")]
pub async fn boards_from_user_with_id(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    user_id: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get a user from the database with the same ID as was provided by the client
    let u = users.find(*user_id).first::<User>(&conn)?;

    // Check that the provided access token matches the one on file
    if u.oauth_token == hash_token(token) {
        // Get the ID of each board, and return a vector of these IDs
        Ok(Json(
            Board::belonging_to(&u)
                .select(schema::boards::id)
                .load::<i32>(&conn)?,
        ))
    } else {
        // The codes don't match, communicate this discrepancy through
        // a 300 (unauth) error
        Err(Error(error::ErrorUnauthorized(
            "The provided access code does not match the recorded token.",
        )))
    }
}

/// Gets a list of notes belonging to a user with the given ID.
///
/// # Arguments
///
/// * `pool` -The connection pool that will be used to connect to the postgres database
/// * `user_id` - A parameter contained in the path, as such: /api/users/my_id, where my_id is the
/// unique identifier assigned to a particular user. The ID assigned to each user is a 32-bit
/// integer
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/api/users/{user_id}/notes")]
pub async fn notes_from_user_with_id(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    user_id: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get a user from the database with the same ID as was provided by the client
    let u = users.find(*user_id).first::<User>(&conn)?;

    // Check that the provided access token matches the one on file
    if u.oauth_token == hash_token(token) {
        // Get the ID of each note, and return a vector of these IDs
        Ok(Json(
            Note::belonging_to(&u)
                .select(schema::notes::id)
                .load::<i32>(&conn)?,
        ))
    } else {
        // The codes don't match, communicate this discrepancy through
        // a 300 (unauth) error
        Err(Error(error::ErrorUnauthorized(
            "The provided access code does not match the recorded token.",
        )))
    }
}

/// Gets the user with given oauth token and ID from the database.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `user_id` - A parameter contained in the path, as such: /api/users/my_id, where my_id is the
/// unique identifier assigned to a particular user. The ID assigned to each user is a 32-bit
/// integer
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/api/users/{user_id}")]
pub async fn user_with_id(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    user_id: Path<i32>,
    req: HttpRequest,
) -> Result<Json<User>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get a user from the database with the same ID as was provided by the client
    let u = users.find(*user_id).first::<User>(&conn)?;

    // Check that the provided access token matches the one on file
    if u.oauth_token == hash_token(token) {
        // Return the user's details
        Ok(Json(u))
    } else {
        // The codes don't match, communicate this discrepancy through
        // a 300 (unauth) error
        Err(Error(error::ErrorUnauthorized(
            "The provided access code does not match the recorded token.",
        )))
    }
}

/// Gets the usernames of each user in the database.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
#[get("/api/users")]
pub async fn all_user_ids(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Json<Vec<i32>>, Error> {
    match pool.get() {
        // We were able to connect to the database, get the requested data (all usernames)
        Ok(conn) => {
            // Get the ID of each user, and return an appropriate repsonse
            match users.select(oauth_id).load::<i32>(&conn) {
                // Respond with each of the user IDs
                Ok(user_ids) => Ok(Json(user_ids)),

                // Since an error was found, return a 300
                Err(e) => Err(Error(error::ErrorInternalServerError(e))),
            }
        }

        // Since an error was found, return a 300
        Err(e) => Err(Error(error::ErrorInternalServerError(e))),
    }
}
