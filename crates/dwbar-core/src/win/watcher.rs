//! Background thread that watches the shell and reports what changed.
//!
//! * `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT)` – foreground changes,
//!   debounced 300 ms before the fullscreen test runs.
//! * `RegisterWindowMessage("TaskbarCreated")` – Explorer restarted; re-embed.
//! * `WM_SETTINGCHANGE` – taskbar alignment / widgets toggled.
//! * A 2 s tick re-checks geometry and z-order (cheap), and a one-shot 2.5 s timer
//!   re-applies placement after boot, when Explorer may still be settling.
//!
//! The thread never touches the capsule window; it only calls the callback.
use super::{fullscreen, wide};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    OnceLock,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, KillTimer,
            RegisterClassW, RegisterWindowMessageW, SetTimer, TranslateMessage,
            EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, MSG, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS,
            WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Fullscreen state changed (already debounced).
    Fullscreen(bool),
    /// Explorer (re)created the taskbar.
    TaskbarCreated,
    /// Taskbar settings changed (alignment, widgets, size).
    SettingsChanged,
    /// Periodic check: placement / z-order.
    Tick,
    /// One-shot re-apply shortly after start.
    Reapply,
}

type Callback = Box<dyn Fn(Event) + Send + Sync>;
static CALLBACK: OnceLock<Callback> = OnceLock::new();
static FULLSCREEN: AtomicBool = AtomicBool::new(false);
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
static mut WINDOW: HWND = std::ptr::null_mut();

const TIMER_TICK: usize = 1;
const TIMER_DEBOUNCE: usize = 2;
const TIMER_REAPPLY: usize = 3;
const TIMER_STYLE: usize = 4;

fn emit(event: Event) {
    if let Some(cb) = CALLBACK.get() {
        cb(event);
    }
}

fn check_fullscreen() {
    let reason = fullscreen::detect();
    let now = reason != fullscreen::Reason::None;
    if FULLSCREEN.swap(now, Ordering::SeqCst) != now {
        crate::log!("fullscreen", "{} ({:?})", if now { "hide" } else { "show" }, reason);
        emit(Event::Fullscreen(now));
    }
}

unsafe extern "system" fn on_win_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // Restart the 300 ms debounce; the test runs when things settle.
    let win = unsafe { WINDOW };
    if !win.is_null() {
        unsafe { SetTimer(win, TIMER_DEBOUNCE, 300, None) };
    }
}

/// A window was shown somewhere: let the window styler look at it.
unsafe extern "system" fn on_show(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if id_object == OBJID_WINDOW && id_child == 0 && !hwnd.is_null() {
        super::styler::on_shown(hwnd as isize);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let created = TASKBAR_CREATED.load(Ordering::Relaxed);
    if created != 0 && msg == created {
        crate::log!("watcher", "TaskbarCreated (Explorer restarted)");
        emit(Event::TaskbarCreated);
        return 0;
    }
    match msg {
        WM_TIMER => {
            match wparam {
                TIMER_TICK => {
                    check_fullscreen();
                    emit(Event::Tick);
                }
                TIMER_DEBOUNCE => {
                    unsafe { KillTimer(hwnd, TIMER_DEBOUNCE) };
                    check_fullscreen();
                }
                TIMER_REAPPLY => {
                    unsafe { KillTimer(hwnd, TIMER_REAPPLY) };
                    emit(Event::Reapply);
                }
                TIMER_STYLE if super::styler::pending() => super::styler::tick(),
                _ => {}
            }
            0
        }
        WM_SETTINGCHANGE => {
            emit(Event::SettingsChanged);
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Start the watcher thread. Call once; later calls are ignored.
pub fn spawn(callback: impl Fn(Event) + Send + Sync + 'static) {
    if CALLBACK.set(Box::new(callback)).is_err() {
        return;
    }
    std::thread::Builder::new()
        .name("dwbar-watcher".into())
        .spawn(|| unsafe { run() })
        .ok();
}

unsafe fn run() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = wide("DWBarWatcher");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..std::mem::zeroed()
        };
        RegisterClassW(&wc);
        // A hidden *top-level* window: message-only windows never get the
        // TaskbarCreated / WM_SETTINGCHANGE broadcasts.
        let title = wide("DW Bar watcher");
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            crate::log!("watcher", "CreateWindowExW failed; watcher disabled");
            return;
        }
        WINDOW = hwnd;
        let name = wide("TaskbarCreated");
        TASKBAR_CREATED.store(RegisterWindowMessageW(name.as_ptr()), Ordering::Relaxed);

        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            std::ptr::null_mut(),
            Some(on_win_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook.is_null() {
            crate::log!("watcher", "SetWinEventHook failed; relying on the 2 s tick");
        }
        SetWinEventHook(
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_SHOW,
            std::ptr::null_mut(),
            Some(on_show),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        SetTimer(hwnd, TIMER_TICK, 2000, None);
        SetTimer(hwnd, TIMER_STYLE, 300, None);
        SetTimer(hwnd, TIMER_REAPPLY, 2500, None);
        crate::log!("watcher", "started");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Last known fullscreen state.
pub fn fullscreen_now() -> bool {
    FULLSCREEN.load(Ordering::SeqCst)
}
