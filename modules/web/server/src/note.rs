use mode::{Mode, geometric::Shape};

/// A string containing one or more characters.
pub struct Expression {
    /// The characters contained in the note
    pub chars: Vec<char>, 

    /// The points contained in a possibly geometric expression
    pub shapes: Optin(Vec<Shape>),

    /// The format in which the characters are written
    pub mode: Mode,
}

/// A note on a given board, authored by a particular user.
pub struct Note {
    /// The author of the note
    author: String,

    /// The last saved contents of the note
    contents: Vec<Expression>,

    /// Any notes contained inside the note
    children: Vec<Node>,

    /// The dimensions of the note
    dimensions: (i8, i8),
}
