use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use snafu::Snafu;
use std::{
    array::TryFromSliceError,
    convert::TryInto,
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, Error as IoError, Read, Write},
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
pub struct Body<'a> {
    /// The file storing the contents of the note
    file: Option<File>,

    /// The number of 8-character strings contained in the body of the note
    num_segments: Option<&'a mut usize>,
}

impl<'a> Default for Body<'a> {
    /// Initializes and returns a zero-value body.
    fn default() -> Self {
        // Return a zero-value body
        Body {
            file: None,
            num_segments: None,
        }
    }
}

impl<'a> Body<'a> {
    /// Gets the number of segments in the note body.
    ///
    /// # Examples
    ///
    /// ```
    /// use server::note::{Note, Body};
    ///
    /// // Make an empty note
    /// let mut n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned());
    ///
    /// let mut b = n.body().unwrap(); // Get a read/writer for the note
    /// assert_eq!(b.num_segments(), n.num_segments);
    /// ```
    pub fn num_segments(&mut self) -> usize {
        // Check that we have a valid reference to the parent note's segment count
        if let Some(num_segments) = self.num_segments.take() {
            let num = (*num_segments); // Get the number of segments contained in the note

            self.num_segments.replace(num_segments); // Again, idk

            return num; // Return the number of segments
        } else {
            0
        }
    }

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
    /// let mut n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
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

            // Check that we have a valid reference to the parent note's number of segments
            if let Some(num_segments) = self.num_segments.take() {
                // Recalculate the number of segments in the body of the note
                *num_segments += num_written;

                // Put the thing back in the body. Idk I'm tired, okay?
                self.num_segments.replace(num_segments);
            }

            // Move the opened file back into the Body struct
            self.file.replace(file);

            Ok(num_written) // Return the number of segments successfully written to the file
        } else {
            Ok(0) // The file isn't open, so we can't write anything to it
        }
    }

    /// Reads n segments of 8 characters from the note's body.
    ///
    /// # Arguments
    ///
    /// * `n_segments` - The number of 8-character segments to read from the note
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::{Note, Body};
    //
    /// // Make a new note
    /// let mut n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned());
    /// let mut b = n.body().unwrap(); // Get the body of the note
    ///
    /// b.write("Some test message!".to_owned()); // Write something to the note
    /// println!("{}", b.read(1)[0]); // => "Some tes"
    /// ```
    pub fn read(&mut self, n_segments: usize) -> Vec<String> {
        // Check that we have actually opened the corresponding file
        if let Some(mut file) = self.file.take() {
            // The result of the read operation should contain n segments. Allocate it as such.
            let mut result: Vec<String> = vec!["".to_owned(); n_segments];

            // Perform the read operation n_segments times
            for i in 0..n_segments {
                Read::by_ref(&mut file)
                    .take(32)
                    .read_to_string(&mut result[i]); // Read 8 characters into the result buf
            }

            result // Return the segments we could read from the file
        } else {
            Vec::new() // Return an empty vector, since we can't read from an unopened file
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
pub struct Note<'a> {
    /// The title of the note
    pub title: String,

    /// The author of the note
    pub author: String,

    /// The number of segments contained in the note
    pub num_segments: usize,

    /// The body of the note
    #[serde(skip)]
    body: Body<'a>,
}

impl<'a> Note<'a> {
    /// Initializes and returns a new note with the given author.
    ///
    /// # Example
    ///
    /// ```
    /// use server::note::Note;
    ///
    /// let n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
    /// ```
    pub fn new(author: String, title: String) -> Note<'a> {
        Note {
            title: author,
            author: title,
            num_segments: 0,
            body: Body {
                file: None,
                num_segments: None,
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
    /// let mut n = Note::new("dowlandaiello@gmail.com".to_owned(), "Untitled".to_owned()); // Make a new note
    /// let b = n.body().unwrap(); // Get a read-writer for the body of the note
    /// ```
    pub fn body(&mut self) -> Result<Body, Error> {
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
                        num_segments: Some(&mut self.num_segments),
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
