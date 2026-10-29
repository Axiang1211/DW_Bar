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
    /// Paths on `host` that are still part of signing in.
    pub flow: &'static [&'static str],
}

const DEFAULT_FLOW: &[&str] = &["/login", "/auth", "/logout", "/magic-link", "/sso", "/verify", "/api/", "/sign-in", "/signin", "/account/login", "/account/register"];

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
    flow: DEFAULT_FLOW,
};

pub const CHATGPT: Service = Service {
    agent: "codex",
    label: "ChatGPT",
    host: "chatgpt.com",
    login_url: "https://chatgpt.com/auth/login",
    probe_url: "https://chatgpt.com/api/auth/session",
    script: CHATGPT_SCRIPT,
    flow: DEFAULT_FLOW,
};

// Gemini app: the persisted UsagePageQuery that gemini.google.com/usage runs.
const GEMINI_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  try {
    const sig = "2/dWbCvkVB0X16r9NPnxifeOAAqdCqMf0xeWvau9yPy7g=";
    const vars = JSON.stringify({
      g1Request: { entrypoints: ["GEMINI_USAGE_SETTINGS_UPGRADE_BANNER"] },
      workspaceRequest: { recommendationsForSurfaceRequests: [] },
    });
    const url = "/_/BardChatUi/graphql/schemas/GEMINI_WEB_GRAPHQL/executeQuery?query_signature=" +
      encodeURIComponent(sig) + "&variables=" + encodeURIComponent(vars);
    const r = await fetch(url, { credentials: "include", headers: { accept: "application/json" } });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    let text = (await r.text()).trim();
    if (text.startsWith("<")) throw new Error("not_signed_in");
    text = text.replace(/^\)\]\}'\s*/, "");
    const parsed = JSON.parse(text);
    const list = Array.isArray(parsed) ? parsed : [parsed];
    const data = list.map((x) => x && x.data).filter(Boolean).find((d) => "viewer" in d);
    if (!data) throw new Error("unrecognised_response");
    if (data.viewer === null) throw new Error("not_signed_in");
    const usage = data.viewer.rpcs && data.viewer.rpcs.getUsageInfo;
    if (!usage) throw new Error("unrecognised_response");
    window.__dwbar = { done: true, ok: true, data: { usage } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

pub const GEMINI: Service = Service {
    agent: "gemini",
    label: "Google",
    host: "gemini.google.com",
    login_url: "https://gemini.google.com/usage",
    probe_url: "https://gemini.google.com/robots.txt",
    script: GEMINI_SCRIPT,
    flow: DEFAULT_FLOW,
};

const CURSOR_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  const get = async (path) => {
    const r = await fetch(path, { credentials: "include", headers: { accept: "application/json" } });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    return r.json();
  };
  try {
    const summary = await get("/api/usage-summary");
    let email = null;
    try { const me = await get("/api/auth/me"); email = me.email || null; } catch (_) {}
    window.__dwbar = { done: true, ok: true, data: { summary, email } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

pub const CURSOR: Service = Service {
    agent: "cursor",
    label: "Cursor",
    host: "cursor.com",
    login_url: "https://cursor.com/dashboard",
    probe_url: "https://cursor.com/robots.txt",
    script: CURSOR_SCRIPT,
    flow: DEFAULT_FLOW,
};

// Windsurf: ConnectRPC GetPlanStatus over protobuf, with the devin_* session from localStorage.
const WINDSURF_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  try {
    const read = (k) => { const v = localStorage.getItem(k); if (!v) return null; try { return JSON.parse(v); } catch (_) { return v; } };
    const token = read("devin_session_token");
    if (!token) throw new Error("not_signed_in");
    const bytes = new TextEncoder().encode(String(token));
    const len = []; let n = bytes.length;
    while (n >= 0x80) { len.push((n & 0x7f) | 0x80); n = Math.floor(n / 128); }
    len.push(n);
    const body = new Uint8Array([0x0a, ...len, ...bytes, 0x10, 0x01]);
    const r = await fetch("/_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus", {
      method: "POST", credentials: "include", body,
      headers: {
        "content-type": "application/proto", "connect-protocol-version": "1",
        "x-auth-token": String(token), "x-devin-session-token": String(token),
        "x-devin-auth1-token": String(read("devin_auth1_token") || ""),
        "x-devin-account-id": String(read("devin_account_id") || ""),
        "x-devin-primary-org-id": String(read("devin_primary_org_id") || ""),
      },
    });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    const buf = new Uint8Array(await r.arrayBuffer());
    // Minimal protobuf reader: returns { field: [values] }, nested messages as Uint8Array.
    const decode = (b) => {
      const out = {}; let i = 0;
      const varint = () => { let x = 0, m = 1, c; do { c = b[i++]; x += (c & 0x7f) * m; m *= 128; } while (c & 0x80); return x; };
      while (i < b.length) {
        const key = varint(), f = Math.floor(key / 8), t = key % 8; let v;
        if (t === 0) v = varint();
        else if (t === 2) { const l = varint(); v = b.subarray(i, i + l); i += l; }
        else if (t === 1) { i += 8; continue; }
        else if (t === 5) { i += 4; continue; }
        else break;
        (out[f] = out[f] || []).push(v);
      }
      return out;
    };
    const top = decode(buf);
    const status = top[1] ? decode(top[1][0]) : {};
    const info = status[1] ? decode(status[1][0]) : {};
    const num = (f) => (status[f] ? status[f][0] : null);
    window.__dwbar = { done: true, ok: true, data: {
      plan: info[2] ? new TextDecoder().decode(info[2][0]) : null,
      daily_remaining: num(14), weekly_remaining: num(15),
      daily_reset: num(17), weekly_reset: num(18),
    } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

pub const WINDSURF: Service = Service {
    agent: "windsurf",
    label: "Windsurf",
    host: "windsurf.com",
    login_url: "https://windsurf.com/profile",
    probe_url: "https://windsurf.com/robots.txt",
    script: WINDSURF_SCRIPT,
    flow: DEFAULT_FLOW,
};

// DeepSeek: API credit balance of the DeepSeek Platform account (what DSH runs on).
const DEEPSEEK_SCRIPT: &str = r#"
window.__dwbar = null;
(async () => {
  try {
    const raw = localStorage.getItem("userToken");
    if (!raw) throw new Error("not_signed_in");
    let token = raw;
    try {
      const j = JSON.parse(raw);
      token = typeof j === "string" ? j : (j.value || j.token || j.access_token || j.accessToken || j.userToken || raw);
    } catch (_) {}
    const r = await fetch("/api/v0/users/get_user_summary", {
      credentials: "include",
      headers: { authorization: "Bearer " + token, accept: "application/json", "x-client-platform": "web" },
    });
    if (r.status === 401 || r.status === 403) throw new Error("not_signed_in");
    if (!r.ok) throw new Error("http_" + r.status);
    const j = await r.json();
    const biz = j && j.data && j.data.biz_data;
    if (!biz) throw new Error("not_signed_in");
    window.__dwbar = { done: true, ok: true, data: { summary: biz } };
  } catch (e) {
    window.__dwbar = { done: true, ok: false, error: String((e && e.message) || e) };
  }
})();
"#;

pub const DEEPSEEK: Service = Service {
    agent: "deepseek",
    label: "DeepSeek",
    host: "platform.deepseek.com",
    login_url: "https://platform.deepseek.com/usage",
    probe_url: "https://platform.deepseek.com/robots.txt",
    script: DEEPSEEK_SCRIPT,
    flow: DEFAULT_FLOW,
};

pub fn service_for(agent: &str) -> Option<Service> {
    match agent {
        "claude" => Some(CLAUDE),
        "codex" => Some(CHATGPT),
        "gemini" => Some(GEMINI),
        "cursor" => Some(CURSOR),
        "windsurf" => Some(WINDSURF),
        "deepseek" => Some(DEEPSEEK),
        _ => None,
    }
}

/// Turn what an agent's script returned into a reading.
pub fn parse(agent: &str, data: &Value, now: i64) -> Reading {
    match agent {
        "claude" => parse_claude(data),
        "codex" => parse_chatgpt(data, now),
        "gemini" => parse_gemini(data),
        "cursor" => parse_cursor(data),
        "windsurf" => parse_windsurf(data),
        _ => parse_deepseek(data),
    }
}

fn base(agent: &str, name: &str) -> Reading {
    let mut r = Reading::empty(agent, name);
    r.source = Some("account".into());
    r.connected = true;
    r
}

fn iso(v: &Value) -> Option<i64> {
    v.as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp())
}

/// gemini.google.com UsagePageQuery → `getUsageInfo` (`percentageUsed` is a fraction).
pub fn parse_gemini(data: &Value) -> Reading {
    let mut r = base("gemini", "Gemini");
    let pools = data["usage"]["pools"].as_array().cloned().unwrap_or_default();
    for (kind, period, label) in [("GXU_FIVE_HOURLY", Period::FiveHour, "5 小時"), ("GXU_WEEKLY", Period::Week, "每週")] {
        let Some(pool) = pools.iter().find(|p| p["poolType"] == kind) else { continue };
        let Some(frac) = pool["percentageUsed"].as_f64().filter(|f| f.is_finite() && *f >= 0.0) else { continue };
        r.windows.push(Window {
            period,
            label: label.into(),
            used_percent: (frac * 100.0).min(100.0),
            resets_at: iso(&pool["gxu"]["windowRefreshesAt"]),
        });
    }
    r.plan = match data["usage"]["subscriptionTier"].as_str() {
        Some("TIER1") => Some("Pro".into()),
        Some("TIER2") => Some("Ultra".into()),
        _ => None,
    };
    r.ok = !r.windows.is_empty();
    if !r.ok {
        r.notice = Some("這個 Google 帳號沒有 Gemini 用量資料".into());
    }
    r
}

/// cursor.com `/api/usage-summary` (+ `/api/auth/me`). Money values are in cents.
pub fn parse_cursor(data: &Value) -> Reading {
    let mut r = base("cursor", "Cursor");
    let s = &data["summary"];
    let plan = &s["individualUsage"]["plan"];
    let pct = |v: &Value| v.as_f64().filter(|p| p.is_finite());
    let used = plan["totalPercentUsed"].as_f64().or_else(|| {
        match (pct(&plan["autoPercentUsed"]), pct(&plan["apiPercentUsed"])) {
            (Some(a), Some(b)) => Some((a + b) / 2.0),
            (a, b) => a.or(b),
        }
    }).or_else(|| {
        let (u, l) = (plan["used"].as_f64()?, plan["limit"].as_f64()?);
        (l > 0.0).then(|| u / l * 100.0)
    }).or_else(|| {
        let o = &s["individualUsage"]["overall"];
        let (u, l) = (o["used"].as_f64()?, o["limit"].as_f64()?);
        (l > 0.0).then(|| u / l * 100.0)
    });
    if let Some(u) = used {
        r.windows.push(Window {
            period: Period::Month,
            label: "本期方案額度".into(),
            used_percent: u.clamp(0.0, 100.0),
            resets_at: iso(&s["billingCycleEnd"]),
        });
    }
    let od = &s["individualUsage"]["onDemand"];
    if let (Some(u), Some(l)) = (od["used"].as_f64(), od["limit"].as_f64()) {
        if l > 0.0 {
            r.balance = Some(format!("按量剩 ${:.2}", ((l - u) / 100.0).max(0.0)));
        }
    }
    r.plan = s["membershipType"].as_str().map(str::to_owned);
    r.account = data["email"].as_str().map(str::to_owned);
    r.ok = !r.windows.is_empty();
    if !r.ok {
        r.notice = Some("Cursor 回傳了無法辨識的用量資料".into());
    }
    r
}

/// windsurf.com GetPlanStatus, decoded by the page script.
pub fn parse_windsurf(data: &Value) -> Reading {
    let mut r = base("windsurf", "Windsurf");
    for (key, reset, period, label) in [
        ("daily_remaining", "daily_reset", Period::Day, "每日"),
        ("weekly_remaining", "weekly_reset", Period::Week, "每週"),
    ] {
        let Some(left) = data[key].as_f64() else { continue };
        r.windows.push(Window {
            period,
            label: label.into(),
            used_percent: (100.0 - left).clamp(0.0, 100.0),
            resets_at: data[reset].as_i64().filter(|t| *t > 0),
        });
    }
    r.plan = data["plan"].as_str().filter(|s| !s.is_empty()).map(str::to_owned);
    r.ok = !r.windows.is_empty();
    if !r.ok {
        r.notice = Some("Windsurf 沒有回傳額度（可能是試用或團隊方案）".into());
    }
    r
}

/// platform.deepseek.com `get_user_summary`: API credit wallets (no time windows).
pub fn parse_deepseek(data: &Value) -> Reading {
    let mut r = base("deepseek", "DeepSeek");
    let mut totals: Vec<(String, f64)> = Vec::new();
    for key in ["normal_wallets", "bonus_wallets"] {
        for w in data["summary"][key].as_array().into_iter().flatten() {
            let amount = w["balance"].as_f64().or_else(|| w["balance"].as_str().and_then(|s| s.parse().ok()));
            let (Some(amount), Some(cur)) = (amount, w["currency"].as_str()) else { continue };
            match totals.iter_mut().find(|(c, _)| c == cur) {
                Some((_, t)) => *t += amount,
                None => totals.push((cur.to_string(), amount)),
            }
        }
    }
    if totals.is_empty() {
        r.notice = Some("DeepSeek 沒有回傳餘額".into());
        return r;
    }
    r.balance = Some(
        totals
            .iter()
            .map(|(c, t)| format!("{}{:.2}", if c == "CNY" { "¥" } else { "$" }, t))
            .collect::<Vec<_>>()
            .join(" / "),
    );
    r.plan = Some("API".into());
    r.ok = true;
    r
}

/// Whether a page the sign-in window landed on means "signed in" (left the login flow).
pub fn looks_signed_in(service: &Service, host: &str, path: &str) -> bool {
    let host_ok = host == service.host || host.ends_with(&format!(".{}", service.host));
    let in_flow = service.flow.iter().any(|p| path.starts_with(p));
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
            Period::Day => "每日".into(),
            Period::Month => "每月".into(),
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
    fn gemini_usage() {
        let r = parse_gemini(&json!({"usage": {"subscriptionTier": "TIER1", "pools": [
            {"poolType": "GXU_FIVE_HOURLY", "percentageUsed": 0.25, "gxu": {"windowRefreshesAt": "2026-10-29T10:00:00Z"}},
            {"poolType": "GXU_WEEKLY", "percentageUsed": 0.5}
        ]}}));
        assert!(r.ok);
        assert_eq!(r.window(Period::FiveHour).unwrap().used_percent, 25.0);
        assert_eq!(r.window(Period::Week).unwrap().used_percent, 50.0);
        assert_eq!(r.plan.as_deref(), Some("Pro"));
    }

    #[test]
    fn cursor_usage() {
        let r = parse_cursor(&json!({"summary": {
            "billingCycleEnd": "2026-11-15T00:00:00.000Z", "membershipType": "pro",
            "individualUsage": {"plan": {"used": 1000, "limit": 2000, "totalPercentUsed": 42.5},
                                "onDemand": {"used": 250, "limit": 1000}}
        }, "email": "x@y.z"}));
        assert!(r.ok);
        assert_eq!(r.window(Period::Month).unwrap().used_percent, 42.5);
        assert_eq!(r.balance.as_deref(), Some("按量剩 $7.50"));
        let r = parse_cursor(&json!({"summary": {"individualUsage": {"plan": {"used": 500, "limit": 2000}}}}));
        assert_eq!(r.window(Period::Month).unwrap().used_percent, 25.0);
    }

    #[test]
    fn windsurf_usage() {
        let r = parse_windsurf(&json!({"plan": "Pro", "daily_remaining": 80, "weekly_remaining": 35, "daily_reset": 1_793_300_000, "weekly_reset": 0}));
        assert!(r.ok);
        assert_eq!(r.window(Period::Day).unwrap().used_percent, 20.0);
        assert_eq!(r.window(Period::Week).unwrap().used_percent, 65.0);
        assert_eq!(r.window(Period::Week).unwrap().resets_at, None);
    }

    #[test]
    fn deepseek_balance() {
        let r = parse_deepseek(&json!({"summary": {
            "normal_wallets": [{"balance": "10.50", "currency": "CNY"}],
            "bonus_wallets": [{"balance": 2.0, "currency": "CNY"}]
        }}));
        assert!(r.ok);
        assert_eq!(r.balance.as_deref(), Some("¥12.50"));
        assert!(r.windows.is_empty());
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
