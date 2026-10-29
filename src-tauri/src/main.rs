// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod platform;
mod state;

use dwbar_core::{config::Config, log};
use state::{AppState, Snapshot};
use std::{sync::mpsc, time::Duration};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

pub const CAPSULE: &str = "capsule";
pub const SETTINGS: &str = "settings";
pub const TIP: &str = "tip";

// ───────────────────────── windows ─────────────────────────

/// On Windows, building a window on the UI thread from inside a command or an event
/// handler deadlocks the event loop (the window appears but stays blank). Every window
/// is therefore built on a short-lived worker thread.
pub fn off_ui_thread(f: impl FnOnce() + Send + 'static) {
    std::thread::spawn(f);
}

static CREATING_CAPSULE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn create_capsule(app: &AppHandle) {
    use std::sync::atomic::Ordering;
    if app.get_webview_window(CAPSULE).is_some() || CREATING_CAPSULE.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    off_ui_thread(move || {
        build_capsule(&app);
        CREATING_CAPSULE.store(false, Ordering::SeqCst);
    });
}

fn build_capsule(app: &AppHandle) {
    let width = app.state::<AppState>().inner.lock().unwrap().capsule_width;
    let built = WebviewWindowBuilder::new(app, CAPSULE, WebviewUrl::App("index.html".into()))
        .title("DW Bar")
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
        .inner_size(width, 40.0)
        .build();
    match built {
        Ok(win) => {
            let app2 = app.clone();
            let _ = app.run_on_main_thread(move || {
                platform::prepare(&win);
                platform::apply(&app2);
            });
            log::write("app", "capsule window created");
        }
        Err(e) => log::write("app", &format!("capsule window failed: {e}")),
    }
}

/// The hover usage card: one hidden, click-through, top-most window reused for every ring.
fn create_tip(app: &AppHandle) {
    let app = app.clone();
    off_ui_thread(move || {
        let built = WebviewWindowBuilder::new(&app, TIP, WebviewUrl::App("tip.html".into()))
            .title("DW Bar")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false)
            .visible(false)
            .always_on_top(true)
            .inner_size(272.0, 180.0)
            .build();
        match built {
            Ok(win) => {
                let _ = win.set_ignore_cursor_events(true);
                let _ = app.run_on_main_thread(move || platform::prepare(&win));
            }
            Err(e) => log::write("app", &format!("tip window failed: {e}")),
        }
    });
}

fn open_settings(app: &AppHandle, page: Option<&str>) {
    if let Some(win) = app.get_webview_window(SETTINGS) {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
        if let Some(page) = page {
            let _ = win.emit("goto", page);
        }
        return;
    }
    let url = match page {
        Some(p) => format!("settings.html#{p}"),
        None => "settings.html".into(),
    };
    let app = app.clone();
    off_ui_thread(move || {
        let built = WebviewWindowBuilder::new(&app, SETTINGS, WebviewUrl::App(url.into()))
            .title("DW Bar 設定")
            .inner_size(820.0, 580.0)
            .min_inner_size(640.0, 460.0)
            .center()
            .build();
        if let Err(e) = built {
            log::write("app", &format!("settings window failed: {e}"));
        }
    });
}

pub fn broadcast(app: &AppHandle) {
    let snap = app.state::<AppState>().inner.lock().unwrap().snapshot();
    let _ = app.emit("state", &snap);
}

// ───────────────────────── quota thread ─────────────────────────

