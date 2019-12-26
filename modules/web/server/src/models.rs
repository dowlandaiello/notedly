use super::schema::{boards, notes, users};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Identifiable, Queryable, Associations, PartialEq, Debug)]
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

    /// The permissions of the board
    pub permissions: Value,
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Associations, PartialEq, Debug)]
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
