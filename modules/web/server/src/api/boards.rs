use super::{
    super::{
        models::{Board, NewBoard, NewPermission, Note, Permission, UpdateBoard, User},
        schema::{self, boards::dsl::*, permissions::dsl::*, users::dsl::*, notes::dsl::*},
    },
    users::{extract_bearer, hash_token, Error},
};
use actix_web::{
    error,
    web::{Data, HttpRequest, HttpResponse, Json, Path},
    Scope as ActixScope,
};
use diesel::{
    dsl::{delete, exists, select, update},
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};

/// Ensures that the provided access token matches that of the provided user.
pub(crate) fn continue_if_authenticated(user: &User, auth_token: &str) -> Result<(), Error> {
    // Return Ok if the tokens match, otherwise unauth
    if hash_token(auth_token) != user.oauth_token {
        Err(Error(error::ErrorUnauthorized(
            "The provided access token does not match the recorded token for this user.",
        )))
    } else {
        Ok(())
    }
}

/// Ensures that the user has the given permissions.
pub(crate) fn continue_if_has_perms(
    conn: &PgConnection,
    board_uid: i32,
    user: &User,
    owner: bool,
    can_read: bool,
    can_write: bool,
) -> Result<(), Error> {
    // Construct a query to get the board with ID from the db
    let b_query = boards.find(board_uid);

    // The board must exist for all to continue. Otherwise, throw a 404.
    if !select(exists(b_query)).get_result(conn)? {
        // Return a 404
        return Err(Error(error::ErrorNotFound(format!(
            "No board with the id '{}' exists.",
            board_uid
        ))));
    }

    // Get the board that matches the given UID from the database
    let matching_board: Board = b_query.first(conn)?;

    // If the end-user wants to enforce that the token matches an owner, so be it
    if owner {
        // If the user is not registered as the owner, return an error.
        if matching_board.user_id != user.id {
            // Respond with an unauth
            return Err(Error(error::ErrorUnauthorized(
                "The provided access token does not match a user with the required privileges.",
            )));
        }

        // The user is authenticated
        return Ok(());
    }

    // Construct a query to get the permission belonging to the user with the board
    let p_query =
        Permission::belonging_to(user).filter(schema::permissions::board_id.eq(board_uid));

    // If this permission doesn't exist, the user isn't even envited to the board
    if !select(exists(p_query)).get_result(conn)? {
        // Return a 404
        return Err(Error(error::ErrorNotFound(
            "The provided access token does not match a user that has been invited to this board.",
        )));
    }

    // Get the permission belonging to the user
    let permission: Permission = Permission::belonging_to(user)
        .filter(schema::permissions::board_id.eq(board_uid))
        .first(conn)?;

    // Ensure the user has the proper permissions to be able to write & read to the file
    if !(permission.write && permission.read || !can_write) {
        // Respond with an unauth
        return Err(Error(error::ErrorUnauthorized(
            "The provided access token does not match a user with the required privileges (write).",
        )));
    }

    // Ensure that the user has the proper permissions to be able to read to the file
    if can_read && !permission.read {
        // Respond with an unauth
        return Err(Error(error::ErrorUnauthorized(
            "THe provided access token does not match a user with the required privileges (read).",
        )));
    }

    // The user should only have gotten this far if each of the preconditions were met--meaning
    // that the user does have sufficient privileges
    Ok(())
}

/// Constructs an actix service group for the boards endpoint.
pub fn build_service_group() -> ActixScope {
    ActixScope::new("/boards")
        .service(viewable_boards)
        .service(specific_board)
        .service(new_board)
        .service(update_specific_board)
        .service(delete_specific_board)
        .service(all_permissions)
        .service(all_notes)
        .service(all_users)
}

/// Gets a list of board IDs that the currently authenticated user is able to view.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("")]
pub async fn viewable_boards(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the currently authenticated user
    let u: User = match users.filter(oauth_token.eq(hash_token(token))).first(&conn) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error(error::ErrorNotFound(
            "The requested user does not exist.",
        ))),
    }?;

    // Get and return any of the boards belonging to the user (includes shared boards)
    Ok(Json(
        boards
            .filter(
                schema::boards::user_id.eq(u.id).or(exists(
                    permissions.filter(
                        schema::permissions::user_id
                            .eq(u.id)
                            .and(schema::permissions::board_id.eq(schema::boards::id)),
                    ),
                )),
            )
            .select(schema::boards::id)
            .load::<i32>(&conn)?,
    ))
}

/// Initializes and puts a new board in the currently authenticated user's db directory.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// * `board` - The JSON request body sent by the user dictating how to create the new board
#[post("")]
pub async fn new_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    req: HttpRequest,
    board: Json<NewBoard>,
) -> Result<Json<Board>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get a reference to the user mentioned in the request, so we can authenticate.
    let u: User = match users.find(board.user_id).first(&conn) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error(error::ErrorNotFound(format!(
            "The requested user (id: {}) does not exist.",
            board.user_id
        )))),
    }?;

    // Ensure that the user is who they say they are
    continue_if_authenticated(&u, token)?;

    // Put the board into the database, and save a reference to its associated JSON encoding
    let written_board: Board = diesel::insert_into(boards)
        .values(&*board)
        .get_result(&conn)?;

    // Put permissions for the owner of this board into the database
    diesel::insert_into(permissions)
        .values(&NewPermission {
            user_id: u.id,
            board_id: written_board.id,
            read: true,
            write: true,
        })
        .execute(&conn)?;

    // Put the new board in the database and return it
    Ok(Json(written_board))
}

