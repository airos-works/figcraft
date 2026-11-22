//! Render a widget into an owned RGBA buffer using the installed Interface Kit.
#![cfg(target_os = "haiku")]
#![deny(unsafe_code)]
use std::ffi::{CString, c_char};
use std::sync::Mutex;

#[allow(unsafe_code)]
unsafe extern "C" {
    fn fc_draw_widget(
        kind: *const c_char,
        label: *const c_char,
        value: *const c_char,
        enabled: bool,
        checked: bool,
        w: u32,
        h: u32,
        pixels: *mut u8,
    ) -> bool;
}

/// The caller supplies logical pixel dimensions. Limit both app_server and client allocations.
pub fn render(kind: &str, label: &str, value: &str, enabled: bool, checked: bool, w: u32, h: u32) -> Option<Vec<u8>> {
    if w == 0 || h == 0 || w > 4096 || h > 4096 || w * h > 1_048_576 || label.len() > 65536 || value.len() > 65536 {
        return None;
    }
    let kind = CString::new(kind).ok()?;
    let label = CString::new(label).ok()?;
    let value = CString::new(value).ok()?;
    let mut pixels = vec![0u8; w as usize * h as usize * 4];
    static NATIVE: Mutex<()> = Mutex::new(());
    let _guard = NATIVE.lock().ok()?;
    // SAFETY: the native call is synchronous, reads three NUL-terminated strings and writes
    // exactly w*h RGBA pixels. All buffers outlive the call. Kit initialization is serialized.
    #[allow(unsafe_code)]
    let drawn = unsafe { fc_draw_widget(kind.as_ptr(), label.as_ptr(), value.as_ptr(), enabled, checked, w, h, pixels.as_mut_ptr()) };
    drawn.then_some(pixels)
}

/// A widget at its actual size in a design page. Native views keep the current display's
/// font rasterization, device scale, accessibility and controls during an operator preview.
pub struct Widget {
    pub kind: String,
    pub label: String,
    pub value: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub enabled: bool,
    pub checked: bool,
}

#[repr(C)]
struct PreviewWidget {
    kind: *const c_char,
    label: *const c_char,
    value: *const c_char,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    enabled: bool,
    checked: bool,
}

#[allow(unsafe_code)]
unsafe extern "C" {
    fn fc_present_design(title: *const c_char, w: u32, h: u32, rgba: *const u8, widgets: *const PreviewWidget, count: usize) -> i32;
}

pub fn present(title: &str, w: u32, h: u32, pixels: &[u8], widgets: Vec<Widget>) -> Option<usize> {
    if w == 0 || h == 0 || w > 4096 || h > 4096 || w * h > 4_194_304 || pixels.len() != w as usize * h as usize * 4 || widgets.len() > 512 {
        return None;
    }
    let title = CString::new(title).ok()?;
    let mut storage = Vec::with_capacity(widgets.len());
    let mut native = Vec::with_capacity(widgets.len());
    for item in widgets {
        if item.width == 0 || item.height == 0 || item.width > 4096 || item.height > 4096 {
            return None;
        }
        let strings = [CString::new(item.kind).ok()?, CString::new(item.label).ok()?, CString::new(item.value).ok()?];
        native.push(PreviewWidget {
            kind: strings[0].as_ptr(),
            label: strings[1].as_ptr(),
            value: strings[2].as_ptr(),
            x: item.x,
            y: item.y,
            width: item.width,
            height: item.height,
            enabled: item.enabled,
            checked: item.checked,
        });
        storage.push(strings);
    }
    static PREVIEW: Mutex<()> = Mutex::new(());
    let _guard = PREVIEW.lock().ok()?;
    // SAFETY: calls updating the shared preview window are serialized here.
    // The native window copies the pixels and strings before returning. Each repr(C)
    // record points into an owned CString in storage, and all buffer lengths were checked above.
    #[allow(unsafe_code)]
    let count = unsafe { fc_present_design(title.as_ptr(), w, h, pixels.as_ptr(), native.as_ptr(), native.len()) };
    usize::try_from(count).ok()
}
