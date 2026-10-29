use dwbar_core::{config::Config, layout::Rect, quota::Reading};
use serde::Serialize;
use std::sync::{mpsc::Sender, Mutex};

/// What the last placement did – shown on the diagnostics page.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Placement {
    pub found_taskbar: bool,
    pub embedded: bool,
    /// Embedding was asked for but refused, so the overlay is in use.
    pub fallback: bool,
    pub visible: bool,
    pub scale: f64,
    pub centered: bool,
    pub widgets: bool,
    pub taskbar: Option<Rect>,
    pub capsule: Option<Rect>,
    /// Work area the floating HUD is placed in.
    pub work: Option<Rect>,
    /// Floating HUD (not on the taskbar).
    pub floating: bool,
}

pub struct Inner {
    pub config: Config,
    pub quotas: Vec<Reading>,
    pub refreshing: bool,
    /// Capsule size the page asked for, in logical px.
    pub capsule_width: f64,
    pub capsule_height: f64,
    pub fullscreen: bool,
    pub placement: Placement,
    pub quitting: bool,
    /// Hovered ring, in screen px, while the usage card is up.
    pub tip_anchor: Option<Rect>,
    /// Start-at-login is on, but switched off in Task Manager → Startup apps.
    pub autostart_blocked: bool,
}

pub struct AppState {
    pub inner: Mutex<Inner>,
    /// Wakes the quota thread for an immediate refresh.
    pub refresh_now: Mutex<Option<Sender<()>>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Mutex::new(Inner {
                config,
                quotas: Vec::new(),
                refreshing: false,
                capsule_width: 96.0,
                capsule_height: 96.0,
                fullscreen: false,
                placement: Placement::default(),
                quitting: false,
                tip_anchor: None,
                autostart_blocked: false,
            }),
            refresh_now: Mutex::new(None),
        }
    }
}

/// Everything the windows need in one payload.
#[derive(Serialize)]
pub struct Snapshot {
    pub config: Config,
    pub quotas: Vec<Reading>,
    pub refreshing: bool,
    pub fullscreen: bool,
    pub placement: Placement,
    pub version: &'static str,
    /// "windows" / "macos" / "linux": the pages hide options that don't apply.
    pub platform: &'static str,
    pub log_path: String,
    pub config_path: String,
    pub autostart_blocked: bool,
    /// Windows 11 rounds the blurred popups (8 px); Windows 10 keeps them square.
    pub round_corners: bool,
}

impl Inner {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            config: self.config.clone(),
            quotas: self.quotas.clone(),
            refreshing: self.refreshing,
            fullscreen: self.fullscreen,
            placement: self.placement.clone(),
            version: env!("CARGO_PKG_VERSION"),
            platform: std::env::consts::OS,
            log_path: dwbar_core::log::path().display().to_string(),
            config_path: Config::path().display().to_string(),
            autostart_blocked: self.autostart_blocked && self.config.autostart,
            round_corners: round_corners(),
        }
    }
}

fn round_corners() -> bool {
    #[cfg(windows)]
    {
        static WIN11: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *WIN11.get_or_init(dwbar_core::win::taskbar::is_windows_11)
    }
    #[cfg(not(windows))]
    true
}
