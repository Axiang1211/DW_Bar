//! Diagnostics log at `%LOCALAPPDATA%\DWBar\logs\dwbar.log`.
//! One line per event, rotated to `dwbar.old.log` at 1 MB. Never panics.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

static LOCK: Mutex<()> = Mutex::new(());
const MAX_BYTES: u64 = 1024 * 1024;

pub fn path() -> PathBuf {
    crate::data_dir().join("logs").join("dwbar.log")
}

/// Append `[tag] message` with a local timestamp.
pub fn write(tag: &str, message: &str) {
    let _guard = LOCK.lock();
    let file = path();
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if fs::metadata(&file).map(|m| m.len() > MAX_BYTES).unwrap_or(false) {
        let _ = fs::rename(&file, file.with_file_name("dwbar.old.log"));
    }
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&file) {
        let _ = writeln!(out, "{stamp} [{tag}] {message}");
    }
    #[cfg(debug_assertions)]
    eprintln!("[{tag}] {message}");
}

/// The last `lines` lines, for the settings window's diagnostics page.
pub fn tail(lines: usize) -> Vec<String> {
    let text = fs::read_to_string(path()).unwrap_or_default();
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[macro_export]
macro_rules! log {
    ($tag:expr, $($arg:tt)*) => { $crate::log::write($tag, &format!($($arg)*)) };
}
