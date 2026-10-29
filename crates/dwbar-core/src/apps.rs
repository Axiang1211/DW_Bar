//! Which AI desktop apps are installed, and the engines they bundle.
//!
//! Cheap file-system checks only (no PowerShell, no registry walks), so this can
//! run on every refresh. Covers the per-user installers and Microsoft Store (MSIX)
//! packages, whose data folders live under `%LOCALAPPDATA%\Packages`.
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Detected {
    /// Claude desktop app (claude.ai).
    pub claude_app: bool,
    /// ChatGPT desktop app.
    pub chatgpt_app: bool,
    /// Codex desktop app.
    pub codex_app: bool,
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

/// Names of the folders directly inside `dir` (lower-cased).
fn folder_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_lowercase))
        .collect()
}

pub fn detect() -> Detected {
    let local = env_path("LOCALAPPDATA");
    let roaming = env_path("APPDATA");
    let packages = local.as_ref().map(|l| folder_names(&l.join("Packages"))).unwrap_or_default();
    let programs = local.as_ref().map(|l| folder_names(&l.join("Programs"))).unwrap_or_default();
    let has_pkg = |needle: &str| packages.iter().any(|p| p.contains(needle));
    let has_prog = |name: &str| programs.iter().any(|p| p == name);
    let exists = |p: Option<PathBuf>| p.is_some_and(|p| p.exists());

    Detected {
        claude_app: exists(local.as_ref().map(|l| l.join("AnthropicClaude")))
            || exists(roaming.as_ref().map(|r| r.join("Claude")))
            || has_prog("claude")
            || packages.iter().any(|p| p.starts_with("claude_") || p.starts_with("anthropic")),
        chatgpt_app: has_pkg("chatgpt") || has_prog("chatgpt"),
        codex_app: (has_pkg("openai.codex") || has_prog("codex"))
            || exists(local.as_ref().map(|l| l.join("OpenAI").join("Codex"))),
    }
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
        }
        "codex" => {
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
