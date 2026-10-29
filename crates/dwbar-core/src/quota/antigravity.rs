//! Google Antigravity: read the quota the Antigravity app shows in its Model Quota
//! panel, from the app's own local language server (only while the app is open).
//!
//! The server listens on 127.0.0.1 with a self-signed certificate and a per-launch
//! CSRF token passed on its command line. A short PowerShell script (hidden window)
//! finds the process, its token and listening ports, and asks it for
//! `RetrieveUserQuotaSummary` (falling back to `GetUserStatus`). Approach documented
//! by CodexBar (MIT); this is an independent implementation.
use super::{Period, Reading, Window};
use serde_json::Value;

pub(crate) const NAME: &str = "Antigravity";

/// Finds Antigravity language servers and queries them. Prints one JSON line:
/// `{"found":bool,"summary":<json|null>,"status":<json|null>,"error":string|null}`.
#[cfg_attr(not(windows), allow(dead_code))]
const PROBE: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$out = @{ found = $false; summary = $null; status = $null; error = $null }
try {
  $procs = Get-CimInstance Win32_Process | Where-Object {
    $_.Name -like 'language_server*' -and $_.CommandLine -match 'antigravity'
  }
  if (-not $procs) { $out.error = 'not_running'; $out | ConvertTo-Json -Compress -Depth 30; exit }
  $out.found = $true
  Add-Type -TypeDefinition @"
using System.Net; using System.Security.Cryptography.X509Certificates;
public class DwBarTrust : ICertificatePolicy { public bool CheckValidationResult(ServicePoint s, X509Certificate c, WebRequest r, int p) { return true; } }
"@
  [System.Net.ServicePointManager]::CertificatePolicy = New-Object DwBarTrust
  [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
  $body = '{"metadata":{"ideName":"antigravity","extensionName":"antigravity","locale":"en","ideVersion":"unknown"}}'
  foreach ($p in $procs) {
    $cmd = $p.CommandLine
    $token = if ($cmd -match '--csrf_token[= ]+(\S+)') { $Matches[1] } else { '' }
    $ports = @(Get-NetTCPConnection -OwningProcess $p.ProcessId -State Listen | Select-Object -ExpandProperty LocalPort -Unique)
    if ($cmd -match '--extension_server_port[= ]+(\d+)') { $ports += [int]$Matches[1] }
    foreach ($port in $ports) {
      foreach ($scheme in @('https', 'http')) {
        $base = "${scheme}://127.0.0.1:$port/exa.language_server_pb.LanguageServerService"
        $h = @{ 'X-Codeium-Csrf-Token' = $token; 'Connect-Protocol-Version' = '1' }
        try {
          $s = Invoke-RestMethod -Method Post -Uri "$base/RetrieveUserQuotaSummary" -Headers $h -ContentType 'application/json' -Body $body -TimeoutSec 4
          $out.summary = $s
        } catch {}
        try {
          $u = Invoke-RestMethod -Method Post -Uri "$base/GetUserStatus" -Headers $h -ContentType 'application/json' -Body $body -TimeoutSec 4
          $out.status = $u
        } catch {}
        if ($out.summary -or $out.status) { $out | ConvertTo-Json -Compress -Depth 30; exit }
      }
    }
  }
  $out.error = 'no_answer'
} catch { $out.error = "$_" }
$out | ConvertTo-Json -Compress -Depth 30
"#;

/// Whether the Antigravity app (or IDE) language server is running, and its answer.
pub fn read() -> Reading {
    #[cfg(windows)]
    {
        match run_probe() {
            Ok(v) => {
                if v["found"] != true {
                    let mut r = Reading::failed("antigravity", NAME, "打開 Antigravity 後就會顯示用量");
                    r.source = Some("app".into());
                    return r;
                }
                let mut r = parse(&v);
                r.source = Some("app".into());
                if !r.ok && r.notice.is_none() {
                    r.notice = Some("Antigravity 正在啟動，稍後自動重試".into());
                }
                r
            }
            Err(e) => Reading::failed("antigravity", NAME, e),
        }
    }
    #[cfg(not(windows))]
    {
        Reading::failed("antigravity", NAME, "目前只支援 Windows")
    }
}

#[cfg(windows)]
fn run_probe() -> Result<Value, String> {
    use std::io::Read;
    use std::os::windows::process::CommandExt;
    use std::time::{Duration, Instant};
    let system = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let ps = std::path::Path::new(&system).join("System32\\WindowsPowerShell\\v1.0\\powershell.exe");
    // -EncodedCommand (UTF-16LE base64) avoids Windows PowerShell's quote stripping.
    let script = encoded(PROBE);
    let mut child = std::process::Command::new(ps)
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", script.as_str()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|e| format!("無法啟動 PowerShell：{e}"))?;
    // Drain stdout on a thread so a large answer can never fill the pipe and stall the child.
    let mut stdout = child.stdout.take().ok_or_else(|| "沒有輸出".to_string())?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Antigravity 沒有在 30 秒內回應".into());
            }
        }
    }
    let text = reader.join().unwrap_or_default();
    serde_json::from_str::<Value>(text.trim().trim_start_matches('\u{feff}'))
        .map_err(|_| "Antigravity 回傳了無法辨識的資料".to_string())
}

