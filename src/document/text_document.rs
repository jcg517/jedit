use std::{error::Error, path::Path};
use crate::document::file_io;

pub struct TextDocument {
    line_offsets: Vec<usize>,
    text_buffer: String,
}

impl TextDocument {

    /// Creates a new, empty TextDocument.
    pub fn new() -> Self {
        TextDocument {
            line_offsets: vec![0],
            text_buffer: String::new(),
        }
    }

    /// Recalculates line offsets based on the current text_buffer.
    /// Assumes line_offsets starts with `vec![0]`.
    fn init_line_offsets(&mut self) -> Result<(), Box<dyn Error>> {
        self.line_offsets.truncate(1);

        for (i, ch) in self.text_buffer.char_indices() {
            if ch == '\n' {
                // Record the offset *after* the newline character
                self.line_offsets.push(i + 1);
            }
        }
        // Note: Doesn't explicitly handle files without trailing newline,
        // but getline logic correctly handles the last line.
        Ok(())
    }

    /// Initializes the document by loading content from a file path.
    /// Clears existing content before loading.
    pub fn init(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.clear();
        self.text_buffer = file_io::load(path)?;
        self.init_line_offsets()?;
        Ok(())
    }

    /// Clears the document content and resets state to empty.
    pub fn clear(&mut self) {
        self.line_offsets = vec![0];
        self.text_buffer.clear();
    }
    
    /// Given a 0-based line number, returns a string slice of that line's text,
    /// excluding the trailing newline character(s).
    pub fn getline(&self, lineno: usize) -> Option<&str> {
        let num_lines = self.line_offsets.len();

        if lineno >= num_lines {
            return None;
        }

        let start_offset = self.line_offsets[lineno];
        let end_offset = if lineno + 1 < num_lines {
            self.line_offsets[lineno + 1]
        } else {
            self.text_buffer.len() // End of the buffer for the last line
        };

        // Basic sanity check
        if start_offset > end_offset || end_offset > self.text_buffer.len() {
             eprintln!("getline error: Invalid offsets {}..{}", start_offset, end_offset); 
             return None; // Indicates an internal error
        }

        let mut line_slice = &self.text_buffer[start_offset..end_offset];

        // Trim trailing newline characters (\n or \r\n)
        // Check for \n first, then \r
        if line_slice.ends_with('\n') {
            line_slice = &line_slice[..line_slice.len() - 1];
        }
        if line_slice.ends_with('\r') {
            line_slice = &line_slice[..line_slice.len() - 1];
        }

        Some(line_slice)
    }

    /// Returns the number of lines in the document.
    pub fn line_count(&self) -> usize {
        self.line_offsets.len()
    }

    /// Returns the total length of the text buffer in bytes.
    pub fn len(&self) -> usize {
        self.text_buffer.len()
    }

    /// Returns a reference to the entire text buffer.
    pub fn get_content(&self) -> &str {
        &self.text_buffer
    }
}