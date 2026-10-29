//! Find an AI CLI on this machine: the user's custom path first, then PATH
//! (with PATHEXT on Windows), then the places installers usually put it.
use std::path::{Path, PathBuf};

/// Candidate file names for a command on this OS, in preference order.
fn names(command: &str) -> Vec<String> {
    if cfg!(windows) {
        // Native .exe first; npm shims (.cmd) work but go through cmd.exe.
        vec![format!("{command}.exe"), format!("{command}.cmd"), format!("{command}.bat")]
    } else {
        vec![command.to_string()]
    }
}

/// Well-known install folders that are often missing from a GUI app's PATH.
fn known_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    if let Some(home) = env("USERPROFILE").or_else(|| env("HOME")) {
        dirs.push(home.join(".local").join("bin")); // Claude Code native installer
        dirs.push(home.join(".claude").join("local")); // older Claude Code local install
        dirs.push(home.join(".cargo").join("bin"));
        dirs.push(home.join("scoop").join("shims"));
    }
    if let Some(appdata) = env("APPDATA") {
        dirs.push(appdata.join("npm")); // npm -g on Windows
    }
    if let Some(local) = env("LOCALAPPDATA") {
        dirs.push(local.join("Microsoft").join("WinGet").join("Links"));
        dirs.push(local.join("pnpm"));
        dirs.push(local.join("Volta").join("bin"));
    }
    dirs
}

/// Resolve `command` (e.g. `"claude"`). `custom` is a file or a folder the user picked.
pub fn find(command: &str, custom: Option<&str>) -> Option<PathBuf> {
    if let Some(custom) = custom.map(str::trim).filter(|s| !s.is_empty()) {
        let p = Path::new(custom);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
        if p.is_dir() {
            if let Some(found) = in_dirs(command, std::iter::once(p.to_path_buf())) {
                return Some(found);
            }
        }
    }
    let path_dirs = std::env::var_os("PATH")
        .map(|v| std::env::split_paths(&v).collect::<Vec<_>>())
        .unwrap_or_default();
    in_dirs(command, path_dirs.into_iter().chain(known_dirs()))
        .or_else(|| crate::apps::bundled_engines(command).into_iter().next())
}

fn in_dirs(command: &str, dirs: impl Iterator<Item = PathBuf>) -> Option<PathBuf> {
    let names = names(command);
    for dir in dirs {
        for name in &names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Whether a path is a batch shim that must run through `cmd.exe`.
pub fn is_batch(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_file_wins() {
        let dir = std::env::temp_dir().join("dwbar-locate-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("my-claude");
        std::fs::write(&file, "").unwrap();
        assert_eq!(find("claude", file.to_str()), Some(file));
    }

    #[test]
    fn batch_detection() {
        assert!(is_batch(Path::new(r"C:\Users\a\AppData\Roaming\npm\codex.CMD")));
        assert!(!is_batch(Path::new(r"C:\x\claude.exe")));
    }
}
