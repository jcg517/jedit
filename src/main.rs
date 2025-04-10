// src/main.rs
mod ui;
mod document;

use std::path::Path;
use windows::core::*;
use ui::main_window::MainWindow;

fn main() -> Result<()> {
    // Initialize the MainWindow from a file path.
    let main_window = MainWindow::new(Path::new("test.txt"))?;
    main_window.run();
    Ok(())
}