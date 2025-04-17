use std::path::Path;
use std::fs;
use std::io::Read;
use std::error::Error;
use crate::document::text_document::TextDocument;

/// Loads the content of a file into a string using OpenOptions.
/// Creates the file if it doesn't exist.
pub fn load(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

/// Saves the content of the TextDocument to the specified path.
pub fn save(doc: &TextDocument, path: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(path, doc.get_content())?; 
    Ok(())
}