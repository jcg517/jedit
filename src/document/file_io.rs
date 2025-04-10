use crate::document::text_document::TextDocument;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Error};

pub fn load_from_file(path: &Path) -> Result<String, Error> {
    // TODO: add checks for file types
    println!("Loading document from file: {}", path.display());

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    Ok(data)
}

pub fn save_to_file(text_buffer: TextDocument, path: &str) {
    // Todo: serialize document and write to disk
    println!("Saving document to file: {}", path);
}