//! Attaching the capsule window to the taskbar. Call from the UI thread only.
//!
//! * Embed  (plan A): the capsule becomes a `WS_CHILD` of `Shell_TrayWnd`, so it moves,
//!   auto-hides and disappears with the taskbar for free. It must sit above the XAML
//!   island that draws the Windows 11 taskbar, so z-order is re-asserted on every tick.
//! * Overlay (fallback): a top-most, non-activating tool window placed over the taskbar.
use super::{h, Hwnd};
use crate::layout::Rect;
use windows_sys::Win32::{
    Foundation::POINT,
    Graphics::Gdi::ScreenToClient,
    UI::WindowsAndMessaging::{
        GetAncestor, GetWindow, GetWindowLongPtrW, SetParent, SetWindowLongPtrW, SetWindowPos,
        ShowWindow, GA_PARENT, GWL_EXSTYLE, GWL_STYLE, GW_HWNDPREV, HWND_TOP, HWND_TOPMOST,
        SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE,
        SW_SHOWNOACTIVATE, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_POPUP,
    },
};

/// Make the window non-activating and keep it out of Alt-Tab and the taskbar.
pub fn prepare(win: Hwnd) {
    unsafe {
        let ex = GetWindowLongPtrW(h(win), GWL_EXSTYLE);
        let ex = (ex | (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize) & !(WS_EX_APPWINDOW as isize);
        SetWindowLongPtrW(h(win), GWL_EXSTYLE, ex);
    }
}

/// Whether `win` is currently a child of `parent`.
pub fn is_child_of(win: Hwnd, parent: Hwnd) -> bool {
    unsafe { GetAncestor(h(win), GA_PARENT) as Hwnd == parent }
}

/// Re-parent into the taskbar. Returns false if Windows refused.
pub fn embed(win: Hwnd, taskbar: Hwnd) -> bool {
    if is_child_of(win, taskbar) {
        return true;
    }
    unsafe {
        let style = GetWindowLongPtrW(h(win), GWL_STYLE);
        let style = (style & !(WS_POPUP as isize)) | (WS_CHILD | WS_CLIPSIBLINGS) as isize;
        SetWindowLongPtrW(h(win), GWL_STYLE, style);
        SetParent(h(win), h(taskbar));
        SetWindowPos(
            h(win),
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
    let ok = is_child_of(win, taskbar);
    crate::log!("embed", "SetParent → Shell_TrayWnd: {}", if ok { "ok" } else { "refused" });
    ok
}

/// Back to a normal top-level popup (for overlay mode, or before Explorer goes away).
pub fn unembed(win: Hwnd) {
    unsafe {
        let style = GetWindowLongPtrW(h(win), GWL_STYLE);
        let style = (style & !(WS_CHILD as isize)) | WS_POPUP as isize;
        SetWindowLongPtrW(h(win), GWL_STYLE, style);
        SetParent(h(win), std::ptr::null_mut());
        SetWindowPos(
            h(win),
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// Place the window at `screen` (screen coordinates, physical px).
pub fn place(win: Hwnd, screen: Rect, parent: Option<Hwnd>) {
    unsafe {
        let (x, y, after) = match parent {
            Some(p) => {
                let mut pt = POINT { x: screen.left, y: screen.top };
                ScreenToClient(h(p), &mut pt);
                (pt.x, pt.y, HWND_TOP)
            }
            None => (screen.left, screen.top, HWND_TOPMOST),
        };
        SetWindowPos(h(win), after, x, y, screen.width(), screen.height(), SWP_NOACTIVATE);
    }
}

/// Keep the embedded capsule above the taskbar's own content. Cheap when already on top.
pub fn ensure_top(win: Hwnd, parent: Option<Hwnd>) {
    unsafe {
        let above = GetWindow(h(win), GW_HWNDPREV);
        if parent.is_some() && above.is_null() {
            return;
        }
        let after = if parent.is_some() { HWND_TOP } else { HWND_TOPMOST };
        SetWindowPos(h(win), after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

pub fn set_visible(win: Hwnd, visible: bool) {
    unsafe {
        ShowWindow(h(win), if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
    }
}
