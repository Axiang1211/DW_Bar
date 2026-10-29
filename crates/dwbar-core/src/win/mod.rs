//! Win32 pieces. Window handles cross threads as `isize` ([`Hwnd`]) because raw
//! pointers are not `Send`; convert at the call site with [`h`].
//!
//! Thread rules (spec §6): the watcher thread only *detects*; every change to a
//! window is done by the app on its UI thread.
pub mod autostart;
pub mod backdrop;
pub mod embed;
pub mod fullscreen;
pub mod styler;
pub mod taskbar;
pub mod watcher;

use crate::layout::Rect;
use windows_sys::Win32::Foundation::{HWND, RECT};

pub type Hwnd = isize;

#[inline]
pub fn h(hwnd: Hwnd) -> HWND {
    hwnd as HWND
}

pub(crate) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn rect(r: RECT) -> Rect {
    Rect::new(r.left, r.top, r.right, r.bottom)
}

/// Screen rect of a window, if it exists.
pub fn window_rect(hwnd: Hwnd) -> Option<Rect> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsWindow};
    unsafe {
        if hwnd == 0 || IsWindow(h(hwnd)) == 0 {
            return None;
        }
        let mut r: RECT = std::mem::zeroed();
        (GetWindowRect(h(hwnd), &mut r) != 0).then(|| rect(r))
    }
}

/// Window class name (for telling the desktop and shell apart from apps).
pub fn class_name(hwnd: Hwnd) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(h(hwnd), buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}
