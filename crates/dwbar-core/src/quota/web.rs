//! Usage read through an account the user signed in to *inside DW Bar*.
//!
//! The app opens the vendor's own sign-in page in a DW Bar window. Afterwards a hidden
//! window on the same site runs [`Service::script`], which calls the site's own usage
//! endpoint with the user's session (same origin, so nothing leaves the vendor's site and
//! DW Bar never sees a password). This module holds the pure parts: which pages, which
//! script, and how to turn the answer into a [`Reading`].
//!
//! These are the endpoints the vendors' own web apps use; they are not public APIs and
//! may change, so every field is read defensively.
use super::{claude, codex, period_for_minutes, Period, Reading, Window};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Service {
    /// The agent this account feeds (`"claude"` / `"codex"`).
    pub agent: &'static str,
    /// Shown in the sign-in window title and buttons.
    pub label: &'static str,
    pub host: &'static str,
    pub login_url: &'static str,
    /// A small same-origin page the hidden window loads before running the script.
    pub probe_url: &'static str,
    pub script: &'static str,
}

/// The script stores its answer in `window.__dwbar`; the app polls [`POLL`] until `done`.
pub const POLL: &str = "JSON.stringify(window.__dwbar || null)";

const CLAUDE_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  const get = async (path) => {
    const r = await fetch(path, { credentials: "include", headers: { accept: "application/json" } });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    return r.json();
  };
  try {
    const orgs = await get("/api/organizations");
    if (!Array.isArray(orgs) || !orgs.length) throw new Error("not_signed_in");
    const org = orgs.find((o) => (o.capabilities || []).includes("chat")) || orgs[0];
    const usage = await get("/api/organizations/" + org.uuid + "/usage");
    let email = null;
    try { const a = await get("/api/account"); email = a.email_address || a.email || null; } catch (_) {}
    window.__dwbar = { done: true, ok: true, data: { usage, capabilities: org.capabilities || [], email } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

const CHATGPT_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  try {
    const sr = await fetch("/api/auth/session", { credentials: "include" });
    const session = sr.ok ? await sr.json() : {};
    if (!session || !session.accessToken) throw new Error("not_signed_in");
    const headers = { authorization: "Bearer " + session.accessToken, accept: "application/json" };
    const account = session.account || {};
    if (account.id) headers["chatgpt-account-id"] = account.id;
    const r = await fetch("/backend-api/wham/usage", { credentials: "include", headers });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    const usage = await r.json();
    window.__dwbar = { done: true, ok: true, data: {
      usage, email: (session.user && session.user.email) || null, plan: account.planType || null } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

pub const CLAUDE: Service = Service {
    agent: "claude",
    label: "Claude",
    host: "claude.ai",
    login_url: "https://claude.ai/login",
    probe_url: "https://claude.ai/api/organizations",
    script: CLAUDE_SCRIPT,
};

pub const CHATGPT: Service = Service {
    agent: "codex",
    label: "ChatGPT",
    host: "chatgpt.com",
    login_url: "https://chatgpt.com/auth/login",
    probe_url: "https://chatgpt.com/api/auth/session",
    script: CHATGPT_SCRIPT,
};

pub fn service_for(agent: &str) -> Option<Service> {
    match agent {
        "claude" => Some(CLAUDE),
        "codex" => Some(CHATGPT),
        _ => None,
    }
}

/// Whether a page the sign-in window landed on means "signed in" (left the login flow).
pub fn looks_signed_in(service: &Service, host: &str, path: &str) -> bool {
    let host_ok = host == service.host || host.ends_with(&format!(".{}", service.host));
    let in_flow = ["/login", "/auth", "/logout", "/magic-link", "/sso", "/verify", "/api/"]
        .iter()
        .any(|p| path.starts_with(p));
    host_ok && !in_flow
}

/// What the script reported.
pub enum Outcome {
    Ok(Value),
    SignedOut,
    Failed(String),
}

pub fn outcome(poll_result: &Value) -> Option<Outcome> {
    if poll_result["done"] != true {
        return None;
    }
    if poll_result["ok"] == true {
        return Some(Outcome::Ok(poll_result["data"].clone()));
    }
    let err = poll_result["error"].as_str().unwrap_or("unknown");
    Some(if err == "not_signed_in" { Outcome::SignedOut } else { Outcome::Failed(err.to_owned()) })
}

/// claude.ai `/api/organizations/{id}/usage` (+ org capabilities, account email).
pub fn parse_claude(data: &Value) -> Reading {
    let mut r = Reading::empty("claude", claude::NAME);
    r.source = Some("account".into());
    r.connected = true;
    r.windows = claude::windows(&data["usage"]);
    let caps: Vec<&str> = data["capabilities"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    r.plan = if caps.contains(&"claude_max") {
        Some("max".into())
    } else if caps.contains(&"claude_pro") {
        Some("pro".into())
    } else if caps.iter().any(|c| c.contains("team") || *c == "raven") {
        Some("team".into())
    } else {
        None
    };
    r.account = data["email"].as_str().map(str::to_owned);
    let extra = &data["usage"]["extra_usage"];
    if extra["is_enabled"] == true {
        if let Some(u) = extra["utilization"].as_f64() {
            r.balance = Some(format!("額外用量剩 {:.0}%", (100.0 - u).clamp(0.0, 100.0)));
        }
    }
    r.ok = !r.windows.is_empty();
    if !r.ok {
        r.notice = Some("Claude 回傳了無法辨識的額度資料".into());
    }
    r
}

/// chatgpt.com `/backend-api/wham/usage` – the Codex limits of a ChatGPT plan.
pub fn parse_chatgpt(data: &Value, now: i64) -> Reading {
    let usage = &data["usage"];
    let mut r = Reading::empty("codex", codex::NAME);
    r.source = Some("account".into());
    r.connected = true;
    for (slot, fallback, label) in [
        ("primary_window", Period::FiveHour, "5 小時"),
        ("secondary_window", Period::Week, "每週"),
    ] {
        let w = &usage["rate_limit"][slot];
        let Some(used) = w["used_percent"].as_f64().filter(|p| p.is_finite()) else { continue };
        let minutes = w["limit_window_seconds"].as_i64().map(|s| s / 60);
        let period = match period_for_minutes(minutes) {
            Period::Other if minutes.is_none() => fallback,
            p => p,
        };
        let label = match period {
            Period::Other => minutes.map(|m| format!("{} 小時", m / 60)).unwrap_or_else(|| label.into()),
            Period::FiveHour => "5 小時".into(),
            Period::Week => "每週".into(),
        };
        let resets_at = w["reset_at"]
            .as_i64()
            .or_else(|| w["reset_after_seconds"].as_i64().map(|s| now + s));
        r.windows.push(Window { period, label, used_percent: used.clamp(0.0, 100.0), resets_at });
    }
    r.plan = usage["plan_type"].as_str().or_else(|| data["plan"].as_str()).map(str::to_owned);
    r.account = data["email"].as_str().or_else(|| usage["email"].as_str()).map(str::to_owned);
    let credits = &usage["credits"];
    if credits["unlimited"] == true {
        r.balance = Some("無限".into());
    } else {
        r.balance = match &credits["balance"] {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        };
    }
    r.ok = !r.windows.is_empty();
    if !r.ok {
        r.notice = Some("ChatGPT 帳號沒有 Codex 額度資料（可能是免費方案）".into());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn claude_web_usage() {
        let r = parse_claude(&json!({
            "usage": {
                "five_hour": {"utilization": 23.0, "resets_at": "2026-10-29T08:00:00.123+00:00"},
                "seven_day": {"utilization": 51.0, "resets_at": "2026-11-02T04:00:00+00:00"},
                "seven_day_opus": null,
                "extra_usage": {"is_enabled": true, "utilization": 10.0}
            },
            "capabilities": ["chat", "claude_max"],
            "email": "me@example.com"
        }));
        assert!(r.ok);
        assert_eq!(r.source.as_deref(), Some("account"));
        assert_eq!(r.plan.as_deref(), Some("max"));
        assert_eq!(r.window(Period::FiveHour).unwrap().used_percent, 23.0);
        assert!(r.window(Period::FiveHour).unwrap().resets_at.is_some());
        assert_eq!(r.windows.len(), 2);
        assert_eq!(r.balance.as_deref(), Some("額外用量剩 90%"));
    }

    #[test]
    fn chatgpt_web_usage() {
        let r = parse_chatgpt(
            &json!({"usage": {
                "plan_type": "plus",
                "rate_limit": {
                    "primary_window": {"used_percent": 30, "limit_window_seconds": 18000, "reset_after_seconds": 600},
                    "secondary_window": {"used_percent": 70, "limit_window_seconds": 604800, "reset_at": 1_792_000_000}
                },
                "credits": {"has_credits": true, "unlimited": false, "balance": "5.00"}
            }, "email": "a@b.c"}),
            1000,
        );
        assert!(r.ok);
        assert_eq!(r.plan.as_deref(), Some("plus"));
        assert_eq!(r.window(Period::FiveHour).unwrap().resets_at, Some(1600));
        assert_eq!(r.window(Period::Week).unwrap().used_percent, 70.0);
        assert_eq!(r.balance.as_deref(), Some("5.00"));
        assert_eq!(r.account.as_deref(), Some("a@b.c"));
    }

    #[test]
    fn signed_in_detection() {
        assert!(looks_signed_in(&CLAUDE, "claude.ai", "/new"));
        assert!(!looks_signed_in(&CLAUDE, "claude.ai", "/login"));
        assert!(!looks_signed_in(&CLAUDE, "accounts.google.com", "/"));
        assert!(looks_signed_in(&CHATGPT, "chatgpt.com", "/"));
        assert!(!looks_signed_in(&CHATGPT, "auth.openai.com", "/log-in"));
        assert!(!looks_signed_in(&CHATGPT, "chatgpt.com", "/auth/login"));
    }

    #[test]
    fn outcomes() {
        assert!(outcome(&json!(null)).is_none());
        assert!(matches!(outcome(&json!({"done": true, "ok": false, "error": "not_signed_in"})), Some(Outcome::SignedOut)));
        assert!(matches!(outcome(&json!({"done": true, "ok": true, "data": {}})), Some(Outcome::Ok(_))));
    }
}