fn start_quota_thread(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<()>();
    *app.state::<AppState>().refresh_now.lock().unwrap() = Some(tx);
    let app = app.clone();
    std::thread::Builder::new()
        .name("dwbar-quota".into())
        .spawn(move || loop {
            let config = {
                let state = app.state::<AppState>();
                let mut s = state.inner.lock().unwrap();
                s.refreshing = true;
                s.config.clone()
            };
            broadcast(&app);
            let readings = accounts::read_all(&app);
            {
                let state = app.state::<AppState>();
                let mut s = state.inner.lock().unwrap();
                s.quotas = readings;
                s.refreshing = false;
            }
            broadcast(&app);
            let wait = Duration::from_secs(config.refresh_minutes as u64 * 60);
            match rx.recv_timeout(wait) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {
                    while rx.try_recv().is_ok() {} // collapse repeated clicks
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        })
        .ok();
}

pub fn request_refresh(app: &AppHandle) {
    if let Some(tx) = app.state::<AppState>().refresh_now.lock().unwrap().as_ref() {
        let _ = tx.send(());
    }
}

// ───────────────────────── shell watcher ─────────────────────────

#[cfg(any(windows, feature = "lint-win"))]
fn start_watcher(app: &AppHandle) {
    use dwbar_core::win::watcher::{self, Event};
    let app = app.clone();
    watcher::spawn(move |event| {
        let app2 = app.clone();
        // Detection happens on the watcher thread; window work on the UI thread.
        let _ = app.run_on_main_thread(move || match event {
            Event::Fullscreen(on) => {
                app2.state::<AppState>().inner.lock().unwrap().fullscreen = on;
                platform::apply(&app2);
            }
            Event::TaskbarCreated => {
                if app2.get_webview_window(CAPSULE).is_none() {
                    create_capsule(&app2);
                } else {
                    platform::apply(&app2);
                }
            }
            Event::SettingsChanged | Event::Reapply => platform::apply(&app2),
            Event::Tick => {
                if app2.get_webview_window(CAPSULE).is_none() {
                    create_capsule(&app2);
                } else {
                    platform::tick(&app2);
                }
            }
        });
    });
}

#[cfg(not(any(windows, feature = "lint-win")))]
fn start_watcher(_app: &AppHandle) {}

// ───────────────────────── tray icon ─────────────────────────

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "設定…", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "立即更新用量", true, None::<&str>)?;
    let replace = MenuItem::with_id(app, "replace", "重新定位膠囊", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "結束 DW Bar", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&settings, &refresh, &replace, &sep, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("DW Bar")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => open_settings(app, None),
            "refresh" => request_refresh(app),
            "replace" => platform::apply(app),
            "quit" => quit_app(app.clone()),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                open_settings(tray.app_handle(), None);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

// ───────────────────────── commands ─────────────────────────

#[tauri::command]
fn get_state(app: AppHandle) -> Snapshot {
    app.state::<AppState>().inner.lock().unwrap().snapshot()
}

#[tauri::command]
fn save_config(app: AppHandle, mut config: Config) -> Result<Snapshot, String> {
    config.clamp();
    let (agents_changed, autostart) = {
        let state = app.state::<AppState>();
        let mut s = state.inner.lock().unwrap();
        // Linking is owned by the sign-in flow; a settings page with an older copy must not undo it.
        for agent in dwbar_core::config::AGENTS {
            config.agent_mut(agent).connected = s.config.agent(agent).connected;
        }
        let changed = dwbar_core::config::AGENTS.iter().any(|a| s.config.agent(a) != config.agent(a))
            || s.config.refresh_minutes != config.refresh_minutes;
        let autostart = (s.config.autostart != config.autostart).then_some(config.autostart);
        s.config = config;
        (changed, autostart)
    };
    let to_save = app.state::<AppState>().inner.lock().unwrap().config.clone();
    to_save.save().map_err(|e| format!("無法儲存設定：{e}"))?;
    if let Some(on) = autostart {
        let launcher = app.autolaunch();
        let result = if on { launcher.enable() } else { launcher.disable() };
        if let Err(e) = result {
            log::write("app", &format!("autostart: {e}"));
        }
    }
    platform::apply(&app);
    if agents_changed {
        request_refresh(&app);
    }
    broadcast(&app);
    log::write("app", "settings saved");
    Ok(app.state::<AppState>().inner.lock().unwrap().snapshot())
}

#[tauri::command]
async fn refresh_quotas(app: AppHandle) {
    request_refresh(&app);
}

#[tauri::command]
fn capsule_resize(app: AppHandle, width: f64, height: Option<f64>) {
    let width = width.clamp(32.0, 900.0);
    let height = height.unwrap_or(40.0).clamp(32.0, 900.0);
    {
        let state = app.state::<AppState>();
        let mut s = state.inner.lock().unwrap();
        if (s.capsule_width - width).abs() < 0.5 && (s.capsule_height - height).abs() < 0.5 {
            return;
        }
        s.capsule_width = width;
        s.capsule_height = height;
    }
    platform::apply(&app);
}

/// The pointer is on a ring: `left/top/width/height` are its box inside the capsule (logical px).
#[tauri::command]
async fn show_tip(app: AppHandle, agent: String, left: f64, top: f64, width: f64, height: f64) {
    {
        let state = app.state::<AppState>();
        let mut s = state.inner.lock().unwrap();
        let Some(cap) = s.placement.capsule else { return };
        let scale = if s.placement.scale > 0.0 { s.placement.scale } else { 1.0 };
        let px = |v: f64| (v * scale).round() as i32;
        s.tip_anchor = Some(dwbar_core::layout::Rect::new(
            cap.left + px(left),
            cap.top + px(top),
            cap.left + px(left + width),
            cap.top + px(top + height),
        ));
    }
    let _ = app.emit_to(TIP, "tip", agent);
}

/// The card page measured itself; place and show it.
#[tauri::command]
async fn tip_resize(app: AppHandle, width: f64, height: f64) {
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || platform::show_tip(&app2, width, height));
}

