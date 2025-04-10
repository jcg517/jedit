use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, TextOutW, CreateFontW, HFONT, FW_NORMAL, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, DEFAULT_PITCH, FF_DONTCARE, HDC};
use windows::core::PCWSTR;

use crate::document::text_document::TextDocument;

pub struct EditorView {
    document: TextDocument,
    caret_pos: usize,
    font_height: i32,
    hfont: HFONT,
}

impl EditorView {
    /// Constructs a new EditorView with the given initial text.
    pub fn new(document: TextDocument) -> Self {
        use std::iter;

        let font_height = 36;
        let font_name: Vec<u16> = "Consolas".encode_utf16().chain(iter::once(0)).collect();

        // Create a font with the specified height (font_height), normal weight, and default settings.
        let hfont = unsafe { CreateFontW(
            font_height,                           // height
            0,                                     // width
            0,                                     // escapement
            0,                                     // orientation
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as i32, // pitch and family (argument 5)
            0,                                     // italic
            0,                                     // underline
            0,                                     // strikeout
            DEFAULT_CHARSET,                       // charset
            OUT_DEFAULT_PRECIS,                    // output precision
            CLIP_DEFAULT_PRECIS,                   // clip precision
            DEFAULT_QUALITY,                       // quality
            FW_NORMAL.0 as u32,                    // weight (swapped to argument 13)
            PCWSTR(font_name.as_ptr())
        ) };

        Self {
            document, 
            caret_pos: 0,
            font_height,
            hfont,
        }
    }
    
    /// WM_PAINT handler for the text view.
    /// This method begins painting, draws the text, and ends painting.
    pub fn on_paint(&self, hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
        let mut ps = PAINTSTRUCT::default();
        unsafe {
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc.0.is_null() {
                return Err("BeginPaint failed".into());
            }

            // Select the custom font into the DC and save the old font
            let old_font = SelectObject(hdc, self.hfont.into());

            // Calculate the first and last line based on the paint area and font height
            let num_lines = self.document.line_count();
            let first_line = ps.rcPaint.top / self.font_height;
            let last_line = std::cmp::min(ps.rcPaint.bottom / self.font_height, num_lines as i32 - 1);
            for line in first_line..=last_line {
                self.paint_line(hdc, line)?;
            }

            // Restore the old font
            SelectObject(hdc, old_font);
            EndPaint(hwnd, &ps);
        }
        Ok(())
    }

    fn paint_line(&self, hdc: HDC, line: i32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(line_text) = self.document.getline(line.try_into().unwrap()) {
            // Convert the Rust string to a null-terminated UTF-16 string
            let text_wide: Vec<u16> = line_text.encode_utf16().chain(std::iter::once(0)).collect();
            // Calculate the Y position based on the line number and font height (with an offset, e.g., 10)
            let y = 10 + line * self.font_height;
            // Draw the text at position (10, y)
            unsafe {
                if TextOutW(hdc, 10, y, &text_wide).as_bool() == false {
                    return Err("TextOutW failed".into());
                }
            }
        }
        Ok(())
    }
    
   // Todo: Additional methods handling scrolling, keyboard, etc.. 
}
