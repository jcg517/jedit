use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, EndPaint, GetDC, GetStockObject, GetTextMetricsW, InvalidateRect,
            ReleaseDC, SelectObject, TextOutW, ANSI_FIXED_FONT, HBRUSH, HDC, HFONT,
            PAINTSTRUCT, TEXTMETRICW, FillRect, COLOR_WINDOW, GetSysColorBrush
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW,
            RegisterClassW, SendMessageW, SetWindowLongPtrW, IDC_ARROW, WINDOW_EX_STYLE,
            WNDCLASSW, WS_CHILD, WS_HSCROLL, WS_VISIBLE, WS_VSCROLL, 
            WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SETFONT, WM_USER, WINDOW_LONG_PTR_INDEX,
        },
    },
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStringExt;
use std::{error::Error, path::Path, ptr};
use crate::document::text_document::TextDocument;

const EDITOR_VIEW_CLASS: PCWSTR = w!("EditorView32");

const EVM_OPENFILE: u32 = WM_USER + 1;
const EVM_CLEARFILE: u32 = WM_USER + 2;

pub struct EditorView {
    hwnd: HWND,
    document: TextDocument,
    caret_pos: usize,
    font_height: i32,
    font_width: i32,
    hfont: HFONT,
    line_count: usize,
}

impl EditorView {
    /// Constructs a new EditorView with the given initial text.
    pub fn new(hwnd: HWND) -> Self {

        // Get the handle to the stock fixed-width font
        let stock_font_handle = unsafe { GetStockObject(ANSI_FIXED_FONT) };
        let hfont = HFONT(stock_font_handle.0);

        // Set the font for the editor window
        unsafe {
            SendMessageW(hwnd, WM_SETFONT, Some(WPARAM(hfont.0 as usize)), Some(LPARAM(1))); // Wrap args in Some()
        }

        let document = TextDocument::new();

        let line_count = 0; // Initial line count

        let mut view = Self {
            hwnd,
            document,
            caret_pos: 0,
            font_height: 0, // Will be set by update_font_metrics
            font_width: 0,  // Will be set by update_font_metrics
            hfont,
            line_count,
        };
        // Calculate initial font metrics, log error if it fails
        if let Err(e) = view.update_font_metrics() {
             eprintln!("ERROR: Failed to calculate initial font metrics: {}", e);
        }
        view
    }

    /// Retrieves a mutable reference to the EditorView instance from the window's extra storage.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it relies on the pointer stored in the window data being valid.
    pub unsafe fn from_hwnd(hwnd: HWND) -> Option<&'static mut Self> {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) } as *mut EditorView; // Add unsafe block
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr }) // Add unsafe block
        }
    }

    /// Calculates and updates font metrics (height and average width) based on the current font.
    fn update_font_metrics(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            let hdc = GetDC(Some(self.hwnd));
            if hdc.is_invalid() {
                return Err("GetDC failed".into());
            }
            let old_font = SelectObject(hdc, self.hfont.into());
            let mut tm = TEXTMETRICW::default();
            if GetTextMetricsW(hdc, &mut tm) == false {
                SelectObject(hdc, old_font);
                ReleaseDC(Some(self.hwnd), hdc);
                return Err("GetTextMetricsW failed".into());
            }
            SelectObject(hdc, old_font);
            ReleaseDC(Some(self.hwnd), hdc);

            self.font_height = tm.tmHeight + tm.tmExternalLeading;
            self.font_width = tm.tmAveCharWidth;
        }
        Ok(())
    }

    /// Handles the WM_SETFONT message. Updates the font and recalculates metrics.
    pub fn on_set_font(&mut self, new_hfont: HFONT) -> Result<(), Box<dyn Error>> {
        self.hfont = new_hfont;
        self.update_font_metrics()?; // Recalculate metrics
        unsafe { InvalidateRect(Some(self.hwnd), None, true); } 
        Ok(())
    }
    /// WM_PAINT handler for the text view.
    /// This method begins painting, draws the text, and ends painting.
    pub fn on_paint(&self) -> Result<(), Box<dyn Error>> {
        let mut ps = PAINTSTRUCT::default();
        unsafe {
            let hdc = BeginPaint(self.hwnd, &mut ps);
            if hdc.0.is_null() {
                return Err("BeginPaint failed".into());
            }
            // Fill background
            FillRect(hdc, &ps.rcPaint, GetSysColorBrush(COLOR_WINDOW));

            // Select the editor's font into the DC
            let old_font = SelectObject(hdc, self.hfont.into());

            // Calculate the first and last line based on the paint area and font height
            let num_lines = self.document.line_count();
            let first_line = ps.rcPaint.top / self.font_height;
            let last_line = std::cmp::min(ps.rcPaint.bottom / self.font_height, num_lines as i32 - 1);
            for line in first_line..=last_line {
                self.paint_line(hdc, line)?;
            }

            // Restore the original font
            SelectObject(hdc, old_font);
            EndPaint(self.hwnd, &ps);
        }
        Ok(())
    }

    fn paint_line(&self, hdc: HDC, line_idx: i32) -> Result<(), Box<dyn Error>> {
        // Safely convert line index (i32) to usize for getline
        if let Ok(line_usize) = usize::try_from(line_idx) {
            if let Some(line_text) = self.document.getline(line_usize) {
                // Convert the Rust string to a null-terminated UTF-16 string
                let text_wide: Vec<u16> = line_text.encode_utf16().chain(std::iter::once(0)).collect();
                // Calculate the Y position based on the line number and font height
                let y = line_idx * self.font_height; // Simple Y calculation
                // Draw the text at position (0, y)
                unsafe {
                    if TextOutW(hdc, 0, y, &text_wide) == false { // Use bool false
                        return Err("TextOutW failed".into());
                    }
                }
            } else {
                eprintln!("Warning: Invalid line index {} encountered during painting.", line_idx); // Keep commented for debugging
            }
        }
        Ok(())
    }

    // File IO message handlers
    pub fn clear_file(&mut self) -> Result<(), Box<dyn Error>> {
        self.document.clear();
        self.line_count = self.document.line_count();
        unsafe { InvalidateRect(Some(self.hwnd), None, true); }
        Ok(())
    }

    pub fn open_file(&mut self, filename_pcwstr: PCWSTR) -> Result<(), Box<dyn Error>> {
        self.clear_file()?;
        // Convert PCWSTR to &Path
        let path_osstr = unsafe { std::ffi::OsString::from_wide(filename_pcwstr.as_wide()) };
        let path = Path::new(&path_osstr);

        self.document.init(path)?; 
        self.line_count = self.document.line_count();
        
        Ok(())
    }

   // TODO: Additional methods handling scrolling, keyboard input, etc.
}

