//! Accounts linked inside DW Bar, and the quota sources for each AI.
//!
//! Sign-in happens on the vendor's own page in a DW Bar window (Google / Apple /
//! Microsoft pop-ups allowed), so DW Bar never handles a password. The session lives
//! in DW Bar's WebView2 profile. Usage is then read by a hidden window on the same
//! site that calls the site's own usage endpoint (see `dwbar_core::quota::web`).
//!
//! Source order per AI: linked account → CLI / engine bundled in a desktop app.
//! Everything here blocks, so call it from background threads only.
use crate::state::AppState;
use dwbar_core::{
    apps::{self, Detected},
    config::{AgentConfig, AGENTS},
    locate, log,
    quota::{
        self,
        web::{self, Outcome, Service},
        Reading,
    },
};
use serde_json::Value;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{
    webview::{NewWindowResponse, PageLoadEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

fn display_name(agent: &str) -> &'static str {
    match agent {
        "claude" => "Claude",
        "codex" => "Codex",
        "gemini" => "Gemini",
        "cursor" => "Cursor",
        "windsurf" => "Windsurf",
        "deepseek" => "DeepSeek",
        _ => "AI",
    }
}

/// The CLI that can also report this AI's usage, when there is one.
fn command(agent: &str) -> Option<&'static str> {
    match agent {
        "claude" => Some("claude"),
        "codex" => Some("codex"),
        _ => None,
    }
}

fn agent_config(app: &AppHandle, agent: &str) -> AgentConfig {
    let state = app.state::<AppState>();
    let s = state.inner.lock().unwrap();
    s.config.agent(agent).clone()
}

fn index(agent: &str) -> usize {
    AGENTS.iter().position(|a| *a == agent).unwrap_or(0)
}

// ───────────────────────── reading ─────────────────────────

static BUSY: [Mutex<()>; 6] = [const { Mutex::new(()) }; 6];

fn busy(service: &Service) -> &'static Mutex<()> {
    &BUSY[index(service.agent)]
}

/// Evaluate `js` in `win` and return its JSON result (the value, not its string form).
fn eval_json(win: &WebviewWindow, js: &str, timeout: Duration) -> Option<Value> {
    let (tx, rx) = mpsc::channel::<String>();
    win.eval_with_callback(js, move |result| {
        let _ = tx.send(result);
    })
    .ok()?;
    let raw = rx.recv_timeout(timeout).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    // JSON.stringify(...) comes back as a JSON string holding JSON.
    match value {
        Value::String(inner) => serde_json::from_str(&inner).ok(),
        other => Some(other),
    }
}

/// Read usage through the linked account. Opens and closes a hidden window.
pub fn read_web(app: &AppHandle, service: Service) -> Outcome {
    let _guard = busy(&service).lock().unwrap_or_else(|e| e.into_inner());
    let label = format!("probe-{}", service.agent);
    if let Some(old) = app.get_webview_window(&label) {
        let _ = old.destroy();
    }
    let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
    let built = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::External(service.probe_url.parse().expect("valid url")),
    )
    .title("DW Bar")
    .visible(false)
    .skip_taskbar(true)
    .focused(false)
    .inner_size(480.0, 360.0)
    .on_page_load(move |_, payload| {
        if payload.event() == PageLoadEvent::Finished {
            let _ = loaded_tx.send(());
        }
    })
    .build();
    let win = match built {
        Ok(w) => w,
        Err(e) => return Outcome::Failed(format!("無法建立讀取視窗：{e}")),
    };

    let result = (|| {
        if loaded_rx.recv_timeout(Duration::from_secs(25)).is_err() {
            return Outcome::Failed("網站沒有回應".into());
        }
        // A Cloudflare check can sit in front of the first load; retry a few times.
        let mut last = Outcome::Failed("沒有回應".into());
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(4));
            }
            if win.eval(service.script).is_err() {
                continue;
            }
            let deadline = Instant::now() + Duration::from_secs(15);
            while Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(400));
                let Some(polled) = eval_json(&win, web::POLL, Duration::from_secs(3)) else { continue };
                match web::outcome(&polled) {
                    None => continue,
                    Some(Outcome::Failed(e)) => {
                        last = Outcome::Failed(e);
                        break;
                    }
                    Some(done) => return done,
                }
            }
        }
        last
    })();
    let _ = win.destroy();
    result
}

/// Read one AI from the best source available.
pub fn read_agent(app: &AppHandle, agent: &str, detected: &Detected) -> Reading {
    let cfg = agent_config(app, agent);
    let name = display_name(agent);
    let cli = command(agent).is_some_and(|c| locate::find(c, cfg.custom_path.as_deref()).is_some());
    let app_installed = detected.has(agent);

    let from_cli = |why: Option<String>| -> Reading {
        if cli {
            quota::read_cli(agent, cfg.custom_path.as_deref())
        } else {
            Reading::failed(agent, name, why.unwrap_or_else(|| "尚未連結帳號".into()))
        }
    };

    let mut reading = if cfg.connected {
        match web::service_for(agent) {
            Some(service) => match read_web(app, service) {
                Outcome::Ok(data) => {
                    let r = web::parse(agent, &data, dwbar_core::now());
                    quota::log_reading(&r);
                    r
                }
                Outcome::SignedOut => from_cli(Some(format!("{} 登入已過期，請重新連結帳號", service.label))),
                Outcome::Failed(e) => {
                    log::write("account", &format!("{agent}: {e}"));
                    from_cli(Some(format!("暫時讀不到用量（{e}），稍後自動重試")))
                }
            },
            None => from_cli(None),
        }
    } else {
        from_cli(None)
    };

    reading.cli = cli;
    reading.app = app_installed;
    reading.connected = cfg.connected;
    reading.installed = cli || app_installed || cfg.connected;
    if !reading.ok && !cfg.connected && reading.notice.as_deref() != Some("尚未連結帳號") {
        let n = reading.notice.take().unwrap_or_default();
        reading.notice = Some(format!("{n}。也可以直接連結帳號。"));
    }
    reading
}

