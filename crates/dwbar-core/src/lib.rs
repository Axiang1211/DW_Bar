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

#[cfg(any(windows, feature = "lint-win"))]
pub mod win;

/// `%LOCALAPPDATA%\DWBar` (falls back to the temp dir off Windows, for tests).
pub fn data_dir() -> std::path::PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("DWBar")
}

/// Seconds since the Unix epoch.
pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
