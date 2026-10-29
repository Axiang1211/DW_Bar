//! Settings, saved as `%LOCALAPPDATA%\DWBar\config.json`.
//! Unknown or missing fields fall back to defaults, so old files keep loading.
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Where on the taskbar the capsule sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// Windows-widgets rule (recommended): centred taskbar → far left; left-aligned → next to the tray.
    #[default]
    Auto,
    /// Always the far left of the taskbar.
    Left,
    /// Always just left of the system tray.
    Tray,
}

/// The 9-grid position: the centre cell is the taskbar, the other eight make the
/// capsule a floating HUD at that edge or corner of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Position {
    #[serde(rename = "tl")]
    TopLeft,
    #[serde(rename = "t")]
    Top,
    #[serde(rename = "tr")]
    TopRight,
    #[serde(rename = "l")]
    Left,
    #[default]
    #[serde(rename = "taskbar")]
    Taskbar,
    #[serde(rename = "r")]
    Right,
    #[serde(rename = "bl")]
    BottomLeft,
    #[serde(rename = "b")]
    Bottom,
    #[serde(rename = "br")]
    BottomRight,
}

impl Position {
    /// Left / right edges stack vertically; everything else is horizontal.
    pub fn vertical(self) -> bool {
        matches!(self, Position::Left | Position::Right)
    }
    /// Corners collapse to a single ring until clicked.
    pub fn corner(self) -> bool {
        matches!(self, Position::TopLeft | Position::TopRight | Position::BottomLeft | Position::BottomRight)
    }
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
    /// 9-grid position (taskbar or a floating edge / corner).
    pub position: Position,
    /// Where on the taskbar, when `position` is the taskbar.
    pub slot: Slot,
    pub attach: Attach,
    /// Extra horizontal offset in logical px (e.g. to clear the widgets button).
    pub offset_x: i32,
    pub theme: Theme,
    /// Show "%" next to rings. Off = icon only, as in the prototype.
    pub show_percent: bool,
    /// Logo style inside the rings: mono / neon / acrylic / pixel / color.
    pub icon_style: String,
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
    pub antigravity: AgentConfig,
    /// 視窗美化 (off by default).
    pub style: StyleConfig,
}

/// Current settings format (bumped when a one-time migration is needed).
pub const CONFIG_VERSION: u32 = 1;

/// Which windows the window styler touches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StyleTarget {
    /// File Explorer windows only.
    #[default]
    Explorer,
    /// Every normal window with a title bar (except excluded programs).
    All,
    /// Only the programs listed in `apps`.
    Custom,
}

/// 視窗美化: DWM effects applied to other programs' windows, from outside —
/// no code is injected into them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StyleConfig {
    pub enabled: bool,
    pub target: StyleTarget,
    /// Executable names for `Custom`, e.g. `notepad.exe`.
    pub apps: Vec<String>,
    /// Never touched, whatever the target.
    pub exclude: Vec<String>,
    /// none / mica / acrylic / tabbed
    pub backdrop: String,
    /// auto / dark / light
    pub titlebar: String,
    /// default / round / small / square
    pub corners: String,
    /// default / none / accent / "#rrggbb"
    pub border: String,
    /// Title bar colour "#rrggbb", or empty for the system's.
    pub caption: String,
    /// Let the backdrop show through the whole window (experimental: some programs draw black).
    pub extend_frame: bool,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target: StyleTarget::Explorer,
            apps: Vec::new(),
            exclude: Vec::new(),
            backdrop: "acrylic".into(),
            titlebar: "auto".into(),
            corners: "round".into(),
            border: "accent".into(),
            caption: String::new(),
            extend_frame: false,
        }
    }
}

/// Every AI DW Bar knows, in display order.
pub const AGENTS: [&str; 7] = ["claude", "codex", "gemini", "antigravity", "cursor", "windsurf", "deepseek"];

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            position: Position::Taskbar,
            slot: Slot::Auto,
            attach: Attach::Embed,
            offset_x: 0,
            theme: Theme::Auto,
            show_percent: false,
            icon_style: "mono".into(),
            hide_fullscreen: true,
            refresh_minutes: 5,
            autostart: false,
            claude: AgentConfig::default(),
            codex: AgentConfig::default(),
            gemini: AgentConfig::default(),
            cursor: AgentConfig::default(),
            windsurf: AgentConfig::default(),
            deepseek: AgentConfig::default(),
            antigravity: AgentConfig::default(),
            style: StyleConfig::default(),
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
            "antigravity" => &self.antigravity,
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
            "antigravity" => &mut self.antigravity,
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
        assert!(!c.claude.enabled);
        assert!(c.codex.enabled);
        assert_eq!(c.refresh_minutes, 5);
        assert!(c.hide_fullscreen);
    }

    #[test]
    fn positions_round_trip() {
        let c: Config = serde_json::from_str(r#"{"position":"br"}"#).unwrap();
        assert_eq!(c.position, Position::BottomRight);
        assert!(c.position.corner());
        assert_eq!(Config::default().position, Position::Taskbar);
        assert_eq!(Config::default().slot, Slot::Auto);
        assert_eq!(serde_json::to_value(Position::Left).unwrap(), "l");
    }

    #[test]
    fn clamp_keeps_refresh_sane() {
        let mut c = Config { refresh_minutes: 0, ..Config::default() };
        c.clamp();
        assert_eq!(c.refresh_minutes, 2);
    }
}
