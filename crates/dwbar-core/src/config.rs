//! Settings, saved as `%LOCALAPPDATA%\DWBar\config.json`.
//! Unknown or missing fields fall back to defaults, so old files keep loading.
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Where on the taskbar the capsule sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// Windows-widgets rule: centred taskbar → far left; left-aligned → next to the tray.
    Auto,
    /// Always the far left of the taskbar.
    Left,
    /// Always just left of the system tray (default).
    #[default]
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
    /// Settings format version, for one-time migrations. Missing (older files) = 0.
    #[serde(default)]
    pub version: u32,
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
    pub gemini: AgentConfig,
    pub cursor: AgentConfig,
    pub windsurf: AgentConfig,
    pub deepseek: AgentConfig,
}

/// Current settings format. 1: default position moved back to "left of the tray".
pub const CONFIG_VERSION: u32 = 1;

/// Every AI DW Bar knows, in display order.
pub const AGENTS: [&str; 6] = ["claude", "codex", "gemini", "cursor", "windsurf", "deepseek"];

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            slot: Slot::Tray,
            attach: Attach::Embed,
            offset_x: 0,
            theme: Theme::Auto,
            show_percent: false,
            hide_fullscreen: true,
            refresh_minutes: 5,
            autostart: false,
            claude: AgentConfig::default(),
            codex: AgentConfig::default(),
            gemini: AgentConfig::default(),
            cursor: AgentConfig::default(),
            windsurf: AgentConfig::default(),
            deepseek: AgentConfig::default(),
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
        config.migrate();
        config.clamp();
        config
    }

    /// One-time upgrades of files written by older versions.
    pub fn migrate(&mut self) {
        if self.version < 1 {
            // v0.1–v0.3 saved the then-default "auto"; the default is left of the tray again.
            if self.slot == Slot::Auto {
                self.slot = Slot::Tray;
            }
        }
        self.version = CONFIG_VERSION;
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

    pub fn agent(&self, id: &str) -> &AgentConfig {
        match id {
            "claude" => &self.claude,
            "codex" => &self.codex,
            "gemini" => &self.gemini,
            "cursor" => &self.cursor,
            "windsurf" => &self.windsurf,
            _ => &self.deepseek,
        }
    }

    pub fn agent_mut(&mut self, id: &str) -> &mut AgentConfig {
        match id {
            "claude" => &mut self.claude,
            "codex" => &mut self.codex,
            "gemini" => &mut self.gemini,
            "cursor" => &mut self.cursor,
            "windsurf" => &mut self.windsurf,
            _ => &mut self.deepseek,
        }
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
        let c: Config = serde_json::from_str(r#"{"slot":"left","claude":{"enabled":false}}"#).unwrap();
        assert_eq!(c.slot, Slot::Left);
        assert_eq!(Config::default().slot, Slot::Tray);
        assert!(!c.claude.enabled);
        assert!(c.codex.enabled);
        assert_eq!(c.refresh_minutes, 5);
        assert!(c.hide_fullscreen);
    }

    #[test]
    fn old_files_move_back_to_the_tray() {
        let mut old: Config = serde_json::from_str(r#"{"slot":"auto"}"#).unwrap();
        old.migrate();
        assert_eq!(old.slot, Slot::Tray);
        let mut chosen: Config = serde_json::from_str(r#"{"version":1,"slot":"auto"}"#).unwrap();
        chosen.migrate();
        assert_eq!(chosen.slot, Slot::Auto);
    }

    #[test]
    fn clamp_keeps_refresh_sane() {
        let mut c = Config { refresh_minutes: 0, ..Config::default() };
        c.clamp();
        assert_eq!(c.refresh_minutes, 2);
    }
}
