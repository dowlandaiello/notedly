use super::modes::{geometric::Shape, Mode};
use serde::{Serialize, Deserialize};

/// A character or sequence of characters.
#[derive(Serialize, Deserialize)]
pub struct Expression {
    /// The expressions contained in the expression
    pub sub_expressions: Option<Vec<Expression>>,

    /// The character contained in the expression
    pub value: Option<char>,

    /// The shape contained in the expression
    pub shape: Option<Shape>,

    /// The format in which the character is written
    pub mode: Mode,
}

impl Expression {
    /// Initializes and returns a new expression with the given mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The format that the expression should be formatted in
    ///
    /// # Example
    ///
    /// ```
    /// use server::{note::Expression, modes::Mode::Algebraic};
    ///
    /// // Make a new algebraic expression
    /// let e = Expression::new(Algebraic);
    /// ```
    pub fn new(mode: Mode) -> Expression {
        Expression {
            sub_expressions: None,
            value: None,
            shape: None,
            mode: mode,
        } // Return the new expression
    }
}

/// A note on a given board, authored by a particular user.
#[derive(Serialize, Deserialize)]
pub struct Note {
    /// The author of the note
    pub author: String,

    /// The last saved contents of the note
    pub contents: Vec<Expression>,

    /// Any notes contained inside the note
    pub children: Vec<Note>,

    /// The dimensions of the note, as a percentage of the viewport
    pub dimensions: (i8, i8),
}

impl Note {
    /// Initializes and returns a new note with the given author.
    ///
    /// # Arguments
    ///
    /// * `author` - The username of the author of the note
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com".to_owned()); // Make a new note
    /// ```
    pub fn new(author: String) -> Note {
        Note {
            author: author,
            contents: Vec::new(),
            children: Vec::new(),
            dimensions: (10, 10),
        } // Return the new note
    }
}
