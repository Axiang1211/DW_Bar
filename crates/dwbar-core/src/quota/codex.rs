//! Codex: ask `codex app-server` over JSON-RPC (stdio).
use super::{engine::Engine, period_for_minutes, Period, Reading, Window};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const NAME: &str = "Codex";

pub fn read(custom: Option<&str>) -> Reading {
    let Some(exe) = crate::locate::find("codex", custom) else {
        return Reading::missing("codex", NAME);
    };
    let mut engine = match Engine::start(&exe, &["app-server", "--listen", "stdio://"], &[]) {
        Ok(e) => e,
        Err(e) => return Reading::failed("codex", NAME, e),
    };
    let init = json!({"id": 1, "method": "initialize", "params": {
        "clientInfo": {"name": "dw_bar", "title": "DW Bar", "version": env!("CARGO_PKG_VERSION")}
    }});
    if let Err(e) = engine.send(&init) {
        return Reading::failed("codex", NAME, e);
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut limits: Option<Value> = None;
    let mut refused: Option<String> = None;
    let mut account = Value::Null;
    let mut account_done = false;
    while let Some(msg) = engine.next(deadline) {
        let Some(id) = msg["id"].as_u64() else { continue };
        if let Some(err) = msg["error"].as_object() {
            let text = err.get("message").and_then(Value::as_str).unwrap_or("app-server error");
            match id {
                2 => refused = Some(text.to_owned()),
                3 => account_done = true, // older CLIs lack account/read; limits still work
                _ => return Reading::failed("codex", NAME, format!("Codex：{text}")),
            }
        } else if let Some(result) = msg.get("result") {
            match id {
                1 => {
                    let _ = engine.send(&json!({"method": "initialized"}));
                    let _ = engine.send(&json!({"id": 2, "method": "account/rateLimits/read"}));
                    let _ = engine.send(&json!({"id": 3, "method": "account/read", "params": {}}));
                }
                2 => limits = Some(result.clone()),
                3 => {
                    account = result["account"].clone();
                    account_done = true;
                }
                _ => {}
            }
        }
        if (limits.is_some() || refused.is_some()) && account_done {
            break;
        }
    }

    match (limits, refused) {
        (Some(l), _) => {
            let mut r = parse_limits(&l);
            r.account = account["email"].as_str().map(str::to_owned);
            if r.plan.is_none() {
                r.plan = account["planType"].as_str().map(str::to_owned);
            }
            r
        }
        (None, Some(why)) => {
            let notice = if account.is_null() {
                "Codex 尚未用 ChatGPT 帳號登入（執行 codex login）".to_string()
            } else if account["type"] == "apiKey" {
                "此登入方式（API key）沒有訂閱額度".to_string()
            } else {
                format!("Codex：{why}")
            };
            Reading::failed("codex", NAME, notice)
        }
        (None, None) => {
            let why = engine
                .stderr_preview(200)
                .unwrap_or_else(|| "Codex 沒有在 30 秒內回覆用量".into());
            Reading::failed("codex", NAME, why)
        }
    }
}

/// Parse the result of `account/rateLimits/read`.
/// Uses the `codex` bucket of `rateLimitsByLimitId` (or the legacy single `rateLimits`).
pub fn parse_limits(result: &Value) -> Reading {
    let bucket = result["rateLimitsByLimitId"]
        .as_object()
        .and_then(|m| m.get("codex").or_else(|| m.values().next()))
        .or_else(|| result["rateLimits"].is_object().then(|| &result["rateLimits"]));
    let mut reading = Reading::empty("codex", NAME);
    let Some(bucket) = bucket else {
        reading.notice = Some("Codex 沒有回傳額度資料".into());
        return reading;
    };
    for (slot, fallback) in [("primary", Period::FiveHour), ("secondary", Period::Week)] {
        let w = &bucket[slot];
        let Some(used) = w["usedPercent"].as_f64().filter(|p| p.is_finite()) else { continue };
        let minutes = w["windowDurationMins"].as_i64();
        let period = match period_for_minutes(minutes) {
            Period::Other if minutes.is_none() => fallback,
            p => p,
        };
        let label = match period {
            Period::FiveHour => "5 小時".to_string(),
            Period::Week => "每週".to_string(),
            Period::Other => minutes.map(|m| format!("{} 小時", m / 60)).unwrap_or_default(),
        };
        reading.windows.push(Window {
            period,
            label,
            used_percent: used.clamp(0.0, 100.0),
            resets_at: w["resetsAt"].as_i64(),
        });
    }
    reading.plan = bucket["planType"].as_str().map(str::to_owned);
    let credits = &bucket["credits"];
    if credits["unlimited"] == true {
        reading.balance = Some("無限".into());
    } else {
        reading.balance = match &credits["balance"] {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        };
    }
    reading.ok = !reading.windows.is_empty();
    if !reading.ok {
        reading.notice = Some("Codex 回傳了無法辨識的額度資料".into());
    }
    reading
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(used: f64, mins: i64) -> Value {
        json!({"usedPercent": used, "windowDurationMins": mins, "resetsAt": 1_792_000_000})
    }

    #[test]
    fn codex_bucket_is_used() {
        let r = parse_limits(&json!({"rateLimitsByLimitId": {
            "gpt-reserve": {"primary": w(1.0, 300)},
            "codex": {"primary": w(10.0, 300), "secondary": w(20.0, 10080), "planType": "pro",
                      "credits": {"hasCredits": true, "unlimited": false, "balance": "12.50"}}
        }}));
        assert!(r.ok);
        assert_eq!(r.plan.as_deref(), Some("pro"));
        assert_eq!(r.window(Period::FiveHour).unwrap().used_percent, 10.0);
        assert_eq!(r.window(Period::Week).unwrap().used_percent, 20.0);
        assert_eq!(r.window(Period::Week).unwrap().resets_at, Some(1_792_000_000));
        assert_eq!(r.balance.as_deref(), Some("12.50"));
    }

    #[test]
    fn legacy_single_bucket() {
        let r = parse_limits(&json!({"rateLimits": {"primary": w(55.0, 300)}}));
        assert!(r.ok);
        assert_eq!(r.windows.len(), 1);
        assert!(r.window(Period::Week).is_none());
    }

    #[test]
    fn empty_is_a_notice() {
        let r = parse_limits(&json!({}));
        assert!(!r.ok && r.notice.is_some());
    }
}
