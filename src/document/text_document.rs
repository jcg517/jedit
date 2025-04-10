use std::str::FromStr;
use std::{error::Error, path::Path};
use crate::document::file_io;

pub struct TextDocument {
    line_offsets: Vec<u64>,
    text_buffer: String,  // For now, just hold data as as a string
}

impl TextDocument {

    /// Process the raw text data and producing an offset mapping of all new lines for easy retrieval
    fn parse_lines(data: &String) -> Result<Vec<u64>, Box<dyn Error + Send + Sync>> {
        let mut line_offsets = vec![0];
        let mut iter = data.char_indices().peekable();
        while let Some((i, ch)) = iter.next() {
            if ch == '\r' {
                // Check if the next character is a newline, indicating a CRLF sequence.
                if let Some(&(j, next_ch)) = iter.peek() {
                    if next_ch == '\n' {
                        iter.next();  // Consume the newline as part of the CRLF combination.
                        line_offsets.push((j + 1) as u64);
                        continue;
                    }
                }
                // If not followed by '\n', treat the carriage return on its own as a newline.
                line_offsets.push((i + 1) as u64);
            } else if ch == '\n' {
                line_offsets.push((i + 1) as u64);
            }
        }
        
        Ok(line_offsets)
    }
    
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Attempt to open and load the file into text buffer
        let text_buffer = file_io::load_from_file(path)?;
        let line_offsets = Self::parse_lines(&text_buffer)?;

        Ok(TextDocument { line_offsets, text_buffer })
    }
    
    /// Given a line number, return string of that line's text
    pub fn getline<'a>(&'a self, lineno: u64) -> Option<&'a str> {
        let num_lines = self.line_offsets.len();
        let idx = lineno as usize;
        
        if idx >= num_lines {
            return None;
        }

        let curr_start = self.line_offsets[idx];
        let next_start = if idx + 1 < num_lines {
            self.line_offsets[idx + 1]
            } else {
                self.text_buffer.len() as u64
            };

        if next_start < curr_start {
            return None;
        }

        let line: &str = &self.text_buffer[curr_start as usize..next_start as usize];
        Some(line)
    }

    pub fn line_count(&self) -> usize {
        self.line_offsets.len()
    }
    
}