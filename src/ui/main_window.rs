// src/ui/main_window.rs
use std::ffi::c_void;
use std::path::Path;
use crate::document::text_document::TextDocument;
use crate::ui::editor_view::EditorView;
use windows::{
    core::*,
    Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, COLORREF},
    Win32::UI::WindowsAndMessaging::{
        WNDCLASSW, RegisterClassW, CreateWindowExW, DefWindowProcW, TranslateMessage,
        DispatchMessageW, GetMessageW, PostQuitMessage, LoadCursorW, WS_OVERLAPPEDWINDOW,
        WS_VISIBLE, CW_USEDEFAULT, WINDOW_EX_STYLE, MSG, IDC_ARROW, WM_DESTROY, WM_PAINT, WM_CREATE,
        SetWindowLongPtrW, GetWindowLongPtrW, GWLP_USERDATA, CREATESTRUCTW,
    },
    Win32::Graphics::Gdi::CreateSolidBrush,
    Win32::System::LibraryLoader::GetModuleHandleW,
};

pub struct MainWindow {
    hwnd: HWND,
}

impl MainWindow {
    pub fn new(path: &Path) -> Result<Self> {
        // Load the document from file.
        let document = match TextDocument::new(path) {
            Ok(doc) => doc,
            Err(e) => {
                panic!("Failed to load {}: {}", path.display(), e);
            }
        };

        // Create the EditorView instance and obtain a raw pointer.
        let editor_view = Box::new(EditorView::new(document));
        let editor_view_ptr = Box::into_raw(editor_view) as *mut c_void;

        unsafe {
            let hinstance = GetModuleHandleW(None)?;
            let class_name = w!("ParentWindowClass");

            let wc = WNDCLASSW {
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                lpfnWndProc: Some(Self::wndproc),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hbrBackground: CreateSolidBrush(COLORREF(15128749)),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("Win32 Text Editor"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                600,
                None,         // No parent window
                None,         // No menu
                Some(hinstance.into()),
                Some(editor_view_ptr), // Pass EditorView pointer as lpParam
            )?;

            // Clean up if window creation fails.
            if hwnd.is_invalid() {
                // The Box::from_raw call reclaims ownership of the EditorView to drop it.
                drop(Box::from_raw(editor_view_ptr as *mut EditorView));
                return Err(Error::from_win32());
            }

            Ok(MainWindow { hwnd })
        }
    }

    pub fn run(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    // Window procedure encapsulated as an associated function.
    extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            // Retrieve the EditorView pointer stored in window's user data.
            let editor_view_ptr = if msg == WM_CREATE {
                0
            } else {
                GetWindowLongPtrW(hwnd, GWLP_USERDATA)
            };

            match msg {
                WM_CREATE => {
                    let create_struct = lparam.0 as *const CREATESTRUCTW;
                    if !create_struct.is_null() {
                        let editor_view_raw_ptr = (*create_struct).lpCreateParams as isize;
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, editor_view_raw_ptr);
                    }
                    LRESULT(0)
                }
                WM_PAINT => {
                    if editor_view_ptr != 0 {
                        let editor_view = &*(editor_view_ptr as *const EditorView);
                        if let Err(e) = editor_view.on_paint(hwnd) {
                            eprintln!("Error painting: {}", e);
                        }
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                    if ptr != 0 {
                        drop(Box::from_raw(ptr as *mut EditorView));
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    }
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }
}