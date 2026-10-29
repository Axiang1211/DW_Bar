//! Pure rules for the window styler (視窗美化): which windows qualify, and the DWM
//! values each setting maps to. The Win32 side lives in `win::styler`.
use crate::config::{StyleConfig, StyleTarget};

/// DWM attribute values for one window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effects {
    /// DWMWA_SYSTEMBACKDROP_TYPE: 0 auto, 1 none, 2 Mica, 3 acrylic, 4 tabbed.
    pub backdrop: u32,
    /// DWMWA_USE_IMMERSIVE_DARK_MODE, or `None` to leave the app's choice.
    pub dark: Option<bool>,
    /// DWMWA_WINDOW_CORNER_PREFERENCE: 0 default, 1 square, 2 round, 3 small.
    pub corners: u32,
    /// DWMWA_BORDER_COLOR as COLORREF (0x00BBGGRR); DEFAULT / NONE specials below.
    pub border: u32,
    /// DWMWA_CAPTION_COLOR, `DEFAULT` for the system colour.
    pub caption: u32,
    pub extend_frame: bool,
}

/// DWMWA_COLOR_DEFAULT
pub const COLOR_DEFAULT: u32 = 0xFFFF_FFFF;
/// DWMWA_COLOR_NONE
pub const COLOR_NONE: u32 = 0xFFFF_FFFE;

/// Everything back to what Windows would do on its own.
pub const RESET: Effects = Effects {
    backdrop: 0,
    dark: None,
    corners: 0,
    border: COLOR_DEFAULT,
    caption: COLOR_DEFAULT,
    extend_frame: false,
};

/// `#rrggbb` → COLORREF (0x00BBGGRR).
pub fn colorref(hex: &str) -> Option<u32> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    let (r, g, b) = ((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
    Some(b << 16 | g << 8 | r)
}

/// Settings → DWM values. `accent` is the Windows accent colour as COLORREF.
pub fn effects(cfg: &StyleConfig, accent: Option<u32>) -> Effects {
    Effects {
        backdrop: match cfg.backdrop.as_str() {
            "none" => 1,
            "mica" => 2,
            "acrylic" => 3,
            "tabbed" => 4,
            _ => 0,
        },
        dark: match cfg.titlebar.as_str() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        },
        corners: match cfg.corners.as_str() {
            "square" => 1,
            "round" => 2,
            "small" => 3,
            _ => 0,
        },
        border: match cfg.border.as_str() {
            "none" => COLOR_NONE,
            "accent" => accent.unwrap_or(COLOR_DEFAULT),
            "default" | "" => COLOR_DEFAULT,
            hex => colorref(hex).unwrap_or(COLOR_DEFAULT),
        },
        caption: colorref(&cfg.caption).unwrap_or(COLOR_DEFAULT),
        extend_frame: cfg.extend_frame,
    }
}

/// Facts about a top-level window, gathered by the Win32 side.
#[derive(Clone, Debug, Default)]
pub struct WindowFacts {
    pub class: String,
    /// Lower-case executable file name, e.g. `explorer.exe`.
    pub exe: String,
    pub has_caption: bool,
    pub tool_window: bool,
    pub visible: bool,
    pub own_process: bool,
}

/// Shell surfaces that must never be restyled.
const SHELL_CLASSES: [&str; 8] = [
    "Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd",
    "Windows.UI.Core.CoreWindow", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland", "XamlExplorerHostIslandWindow",
];

/// Programs left alone by default: games and anti-cheat launchers, remote desktop,
/// and system prompts.
pub const DEFAULT_EXCLUDE: [&str; 10] = [
    "consent.exe", "lockapp.exe", "logonui.exe", "mstsc.exe", "steam.exe",
    "epicgameslauncher.exe", "riotclientservices.exe", "battle.net.exe", "easyanticheat.exe", "vgtray.exe",
];

pub fn qualifies(w: &WindowFacts, cfg: &StyleConfig) -> bool {
    if !cfg.enabled || !w.visible || w.own_process || w.tool_window || !w.has_caption {
        return false;
    }
    if SHELL_CLASSES.contains(&w.class.as_str()) {
        return false;
    }
    let listed = |list: &[String]| list.iter().any(|a| a.trim().eq_ignore_ascii_case(&w.exe));
    if DEFAULT_EXCLUDE.contains(&w.exe.as_str()) || listed(&cfg.exclude) {
        return false;
    }
    match cfg.target {
        StyleTarget::Explorer => w.class == "CabinetWClass",
        StyleTarget::All => true,
        StyleTarget::Custom => listed(&cfg.apps),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(class: &str, exe: &str) -> WindowFacts {
        WindowFacts { class: class.into(), exe: exe.into(), has_caption: true, visible: true, ..Default::default() }
    }

    #[test]
    fn colours() {
        assert_eq!(colorref("#ff8000"), Some(0x0000_80FF));
        assert_eq!(colorref("zz"), None);
        let cfg = StyleConfig { border: "none".into(), ..Default::default() };
        assert_eq!(effects(&cfg, None).border, COLOR_NONE);
        let cfg = StyleConfig { border: "accent".into(), ..Default::default() };
        assert_eq!(effects(&StyleConfig::default(), None).border, COLOR_DEFAULT);
        assert_eq!(effects(&cfg, Some(0x123456)).border, 0x123456);
        assert_eq!(effects(&StyleConfig::default(), None).backdrop, 3);
    }

    #[test]
    fn who_gets_styled() {
        let on = StyleConfig { enabled: true, target: StyleTarget::Explorer, ..Default::default() };
        assert!(qualifies(&win("CabinetWClass", "explorer.exe"), &on));
        assert!(!qualifies(&win("Notepad", "notepad.exe"), &on));
        assert!(!qualifies(&win("Shell_TrayWnd", "explorer.exe"), &StyleConfig { target: StyleTarget::All, ..on.clone() }));
        let all = StyleConfig { target: StyleTarget::All, exclude: vec!["Code.exe".into()], ..on.clone() };
        assert!(qualifies(&win("Notepad", "notepad.exe"), &all));
        assert!(!qualifies(&win("Chrome_WidgetWin_1", "code.exe"), &all));
        assert!(!qualifies(&win("X", "steam.exe"), &all));
        let custom = StyleConfig { target: StyleTarget::Custom, apps: vec!["notepad.exe".into()], ..on.clone() };
        assert!(qualifies(&win("Notepad", "notepad.exe"), &custom));
        assert!(!qualifies(&win("CabinetWClass", "explorer.exe"), &custom));
        let mut tool = win("CabinetWClass", "explorer.exe");
        tool.tool_window = true;
        assert!(!qualifies(&tool, &on));
        assert!(!qualifies(&win("CabinetWClass", "explorer.exe"), &StyleConfig::default()));
    }
}
