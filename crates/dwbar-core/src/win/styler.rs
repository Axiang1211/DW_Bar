//! 視窗美化 — apply DWM effects to other programs' top-level windows.
//!
//! Only public DWM window attributes are set, from DW Bar's own process
//! (`DwmSetWindowAttribute`), so nothing is injected into other programs and
//! Explorer cannot be crashed by it. Windows that appear later are styled as they
//! show (EVENT_OBJECT_SHOW from the watcher thread) and once more shortly after,
//! because some programs set their own attributes during start-up.
use super::{class_name, h, Hwnd};
use crate::{
    config::StyleConfig,
    style::{self, Effects, WindowFacts},
};
use std::{
    collections::HashSet,
    sync::{Mutex, RwLock},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM},
    Graphics::Dwm::{
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR,
        DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE,
    },
    System::Threading::{GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
    UI::{
        Controls::MARGINS,
        WindowsAndMessaging::{
            EnumWindows, GetAncestor, GetWindowLongPtrW, GetWindowThreadProcessId, IsWindowVisible, GA_ROOT,
            GWL_EXSTYLE, GWL_STYLE, WS_CAPTION, WS_EX_TOOLWINDOW,
        },
    },
};

static CONFIG: RwLock<Option<StyleConfig>> = RwLock::new(None);
/// Windows DW Bar has changed, so they can be put back.
static TOUCHED: Mutex<Option<HashSet<Hwnd>>> = Mutex::new(None);
/// Windows to style again a moment after they appeared.
static LATER: Mutex<Vec<(Hwnd, Instant)>> = Mutex::new(Vec::new());

fn exe_name(pid: u32) -> String {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return String::new();
        }
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) != 0;
        CloseHandle(process);
        if !ok {
            return String::new();
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/']).next().unwrap_or_default().to_lowercase()
    }
}

fn facts(hwnd: Hwnd) -> WindowFacts {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(h(hwnd), &mut pid);
        let style = GetWindowLongPtrW(h(hwnd), GWL_STYLE) as u32;
        let ex = GetWindowLongPtrW(h(hwnd), GWL_EXSTYLE) as u32;
        WindowFacts {
            class: class_name(hwnd),
            exe: exe_name(pid),
            has_caption: style & WS_CAPTION == WS_CAPTION,
            tool_window: ex & WS_EX_TOOLWINDOW != 0,
            visible: IsWindowVisible(h(hwnd)) != 0 && GetAncestor(h(hwnd), GA_ROOT) as Hwnd == hwnd,
            own_process: pid == GetCurrentProcessId(),
        }
    }
}

fn set_u32(hwnd: Hwnd, attr: i32, value: u32) {
    unsafe {
        DwmSetWindowAttribute(h(hwnd), attr as u32, &value as *const u32 as *const _, 4);
    }
}

fn apply(hwnd: Hwnd, e: &Effects) {
    if let Some(dark) = e.dark {
        set_u32(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, dark as u32);
    }
    set_u32(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, e.corners);
    set_u32(hwnd, DWMWA_BORDER_COLOR, e.border);
    set_u32(hwnd, DWMWA_CAPTION_COLOR, e.caption);
    set_u32(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, e.backdrop);
    let m = if e.extend_frame { -1 } else { 0 };
    let margins = MARGINS { cxLeftWidth: m, cxRightWidth: m, cyTopHeight: m, cyBottomHeight: m };
    unsafe {
        DwmExtendFrameIntoClientArea(h(hwnd), &margins);
    }
}

/// The Windows accent colour (Settings → Personalization → Colors) as COLORREF.
fn accent() -> Option<u32> {
    super::taskbar::reg_dword_pub(r"Software\Microsoft\Windows\DWM", "AccentColor").map(|abgr| abgr & 0x00FF_FFFF)
}

fn current() -> Option<StyleConfig> {
    CONFIG.read().ok()?.clone()
}

/// Style one window if the rules say so; returns whether it was styled.
fn consider(hwnd: Hwnd, cfg: &StyleConfig, effects: &Effects) -> bool {
    if !style::qualifies(&facts(hwnd), cfg) {
        return false;
    }
    apply(hwnd, effects);
    if let Ok(mut t) = TOUCHED.lock() {
        t.get_or_insert_with(HashSet::new).insert(hwnd);
    }
    true
}

unsafe extern "system" fn enum_proc(hwnd: HWND, _: LPARAM) -> windows_sys::core::BOOL {
    if let Some(cfg) = current() {
        let e = style::effects(&cfg, accent());
        consider(hwnd as Hwnd, &cfg, &e);
    }
    1
}

/// New settings: undo windows that no longer qualify, then style every open window.
/// Safe to call from any thread.
pub fn set_config(cfg: StyleConfig) {
    let previous = TOUCHED.lock().ok().and_then(|mut t| t.take()).unwrap_or_default();
    if let Ok(mut c) = CONFIG.write() {
        *c = Some(cfg.clone());
    }
    for hwnd in previous {
        if super::window_rect(hwnd).is_some() {
            apply(hwnd, &style::RESET);
        }
    }
    if cfg.enabled {
        unsafe {
            EnumWindows(Some(enum_proc), 0);
        }
        let n = TOUCHED.lock().ok().and_then(|t| t.as_ref().map(|s| s.len())).unwrap_or(0);
        crate::log!("styler", "on: {:?}, {} window(s) styled", cfg.target, n);
    } else {
        crate::log!("styler", "off");
    }
}

/// A window just appeared (watcher thread).
pub fn on_shown(hwnd: Hwnd) {
    let Some(cfg) = current().filter(|c| c.enabled) else { return };
    let e = style::effects(&cfg, accent());
    if consider(hwnd, &cfg, &e) {
        if let Ok(mut l) = LATER.lock() {
            l.push((hwnd, Instant::now() + Duration::from_millis(700)));
        }
    }
}

/// Re-apply to windows that appeared a moment ago (called from the watcher's timer).
pub fn tick() {
    let due: Vec<Hwnd> = {
        let Ok(mut l) = LATER.lock() else { return };
        let now = Instant::now();
        let (due, wait): (Vec<_>, Vec<_>) = l.drain(..).partition(|(_, t)| *t <= now);
        *l = wait;
        due.into_iter().map(|(h, _)| h).collect()
    };
    if due.is_empty() {
        return;
    }
    let Some(cfg) = current().filter(|c| c.enabled) else { return };
    let e = style::effects(&cfg, accent());
    for hwnd in due {
        consider(hwnd, &cfg, &e);
    }
}

pub fn pending() -> bool {
    LATER.lock().map(|l| !l.is_empty()).unwrap_or(false)
}

/// Put every touched window back (on quit).
pub fn restore_all() {
    let previous = TOUCHED.lock().ok().and_then(|mut t| t.take()).unwrap_or_default();
    for hwnd in previous {
        if super::window_rect(hwnd).is_some() {
            apply(hwnd, &style::RESET);
        }
    }
}