pub fn init_editor_view() -> Result<(), Box<dyn Error>> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;

        // Load the standard arrow cursor
        let hcursor = LoadCursorW(None, IDC_ARROW)?;

        let wc = WNDCLASSW {
            hInstance: hinstance.into(),
            lpszClassName: EDITOR_VIEW_CLASS,
            lpfnWndProc: Some(wndproc),
            hCursor: hcursor,
            hbrBackground: HBRUSH(ptr::null_mut()), // No background brush (we handle painting)
            cbWndExtra: std::mem::size_of::<*mut EditorView>() as i32, // Reserve extra space for pointer
            ..Default::default()
        };

        // Register the window class
        if RegisterClassW(&wc) == 0 {
            return Err(windows::core::Error::from_win32().into());
        }
    }
    Ok(())
}

pub fn create_editor_view(hwnd_parent: HWND) -> Result<HWND, Box<dyn Error>> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        eprintln!("hinstance: {:?}", hinstance);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),                      // Optional window styles
            EDITOR_VIEW_CLASS,                               // Window class name
            EDITOR_VIEW_CLASS,                                          // Window title (none)
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_HSCROLL, // Window styles
            0, 0, 0, 0,                                      // Position and size (set later)
            Some(hwnd_parent),                               // Wrap hwnd_parent in Some()
            None,                                            // No menu or child ID
            Some(hinstance.into()),                          // Wrap hinstance in Some() and convert
            None,                                            // No additional application data
        )?;

        eprintln!("CreateWindowExW result: {:?}", hwnd);

        Ok(hwnd)
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {

        match msg {
            // First message received by any window:
            WM_NCCREATE => {
                eprintln!("WM_NCCREATE received");
                // Create the EditorView instance when the window is created
                let editor_view = Box::new(EditorView::new(hwnd));
                // Store a raw pointer to the EditorView in the window's extra data (at offset 0)
                SetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0), Box::into_raw(editor_view) as isize); // Use WINDOW_LONG_PTR_INDEX
                return LRESULT(1); 
            }
            // Last message received - clean up the EditorView instance
            WM_NCDESTROY => {
                // Retrieve the pointer from window extra storage
                let ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut EditorView;
                if !ptr.is_null() {
                    // Convert the raw pointer back to a Box to allow Rust to drop it
                    let _ = Box::from_raw(ptr);
                    // Clear the pointer from window storage to prevent double-free
                    SetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0), 0); // Use WINDOW_LONG_PTR_INDEX
                }
                return LRESULT(0);
            }
            WM_PAINT => {
                if let Some(editor_view) = EditorView::from_hwnd(hwnd) {
                    if let Err(_e) = editor_view.on_paint() {
                        // eprintln!("on_paint error: {:?}", e); // Keep commented for debugging
                    }
                }
                return LRESULT(0);
            }
            WM_SETFONT => {
                let hfont = HFONT(wparam.0 as _); // Cast usize directly to *mut c_void implicitly
                let redraw = lparam != LPARAM(0);

                if let Some(editor_view) = EditorView::from_hwnd(hwnd) {
                    if let Err(_e) = editor_view.on_set_font(hfont) {
                        // eprintln!("WM_SETFONT error: {:?}", e); // Keep commented for debugging
                    } else if redraw {
                        InvalidateRect(Some(hwnd), None, true);
                    }
                }
                return LRESULT(0);
            }
            EVM_OPENFILE => {
                let filename_pcwstr = PCWSTR(lparam.0 as *const u16); // lparam is PCWSTR
                let mut success = false;

                if let Some(editor_view) = EditorView::from_hwnd(hwnd) {
                    match editor_view.open_file(filename_pcwstr) {
                        Ok(_) => success = true,
                        Err(_e) => {
                            // eprintln!("EVM_OPENFILE error: {:?}", e); // Keep commented for debugging
                            // TODO: Show a message box to the user on error
                        }
                    }
                }
                // Return 1 for success, 0 for failure
                return LRESULT(if success { 1 } else { 0 });
            }
            EVM_CLEARFILE => {
                let mut success = false;
                if let Some(editor_view) = EditorView::from_hwnd(hwnd) {
                     match editor_view.clear_file() {
                        Ok(_) => success = true,
                        Err(_e) => {
                            // eprintln!("EVM_CLEARFILE error: {:?}", e); // Keep commented for debugging
                        }
                    }
                }
                 // Return 1 for success, 0 for failure
                return LRESULT(if success { 1 } else { 0 });
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
