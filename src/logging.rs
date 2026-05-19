//! File-based [`tracing`] subscriber for background diagnostics (SPEC §38.1).

use std::path::PathBuf;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Installs a global tracing subscriber. Prefer calling once from `main` before TUI setup.
///
/// Default log path: `{cache_dir}/stockterm/logs/stockterm.log` (override with `STOCKTERM_LOG_DIR`).
/// On log-file creation failure, prints one line to stderr and continues with stderr-only logging.
pub fn init() {
    let filter = env_filter();

    match try_init_file_subscriber(&filter) {
        Ok(guard) => {
            let _ = LOG_GUARD.set(guard);
        }
        Err(e) => {
            eprintln!("stockterm: could not open log file ({e}); using stderr-only logging");
            init_stderr_subscriber(&filter);
        }
    }
}

fn env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,stockterm=warn"))
}

fn log_stderr_mirror_enabled() -> bool {
    std::env::var("STOCKTERM_LOG_STDERR")
        .map(|s| s == "1")
        .unwrap_or(false)
}

fn resolve_log_dir() -> Result<PathBuf, String> {
    if let Ok(raw) = std::env::var("STOCKTERM_LOG_DIR") {
        return expand_tilde_path(&raw);
    }
    let cache = dirs::cache_dir()
        .ok_or_else(|| "could not resolve cache directory for log file".to_string())?;
    Ok(cache.join("stockterm").join("logs"))
}

fn expand_tilde_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| "STOCKTERM_LOG_DIR uses ~ but HOME is unset".to_string())?;
        return Ok(home.join(rest));
    }
    if trimmed == "~" {
        let home = dirs::home_dir()
            .ok_or_else(|| "STOCKTERM_LOG_DIR is ~ but HOME is unset".to_string())?;
        return Ok(home);
    }
    Ok(PathBuf::from(trimmed))
}

fn try_init_file_subscriber(filter: &EnvFilter) -> Result<WorkerGuard, String> {
    let log_dir = resolve_log_dir()?;
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("create_dir_all {}: {e}", log_dir.display()))?;

    let file_appender = tracing_appender::rolling::never(&log_dir, "stockterm.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let registry = tracing_subscriber::registry().with(filter.clone()).with(file_layer);

    if log_stderr_mirror_enabled() {
        let stderr_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(false);
        registry.with(stderr_layer).init();
    } else {
        registry.init();
    }

    Ok(guard)
}

fn init_stderr_subscriber(filter: &EnvFilter) {
    tracing_subscriber::fmt()
        .with_env_filter(filter.clone())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_joins_home() {
        let Some(home) = dirs::home_dir() else {
            return;
        };
        let got = expand_tilde_path("~/stockterm-logs").expect("expand");
        assert_eq!(got, home.join("stockterm-logs"));
    }

    #[test]
    fn resolve_log_dir_uses_cache_when_env_unset() {
        let prev = std::env::var("STOCKTERM_LOG_DIR").ok();
        std::env::remove_var("STOCKTERM_LOG_DIR");
        let got = resolve_log_dir().expect("cache dir");
        if let Some(cache) = dirs::cache_dir() {
            assert_eq!(got, cache.join("stockterm").join("logs"));
        }
        if let Some(p) = prev {
            std::env::set_var("STOCKTERM_LOG_DIR", p);
        }
    }
}
