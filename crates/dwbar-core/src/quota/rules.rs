//! Online reading rules (線上規則).
//!
//! When an AI site changes, DW Bar can be fixed without a new release: the site publishes
//! `rules.json` + `rules.json.sig` (minisign, the same key as the app updater). A rule may
//! replace a service's sign-in / probe URL, sign-in paths and reading script. A replacement
//! script can also return the standard result (`{"dwbar": 1, "windows": [...]}`, see
//! `web::parse_standard`), so a changed API needs no Rust parser.
//!
//! Only signed rules are used; anything else (bad signature, too new, unreadable) keeps the
//! rules built into this version.
use super::web::{self, Service};
use base64::Engine;
use serde::Deserialize;
use std::{collections::HashMap, sync::RwLock};

/// minisign public key, base64 of the key file (same as `plugins.updater.pubkey`).
pub const PUBKEY: &str = include_str!("../../../../src-tauri/updater-pubkey.txt");
/// Where the website serves the rules.
pub const URL: &str = "https://dwbar.app.dreamsworld.top/rules/v1/";
/// The rules format this version understands.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RemoteService {
    label: Option<String>,
    host: Option<String>,
    login_url: Option<String>,
    probe_url: Option<String>,
    script: Option<String>,
    flow: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Rules {
    schema: u32,
    version: u32,
    /// Oldest DW Bar version these rules are written for (e.g. "1.1.0").
    #[serde(default)]
    min_app: Option<String>,
    #[serde(default)]
    services: HashMap<String, RemoteService>,
}

struct Active {
    version: u32,
    services: HashMap<String, Service>,
}

static ACTIVE: RwLock<Option<Active>> = RwLock::new(None);

/// What `install` did.
#[derive(Debug, PartialEq, Eq)]
pub enum Installed {
    /// New rules are in use.
    Updated(u32),
    /// Same or older than what is in use.
    Unchanged(u32),
}

fn decode_b64(s: &str) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|e| format!("base64: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("utf-8: {e}"))
}

/// Check a minisign signature (`sig` as written by `tauri signer sign`, base64).
pub fn verify(data: &[u8], sig: &str, pubkey: &str) -> Result<(), String> {
    let key = minisign_verify::PublicKey::decode(&decode_b64(pubkey)?).map_err(|e| format!("key: {e}"))?;
    let sig = minisign_verify::Signature::decode(&decode_b64(sig)?).map_err(|e| format!("signature: {e}"))?;
    key.verify(data, &sig, true).map_err(|e| format!("verify: {e}"))
}

