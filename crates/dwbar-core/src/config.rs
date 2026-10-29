//! Settings, saved as `%LOCALAPPDATA%\DWBar\config.json`.
//! Unknown or missing fields fall back to defaults, so old files keep loading.
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Where on the taskbar the capsule sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// Windows-widgets rule: centred taskbar → far left; left-aligned → next to the tray.
    #[default]
    Auto,
    /// Always the far left of the taskbar.
    Left,
    /// Always just left of the system tray.
    Tray,
}

/// How the capsule window is attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Attach {
    /// Child window of `Shell_TrayWnd` (plan A). Moves and hides with the taskbar.
    #[default]
    Embed,
    /// Top-most tool window drawn over the taskbar. Fallback when embedding misbehaves.
    Overlay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    pub enabled: bool,
    /// The user linked an account inside DW Bar (claude.ai / chatgpt.com).
    pub connected: bool,
    /// Full path to the CLI when it is not on PATH (`claude.exe`, `codex.cmd`, …).
    pub custom_path: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { enabled: true, connected: false, custom_path: None }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub slot: Slot,
    pub attach: Attach,
    /// Extra horizontal offset in logical px (e.g. to clear the widgets button).
    pub offset_x: i32,
    pub theme: Theme,
    /// Show "%" next to rings. Off = icon only, as in the prototype.
    pub show_percent: bool,
    /// Hide while a fullscreen app, game or presentation is in front.
    pub hide_fullscreen: bool,
    /// Minutes between quota refreshes (clamped 2..=60).
    pub refresh_minutes: u32,
    pub autostart: bool,
    pub claude: AgentConfig,
    pub codex: AgentConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            slot: Slot::Auto,
            attach: Attach::Embed,
            offset_x: 0,
            theme: Theme::Auto,
            show_percent: false,
            hide_fullscreen: true,
            refresh_minutes: 5,
            autostart: false,
            claude: AgentConfig::default(),
            codex: AgentConfig::default(),
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        crate::data_dir().join("config.json")
    }

    pub fn load() -> Self {
        let mut config: Config = fs::read_to_string(Self::path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        config.clamp();
        config
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // Write-then-rename so a crash never leaves half a file.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self).unwrap_or_default())?;
        fs::rename(tmp, path)
    }

    pub fn clamp(&mut self) {
        self.refresh_minutes = self.refresh_minutes.clamp(2, 60);
        self.offset_x = self.offset_x.clamp(-2000, 2000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_files_fill_in_defaults() {
        let c: Config = serde_json::from_str(r#"{"slot":"tray","claude":{"enabled":false}}"#).unwrap();
        assert_eq!(c.slot, Slot::Tray);
        assert!(!c.claude.enabled);
        assert!(c.codex.enabled);
        assert_eq!(c.refresh_minutes, 5);
        assert!(c.hide_fullscreen);
    }

    #[test]
    fn clamp_keeps_refresh_sane() {
        let mut c = Config { refresh_minutes: 0, ..Config::default() };
        c.clamp();
        assert_eq!(c.refresh_minutes, 2);
    }
}
