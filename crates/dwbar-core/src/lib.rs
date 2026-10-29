//! DW Bar core: everything that does not need the webview.
//!
//! * [`layout`]  – where the capsule goes on the taskbar (pure, unit-tested)
//! * [`config`]  – settings persisted as JSON
//! * [`quota`]   – Claude Code / Codex plan usage, read through their own CLIs
//! * [`win`]     – Win32: taskbar lookup, embedding, fullscreen detection, watcher thread
//! * [`log`]     – tiny rotating diagnostics log

pub mod apps;
pub mod config;
pub mod layout;
pub mod locate;
pub mod log;
pub mod quota;
pub mod startup;
pub mod style;

#[cfg(any(windows, feature = "lint-win"))]
pub mod win;

/// `%LOCALAPPDATA%\DWBar` (falls back to the temp dir off Windows, for tests).
pub fn data_dir() -> std::path::PathBuf {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return std::path::PathBuf::from(local).join("DWBar");
    }
    // macOS: ~/Library/Application Support/DWBar; other Unix: ~/.local/share/DWBar.
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        let base = if cfg!(target_os = "macos") {
            home.join("Library").join("Application Support")
        } else {
            home.join(".local").join("share")
        };
        return base.join("DWBar");
    }
    std::env::temp_dir().join("DWBar")
}

/// Seconds since the Unix epoch.
pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
