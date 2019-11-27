use std::fmt::{Display, Formatter, Result};

/// A vector body constructed of a sequences of points and their associated
/// links.
pub struct Shape {
    /// The points in the shape
    pub points: Vec<Point>,

    /// The connections between each point
    pub connections: Vec<[usize; 2]>,
}

impl Shape {
    /// Initializes and returns a new empty shape.
    pub fn new() -> Shape {
        Shape {
            points: Vec::new(),
            connections: Vec::new(),
        } // Return an empty shape
    }
}

/// A point on a euclidean plane represented by a tuple of pixels.
pub struct Point(pub i64, pub i64);

impl Point {
    /// Returns a new point with the given coordinates.
    pub fn new(x: i64, y: i64) -> Self {
        Self(x, y) // Return the point
    }
}

impl Display for Point {
    /// Converts the given point to a coordinate forammted as such: (x, y).
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {})", self.0, self.1) // Return the coordinate as a string
    }
}
