//! Finding the taskbar and reading how the user set it up.
use super::{h, window_rect, wide, Hwnd};
use crate::layout::TaskbarInfo;
use windows_sys::Win32::{
    System::Registry::{RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ},
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{FindWindowExW, FindWindowW},
    },
};

const ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";

/// The primary taskbar (`Shell_TrayWnd`).
pub fn primary() -> Option<Hwnd> {
    let class = wide("Shell_TrayWnd");
    let hwnd = unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) } as Hwnd;
    (hwnd != 0).then_some(hwnd)
}

/// Taskbars on other monitors (`Shell_SecondaryTrayWnd`), for later multi-monitor support.
pub fn secondary() -> Vec<Hwnd> {
    let class = wide("Shell_SecondaryTrayWnd");
    let mut out = Vec::new();
    let mut after: Hwnd = 0;
    loop {
        let next = unsafe {
            FindWindowExW(std::ptr::null_mut(), h(after), class.as_ptr(), std::ptr::null())
        } as Hwnd;
        if next == 0 || out.len() > 16 {
            break;
        }
        out.push(next);
        after = next;
    }
    out
}

fn child(parent: Hwnd, class: &str) -> Option<Hwnd> {
    let class = wide(class);
    let hwnd = unsafe {
        FindWindowExW(h(parent), std::ptr::null_mut(), class.as_ptr(), std::ptr::null())
    } as Hwnd;
    (hwnd != 0).then_some(hwnd)
}

fn reg_dword(root: HKEY, key: &str, value: &str) -> Option<u32> {
    let (key, value) = (wide(key), wide(value));
    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    let rc = unsafe {
        RegGetValueW(
            root,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut _,
            &mut size,
        )
    };
    (rc == 0).then_some(data)
}

fn reg_string(root: HKEY, key: &str, value: &str) -> Option<String> {
    let (key, value) = (wide(key), wide(value));
    let mut buf = [0u16; 128];
    let mut size = (buf.len() * 2) as u32;
    let rc = unsafe {
        RegGetValueW(
            root,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
            &mut size,
        )
    };
    if rc != 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}

/// Windows build number (22000+ is Windows 11).
pub fn build() -> u32 {
    reg_string(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "CurrentBuildNumber")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

pub fn is_windows_11() -> bool {
    build() >= 22000
}

/// `TaskbarAl`: 1 = centred, 0 = left. Missing means the OS default (centred on Windows 11).
pub fn centered() -> bool {
    match reg_dword(HKEY_CURRENT_USER, ADVANCED, "TaskbarAl") {
        Some(v) => v == 1,
        None => is_windows_11(),
    }
}

/// `TaskbarDa`: the widgets button. Missing means shown (Windows 11 default).
pub fn widgets() -> bool {
    match reg_dword(HKEY_CURRENT_USER, ADVANCED, "TaskbarDa") {
        Some(v) => v != 0,
        None => is_windows_11(),
    }
}

/// Show or hide the Windows widgets button (the weather at the left of the taskbar),
/// the same switch as Settings → Personalization → Taskbar → Widgets.
///
/// Some Windows 11 builds protect this value from other apps; then this returns
/// `Err` and the caller should send the user to that Settings page instead.
pub fn set_widgets(show: bool) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::ERROR_ACCESS_DENIED,
        System::Registry::{RegSetKeyValueW, REG_DWORD},
        UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE},
    };
    let (key, name) = (wide(ADVANCED), wide("TaskbarDa"));
    let value: u32 = if show { 1 } else { 0 };
    let rc = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            REG_DWORD,
            &value as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if rc != 0 {
        crate::log!("taskbar", "TaskbarDa write refused ({rc})");
        return Err(if rc == ERROR_ACCESS_DENIED { "protected".into() } else { format!("error {rc}") });
    }
    // Tell Explorer to re-read its taskbar settings (never block on a hung window).
    let area = wide("TraySettings");
    let mut result = 0usize;
    unsafe {
        SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, area.as_ptr() as isize, SMTO_ABORTIFHUNG, 200, &mut result);
    }
    // Some builds only notice after a moment; confirm by reading it back.
    std::thread::sleep(std::time::Duration::from_millis(300));
    if widgets() == show {
        crate::log!("taskbar", "widgets {}", if show { "shown" } else { "hidden" });
        Ok(())
    } else {
        Err("not applied".into())
    }
}

/// Everything layout needs about the primary taskbar, or `None` while Explorer is down.
pub fn info() -> Option<(Hwnd, TaskbarInfo)> {
    let tb = primary()?;
    let rect = window_rect(tb)?;
    if rect.is_empty() {
        return None;
    }
    let tray = child(tb, "TrayNotifyWnd").and_then(window_rect).filter(|r| !r.is_empty());
    let dpi = unsafe { GetDpiForWindow(h(tb)) };
    let scale = if dpi == 0 { 1.0 } else { dpi as f64 / 96.0 };
    Some((tb, TaskbarInfo { rect, tray, centered: centered(), widgets: widgets(), scale }))
}
