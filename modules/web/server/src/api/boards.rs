use super::{
    super::{
        models::{User, Board},
        schema::{self, boards::dsl::*, permissions::dsl::*, users::dsl::*},
    },
    users::{extract_bearer, hash_token, Error},
};
use actix_web::{web::{Data, HttpRequest, Json, Path}, error};
use diesel::{
    dsl::exists,
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};

/// Gets a list of board IDs that the currently authenticated user is able to view.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/api/boards")]
pub async fn viewable_boards(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the currently authenticated user
    let u: User = users
        .filter(oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Get and return any of the boards belonging to the user (includes shared boards)
    Ok(Json(
        boards
            .select(schema::boards::id)
            .filter(
                schema::boards::user_id.eq(u.id).or(exists(
                    permissions.filter(
                        schema::permissions::board_id
                            .eq(schema::boards::id)
                            .and(schema::permissions::user_id.eq(u.id)),
                    ),
                )),
            )
            .load::<i32>(&conn)?,
    ))
}

/// Gets a specific board by its ID.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/api/boards/{board_id}")]
pub async fn specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_id: Path<String>,
    req: HttpRequest,
) -> Result<Json<Board>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the currently authenticated user
    let u: User = users
        .filter(oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Look at the request path, extract the board ID, and find the matching board.
    let matching_board: Board = boards.find(board_id).first(&conn);

    // Get the permissions of the user in the board
    let permission = permissions
        .filter(
            schema::permissions::board_id
                .eq(schema::boards::id)
                .and(schema::permissions::user_id.eq(u.id)),
        )
        .first(&conn)?;

    // If the user can read or write, then we can return the board!
    if permission.read || permission.write {
        // Return the found board
        Ok(Json(matching_board))
    } else {
        // Return an authorization error, since the user does not have the proper permissions to
        // view this board
        Err(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to view this board."))
    }
}
