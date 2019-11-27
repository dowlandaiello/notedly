use super::modes::{geometric::Shape, Mode};

/// A character or sequence of characters.
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
    pub fn new(mode: Mode) -> Expression {
        Expression{
            sub_expression: None,
            value: None,
            shape: None,
            mode: mode,
        } // Return the new expression
    }
}

/// A note on a given board, authored by a particular user.
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
    pub fn new(author: String) -> Note {
        Note {
            author: author,
            contents: Vec::new(),
            children: Vec::new(),
            dimensions: (10, 10),
        } // Return the new note
    }
}
