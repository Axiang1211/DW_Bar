//! Start-at-login command line (pure; the registry side is `win::autostart`).
//!
//! The install folder is `…\AppData\Local\DW Bar\`, so the path must be quoted:
//! an unquoted path with a space is not reliably started from the Run key.

/// Marks launches that came from login, for the log.
pub const ARG: &str = "--autostart";

/// Value written to `HKCU\…\Run`.
pub fn command(exe: &str) -> String {
    format!("\"{exe}\" {ARG}")
}

/// True when the registered value already starts this exact executable, quoted.
pub fn is_current(value: &str, exe: &str) -> bool {
    value.trim() == command(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted() {
        let exe = r"C:\Users\Ann Lee\AppData\Local\DW Bar\dw-bar.exe";
        assert_eq!(command(exe), r#""C:\Users\Ann Lee\AppData\Local\DW Bar\dw-bar.exe" --autostart"#);
        assert!(is_current(&command(exe), exe));
        // What the old plugin wrote: unquoted, trailing space.
        assert!(!is_current(&format!("{exe} "), exe));
        assert!(!is_current(&command(r"C:\Old\dw-bar.exe"), exe));
    }
}
