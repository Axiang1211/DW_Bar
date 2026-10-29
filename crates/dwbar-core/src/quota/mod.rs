//! Plan usage for each AI, read through the vendor's own CLI so DW Bar never
//! touches tokens or cookies:
//!
//! * Claude Code – `claude -p --input-format stream-json …` + `get_usage` control request
//! * Codex       – `codex app-server --listen stdio://` + `account/rateLimits/read`
//!
//! The approach follows Agent HUD Open (Apache-2.0, jazzenchen/agent-hud-open);
//! the code here is an independent, much smaller implementation.
pub mod antigravity;
mod claude;
mod codex;
mod engine;
pub mod web;

pub use claude::parse_usage as parse_claude;
pub use codex::parse_limits as parse_codex;

use serde::Serialize;

/// Which period a window covers, so the capsule knows which ring to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    /// Rolling 5-hour window → the upper half arc.
    FiveHour,
    /// Daily window (Windsurf) → the upper half arc when there is no 5-hour window.
    Day,
    /// Weekly window → the inner circle.
    Week,
    /// Billing month (Cursor) → the inner circle when there is no weekly window.
    Month,
    /// Anything else (model-specific weekly caps, etc.) – shown in the card only.
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Window {
    pub period: Period,
    pub label: String,
    /// 0–100.
    pub used_percent: f64,
    /// Unix seconds.
    pub resets_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Reading {
    /// `"claude"` or `"codex"`.
    pub agent: String,
    pub name: String,
    /// Shown on the taskbar: a desktop app, CLI or linked account was found.
    pub installed: bool,
    /// Where the numbers came from: `"account"` (signed in inside DW Bar) or `"cli"`.
    pub source: Option<String>,
    /// The CLI (or an engine bundled in a desktop app) was found.
    pub cli: bool,
    /// The vendor's desktop app is installed.
    pub app: bool,
    /// An account is linked inside DW Bar.
    pub connected: bool,
    pub ok: bool,
    pub plan: Option<String>,
    pub account: Option<String>,
    /// Remaining paid credits as the vendor reports them, when it does.
    pub balance: Option<String>,
    pub windows: Vec<Window>,
    /// Human-readable reason when there are no numbers (signed out, API key, error).
    pub notice: Option<String>,
    /// Unix seconds of this reading.
    pub updated: i64,
}

impl Reading {
    pub(crate) fn empty(agent: &str, name: &str) -> Self {
        Self {
            agent: agent.into(),
            name: name.into(),
            installed: true,
            source: None,
            cli: false,
            app: false,
            connected: false,
            ok: false,
            plan: None,
            account: None,
            balance: None,
            windows: Vec::new(),
            notice: None,
            updated: crate::now(),
        }
    }
    pub fn failed(agent: &str, name: &str, notice: impl Into<String>) -> Self {
        let mut r = Self::empty(agent, name);
        r.notice = Some(notice.into());
        r
    }
    pub(crate) fn missing(agent: &str, name: &str) -> Self {
        let mut r = Self::failed(agent, name, "尚未連結帳號");
        r.installed = false;
        r
    }
    pub fn window(&self, period: Period) -> Option<&Window> {
        self.windows.iter().find(|w| w.period == period)
    }
}

/// Read one agent through its CLI (or an engine bundled in its desktop app).
/// Blocking: spawns a process and can take seconds, so call it off the UI thread.
pub fn read_cli(agent: &str, custom_path: Option<&str>) -> Reading {
    let r = match agent {
        "claude" => claude::read(custom_path),
        "codex" => codex::read(custom_path),
        other => Reading::failed(other, other, "未知的 AI"),
    };
    log_reading(&r);
    r
}

pub fn log_reading(r: &Reading) {
    if let Some(n) = &r.notice {
        crate::log!("quota", "{} ({}): {}", r.agent, r.source.as_deref().unwrap_or("-"), n);
    } else {
        let w = |p| r.window(p).map(|w| format!("{:.0}%", w.used_percent)).unwrap_or("-".into());
        crate::log!(
            "quota",
            "{} ({}): used 5h {} / week {}",
            r.agent,
            r.source.as_deref().unwrap_or("-"),
            w(Period::FiveHour),
            w(Period::Week)
        );
    }
}

/// Classify a window by its length in minutes.
pub(crate) fn period_for_minutes(minutes: Option<i64>) -> Period {
    match minutes {
        Some(m) if (240..=360).contains(&m) => Period::FiveHour,
        Some(m) if (1380..=1500).contains(&m) => Period::Day,
        Some(m) if (6 * 1440..=8 * 1440).contains(&m) => Period::Week,
        Some(m) if (27 * 1440..=32 * 1440).contains(&m) => Period::Month,
        _ => Period::Other,
    }
}
