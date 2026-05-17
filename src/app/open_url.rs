//! Best-effort open URL in the user's browser and clipboard copy (Issues #11, #58, #59).

use std::io::Write;
use std::process::{Command, Stdio};

const HTTP_PREFIX: &str = "http://";
const HTTPS_PREFIX: &str = "https://";

/// Trim, validate `http`/`https`, return normalized URL for OS helpers (§27.2.1).
pub(crate) fn normalize_article_url(url: &str) -> Result<String, &'static str> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Empty URL");
    }
    let lower = trimmed.to_ascii_lowercase();
    let rest = if let Some(r) = lower.strip_prefix(HTTPS_PREFIX) {
        r
    } else if let Some(r) = lower.strip_prefix(HTTP_PREFIX) {
        r
    } else {
        return Err("Only http(s) URLs can be opened");
    };
    if rest.bytes().any(|b| b == 0 || b.is_ascii_control()) {
        return Err("Only http(s) URLs can be opened");
    }
    Ok(trimmed.to_string())
}

pub(crate) fn open_article_url_blocking(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())
            .and_then(|s| {
                if s.success() {
                    Ok(())
                } else {
                    Err("open exited with non-zero status".to_string())
                }
            })
    }

    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map_err(|e| e.to_string())
            .and_then(|s| {
                if s.success() {
                    Ok(())
                } else {
                    Err("start exited with non-zero status".to_string())
                }
            })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())
            .and_then(|s| {
                if s.success() {
                    Ok(())
                } else {
                    Err("xdg-open exited with non-zero status".to_string())
                }
            })
    }

    #[cfg(not(any(
        target_os = "macos",
        windows,
        all(unix, not(target_os = "macos"))
    )))]
    {
        let _ = url;
        Err("Unsupported platform".to_string())
    }
}

pub(crate) fn copy_article_url_blocking(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        write_url_to_stdin_command("pbcopy", url)
    }

    #[cfg(windows)]
    {
        write_url_to_stdin_command("clip", url)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if command_exists("wl-copy") {
            return write_url_to_stdin_command("wl-copy", url);
        }
        if command_exists("xclip") {
            let mut child = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            {
                let stdin = child.stdin.as_mut().ok_or("xclip stdin unavailable")?;
                stdin.write_all(url.as_bytes()).map_err(|e| e.to_string())?;
            }
            let status = child.wait().map_err(|e| e.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err("xclip exited with non-zero status".to_string())
            }
        } else {
            Err("No clipboard helper found (install wl-copy or xclip)".to_string())
        }
    }

    #[cfg(not(any(target_os = "macos", windows, all(unix, not(target_os = "macos")))))]
    {
        let _ = url;
        Err("Unsupported platform".to_string())
    }
}

fn write_url_to_stdin_command(program: &str, url: &str) -> Result<(), String> {
    let mut child = Command::new(program)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    {
        let stdin = child.stdin.as_mut().ok_or("clipboard stdin unavailable")?;
        stdin.write_all(url.as_bytes()).map_err(|e| e.to_string())?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with non-zero status"))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn command_exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|_| {
            Command::new(program)
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
}

/// Run inside `spawn_blocking`: open with optional clipboard fallback (§27.2.3).
pub(crate) fn run_open_with_copy_fallback(url: &str) -> (Result<(), String>, Option<NewsUrlFlashHint>) {
    run_open_with_copy_fallback_ops(
        url,
        open_article_url_blocking,
        copy_article_url_blocking,
    )
}

pub(crate) fn run_open_with_copy_fallback_ops(
    url: &str,
    open: fn(&str) -> Result<(), String>,
    copy: fn(&str) -> Result<(), String>,
) -> (Result<(), String>, Option<NewsUrlFlashHint>) {
    match open(url) {
        Ok(()) => (Ok(()), Some(NewsUrlFlashHint::Opened)),
        Err(open_err) => match copy(url) {
            Ok(()) => (Ok(()), Some(NewsUrlFlashHint::OpenFailedCopied)),
            Err(copy_err) => (
                Err(format!("Could not open URL: {open_err}; copy failed: {copy_err}")),
                None,
            ),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NewsUrlFlashHint {
    Opened,
    Copied,
    OpenFailedCopied,
}

impl NewsUrlFlashHint {
    pub(crate) fn status_text(self) -> &'static str {
        match self {
            Self::Opened => "Opened URL",
            Self::Copied => "URL copied",
            Self::OpenFailedCopied => "Open failed; URL copied",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn normalize_article_url_accepts_http_https() {
        assert!(normalize_article_url("https://example.com/path").is_ok());
        assert!(normalize_article_url("http://host").is_ok());
        assert_eq!(
            normalize_article_url("  HTTPS://Foo.Bar  ").unwrap(),
            "HTTPS://Foo.Bar"
        );
    }

    #[test]
    fn normalize_article_url_trims_whitespace() {
        assert_eq!(
            normalize_article_url("  https://example.com  ").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn normalize_article_url_rejects_unsafe_or_empty() {
        assert_eq!(normalize_article_url(""), Err("Empty URL"));
        assert_eq!(normalize_article_url("   "), Err("Empty URL"));
        assert_eq!(
            normalize_article_url("javascript:alert(1)"),
            Err("Only http(s) URLs can be opened")
        );
        assert_eq!(
            normalize_article_url("file:///etc/passwd"),
            Err("Only http(s) URLs can be opened")
        );
        assert_eq!(
            normalize_article_url("https://evil.com/\x01"),
            Err("Only http(s) URLs can be opened")
        );
    }

    fn ok(_: &str) -> Result<(), String> {
        Ok(())
    }

    fn fail_open(_: &str) -> Result<(), String> {
        Err("open".to_string())
    }

    fn fail_copy(_: &str) -> Result<(), String> {
        Err("copy".to_string())
    }

    #[test]
    fn open_with_copy_fallback_open_ok() {
        static COPY_CALLS: AtomicUsize = AtomicUsize::new(0);
        fn counting_copy(_: &str) -> Result<(), String> {
            COPY_CALLS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        let (result, flash) = run_open_with_copy_fallback_ops("https://x.test", ok, counting_copy);
        assert!(result.is_ok());
        assert_eq!(flash, Some(NewsUrlFlashHint::Opened));
        assert_eq!(COPY_CALLS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn open_with_copy_fallback_open_fail_copy_ok() {
        let (result, flash) =
            run_open_with_copy_fallback_ops("https://x.test", fail_open, ok);
        assert!(result.is_ok());
        assert_eq!(flash, Some(NewsUrlFlashHint::OpenFailedCopied));
    }

    #[test]
    fn open_with_copy_fallback_both_fail() {
        let (result, flash) =
            run_open_with_copy_fallback_ops("https://x.test", fail_open, fail_copy);
        assert!(result.is_err());
        assert!(flash.is_none());
        let err = result.unwrap_err();
        assert!(err.contains("open"));
        assert!(err.contains("copy"));
    }
}
