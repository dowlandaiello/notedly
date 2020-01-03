use super::{
    super::{
        models::{Note, UpdateNote, User},
        schema::{self, notes::dsl::*, users::dsl::*},
    },
    users::{extract_bearer, hash_token, Error},
};
use actix_web::{
    error,
    web::{Data, HttpRequest, Json, Path},
};
use diesel::{
    dsl::update,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};

/// Gets a specific note from the database.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
/// * `note_id` - The unique identifier assigned to the note that the user wishes to read
#[get("/api/notes/{note_id}")]
pub async fn specific_note(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
    note_id: Path<i32>,
) -> Result<Json<Note>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get the note from the database, then do some authentication checking with the
    // provided token.
    let matching_note: Note = notes.find(*note_id).first(&conn)?;

    // Get the user's details from the provided token
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(extract_bearer(&req)?)))
        .first(&conn)?;

    // Ensure that the user is in fact the owner of the note
    if matching_note.user_id != matching_user.id {
        return Err(Error(error::ErrorUnauthorized("The provided access token does not match a user with sufficient privileges to read this note.")));
    }

    // Reteurn the note
    Ok(Json(matching_note))
}

/// Updates a specific note.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
/// * `note_id` - The unique identifier assigned to the note that the user wishes to read
#[patch("/api/notes/{note_id}")]
pub async fn update_specific_note(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
    note_id: Path<i32>,
    mut updated_note: Json<UpdateNote>,
) -> Result<Json<Note>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel
    let conn = pool.get()?;

    // Get the note from the database, then do some authtnication checking with the
    // provided token.
    let matching_note: Note = notes.find(*note_id).first(&conn)?;

    // Get the user's details from the provided token
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(extract_bearer(&req)?)))
        .first(&conn)?;

    // Ensure that the user is in fact the owner of the ntoe
    if matching_note.user_id != matching_user.id {
        return Err(Error(error::ErrorUnauthorized("The provided access token does not match a user with sufficient privileges to update this note.")));
    }

    // Merge the updated note and the old note, in case the user didn't update some of the fields
    let new_note: Note = updated_note.new_note(matching_note);

    // Return the new note after updating whatever is already in the database
    Ok(Json(update(notes).set(&new_note).get_result(&conn)?))
}
