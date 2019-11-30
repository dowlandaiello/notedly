pub mod geometric; // Export the geometric mode

use serde::{Deserialize, Serialize};

/// Any particular mode of textual content representation.
#[derive(Serialize, Deserialize)]
pub enum Mode {
    /// The default format used to represent English writing
    Anglicized,

    /// A format used to represent mathematical writing
    Algebraic,

    /// A format used to represent technical writing
    Programming,

    /// A format used to represent drawings
    Geometric,

    /// A format based on the markdown language
    Markdown,
}