/// PowerShell `-EncodedCommand`: base64 of the UTF-16LE script.
#[cfg_attr(not(windows), allow(dead_code))]
fn encoded(script: &str) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn reset_of(v: &Value) -> Option<i64> {
    for key in ["resetTime", "reset_time", "resetsAt", "nextResetTime"] {
        let r = &v[key];
        if let Some(s) = r.as_str() {
            if let Ok(d) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(d.timestamp());
            }
            if let Ok(n) = s.parse::<i64>() {
                return Some(n);
            }
        }
        if let Some(n) = r.as_i64() {
            return Some(n);
        }
    }
    None
}

fn period_of(text: &str) -> Period {
    let t = text.to_lowercase();
    if t.contains("week") || t.contains("7d") {
        Period::Week
    } else if t.contains("5h") || t.contains("hour") || t.contains("session") {
        Period::FiveHour
    } else {
        Period::Other
    }
}

/// Parse the probe output: quota summary groups first, else per-model quotas from user status.
pub fn parse(v: &Value) -> Reading {
    let mut r = Reading::empty("antigravity", NAME);
    let summary = if v["summary"]["response"].is_object() { &v["summary"]["response"] } else { &v["summary"] };
    for group in summary["groups"].as_array().into_iter().flatten() {
        let gname = group["displayName"].as_str().unwrap_or("模型");
        for b in group["buckets"].as_array().into_iter().flatten() {
            let Some(frac) = b["remaining"]["remainingFraction"].as_f64().or_else(|| b["remainingFraction"].as_f64()) else {
                continue;
            };
            let cadence = [&b["window"], &b["bucketId"], &b["displayName"]]
                .iter()
                .filter_map(|x| x.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let period = period_of(&cadence);
            let label = match period {
                Period::FiveHour => format!("{gname} · 5 小時"),
                Period::Week => format!("{gname} · 每週"),
                _ => format!("{gname} · {}", b["displayName"].as_str().unwrap_or("額度")),
            };
            r.windows.push(Window {
                period,
                label,
                used_percent: ((1.0 - frac) * 100.0).clamp(0.0, 100.0),
                resets_at: reset_of(&b["remaining"]).or_else(|| reset_of(b)),
            });
        }
    }
    if r.windows.is_empty() {
        // Legacy: per-model quota; the most constrained model stands for the session.
        let models = v["status"]["userStatus"]["cascadeModelConfigData"]["clientModelConfigs"].as_array().cloned().unwrap_or_default();
        let lowest = models
            .iter()
            .filter_map(|m| Some((m["quotaInfo"]["remainingFraction"].as_f64()?, m)))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((frac, m)) = lowest {
            r.windows.push(Window {
                period: Period::FiveHour,
                label: "模型額度".into(),
                used_percent: ((1.0 - frac) * 100.0).clamp(0.0, 100.0),
                resets_at: reset_of(&m["quotaInfo"]),
            });
        }
    }
    // The tightest window of each period comes first, so the rings show what runs out first.
    r.windows.sort_by(|a, b| b.used_percent.total_cmp(&a.used_percent));
    let us = &v["status"]["userStatus"];
    r.account = us["email"].as_str().map(str::to_owned);
    r.plan = us["planStatus"]["planInfo"]["planName"]
        .as_str()
        .or_else(|| us["userTier"]["name"].as_str())
        .map(str::to_owned);
    r.ok = !r.windows.is_empty();
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn summary_groups() {
        let r = parse(&json!({"found": true, "summary": {"response": {"groups": [
            {"displayName": "Gemini Models", "buckets": [
                {"bucketId": "gemini-5h", "remaining": {"remainingFraction": 0.8, "resetTime": "2026-10-29T12:00:00Z"}},
                {"bucketId": "gemini-weekly", "remaining": {"remainingFraction": 0.4}}
            ]},
            {"displayName": "Claude and GPT models", "buckets": [
                {"bucketId": "claude-gpt-5h", "window": "5h", "remaining": {"remainingFraction": 0.1}}
            ]}
        ]}}, "status": {"userStatus": {"email": "a@b.c"}}}));
        assert!(r.ok);
        assert_eq!(r.windows.len(), 3);
        // tightest five-hour window first
        assert_eq!(r.window(Period::FiveHour).unwrap().label, "Claude and GPT models · 5 小時");
        assert!((r.window(Period::Week).unwrap().used_percent - 60.0).abs() < 1e-9);
        assert_eq!(r.account.as_deref(), Some("a@b.c"));
    }

    #[test]
    fn encoded_command() {
        // "ab" in UTF-16LE = 61 00 62 00
        assert_eq!(encoded("ab"), "YQBiAA==");
        assert_eq!(encoded("a"), "YQA=");
    }

    #[test]
    fn legacy_model_quota() {
        let r = parse(&json!({"found": true, "summary": null, "status": {"userStatus": {"cascadeModelConfigData": {"clientModelConfigs": [
            {"quotaInfo": {"remainingFraction": 0.9}},
            {"quotaInfo": {"remainingFraction": 0.25, "resetTime": "2026-10-29T12:00:00Z"}}
        ]}}}}));
        assert!(r.ok);
        assert_eq!(r.window(Period::FiveHour).unwrap().used_percent, 75.0);
    }
}
