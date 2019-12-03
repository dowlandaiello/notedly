#[macro_use]
extern crate diesel_as_jsonb;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serializable, Deserializable, Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(email)]
#[table_name = "users"]
pub struct User {
    /// The email of the user
    pub email: String,

    /// The hash of the user's oauth access token
    pub id: String,
}

#[derive(Serializable, Deserializable, Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(User)]
#[table_name = "board"]
pub struct Board {
    /// The hash of the board's and owner
    pub id: String,

    /// The email of the board's owner
    pub owner: String,

    /// The privacy settinig of the board (0 => private, 1 => public [accessable by link])
    pub visibility: i8,

    /// The permissions of the board
    pub permissions: PermissionRule,
}

/// A permission for a particular user.
#[derive(Serialize, Deserialize)]
pub enum Permission {
    /// A user can read, delete, and write notes on a board
    Admin,

    /// A user can read notes on a board
    Read,

    /// A user can write new notes on a board
    Write,
}

/// A rule regarding permissions of some users.
#[derive(AsJsonb)]
pub struct PermissionRule {
    /// The permissions lol
    pub permissions: HashMap<String, Permission>,
}

#[derive(Serializable, Deserializable, Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[belongs_to(User)]
#[table_name = "notes"]
pub struct Note {
    /// The hash of the note's name and author
    pub id: String,

    /// The author of the note
    pub author: String,

    /// The title of the note
    pub title: String,

    /// The text contained in the note
    pub body: String,
}