/// "1.2.3" → (1, 2, 3); missing parts are 0.
fn semver(v: &str) -> (u32, u32, u32) {
    let mut it = v.trim().trim_start_matches('v').split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

/// Rules are rare and few, so their strings live for the rest of the run.
fn keep(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn merge(base: Service, r: RemoteService) -> Service {
    Service {
        agent: base.agent,
        label: r.label.map(keep).unwrap_or(base.label),
        host: r.host.map(keep).unwrap_or(base.host),
        login_url: r.login_url.map(keep).unwrap_or(base.login_url),
        probe_url: r.probe_url.map(keep).unwrap_or(base.probe_url),
        script: r.script.map(keep).unwrap_or(base.script),
        flow: match r.flow {
            Some(f) => Box::leak(f.into_iter().map(keep).collect::<Vec<_>>().into_boxed_slice()),
            None => base.flow,
        },
    }
}

/// Verify and parse; `Err` says why the rules can't be used.
fn check(text: &str, sig: &str, pubkey: &str, app_version: &str) -> Result<Rules, String> {
    verify(text.as_bytes(), sig, pubkey)?;
    let rules: Rules = serde_json::from_str(text).map_err(|e| format!("json: {e}"))?;
    if rules.schema != SCHEMA {
        return Err(format!("schema {} (this version reads {SCHEMA})", rules.schema));
    }
    if let Some(min) = &rules.min_app {
        if semver(app_version) < semver(min) {
            return Err(format!("needs DW Bar {min}"));
        }
    }
    Ok(rules)
}

fn activate(rules: Rules) -> u32 {
    let services = rules
        .services
        .into_iter()
        .filter_map(|(agent, r)| web::builtin(&agent).map(|base| (agent, merge(base, r))))
        .collect();
    *ACTIVE.write().unwrap() = Some(Active { version: rules.version, services });
    rules.version
}

fn cache_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = crate::data_dir();
    (dir.join("rules.json"), dir.join("rules.json.sig"))
}

/// Use newly downloaded rules if they are signed, readable and newer. Saved for next start.
pub fn install(text: &str, sig: &str, app_version: &str) -> Result<Installed, String> {
    let rules = check(text, sig, PUBKEY, app_version)?;
    if let Some(v) = version() {
        if rules.version <= v {
            return Ok(Installed::Unchanged(v));
        }
    }
    let v = activate(rules);
    let (json, sig_path) = cache_paths();
    let _ = std::fs::create_dir_all(crate::data_dir());
    let _ = std::fs::write(json, text).and_then(|_| std::fs::write(sig_path, sig));
    crate::log!("rules", "using online rules v{v}");
    Ok(Installed::Updated(v))
}

/// At start: the rules saved last time, checked again.
pub fn load_cached(app_version: &str) {
    let (json, sig) = cache_paths();
    let (Ok(text), Ok(sig)) = (std::fs::read_to_string(json), std::fs::read_to_string(sig)) else { return };
    match check(&text, &sig, PUBKEY, app_version) {
        Ok(rules) => {
            let v = activate(rules);
            crate::log!("rules", "saved online rules v{v}");
        }
        Err(e) => crate::log!("rules", "saved rules not used: {e}"),
    }
}

/// Version of the online rules in use, if any.
pub fn version() -> Option<u32> {
    ACTIVE.read().unwrap().as_ref().map(|a| a.version)
}

/// The online rule for `agent`, when one is in use.
pub fn service(agent: &str) -> Option<Service> {
    ACTIVE.read().unwrap().as_ref().and_then(|a| a.services.get(agent).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(semver("1.0.9") < semver("1.1.0"));
        assert_eq!(semver("v2"), (2, 0, 0));
    }

    #[test]
    fn rejects_bad_signatures() {
        // A real key, a signature for other data.
        let err = check(r#"{"schema":1,"version":9}"#, "bm90IGEgc2lnbmF0dXJl", PUBKEY, "9.9.9").unwrap_err();
        assert!(err.starts_with("signature") || err.starts_with("verify"), "{err}");
    }

    #[test]
    fn accepts_signed_rules() {
        // Signed with the release key by `tauri signer sign` (the CI step).
        let text = "{\"schema\":1,\"version\":1,\"services\":{}}\n";
        let sig = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTcWlmZDYyL3VnVWRiRk53aTJrRDV5SlBtcjhwMmt6MjhJVWV2MWpmU3RyVjd6akhtcTU1ZFVxdFNNdlRYdFdtQ0ZERm4wMUIyb25ja1J2aytGTFRvWlJRdE0yMkVBb1E0PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkzODQ4NjI0CWZpbGU6c2FtcGxlLmpzb24KMTVWc3B5NHV0VUx4NlJpdlNueVhZVHpBNm5vekxiQWpvWmtLSWI0UTZSeDU2anM5ZFo1U3g5U0MwUmVZQlVVcVhGcWV3WUhZZTZBdU0xeVNVRW9LQlE9PQo=";
        let rules = check(text, sig, PUBKEY, "1.1.0").expect("valid");
        assert_eq!(rules.version, 1);
        assert!(check(&text.replace('1', "2"), sig, PUBKEY, "1.1.0").is_err());
    }

    #[test]
    fn merges_over_builtin() {
        let base = web::builtin("claude").unwrap();
        let s = merge(base, RemoteService { probe_url: Some("https://claude.ai/x".into()), ..Default::default() });
        assert_eq!(s.probe_url, "https://claude.ai/x");
        assert_eq!(s.login_url, base.login_url);
        assert_eq!(s.agent, "claude");
    }
}