/// Read every enabled AI, in parallel.
pub fn read_all(app: &AppHandle) -> Vec<Reading> {
    let detected = apps::detect();
    let enabled: Vec<&str> = {
        let state = app.state::<AppState>();
        let s = state.inner.lock().unwrap();
        AGENTS.into_iter().filter(|a| s.config.agent(a).enabled).collect()
    };
    let jobs: Vec<_> = enabled
        .into_iter()
        .map(|agent| {
            let app = app.clone();
            let detected = detected.clone();
            std::thread::spawn(move || read_agent(&app, agent, &detected))
        })
        .collect();
    jobs.into_iter().filter_map(|j| j.join().ok()).collect()
}

// ───────────────────────── linking ─────────────────────────

static LINKING: [AtomicBool; 6] = [const { AtomicBool::new(false) }; 6];

fn linking(service: &Service) -> &'static AtomicBool {
    &LINKING[index(service.agent)]
}

fn login_label(service: &Service) -> String {
    format!("login-{}", service.agent)
}

/// Open the vendor's sign-in page. When the window leaves the sign-in flow, the
/// account is verified by reading usage once, then the window closes by itself.
pub fn open_login(app: &AppHandle, agent: &str) {
    let Some(service) = web::service_for(agent) else { return };
    let label = login_label(&service);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let app = app.clone();
    crate::off_ui_thread(move || build_login(&app, service, &label));
}

fn build_login(app: &AppHandle, service: Service, label: &str) {
    let built = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::External(service.login_url.parse().expect("valid url")),
    )
    .title(format!("登入 {} · DW Bar", service.label))
    .inner_size(500.0, 760.0)
    .center()
    // Google / Apple / Microsoft sign-in opens pop-ups; let them share this session.
    .on_new_window(|_, _| NewWindowResponse::Allow)
    .build();
    match built {
        Ok(_) => {
            log::write("account", &format!("sign-in window opened for {}", service.label));
            let app = app.clone();
            std::thread::spawn(move || watch_login(app, service));
        }
        Err(e) => log::write("account", &format!("sign-in window failed: {e}")),
    }
}

/// Poll the sign-in window's location (covers single-page apps that never reload).
fn watch_login(app: AppHandle, service: Service) {
    let label = login_label(&service);
    let flag = linking(&service);
    // Some sites show a public page on the same address before sign-in (Gemini), so a
    // failed check is only repeated when the page changes, or every 15 s.
    let mut last_try: Option<(String, Instant)> = None;
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let Some(win) = app.get_webview_window(&label) else { return };
        let Some(loc) = eval_json(&win, "JSON.stringify([location.host, location.pathname])", Duration::from_secs(3)) else {
            continue;
        };
        let host = loc[0].as_str().unwrap_or_default();
        let path = loc[1].as_str().unwrap_or_default();
        if !web::looks_signed_in(&service, host, path) {
            continue;
        }
        let here = format!("{host}{path}");
        if last_try.as_ref().is_some_and(|(at, when)| *at == here && when.elapsed() < Duration::from_secs(15)) {
            continue;
        }
        if flag.swap(true, Ordering::SeqCst) {
            continue;
        }
        last_try = Some((here, Instant::now()));
        let linked = finish_link(&app, service);
        flag.store(false, Ordering::SeqCst);
        if linked {
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.close();
            }
            return;
        }
    }
}

/// Verify the session by reading usage; on success remember the link and show the numbers.
fn finish_link(app: &AppHandle, service: Service) -> bool {
    match read_web(app, service) {
        Outcome::Ok(data) => {
            let mut reading = web::parse(service.agent, &data, dwbar_core::now());
            reading.installed = true;
            let config = {
                let state = app.state::<AppState>();
                let mut s = state.inner.lock().unwrap();
                let entry = s.config.agent_mut(service.agent);
                entry.connected = true;
                entry.enabled = true;
                reading.cli = entry.custom_path.is_some();
                s.quotas.retain(|q| q.agent != service.agent);
                s.quotas.push(reading.clone());
                s.quotas.sort_by_key(|q| AGENTS.iter().position(|a| *a == q.agent));
                s.config.clone()
            };
            if let Err(e) = config.save() {
                log::write("account", &format!("save failed: {e}"));
            }
            log::write("account", &format!("{} linked", service.label));
            crate::broadcast(app);
            let _ = app.emit("account-linked", service.agent);
            crate::request_refresh(app);
            true
        }
        Outcome::SignedOut => false,
        Outcome::Failed(e) => {
            log::write("account", &format!("{} link check failed: {e}", service.label));
            false
        }
    }
}

/// Forget the link. The session stays in the profile until "清除登入資料".
pub fn unlink(app: &AppHandle, agent: &str) {
    let config = {
        let state = app.state::<AppState>();
        let mut s = state.inner.lock().unwrap();
        s.config.agent_mut(agent).connected = false;
        s.config.clone()
    };
    let _ = config.save();
    log::write("account", &format!("{agent} unlinked"));
}

/// Sign out everywhere: wipe DW Bar's browser profile (cookies, storage).
pub fn clear_sessions(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(crate::SETTINGS).or_else(|| app.get_webview_window(crate::CAPSULE)) {
        if let Err(e) = win.clear_all_browsing_data() {
            log::write("account", &format!("clear browsing data failed: {e}"));
        }
    }
    for agent in AGENTS {
        unlink(app, agent);
    }
}