/// Gets a specific board by its ID.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/{board_id}")]
pub async fn specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Board>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel.
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the requested user from the database, and return a 404 if it doesn't exist.
    let u: User = match users.filter(oauth_token.eq(hash_token(token))).first(&conn) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error(error::ErrorNotFound(
            "The requested user does not exist.",
        ))),
    }?;

    // Get the requested board from the database
    match boards.find(*board_uid).first::<Board>(&conn) {
        // The board exists, let's return it
        Ok(board) => {
            // Ensure the user is able to read from the board
            continue_if_has_perms(&conn, *board_uid, &u, false, true, false)?;

            // Return the board
            Ok(Json(board))
        }

        // The board does not exist, return a 404
        Err(_) => Err(Error(error::ErrorNotFound(format!(
            "The requested board (id: {}) does not exist.",
            *board_uid
        )))),
    }
}

/// Updates a specific board by its ID.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `new_board` - A JSON request detailing how to update the board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[patch("/{board_id}")]
pub async fn update_specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    mut update_to_board: Json<UpdateBoard>,
    req: HttpRequest,
) -> Result<Json<Board>, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Look at the request path, extract the board ID, and find the matching board in the database
    let board_entry: Board = match boards.find(*board_uid).first(&conn) {
        Ok(b) => Ok(b),
        Err(_) => Err(Error(error::ErrorNotFound(format!(
            "The requested board (id: {}) does not exist.",
            *board_uid
        )))),
    }?;

    // Get the matching user from the request so that we can authenticate
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Ensure that the user is actually the owner of the board
    continue_if_has_perms(&conn, *board_uid, &matching_user, true, false, false)?;

    // Merge the old and new boards
    let merged_boards: Board = update_to_board.new_board(board_entry);

    // Update the board in the table
    Ok(Json(update(boards).set(&merged_boards).get_result(&conn)?))
}

/// Deletes a board with the given ID.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[delete("/{board_id}")]
pub async fn delete_specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the user that was mentioned in the request
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Ensure that the user is the owner of the board
    continue_if_has_perms(&conn, *board_uid, &matching_user, true, false, false)?;

    // Delete the board
    delete(boards.find(*board_uid)).execute(&conn)?;

    // Delete the associated permissions
    delete(permissions.filter(schema::permissions::board_id.eq(*board_uid))).execute(&conn)?;

    // Delete the associated notes
    delete(notes.filter(schema::notes::board_id.eq(*board_uid))).execute(&conn)?;

    // Update the board in the table
    Ok(HttpResponse::Ok().finish())
}

/// Gets a list of permissions for the board.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/{board_id}/permissions")]
pub async fn all_permissions(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Vec<Permission>>, Error> {
    // Get a connection from the provided connection pool, so we can start using dieisel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Look at the request path, extract the board ID, and find the matching board.
    let matching_board: Board = boards.find(*board_uid).first(&conn)?;

    // Get the user making the request
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Ensure that the requesting user is in fact a user that is able to view the board
    continue_if_has_perms(&conn, *board_uid, &matching_user, false, true, false)?;

    // Return each of the permissions belonging to the board
    Ok(Json(
        Permission::belonging_to(&matching_board).get_results(&conn)?,
    ))
}

/// Gets a list of notes belonging to the board.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/{board_id}/notes")]
pub async fn all_notes(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using dieisel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Look at the request path, extract the board ID, and find the matching board.
    let matching_board: Board = boards.find(*board_uid).first(&conn)?;

    // Get the user making the request
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Ensure that the user is in fact the owner of the board
    continue_if_has_perms(&conn, *board_uid, &matching_user, false, true, true)?;

    // Return each of the notes belonging to the board (just their IDs)
    Ok(Json(
        Note::belonging_to(&matching_board)
            .select(schema::notes::id)
            .get_results(&conn)?,
    ))
}

/// Gets a list of users associated with the board.
///
/// # Arguments
///
/// * `pool` - The connection pool that will be used to connect to the postgres database
/// * `board_uid` - The ID of the requested board
/// * `req` - An HTTP request provided by the caller of this method. Used to obtain the bearer
/// token (required) of the user
#[get("/{board_id}/users")]
pub async fn all_users(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<Json<Vec<i32>>, Error> {
    // Get a connection from the provided connection pool, so we can start using dieisel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Look at the request path, extract the board ID, and find the matching board.
    let matching_board: Board = boards.find(*board_uid).first(&conn)?;

    // Get the user from the database with the provided oauth token
    let matching_user: User = users
        .filter(schema::users::oauth_token.eq(hash_token(token)))
        .first(&conn)?;

    // Ensure that the requesting user has access to the board (read, at least)
    continue_if_has_perms(&conn, *board_uid, &matching_user, false, true, false)?;

    // Return a list of invited users
    Ok(Json(
        Permission::belonging_to(&matching_board)
            .select(schema::permissions::user_id)
            .get_results(&conn)?,
    ))
}
