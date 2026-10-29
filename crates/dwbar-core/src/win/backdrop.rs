//! Desktop blur behind DW Bar's own popups (panel, hover card).
//!
//! Uses `SetWindowCompositionAttribute(WCA_ACCENT_POLICY)` rather than the Windows 11
//! system backdrop: the system backdrop turns into a flat colour while a window is
//! inactive, and the hover card never takes focus. Only DW Bar's own windows are
//! touched (no other process).
use super::{h, wide, Hwnd};
use crate::config::Backdrop;
use std::sync::OnceLock;
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Dwm::DwmSetWindowAttribute,
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
};

#[repr(C)]
struct AccentPolicy {
    state: u32,
    flags: u32,
    /// Tint as 0xAABBGGRR.
    gradient: u32,
    animation: u32,
}

#[repr(C)]
struct CompositionData {
    attribute: u32,
    data: *mut core::ffi::c_void,
    size: usize,
}

const WCA_ACCENT_POLICY: u32 = 19;
const ACCENT_DISABLED: u32 = 0;
const ACCENT_ENABLE_BLURBEHIND: u32 = 3;
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_DEFAULT: u32 = 0;
const DWMWCP_ROUND: u32 = 2;

type SetWca = unsafe extern "system" fn(HWND, *mut CompositionData) -> i32;

fn set_wca() -> Option<SetWca> {
    static F: OnceLock<Option<usize>> = OnceLock::new();
    let addr = *F.get_or_init(|| unsafe {
        let name = wide("user32.dll");
        let mut module = GetModuleHandleW(name.as_ptr());
        if module.is_null() {
            module = LoadLibraryW(name.as_ptr());
        }
        if module.is_null() {
            return None;
        }
        GetProcAddress(module, c"SetWindowCompositionAttribute".as_ptr() as *const u8).map(|f| f as usize)
    });
    addr.map(|a| unsafe { std::mem::transmute::<usize, SetWca>(a) })
}

/// Blur (or stop blurring) what is behind `hwnd`. The page paints the tint and edges.
pub fn set(hwnd: Hwnd, backdrop: Backdrop) {
    let Some(f) = set_wca() else {
        crate::log!("backdrop", "SetWindowCompositionAttribute unavailable");
        return;
    };
    let state = match backdrop {
        Backdrop::None => ACCENT_DISABLED,
        Backdrop::Blur => ACCENT_ENABLE_BLURBEHIND,
        Backdrop::Acrylic => ACCENT_ENABLE_ACRYLICBLURBEHIND,
    };
    // Acrylic needs a non-zero tint alpha; keep it faint, the page does the colour.
    let mut policy = AccentPolicy { state, flags: 2, gradient: 0x10_80_80_80, animation: 0 };
    let mut data = CompositionData {
        attribute: WCA_ACCENT_POLICY,
        data: &mut policy as *mut AccentPolicy as *mut _,
        size: std::mem::size_of::<AccentPolicy>(),
    };
    let ok = unsafe { f(h(hwnd), &mut data) };
    // Windows 11: round the blurred rectangle to match the page (8 px).
    let corner: u32 = if backdrop == Backdrop::None { DWMWCP_DEFAULT } else { DWMWCP_ROUND };
    unsafe {
        DwmSetWindowAttribute(h(hwnd), DWMWA_WINDOW_CORNER_PREFERENCE, &corner as *const u32 as *const _, 4);
    }
    crate::log!("backdrop", "{backdrop:?} → {}", if ok != 0 { "ok" } else { "refused" });
}
