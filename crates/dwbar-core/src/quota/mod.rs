//! Plan usage for each AI, read through the vendor's own CLI so DW Bar never
//! touches tokens or cookies:
//!
//! * Claude Code – `claude -p --input-format stream-json …` + `get_usage` control request
//! * Codex       – `codex app-server --listen stdio://` + `account/rateLimits/read`
//!
//! The approach follows Agent HUD Open (Apache-2.0, jazzenchen/agent-hud-open);
//! the code here is an independent, much smaller implementation.
mod claude;
mod codex;
mod engine;

pub use claude::parse_usage as parse_claude;
pub use codex::parse_limits as parse_codex;

use crate::config::Config;
use serde::Serialize;

/// Which period a window covers, so the capsule knows which ring to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    /// Rolling 5-hour window → the upper half arc.
    FiveHour,
    /// Weekly window → the inner circle.
    Week,
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
    /// CLI found on this machine.
    pub installed: bool,
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
            ok: false,
            plan: None,
            account: None,
            balance: None,
            windows: Vec::new(),
            notice: None,
            updated: crate::now(),
        }
    }
    pub(crate) fn failed(agent: &str, name: &str, notice: impl Into<String>) -> Self {
        let mut r = Self::empty(agent, name);
        r.notice = Some(notice.into());
        r
    }
    pub(crate) fn missing(agent: &str, name: &str) -> Self {
        let mut r = Self::failed(agent, name, "未偵測到此 AI 的命令列工具");
        r.installed = false;
        r
    }
    pub fn window(&self, period: Period) -> Option<&Window> {
        self.windows.iter().find(|w| w.period == period)
    }
}

/// Read every enabled agent. Blocking (each read spawns a CLI and can take seconds),
/// so call it from a background thread. Agents are read in parallel.
pub fn read_all(config: &Config) -> Vec<Reading> {
    let mut jobs = Vec::new();
    if config.claude.enabled {
        let custom = config.claude.custom_path.clone();
        jobs.push(std::thread::spawn(move || claude::read(custom.as_deref())));
    }
    if config.codex.enabled {
        let custom = config.codex.custom_path.clone();
        jobs.push(std::thread::spawn(move || codex::read(custom.as_deref())));
    }
    jobs.into_iter()
        .filter_map(|job| job.join().ok())
        .inspect(|r| {
            if let Some(n) = &r.notice {
                crate::log!("quota", "{}: {}", r.agent, n);
            } else {
                let w = |p| r.window(p).map(|w| format!("{:.0}%", w.used_percent)).unwrap_or("-".into());
                crate::log!("quota", "{}: 5h {} / week {}", r.agent, w(Period::FiveHour), w(Period::Week));
            }
        })
        .collect()
}

/// Classify a window by its length in minutes.
pub(crate) fn period_for_minutes(minutes: Option<i64>) -> Period {
    match minutes {
        Some(m) if (240..=360).contains(&m) => Period::FiveHour,
        Some(m) if (6 * 1440..=8 * 1440).contains(&m) => Period::Week,
        _ => Period::Other,
    }
}
