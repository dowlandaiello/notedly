use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use snafu::Snafu;
use std::{
    array::TryFromSliceError,
    convert::TryInto,
    fs::{File, OpenOptions},
    io::{BufReader, Error as IoError, Write},
};

/// An error encountered while dealing with a note.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open note titled {}: {}", filename, source))]
    OpenNote { filename: String, source: IoError },

    #[snafu(display("Could not find appropriate metadata for the note"))]
    NoMetadata,

    #[snafu(display("Could not write to the note titled {}: {}", filename, source))]
    WriteNote { filename: String, source: IoError },
}

/// The contents of a note.
#[derive(Serialize, Deserialize)]
pub struct Body {
    /// The file storing the contents of the note
    #[serde(skip)]
    file: Option<File>,

    /// The number of 8-character strings contained in the body of the note
    pub num_segments: i64,
}

impl Body {
    /// Writes the string to the body of a note, and returns the number of segments written to the
    /// note.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to write to the note
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::{Note, Body};
    ///
    /// let n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
    /// let mut b = n.body().unwrap(); // Get a read/writer for the body of the note
    /// b.write("Hello, world!".to_string()); // Write a few characters to the note
    /// ```
    pub fn write(&mut self, s: String) -> Result<usize, Error> {
        if let Some(mut file) = self.file.take() {
            // Each of the segments that will be written to the note
            let mut segments: Vec<String> = vec![s];

            // The number of segments successfully written to the file
            let mut num_written: usize = 0;

            let mut i = 0; // An iterator

            // Keep adding 8-char segments of the inputted string to the note
            loop {
                // The original segment, before slicing--if necessary
                let mut segment = segments[i].clone();

                // If the segment is too big, slice it up
                if segment.len() > 8 {
                    segments.push(segment[8..].to_owned()); // Put the rest on the end of the segments vec
                    segment = segment[..8].to_owned(); // Slice it up
                }

                // Write the segment to the file
                match file.write(segment.as_bytes()) {
                    Ok(_) => {
                        num_written += 1;
                    }
                    Err(_) => (),
                };

                // Check if the loop should be done
                if segment.len() < 8 {
                    break; // Break
                }

                i += 1; // Increment the iterator
            }

            // Recalculate the number of segments in the body of the note
            self.num_segments += num_written as i64;

            // Move the opened file back into the Body struct
            self.file.replace(file);

            Ok(num_written) // Return the number of segments successfully written to the file
        } else {
            Ok(0) // The file isn't open, so we can't write anything to it
        }
    }

    /// Gets a reader for the note. If the file isn't already open, None is returned.
    pub fn reader(self) -> Option<BufReader<File>> {
        if let Some(file) = self.file {
            Some(BufReader::new(file)) // Return a buff reader for the file
        } else {
            None // Return nothing
        }
    }
}

/// A note on a given board, authored by a particular user.
#[derive(Serialize, Deserialize)]
pub struct Note {
    /// The title of the note
    pub title: String,

    /// The author of the note
    pub author: String,

    /// The body of the note
    body: Body,
}

impl Note {
    /// Initializes and returns a new note with the given author.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
    /// ```
    pub fn new(author: String, title: String) -> Note {
        Note {
            title: author,
            author: title,
            body: Body {
                file: None,
                num_segments: 0,
            },
        } // Return the new note
    }

    /// Gets an read/writer for the body of the note.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
    /// let b = n.body().unwrap(); // Get a read-writer for the body of the note
    /// ```
    pub fn body(&self) -> Result<Body, Error> {
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
                match OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(&f_name)
                {
                    Ok(file) => Ok(Body {
                        file: Some(file),
                        num_segments: self.body.num_segments,
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
