mod ui;
mod document;

use windows::{
    core::{Result, HSTRING},
    Win32::{
        Foundation::E_FAIL,
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
    },
};

use crate::ui::editor_view::*; 
use crate::ui::main_window::*; 

fn main() -> Result<()> { // Revert return type to windows::core::Result<()>
    // Initialize window classes
    init_main_window()?;
    init_editor_view().map_err(|e| windows::core::Error::new(E_FAIL, format!("init_editor_view failed: {}", e)))?;

    // Create the main window
    let _hwnd_main = create_main_window().map_err(|e| windows::core::Error::new(E_FAIL, format!("create_main_window failed: {}", e)))?;

    // Run the message loop for main window
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}