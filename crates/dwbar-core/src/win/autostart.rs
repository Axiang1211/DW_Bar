//! Start DW Bar at login through `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
//!
//! Replaces tauri-plugin-autostart on Windows, which writes the exe path unquoted;
//! with the default install folder (`…\Local\DW Bar\`) that path has a space.
use super::wide;
use crate::startup;
use windows_sys::Win32::System::Registry::{
    RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_BINARY, REG_SZ, RRF_RT_REG_BINARY,
    RRF_RT_REG_SZ,
};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Task Manager's "Startup apps" switch. First byte 2 = enabled, 3 = disabled.
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
/// Same value name the old plugin used (the product name), so it is overwritten.
pub const NAME: &str = "DW Bar";

/// Running from the Microsoft Store (MSIX) package. Packaged apps start at login through
/// the manifest's StartupTask instead of the Run key (their registry writes are virtualized).
pub fn packaged() -> bool {
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    let mut len: u32 = 0;
    // ERROR_INSUFFICIENT_BUFFER (122) = has a package; APPMODEL_ERROR_NO_PACKAGE (15700) = not.
    unsafe { GetCurrentPackageFullName(&mut len, std::ptr::null_mut()) == 122 }
}

#[cfg(windows)]
mod store {
    use windows::{
        core::HSTRING,
        ApplicationModel::{StartupTask, StartupTaskState},
    };
    const TASK: &str = "DWBarStartup";

    fn task() -> windows::core::Result<StartupTask> {
        StartupTask::GetAsync(&HSTRING::from(TASK))?.join()
    }

    /// Returns (enabled, blocked by the user / policy in Task Manager).
    pub fn state() -> Option<(bool, bool)> {
        let s = task().ok()?.State().ok()?;
        Some((
            s == StartupTaskState::Enabled || s == StartupTaskState::EnabledByPolicy,
            s == StartupTaskState::DisabledByUser || s == StartupTaskState::DisabledByPolicy,
        ))
    }

    pub fn set(enabled: bool) -> Result<(), String> {
        let t = task().map_err(|e| format!("StartupTask: {e}"))?;
        if enabled {
            let s = t.RequestEnableAsync().and_then(|op| op.join()).map_err(|e| format!("StartupTask enable: {e}"))?;
            if s != StartupTaskState::Enabled && s != StartupTaskState::EnabledByPolicy {
                return Err(format!("StartupTask is {}", s.0));
            }
        } else {
            t.Disable().map_err(|e| format!("StartupTask disable: {e}"))?;
        }
        Ok(())
    }
}

/// `lint-win` on another OS: no WinRT, so the Store path is a stub.
#[cfg(not(windows))]
mod store {
    pub fn state() -> Option<(bool, bool)> {
        None
    }
    pub fn set(_enabled: bool) -> Result<(), String> {
        Err("not Windows".into())
    }
}

fn exe() -> Option<String> {
    std::env::current_exe().ok().map(|p| p.display().to_string())
}

/// The command registered under Run, if any.
pub fn registered() -> Option<String> {
    let (key, name) = (wide(RUN), wide(NAME));
    let mut buf = vec![0u16; 2048];
    let mut size = (buf.len() * 2) as u32;
    let rc = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), buf.as_mut_ptr() as *mut _, &mut size)
    };
    if rc != 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}

/// False when the user switched DW Bar off in Task Manager → Startup apps.
pub fn approved() -> bool {
    let (key, name) = (wide(APPROVED), wide(NAME));
    let mut data = [0u8; 12];
    let mut size = data.len() as u32;
    let rc = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_BINARY, std::ptr::null_mut(), data.as_mut_ptr() as *mut _, &mut size)
    };
    // Missing entry means enabled; odd first byte (3) means disabled.
    rc != 0 || data[0] & 1 == 0
}

fn write_run(command: &str) -> Result<(), String> {
    let (key, name, value) = (wide(RUN), wide(NAME), wide(command));
    let rc = unsafe {
        RegSetKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), REG_SZ, value.as_ptr() as *const _, (value.len() * 2) as u32)
    };
    if rc == 0 { Ok(()) } else { Err(format!("Run write failed ({rc})")) }
}

fn approve() {
    let (key, name) = (wide(APPROVED), wide(NAME));
    let data: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), REG_BINARY, data.as_ptr() as *const _, data.len() as u32) };
}

/// Turn start-at-login on or off. Turning it on from the settings also clears a
/// Task Manager "disabled" mark, since the user just asked for it.
pub fn set(enabled: bool) -> Result<(), String> {
    if packaged() {
        store::set(enabled)?;
        crate::log!("autostart", "{} (StartupTask)", if enabled { "on" } else { "off" });
        return Ok(());
    }
    if !enabled {
        let (key, name) = (wide(RUN), wide(NAME));
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) };
        crate::log!("autostart", "off");
        return Ok(());
    }
    let exe = exe().ok_or("no exe path")?;
    write_run(&startup::command(&exe))?;
    approve();
    crate::log!("autostart", "on");
    Ok(())
}

/// At every start: make the Run entry match the setting and this exe (fixes the old
/// unquoted value and a moved install). Leaves the Task Manager switch alone.
/// Returns true when start-at-login is on but blocked in Task Manager.
pub fn sync(enabled: bool) -> bool {
    if packaged() {
        let Some((on, blocked)) = store::state() else { return false };
        if on != enabled && !blocked {
            if let Err(e) = store::set(enabled) {
                crate::log!("autostart", "{e}");
            }
        }
        if blocked && enabled {
            crate::log!("autostart", "StartupTask switched off in Task Manager → Startup apps");
        }
        return blocked && enabled;
    }
    let current = registered();
    if !enabled {
        if current.is_some() {
            let _ = set(false);
        }
        return false;
    }
    let Some(exe) = exe() else { return false };
    if !current.as_deref().is_some_and(|v| startup::is_current(v, &exe)) {
        match write_run(&startup::command(&exe)) {
            Ok(()) => crate::log!("autostart", "repaired Run entry (was {})", if current.is_some() { "outdated" } else { "missing" }),
            Err(e) => crate::log!("autostart", "{e}"),
        }
    }
    let blocked = !approved();
    if blocked {
        crate::log!("autostart", "switched off in Task Manager → Startup apps");
    }
    blocked
}
