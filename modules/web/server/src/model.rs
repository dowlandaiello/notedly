use super::schema::{boards, notes, users};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(email)]
#[table_name = "users"]
pub struct User {
    /// The email of the user
    pub email: String,

    /// The hash of the user's oauth access token
    pub id: i32,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    /// The email of the new user
    pub email: &'a str,
}

#[derive(
    Serialize, Deserialize, Insertable, Identifiable, Queryable, Associations, PartialEq, Debug,
)]
#[belongs_to(User)]
#[table_name = "boards"]
pub struct Board {
    /// The hash of the board's title and owner
    pub id: String,

    /// The ID of the user that the board is owned by
    pub user_id: i32,

    /// The title of the board
    pub title: String,

    /// The privacy setting of the board (0 => private, 1 => public [accessable by link])
    pub visibility: i16,

    /// The permissions of the board
    pub permissions: Value,
}

#[derive(
    Serialize, Deserialize, Insertable, Identifiable, Queryable, Associations, PartialEq, Debug,
)]
#[belongs_to(User)]
#[table_name = "notes"]
pub struct Note {
    /// The hash of the note's name and author
    pub id: String,

    /// The ID of the user that the note is owned by
    pub user_id: i32,

    /// The title of the note
    pub title: String,

    /// The text contained in the note
    pub body: String,
}
