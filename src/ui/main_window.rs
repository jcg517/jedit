use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr,
};

use crate::ui::editor_view;

use windows::{
    core::*,
    Win32::{
        Foundation::*, 
        Graphics::Gdi::HBRUSH,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::Dialogs::{
                GetOpenFileNameW,
                OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
            },
            WindowsAndMessaging::*,
        },
    },
};

const APP_TITLE: PCWSTR = w!("Jedit");

// --- Menu Item IDs --- (typically be defined in a resource file (.rc) and header (.h))
const IDM_FILE_NEW: u16 = 1001;
const IDM_FILE_OPEN: u16 = 1002;
const IDM_HELP_ABOUT: u16 = 2001;

// Custom application messages for communcation with editor view control
const EVM_OPENFILE: u32 = WM_USER + 1;
const EVM_CLEARFILE: u32 = WM_USER + 2;

// Helper function to replicate the LOWORD macro
#[inline]
fn loword(dword: usize) -> u16 {
    (dword & 0xFFFF) as u16
}

/// Sets the title text of the main window.
/// Prepends the application title to the given file name.
fn set_window_file_name(hwnd: HWND, file_name: PCWSTR) -> Result<()> {
    unsafe {
        let app_title_str = APP_TITLE.to_string().unwrap_or_else(|_| "Jedit".to_string());
        let file_name_str = file_name.to_string().unwrap_or_else(|_| "Untitled".to_string());

        let combined_title = format!("{} - {}", file_name_str, app_title_str);

        // Convert the combined title to a null-terminated wide string (Vec<u16>)
        let title_wide: Vec<u16> = combined_title
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        if SetWindowTextW(hwnd, PCWSTR(title_wide.as_ptr())).is_err() { // Check Result with is_err()
            return Err(Error::from_win32());
        }
    }
    Ok(())
}

/// Shows the standard Windows "Open File" common dialog.
/// Returns Option<(PathBuf, String)> containing the full path and the file name (title)
/// if the user selects a file, otherwise returns None.
fn show_open_file_dialog(hwnd: HWND) -> Option<(PathBuf, String)> {
    unsafe {
        let mut file_buffer: [u16; 260] = [0; 260];
        let mut title_buffer: [u16; 260] = [0; 260];

        // Define the filter string (null-terminated pairs, double-null terminated at the end)
        let filter: Vec<u16> = "Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0\0"
            .encode_utf16()
            .collect();

        // Initialize the OPENFILENAMEW structure.
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            lpstrFileTitle: PWSTR(title_buffer.as_mut_ptr()),
            nMaxFileTitle: title_buffer.len() as u32,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            nFilterIndex: 1,
            lpstrInitialDir: PCWSTR::null(),
            Flags: OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST,
            ..Default::default()
        };

        if GetOpenFileNameW(&mut ofn) == TRUE { 
            // Find the actual length of the returned path string
            let path_len = file_buffer.iter().position(|&c| c == 0).unwrap_or(file_buffer.len());
            let file_path = PathBuf::from(OsString::from_wide(&file_buffer[..path_len]));

            // Find the actual length of the returned title string
            let title_len = title_buffer.iter().position(|&c| c == 0).unwrap_or(title_buffer.len());
            let file_title = String::from_utf16_lossy(&title_buffer[..title_len]);

            Some((file_path, file_title))
        } else {
            // User cancelled or an error occurred. Check CommDlgExtendedError for details if needed.
            None
        }
    }
}

/// Displays a simple "About" message box.
fn show_about_dialog(hwnd: HWND) {
    let text = w!("Jedit - Simple Rust Text Editor\nVersion 0.1");
    let caption = w!("About Jedit");
    unsafe {
        MessageBoxW(Some(hwnd), text, caption, MB_OK | MB_ICONINFORMATION); // Wrap hwnd in Some()
    }
}

fn create_menu_bar() -> Result<HMENU> {
    let hmenu = unsafe { CreateMenu()? };
    let hsubmenu = unsafe { CreatePopupMenu()? };

    let result = unsafe {
        AppendMenuW(hsubmenu, MF_STRING, IDM_FILE_NEW as usize, w!("New"))?;
        AppendMenuW(hsubmenu, MF_STRING, IDM_FILE_OPEN as usize, w!("Open"))?;
        AppendMenuW(hsubmenu, MF_SEPARATOR, 0, None)?;
        AppendMenuW(hsubmenu, MF_STRING, IDM_HELP_ABOUT as usize, w!("About"))?;
        AppendMenuW(hmenu, MF_POPUP, hsubmenu.0 as usize, w!("File"))?;
        Ok(())
    };

    if let Err(e) = result {
        unsafe { DestroyMenu(hmenu); DestroyMenu(hsubmenu); }
        return Err(e);
    }

    Ok(hmenu)
}

/// Register Main window class
pub fn init_main_window() -> Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;

        let hcursor = LoadCursorW(None, IDC_ARROW)?;

        let wc = WNDCLASSW {
            hInstance: hinstance.into(),
            lpszClassName: APP_TITLE,
            lpfnWndProc: Some(wndproc),
            hCursor: hcursor,
            hbrBackground: HBRUSH(ptr::null_mut()),
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err(Error::from_win32());
        }
    }
    Ok(())
}
    
