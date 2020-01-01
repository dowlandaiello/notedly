use super::{
    super::{
        models::{Board, NewBoard, NewPermission, Note, Permission, UpdateBoard, User},
        schema::{self, boards::dsl::*, permissions::dsl::*, users::dsl::*},
    },
    boards::{continue_if_authenticated, continue_if_has_perms},
    users::{extract_bearer, hash_token, Error},
};
use actix_web::{
    error,
    web::{Data, HttpRequest, HttpResponse, Json, Path},
};
use diesel::{
    dsl::{delete, exists, select, update},
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
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

    // Ensure that the user can read from the associated board
    continue_if_has_perms(
        &conn,
        matching_note.board_id,
        &matching_user,
        falsee,
        true,
        false,
    )?;

    // Reteurn the note
    Ok(Json(matching_note))
}
