use super::schema::{boards, notes, permissions, users};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(oauth_id)]
#[table_name = "users"]
pub struct User {
    /// The user's oauth ID provided by google or GitHub
    pub oauth_id: i32,

    /// A hash of the user's current oauth access token
    pub oauth_token: String,

    /// The email of the user
    pub email: String,

    /// The user's unique identifier
    pub id: i32,
}

/// An owned representation of the user struct. Usually used in server responses.
#[derive(Serialize)]
pub struct OwnedUser {
    /// The unique ID issued by the user's oauth provider (i.e. Google or GitHub)
    pub oauth_id: i32,

    /// The raw oauth token of the user
    pub oauth_token: String,

    /// The user's email
    pub email: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    /// The user's oauth identifier
    pub oauth_id: i32,

    /// The user's current oauth access token hash
    pub oauth_token: &'a str,

    /// The email of the new user
    pub email: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    /// The update oauth access token hash of the user
    pub oauth_token: &'a str,

    /// The updated email of the user
    pub email: &'a str,
}

#[derive(
    Serialize, Deserialize, Identifiable, Queryable, Associations, AsChangeset, PartialEq, Debug,
)]
#[belongs_to(User)]
#[table_name = "boards"]
pub struct Board {
    /// The hash of the board's title and owner
    pub id: i32,

    /// The ID of the user that the board is owned by
    pub user_id: i32,

    /// The title of the board
    pub title: String,

    /// The privacy setting of the board (0 => private, 1 => public [accessable by link])
    pub visibility: i16,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "boards"]
pub struct NewBoard {
    /// The ID of the user that the board is owned by
    pub user_id: i32,

    /// The title of the board
    pub title: String,

    /// The privacy setting of the board (0 => private, 1 => public [accessable by link])
    pub visibility: i16,
}

#[derive(Deserialize)]
pub struct UpdateBoard {
    /// The ID of the user that the board is owned by
    pub user_id: Option<i32>,

    /// The title of the board
    pub title: Option<String>,

    /// The privacy setting of the board (0 => private, 1 => public [accessable by link])
    pub visibility: Option<i16>,
}

impl UpdateBoard {
    /// Consumes the request to update the given board, and returns a new board given potentially
    /// empty fields in the current request to update & an old, completed request.
    pub fn new_board(&mut self, old: Board) -> Board {
        // Use fields from the new instance if they exist, but fall back to the old instance if
        // they don't
        Board {
            id: old.id,
            user_id: if let Some(usr_id) = self.user_id.take() {
                usr_id
            } else {
                old.user_id
            },
            title: if let Some(title) = self.title.take() {
                title
            } else {
                old.title
            },
            visibility: if let Some(vis) = self.visibility.take() {
                vis
            } else {
                old.visibility
            },
        }
    }
}

#[derive(
    Serialize, Deserialize, Identifiable, Queryable, Associations, AsChangeset, PartialEq, Debug,
)]
#[belongs_to(User)]
#[belongs_to(Board)]
#[table_name = "notes"]
pub struct Note {
    /// The hash of the note's name and author
    pub id: i32,

    /// The ID of the user that the note is owned by
    pub user_id: i32,

    /// The ID of the board that the note is owned by
    pub board_id: i32,

    /// The title of the note
    pub title: String,

    /// The text contained in the note
    pub body: String,
}

#[derive(Deserialize)]
pub struct UpdateNote {
    /// The ID of the user that the note is owned by
    pub user_id: Option<i32>,

    /// The ID of the board that the note is owned by
    pub board_id: Option<i32>,

    /// The title of the note
    pub title: Option<String>,

    /// The text contained in the note
    pub body: Option<String>,
}

impl UpdateNote {
    /// Initializes a new note from the provided old note, as well as a partially constructed
    /// UpdateNote.
    pub fn new_note(&mut self, old: Note) -> Note {
        // Return the new note
        Note {
            // The ID of this note CANNOT change
            id: old.id,

            // Use the new user ID if it exists, fallback to the old one
            user_id: if let Some(user_id) = self.user_id.take() {
                user_id
            } else {
                old.user_id
            },

            // Use the new board ID if it exists, fallback to the old one
            board_id: if let Some(board_id) = self.board_id.take() {
                board_id
            } else {
                old.board_id
            },

            // Use the new title if it exists, fallback to the old one
            title: if let Some(title) = self.title.take() {
                title
            } else {
                old.title
            },

            // Use the new note body if it exists, fallback to the old one
            body: if let Some(body) = self.body.take() {
                body
            } else {
                old.body
            },
        }
    }
}

#[derive(
    Serialize, Deserialize, Identifiable, Insertable, Queryable, Associations, PartialEq, Debug,
)]
#[belongs_to(User)]
#[belongs_to(Board)]
#[primary_key(user_id)]
#[table_name = "permissions"]
pub struct Permission {
    /// The ID of the permission
    pub id: i32,

    /// The ID of the user associated with the permission
    pub user_id: i32,

    /// The ID of the board associated with the permission
    pub board_id: i32,

    /// Whether or not the user can read from this board
    pub read: bool,

    /// Whether or not the user can write to this board
    pub write: bool,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "permissions"]
pub struct NewPermission {
    /// The ID of the user associated with the permission
    pub user_id: i32,

    /// The ID of the board associated with the permission
    pub board_id: i32,

    /// Whether or not the user can read from the board
    pub read: bool,

    /// Whether or not the user can write to the board
    pub write: bool,
}
