//! App updates for the installer / DMG versions (the Microsoft Store version updates
//! through the Store). At start an available update is downloaded and installed right
//! away; later checks (every 6 hours, or "檢查更新") download it and wait for the user.
//! Updates come from the GitHub release's `latest.json` and are verified with the
//! minisign key in `tauri.conf.json` before they run.
use crate::state::AppState;
use serde::Serialize;
use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// What the settings page and panel show.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Status {
    /// "idle" | "checking" | "none" | "downloading" | "ready" | "installing" | "error"
    pub state: &'static str,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// A downloaded update waiting for "重新啟動並更新".
#[derive(Default)]
pub struct Pending(pub Mutex<Option<(Update, Vec<u8>)>>);

/// This build updates itself (not the Store package, not Linux).
pub fn supported() -> bool {
    #[cfg(windows)]
    {
        !dwbar_core::win::autostart::packaged()
    }
    #[cfg(target_os = "macos")]
    {
        true
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        false
    }
}

fn set(app: &AppHandle, status: Status) {
    app.state::<AppState>().inner.lock().unwrap().update = status;
    crate::broadcast(app);
}

/// Check, download, and either install now (at start) or keep it for the user.
pub async fn check(app: &AppHandle, install_now: bool) {
    if !supported() {
        return;
    }
    set(app, Status { state: "checking", ..Default::default() });
    let found = match app.updater() {
        Ok(u) => u.check().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    let update = match found {
        Ok(Some(u)) => u,
        Ok(None) => return set(app, Status { state: "none", ..Default::default() }),
        Err(e) => {
            dwbar_core::log::write("update", &format!("check failed: {e}"));
            return set(app, Status { state: "error", error: Some(e), ..Default::default() });
        }
    };
    let version = Some(update.version.clone());
    dwbar_core::log::write("update", &format!("downloading {}", update.version));
    set(app, Status { state: "downloading", version: version.clone(), ..Default::default() });
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => {
            if install_now {
                install(app, update, bytes);
            } else {
                *app.state::<Pending>().0.lock().unwrap() = Some((update, bytes));
                set(app, Status { state: "ready", version, ..Default::default() });
            }
        }
        Err(e) => {
            dwbar_core::log::write("update", &format!("download failed: {e}"));
            set(app, Status { state: "error", version, error: Some(e.to_string()) });
        }
    }
}

/// Run the downloaded installer. Windows closes DW Bar and reopens it after a short,
/// unattended install; macOS replaces the app and restarts it here.
pub fn install(app: &AppHandle, update: Update, bytes: Vec<u8>) {
    dwbar_core::log::write("update", &format!("installing {}", update.version));
    set(app, Status { state: "installing", version: Some(update.version.clone()), ..Default::default() });
    app.state::<AppState>().inner.lock().unwrap().quitting = true;
    #[cfg(any(windows, feature = "lint-win"))]
    dwbar_core::win::styler::restore_all();
    match update.install(bytes) {
        Ok(()) => app.restart(),
        Err(e) => {
            app.state::<AppState>().inner.lock().unwrap().quitting = false;
            dwbar_core::log::write("update", &format!("install failed: {e}"));
            set(app, Status { state: "error", version: Some(update.version.clone()), error: Some(e.to_string()) });
        }
    }
}

/// Start-up check (installs right away), then every 6 hours (downloads and waits).
pub fn start(app: &AppHandle) {
    if !supported() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio_sleep(Duration::from_secs(8)).await;
        let mut first = true;
        loop {
            let auto = app.state::<AppState>().inner.lock().unwrap().config.auto_update;
            let waiting = app.state::<Pending>().0.lock().unwrap().is_some();
            if auto && !waiting {
                check(&app, first).await;
            }
            first = false;
            tokio_sleep(Duration::from_secs(6 * 3600)).await;
        }
    });
}

async fn tokio_sleep(d: Duration) {
    // tauri's async runtime is tokio; a blocking sleep on a spawned blocking thread keeps
    // this free of an extra tokio dependency.
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}
