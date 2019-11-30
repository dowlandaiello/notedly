use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use snafu::Snafu;
use std::{
    array::TryFromSliceError,
    convert::TryInto,
    fs::File,
    io::{BufReader, Error as IoError, Read},
    iter::Iterator,
};

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
    file_reader: Option<BufReader<File>>,
}

impl Iterator for Body {
    // 8 chars should be stored in each segment
    type Item = String;

    /// Gets the next sequence of character's in the notes body.
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer: String = String::new(); // Get a buffer to read into

        if let Some(file_reader) = self.file_reader.take() {
            // Read into the buffer
            match file_reader.take(8).read_to_string(&mut buffer) {
                // If we could read anything from the buffer, return it
                Ok(_) => Some(buffer),

                // Otherwise, don't return aything
                Err(_) => None,
            }
        } else {
            None // We don't have anything to read from, so we're done here!
        }
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
                match File::open(&f_name) {
                    Ok(file) => Ok(Body {
                        file_reader: Some(BufReader::new(file)),
                    }),
                    Err(e) => Err(Error::OpenNote {
                        filename: f_name,
                        source: e,
                    }),
                }
            }

            // Otherwise, don't return anything
            Err(_) => Err(Error::NoMetadata),
        }
    }
}
