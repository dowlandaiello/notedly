use super::super::{models::User, schema::users::dsl::*};
use actix_web::{
    error,
    web::{Data, Json, Path},
    Error,
};
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    RunQueryDsl,
};

/// Gets the user with given oauth token from the database.
#[get("/api/user")]
pub async fn user_with_id(
    info: Path<i32>,
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<Json<User>, Error> {
    match pool.get() {
        // We were able to connect to the database, get the requested data (all usernames)
        Ok(conn) => {
            // Get the user with the given ID
            match users.find(*info).first(&conn) {
                // The user was found
                Ok(u) => Ok(Json(u)),
                
                // The user was not found, or an error was returned
                Err(e) => Err(error::ErrorInternalServerError(e)),
            }
        }

        // Since an error was found, return a 300
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}

/// Gets the usernames of each user in the database.
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
                Err(e) => Err(error::ErrorInternalServerError(e)),
            }
        }

        // Since an error was found, return a 300
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}
