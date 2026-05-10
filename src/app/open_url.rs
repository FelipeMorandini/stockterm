//! Best-effort open URL in the user's browser (Issue #11).

use std::process::Command;

pub(crate) fn open_article_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Empty URL".to_string());
    }

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
        Err("Unsupported platform".to_string())
    }
}