/// Creates the window
pub fn create_main_window() -> Result<HWND> {
    let hinstance = unsafe { GetModuleHandleW(None)? };

    let hwnd = unsafe { CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        APP_TITLE,
        APP_TITLE,
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 600, 400,
        None,
        None,
        Some(hinstance.into()), // Wrap hinstance in Some() and convert
        None
    )}?;

    Ok(hwnd)
}

// Standalone window procedure function
extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            // Create the editor view child window first
            let hwnd_editor = match editor_view::create_editor_view(hwnd) {
                Ok(hwnd_editor) => {
                    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, hwnd_editor.0 as isize) };
                    hwnd_editor // Store the handle if successful
                }
                Err(e) => {
                    eprintln!("Failed to create editor view: {}", e);
                    return LRESULT(-1); // Return -1 to indicate failure to create window
                }
            };

            let hmenu = match create_menu_bar() {
                Ok(menu) => menu,
                Err(e) => {
                    eprintln!("{}", e);
                    return LRESULT(-1);
                }
            };

            // Set the menu for the window
            if unsafe { SetMenu(hwnd, Some(hmenu)) }.is_err() {
                eprintln!("SetMenu failed");
                unsafe { DestroyMenu(hmenu); } // includes the submenu
                return LRESULT(-1);
            }

            // Draw the menu bar
            if unsafe { DrawMenuBar(hwnd) }.is_err() {
                eprintln!("DrawMenuBar failed");
                // If DrawMenuBar fails, the menu *is* associated, so it still needs cleanup on WM_DESTROY.
                // We don't destroy it here.
                return LRESULT(-1);
            }

            // Menu creation successful
            LRESULT(0)
        }
        WM_SIZE => {
            let hwnd_editor_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) }; // Add unsafe block
            let hwnd_editor = HWND(hwnd_editor_ptr as *mut _); // Cast isize to *mut c_void
            if !hwnd_editor.0.is_null() { // Compare pointer with is_null()
                let mut rect = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rect) }; // Add unsafe block
                unsafe { SetWindowPos( // Add unsafe block
                    hwnd_editor,
                    None,
                    0, 0,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOMOVE
                ) };
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let command_id = loword(wparam.0); // Use helper function
            let hwnd_editor_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) }; // Add unsafe block
            let hwnd_editor = HWND(hwnd_editor_ptr as *mut _); // Cast isize to *mut c_void

            match command_id {
                IDM_FILE_NEW => {
                    // println!("WM_COMMAND: IDM_FILE_NEW"); // Keep commented for debugging
                    if let Err(e) = set_window_file_name(hwnd, w!("Untitled")) { // Removed underscore from _e
                        eprintln!("Failed to set window title for New File: {}", e); // Keep commented for debugging
                    }

                    // Send message to editor view to clear its content
                    unsafe { SendMessageW(hwnd_editor, EVM_CLEARFILE, Some(WPARAM(0)), Some(LPARAM(0))) }; // Add unsafe block

                    LRESULT(0)
                }
                IDM_FILE_OPEN => {
                    // println!("WM_COMMAND: IDM_FILE_OPEN"); // Keep commented for debugging
                    if let Some((file_path, file_title)) = show_open_file_dialog(hwnd) {
                        println!("  -> File selected: {}", file_path.display()); // Keep commented for debugging

                        let file_path_wide: Vec<u16> = file_path
                            .as_os_str()
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();
                        let file_ptr = file_path_wide.as_ptr();

                        // Send message to editor view to open the file
                        // EVM_OPENFILE returns LRESULT(1) on success, LRESULT(0) on failure
                        let open_result = unsafe { SendMessageW(hwnd_editor, EVM_OPENFILE, Some(WPARAM(0)), Some(LPARAM(file_ptr as isize))) }; // Add unsafe block
                        let open_success = open_result == LRESULT(1);

                        if open_success {
                            // Update the main window title
                            let file_title_pcwstr = OsString::from(file_title)
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect::<Vec<_>>();
                            if let Err(e) = set_window_file_name(hwnd, PCWSTR(file_title_pcwstr.as_ptr())) {
                                eprintln!("Failed to set window title after Open File: {}", e); // Keep commented for debugging
                            }
                        } else {
                            // Show error message if opening failed
                            let error_text = w!("Error opening file.");
                            unsafe { MessageBoxW(Some(hwnd), error_text, APP_TITLE, MB_OK | MB_ICONEXCLAMATION) }; // Add unsafe block
                        }
                    } else {
                        println!("  -> File open dialog cancelled."); // Keep commented for debugging
                    }
                    LRESULT(0)
                }

                IDM_HELP_ABOUT => {
                    println!("WM_COMMAND: IDM_HELP_ABOUT"); // Keep commented for debugging
                    show_about_dialog(hwnd);
                    LRESULT(0)
                }

                _ => {
                    println!("WM_COMMAND: Unhandled ID {}", command_id); // Keep commented for debugging
                    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) } 
                }
            }
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            // Clean up user data when the main window is destroyed
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
            // Terminate the application's message loop
            unsafe { PostQuitMessage(0) }; 
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