#[tauri::command]
async fn hide_tip(app: AppHandle) {
    app.state::<AppState>().inner.lock().unwrap().tip_anchor = None;
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || platform::hide_tip(&app2));
}

#[tauri::command]
async fn show_settings(app: AppHandle, page: Option<String>) {
    open_settings(&app, page.as_deref());
}

#[tauri::command]
fn read_log(lines: Option<usize>) -> Vec<String> {
    log::tail(lines.unwrap_or(200))
}

/// Hide / show the Windows widgets (weather) button. When Windows protects the
/// setting, open the Taskbar settings page so the user can flip it there.
#[tauri::command]
async fn set_widgets(app: AppHandle, show: bool) -> Result<String, String> {
    #[cfg(any(windows, feature = "lint-win"))]
    {
        let result = dwbar_core::win::taskbar::set_widgets(show);
        let _ = app.run_on_main_thread({
            let app = app.clone();
            move || platform::apply(&app)
        });
        broadcast(&app);
        match result {
            Ok(()) => Ok(if show { "已顯示小工具".into() } else { "已關閉小工具（天氣）".into() }),
            Err(_) => {
                let _ = std::process::Command::new("explorer.exe").arg("ms-settings:taskbar").spawn();
                Err("Windows 不允許其他程式更改這個設定，已幫你打開「工作列設定」，請把「小工具」關掉。".into())
            }
        }
    }
    #[cfg(not(any(windows, feature = "lint-win")))]
    {
        let _ = (app, show);
        Err("只支援 Windows".into())
    }
}

#[tauri::command]
async fn connect_account(app: AppHandle, agent: String) {
    accounts::open_login(&app, &agent);
}

#[tauri::command]
async fn disconnect_account(app: AppHandle, agent: String) {
    accounts::unlink(&app, &agent);
    request_refresh(&app);
    broadcast(&app);
}

#[tauri::command]
async fn clear_accounts(app: AppHandle) {
    accounts::clear_sessions(&app);
    request_refresh(&app);
    broadcast(&app);
}

#[tauri::command]
fn quit(app: AppHandle) {
    quit_app(app);
}

fn quit_app(app: AppHandle) {
    app.state::<AppState>().inner.lock().unwrap().quitting = true;
    log::write("app", "quit");
    app.exit(0);
}

// ───────────────────────── main ─────────────────────────

fn main() {
    log::write("app", &format!("DW Bar {} starting", env!("CARGO_PKG_VERSION")));
    let config = Config::load();

    let app = tauri::Builder::default()
        // Must be first: a second launch just opens the settings of the running one.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| open_settings(app, None)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .manage(AppState::new(config))
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_config,
            refresh_quotas,
            capsule_resize,
            show_settings,
            read_log,
            connect_account,
            disconnect_account,
            set_widgets,
            show_tip,
            tip_resize,
            hide_tip,
            clear_accounts,
            quit
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            if let Err(e) = build_tray(&handle) {
                log::write("app", &format!("tray icon failed: {e}"));
            }
            create_capsule(&handle);
            create_tip(&handle);
            start_watcher(&handle);
            start_quota_thread(&handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build DW Bar");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { code, api, .. } = event {
            // Closing the settings window, or Explorer taking the capsule down with it
            // on restart, must not quit the app. Only "結束 DW Bar" does.
            let quitting = app.state::<AppState>().inner.lock().unwrap().quitting;
            if code.is_none() && !quitting {
                api.prevent_exit();
            }
        }
    });
}
