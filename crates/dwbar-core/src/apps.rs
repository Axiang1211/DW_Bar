//! Which AI desktop apps are installed, and the engines they bundle.
//!
//! Cheap file-system checks only (no PowerShell, no registry walks), so this can
//! run on every refresh. Covers the per-user installers and Microsoft Store (MSIX)
//! packages, whose data folders live under `%LOCALAPPDATA%\Packages`.
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// The AIs whose desktop app (or installed web app) was found, by agent id.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Detected(pub BTreeSet<String>);

impl Detected {
    pub fn has(&self, agent: &str) -> bool {
        self.0.contains(agent)
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

/// Lower-cased names (folders and files, extension stripped) inside `dir`, `depth` levels deep.
fn names_in(dir: &Path, depth: usize, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().filter_map(Result::ok) {
        let path = entry.path();
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_lowercase());
        }
        if depth > 1 && path.is_dir() {
            names_in(&path, depth - 1, out);
        }
    }
}

/// Words that identify each AI's app in folder / shortcut names.
const NEEDLES: [(&str, &[&str]); 7] = [
    ("claude", &["claude", "anthropic"]),
    ("codex", &["chatgpt", "codex", "openai.chatgpt"]),
    ("gemini", &["gemini"]),
    ("antigravity", &["antigravity"]),
    ("cursor", &["cursor"]),
    ("windsurf", &["windsurf"]),
    ("deepseek", &["deepseek"]),
];

fn matches(agent: &str, needles: &[&str], name: &str) -> bool {
    if needles.iter().any(|n| name.contains(n)) {
        return true;
    }
    // "DSH" is too short to match anywhere in a name; require it as a whole word.
    agent == "deepseek" && name.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| w == "dsh")
}

/// Scan the usual install locations: per-user and machine-wide program folders,
/// Microsoft Store package folders, and Start menu shortcuts (which also catch
/// web apps installed from Chrome or Edge).
pub fn detect() -> Detected {
    let mut names = Vec::new();
    let local = env_path("LOCALAPPDATA");
    let roaming = env_path("APPDATA");
    if let Some(l) = &local {
        names_in(&l.join("Programs"), 1, &mut names);
        names_in(&l.join("Packages"), 1, &mut names);
        names_in(l, 1, &mut names); // e.g. AnthropicClaude
    }
    for key in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(p) = env_path(key) {
            names_in(&p, 1, &mut names);
        }
    }
    if let Some(r) = &roaming {
        names_in(&r.join("Microsoft").join("Windows").join("Start Menu").join("Programs"), 3, &mut names);
    }
    if let Some(p) = env_path("ProgramData") {
        names_in(&p.join("Microsoft").join("Windows").join("Start Menu").join("Programs"), 3, &mut names);
    }
    // macOS: /Applications and ~/Applications (Chrome web apps live in ~/Applications/Chrome Apps.localized).
    if cfg!(target_os = "macos") {
        names_in(Path::new("/Applications"), 1, &mut names);
        if let Some(home) = env_path("HOME") {
            names_in(&home.join("Applications"), 2, &mut names);
        }
    }
    let mut found = BTreeSet::new();
    for (agent, needles) in NEEDLES {
        if names.iter().any(|n| matches(agent, needles, n)) {
            found.insert(agent.to_string());
        }
    }
    Detected(found)
}

/// Newest `<dir>/<version>/<exe>` (by modification time).
fn newest_in(dir: &Path, exe: &str) -> Option<PathBuf> {
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path().join(exe))
        .filter(|p| p.is_file())
        .filter_map(|p| Some((std::fs::metadata(&p).ok()?.modified().ok()?, p)))
        .collect();
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found.into_iter().next().map(|(_, p)| p)
}

/// Engines bundled inside desktop apps, newest first. Used when the CLI is not on PATH.
pub fn bundled_engines(command: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let local = env_path("LOCALAPPDATA");
    let roaming = env_path("APPDATA");
    match command {
        "claude" => {
            // The Claude desktop app downloads Claude Code here for its Code tab.
            if let Some(p) = roaming.as_ref().and_then(|r| newest_in(&r.join("Claude").join("claude-code"), "claude.exe")) {
                out.push(p);
            }
            if let Some(home) = env_path("HOME").filter(|_| cfg!(target_os = "macos")) {
                let dir = home.join("Library").join("Application Support").join("Claude").join("claude-code");
                if let Some(p) = newest_in(&dir, "claude") {
                    out.push(p);
                }
            }
        }
        "codex" => {
            if cfg!(target_os = "macos") {
                for app in ["/Applications/Codex.app", "/Applications/ChatGPT.app"] {
                    let p = Path::new(app).join("Contents").join("Resources").join("codex");
                    if p.is_file() {
                        out.push(p);
                    }
                }
            }
            if let Some(l) = &local {
                // Store builds copy their runtime to OpenAI\Codex\bin\<hash>\.
                if let Some(p) = newest_in(&l.join("OpenAI").join("Codex").join("bin"), "codex.exe") {
                    out.push(p);
                }
                for app in ["Codex", "ChatGPT"] {
                    let p = l.join("Programs").join(app).join("resources").join("codex.exe");
                    if p.is_file() {
                        out.push(p);
                    }
                }
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_matching() {
        assert!(matches("deepseek", &["deepseek"], "dsh desktop"));
        assert!(!matches("deepseek", &["deepseek"], "dashboard"));
        assert!(matches("codex", &["chatgpt", "codex"], "openai.chatgpt-desktop_2p2nqsd0c76g0"));
        assert!(matches("claude", &["claude", "anthropic"], "anthropicclaude"));
    }
}
