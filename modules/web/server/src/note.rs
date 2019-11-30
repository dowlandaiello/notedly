use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use snafu::Snafu;
use std::{fs::File, io::Error as IoError, iter::Iterator, convert::TryInto, array::TryFromSliceError};

/// An error encountered while dealing with a note.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open note titled {}: {}", filename, source))]
    OpenNote { filename: String, source: IoError },
    #[snafu(display("Could not find appropriate metadata for the note"))]
    NoMetadata,
}

/// The contents of a note.
pub struct Body {
    /// The file storing the contents of the note
    file: File,
}

impl Iterator for Body {
    // 8 chars should be stored in each segment
    type Item = [char; 8];

    /// Gets the next sequence of character's in the notes body.
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer: [char; 8] = [0; 8]; // Get a buffer to read into

        // Read into the buffer
        self.file.read(&mut buffer);
    }
}

/// A note on a given board, authored by a particular user.
#[derive(Serialize, Deserialize)]
pub struct Note {
    /// The number of 8-letter segments contained in the note
    pub num_segments: i64,

    /// The title of the note
    pub title: String,

    /// The author of the note
    pub author: String,
}

impl Note {
    /// Initializes and returns a new note with the given author.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com", "Untitled"); // Make a new note
    /// ```
    pub fn new(author: String, title: String) -> Note {
        Note {
            num_segments: 0,
            title: author,
            author: title,
        } // Return the new note
    }

    /// Gets an iterator over the contents of the note.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com", "Untitled"); // Make a new note
    ///
    /// // Iterate through each of the 8-char segments in the note.
    /// // Note: The below print statement will never run, as nothing has been
    /// // to the note.
    /// for segment in n.contents().unwrap() {
    ///     println!("{}", segment); // Print the segment
    /// }
    /// ```
    pub fn contents(&self) -> Result<Body, Error> {
        let mut hasher = Sha3_256::new(); // Make a new hasher

        // We'll want to generate a hash comprising of the note's author_title.
        hasher.input(format!("{}_{}", self.author, self.title).as_bytes());

        // Get the result of hashing the note's metadata
        let hash: Result<[u8; 32], TryFromSliceError> = hasher.result()[..].try_into();

        // Make sure that we could successfully convert the hash into a 32-byte array
        match hash {
            Ok(h) => {
                // The name of the corresponding file should be formatted as such: hexhash.note
                let f_name: String = format!("{}.note", hex::encode(h));

                // Try opening the file
                match File::open(f_name) {
                    Ok(file) => Ok(Body { file: file }),
                    Err(e) => Err(Error::OpenNote {
                        filename: f_name,
                        source: e,
                    }),
                }
            },
            Err(e) => Err(Error::NoMetadata),
        }
    }
}
