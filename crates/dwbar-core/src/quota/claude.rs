//! Claude Code: ask its own engine for `get_usage`.
use super::{engine::Engine, Period, Reading, Window};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub(crate) const NAME: &str = "Claude";

/// Claude Code's profile file, which names the signed-in account.
fn profile() -> Value {
    let base = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from));
    base.map(|b| b.join(".claude.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .map(|v| v["oauthAccount"].clone())
        .unwrap_or(Value::Null)
}

/// Settings file handed to the engine so the user's hooks don't fire for our query.
/// Passed as a path (not inline JSON) so no quoting goes through cmd.exe for npm shims.
fn quiet_settings() -> Option<PathBuf> {
    let path = crate::data_dir().join("engine-settings.json");
    if !path.exists() {
        std::fs::create_dir_all(path.parent()?).ok()?;
        std::fs::write(&path, r#"{"disableAllHooks":true}"#).ok()?;
    }
    Some(path)
}

pub fn read(custom: Option<&str>) -> Reading {
    let Some(exe) = crate::locate::find("claude", custom) else {
        return Reading::missing("claude", NAME);
    };
    let settings = quiet_settings();
    let settings_arg = settings.as_ref().map(|p| p.to_string_lossy().into_owned());
    let mut args = vec![
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
    ];
    if let Some(s) = settings_arg.as_deref() {
        args.extend(["--settings", s]);
    }
    let mut engine = match Engine::start(&exe, &args, &[]) {
        Ok(e) => e,
        Err(e) => return Reading::failed("claude", NAME, e),
    };
    let request = json!({
        "type": "control_request",
        "request_id": "dwbar-usage",
        "request": {"subtype": "get_usage", "skip_behaviors": true}
    });
    if let Err(e) = engine.send(&request) {
        return Reading::failed("claude", NAME, e);
    }
    let deadline = Instant::now() + Duration::from_secs(40);
    while let Some(message) = engine.next(deadline) {
        if message["type"] == "control_response" {
            let mut reading = parse_usage(&message["response"]);
            reading.source = Some("cli".into());
            reading.account = profile()["emailAddress"].as_str().map(str::to_owned);
            return reading;
        }
    }
    let why = engine
        .stderr_preview(200)
        .unwrap_or_else(|| "Claude Code 沒有在 40 秒內回覆用量".into());
    Reading::failed("claude", NAME, why)
}

/// Parse the `response` object of a `control_response` to `get_usage`.
pub fn parse_usage(response: &Value) -> Reading {
    if response["subtype"] == "error" {
        let why = response["error"].as_str().unwrap_or("unknown error");
        return Reading::failed("claude", NAME, format!("Claude Code 查詢失敗：{why}"));
    }
    let payload = &response["response"];
    if payload["rate_limits_available"] != true {
        return Reading::failed(
            "claude",
            NAME,
            "此登入方式沒有訂閱額度（API key、第三方平台，或尚未登入）",
        );
    }
    let limits = &payload["rate_limits"];
    if !limits.is_object() {
        return Reading::failed("claude", NAME, "這次沒有讀到 Claude 額度，稍後自動重試");
    }
    let mut reading = Reading::empty("claude", NAME);
    reading.windows = windows(limits);
    if reading.windows.is_empty() {
        reading.notice = Some("Claude 回傳了無法辨識的額度資料".into());
        return reading;
    }
    reading.ok = true;
    reading.plan = payload["subscription_type"].as_str().map(str::to_owned);
    reading
}

/// The windows of a Claude usage object (`five_hour`, `seven_day`, …). The CLI's
/// `rate_limits` and claude.ai's `/usage` share this shape.
pub(crate) fn windows(limits: &Value) -> Vec<Window> {
    let window = |key: &str, period: Period, label: &str| {
        let v = &limits[key];
        let used = v["utilization"].as_f64().filter(|p| p.is_finite())?;
        let resets_at = v["resets_at"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.timestamp());
        Some(Window { period, label: label.into(), used_percent: used.clamp(0.0, 100.0), resets_at })
    };
    let mut out = Vec::new();
    out.extend(window("five_hour", Period::FiveHour, "5 小時"));
    out.extend(window("seven_day", Period::Week, "每週（全部模型）"));
    for (key, label) in [("seven_day_opus", "每週 · Opus"), ("seven_day_sonnet", "每週 · Sonnet")] {
        out.extend(window(key, Period::Other, label));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_both_windows() {
        let r = parse_usage(&json!({"subtype": "success", "response": {
            "rate_limits_available": true, "subscription_type": "max",
            "rate_limits": {
                "five_hour": {"utilization": 12.5, "resets_at": "2026-10-27T15:00:00Z"},
                "seven_day": {"utilization": 40.0},
                "seven_day_opus": {"utilization": 3.0}
            }
        }}));
        assert!(r.ok);
        assert_eq!(r.plan.as_deref(), Some("max"));
        assert_eq!(r.window(Period::FiveHour).unwrap().used_percent, 12.5);
        assert_eq!(r.window(Period::FiveHour).unwrap().resets_at, Some(1793113200));
        assert_eq!(r.window(Period::Week).unwrap().used_percent, 40.0);
        assert_eq!(r.windows.len(), 3);
    }

    #[test]
    fn no_plan_and_errors_are_notices() {
        let r = parse_usage(&json!({"subtype": "success", "response": {"rate_limits_available": false}}));
        assert!(!r.ok && r.notice.is_some());
        let r = parse_usage(&json!({"subtype": "error", "error": "boom"}));
        assert!(r.notice.unwrap().contains("boom"));
        let r = parse_usage(&json!({"subtype": "success", "response": {"rate_limits_available": true, "rate_limits": null}}));
        assert!(!r.ok);
    }
}
