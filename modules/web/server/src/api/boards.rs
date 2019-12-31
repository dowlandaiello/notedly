use super::{
    super::{
        models::{Board, NewBoard, NewPermission, Note, Permission, UpdateBoard, User},
        schema::{self, boards::dsl::*, permissions::dsl::*, users::dsl::*},
    },
    users::{extract_bearer, hash_token, Error},
};
use actix_web::{
    error,
    web::{Data, HttpRequest, HttpResponse, Json, Path},
};
use diesel::{
    dsl::{delete, exists, update},
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};

/// Ensures that the user has the given permissions.
fn has_permissions(
    conn: PgConnection,
    board_uid: i32,
    user: User,
    owner: bool,
    read: bool,
    write: bool,
) -> Result<(), Error> {
    // Construct a query to get the board with ID from the db
    let b_query = boards.find(board_uid);

    // The board must exist for all to continue. Otherwise, throw a 404.
    if !exists(b_query).get_result(&conn)? {
        // Return a 404
        return Err(Error(error::ErrorNotFound()));
    }

    // Get the board that matches the given UID from the database
    let matching_board = b_query.first(&conn)?;

    // If the end-user wants to enforce that the token matches an owner, so be it
    if owner {
        // If the user is not registered as the owner, return an error.
        if matching_boarduser_id != user.id {
            // Respond with an unauth
            return Err(Error(error::ErrorUnauthorized()));
        }

        // The user is authenticated
        return Ok(());
    }

    // Construct a query to get the permission belonging to the user with the board
    let p_query =
        Permission::belonging_to(&user).filter(schema::permissions::board_id.eq(board_uid));

    // If this permission doesn't exist, the user isn't even envited to the board
    if !exists(p_query).get_result(&conn)? {
        // Return a 404
        return Err(Error(error::ErrorNotFound()));
    }

    // Get the permission belonging to the user
    let permission: Permission = Permission::belonging_to(&user)
        .filter(schema::permissions::board_id.eq(board_uid))
        .first(&conn)?;

    // Ensure the user has the proper permissions to be able to write & read to the file
    if write && !permission.write || write && !permissions.read {
        // Respond with an unauth
        return Err(Error(error::ErrorUnauthorized()));
    }

    // Ensure that the user has the proper permissions to be able to read to the file
    if read && !permissions.read {
        // Respond with an unauth
        return Err(Error(error::ErrorUnauthorized()));
    }

    // The user should only have gotten this far if each of the preconditions were met--meaning
    // that the user does have sufficient privileges
    Ok(())
}

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
            .filter(
                schema::boards::user_id.eq(u.oauth_id).or(exists(
                    permissions.filter(
                        schema::permissions::user_id
                            .eq(u.oauth_id)
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
#[post("/api/boards")]
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
    let u: User = users.find(board.user_id).first(&conn)?;

    // Ensure that no funky business is going on...
    if u.oauth_token != hash_token(token) {
        // Return an authorization error, since the user is not allowed to create a board with this
        // username without having the proper credentials
        return Err(Error(error::ErrorUnauthorized(
            "The provided access token does not match the recorded authoring token hash.",
        )));
    }

    // Put the board into the database, and save a reference to its associated JSON encoding
    let written_board: Board = diesel::insert_into(boards)
        .values(&*board)
        .get_result(&conn)?;

    // Put permissions for the owner of this board into the database
    diesel::insert_into(permissions)
        .values(&NewPermission {
            user_id: u.oauth_id,
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
#[get("/api/boards/{board_id}")]
pub async fn specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
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
    let matching_board: Board = boards.find(*board_uid).first(&conn)?;

    // Get the permissions of the post corresponding to the user indicated by the bearer token
    match Permission::belonging_to(&matching_board)
        .filter(schema::permissions::user_id.eq(u.oauth_id))
        .first::<Permission>(&conn)
    {
        // The permission exists in the DB!
        Ok(perm) => {
            // If the user can read or write, then we can return the board!
            if perm.read || perm.write {
                // Return the found board
                Ok(Json(matching_board))
            } else {
                // Return an authorization error, since the user does not have the proper permissions to
                // view this board
                Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to view this board.")))
            }
        }

        Err(e) => {
            // Return the error in the form of a 500 unauth
            Err(Error(error::ErrorUnauthorized(e)))
        }
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
#[patch("/api/boards/{board_id}")]
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
    let board_entry: Board = boards.find(*board_uid).first(&conn)?;

    // Get the owner of the matching board
    let owner: User = users.find(board_entry.user_id).first(&conn)?;

    // Ensure that the user is the owner of the board
    if owner.oauth_token != hash_token(token) {
        // Return an authorization error
        return Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to edit this board.")));
    };

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
#[delete("/api/boards/{board_id}")]
pub async fn delete_specific_board(
    pool: Data<Pool<ConnectionManager<PgConnection>>>,
    board_uid: Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Get a connection from the provided connection pool, so we can start using diesel
    let conn = pool.get()?;

    // Get an authorization token from the headers sent with the request
    let token = extract_bearer(&req)?;

    // Get the owner of the board that the user wants to delete
    let owner: User = users
        .find(
            boards
                .find(*board_uid)
                .select(schema::boards::user_id)
                .first::<i32>(&conn)?,
        )
        .first(&conn)?;

    // Ensure that the user is the owner of the board
    if owner.oauth_token != hash_token(token) {
        // Return an authorization error
        return Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to edit this board.")));
    };

    // Delete the board
    delete(boards.find(*board_uid)).execute(&conn)?;

    // Delete the associated permissions
    delete(permissions.find(*board_uid)).execute(&conn)?;

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
#[get("/api/boards/{board_id}/permissions")]
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

    // Get the owner of the board that the user wants to delete
    let owner: User = users.find(matching_board.user_id).first(&conn)?;

    // Ensure that the user is the owner of the board
    if owner.oauth_token != hash_token(token) {
        // Return an authorization error
        return Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to edit this board.")));
    };

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
#[get("/api/boards/{board_id}/notes")]
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

    // Get the owner of the board that the user wants to delete
    let owner: User = users.find(matching_board.user_id).first(&conn)?;

    // Ensure that the user is the owner of the board
    if owner.oauth_token != hash_token(token) {
        // Return an authorization error
        return Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to edit this board.")));
    };

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

    // Ensure that this user is capable of reading from this board
    if !exists(
        Permission::belonging_to(&matching_board).filter(
            schema::permissions::user_id
                .eq(matching_user.id)
                .and(schema::permissions::read.eq(true)),
        ),
    ) {
        return Err(Error(error::ErrorUnauthorized("The provided bearer token does not match a user with the required permissions to view this board.")));
    }
}
