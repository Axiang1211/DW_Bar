//! A child CLI that speaks one JSON object per line on stdin/stdout.
//! Killed when dropped, so a hung CLI never outlives the read.
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct Engine {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Value>,
    stderr: Arc<Mutex<String>>,
}

impl Engine {
    pub fn start(program: &Path, args: &[&str], envs: &[(&str, String)]) -> Result<Self, String> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in envs {
            cmd.env(k, v);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("無法啟動 {}：{e}", program.display()))?;

        let stdout = child.stdout.take().ok_or("沒有 stdout")?;
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(value) = serde_json::from_str::<Value>(line.trim()) {
                    if tx.send(value).is_err() {
                        break;
                    }
                }
            }
        });

        let stderr = Arc::new(Mutex::new(String::new()));
        if let Some(mut err) = child.stderr.take() {
            let sink = stderr.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 1024];
                while let Ok(n) = err.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut s) = sink.lock() {
                        if s.len() < 4096 {
                            s.push_str(&String::from_utf8_lossy(&buf[..n]));
                        }
                    }
                }
            });
        }

        let stdin = child.stdin.take();
        Ok(Self { child, stdin, lines, stderr })
    }

    pub fn send(&mut self, message: &Value) -> Result<(), String> {
        let stdin = self.stdin.as_mut().ok_or("stdin 已關閉")?;
        let mut line = message.to_string();
        line.push('\n');
        stdin
            .write_all(line.as_bytes())
            .and_then(|_| stdin.flush())
            .map_err(|e| format!("寫入失敗：{e}"))
    }

    /// The next JSON line, or `None` once `deadline` passes or the CLI exits.
    pub fn next(&mut self, deadline: Instant) -> Option<Value> {
        let wait = deadline.checked_duration_since(Instant::now())?;
        match self.lines.recv_timeout(wait) {
            Ok(v) => Some(v),
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => None,
        }
    }

    /// First `max` characters of stderr, for error messages.
    pub fn stderr_preview(&self, max: usize) -> Option<String> {
        let s = self.stderr.lock().ok()?;
        let t = s.trim();
        (!t.is_empty()).then(|| t.chars().take(max).collect())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
