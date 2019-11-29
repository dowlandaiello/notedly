use super::modes::{geometric::Shape, Mode};
use serde::{Serialize, Deserialize};

/// A note on a given board, authored by a particular user.
#[derive(Serialize, Deserialize)]
pub struct Note {
    /// The last saved contents of the note
    pub contents: Vec<String>,
}

impl Note {
    /// Initializes and returns a new note with the given author.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new(); // Make a new note
    /// ```
    pub fn new(author: String) -> Note {
        Note {
            contents: Vec::new(),
        } // Return the new note
    }
}
