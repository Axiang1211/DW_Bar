//! "Is something fullscreen in front right now?" (spec §3).
//!
//! 1. `SHQueryUserNotificationState` – the shell's own answer for D3D games,
//!    presentations and busy fullscreen apps.
//! 2. Otherwise, the foreground window covers its whole monitor and is not the
//!    desktop, the shell or one of our own windows.
use super::{class_name, h, window_rect, Hwnd};
use crate::layout::Rect;
use windows_sys::Win32::{
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    System::Threading::GetCurrentProcessId,
    UI::{
        Shell::{SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN},
        WindowsAndMessaging::{GetForegroundWindow, GetShellWindow, GetWindowThreadProcessId},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    None,
    D3D,
    Presentation,
    Busy,
    CoversMonitor,
}

pub fn shell_state() -> Reason {
    let mut state = 0;
    let hr = unsafe { SHQueryUserNotificationState(&mut state) };
    if hr < 0 {
        return Reason::None;
    }
    match state {
        QUNS_RUNNING_D3D_FULL_SCREEN => Reason::D3D,
        QUNS_PRESENTATION_MODE => Reason::Presentation,
        QUNS_BUSY => Reason::Busy,
        _ => Reason::None,
    }
}

fn monitor_rect(hwnd: Hwnd) -> Option<Rect> {
    unsafe {
        let mon = MonitorFromWindow(h(hwnd), MONITOR_DEFAULTTONEAREST);
        if mon.is_null() {
            return None;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        (GetMonitorInfoW(mon, &mut info) != 0).then(|| super::rect(info.rcMonitor))
    }
}

const SHELL_CLASSES: [&str; 5] = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "Windows.UI.Core.CoreWindow"];

pub fn foreground_covers_monitor() -> bool {
    unsafe {
        let fg = GetForegroundWindow() as Hwnd;
        if fg == 0 || fg == GetShellWindow() as Hwnd {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(h(fg), &mut pid);
        if pid == GetCurrentProcessId() {
            return false;
        }
        if SHELL_CLASSES.contains(&class_name(fg).as_str()) {
            return false;
        }
        match (window_rect(fg), monitor_rect(fg)) {
            (Some(w), Some(m)) => w.covers(&m),
            _ => false,
        }
    }
}

pub fn detect() -> Reason {
    match shell_state() {
        Reason::None if foreground_covers_monitor() => Reason::CoversMonitor,
        r => r,
    }
}
