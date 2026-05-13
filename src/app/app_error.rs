//! Issue #20 — structured runtime errors, categories, and log entries ([`docs/SPEC.md`](../../docs/SPEC.md) §20).

use std::collections::VecDeque;
use std::time::Duration;

use chrono::{DateTime, Local};
use ratatui::style::{Color, Style};

use crate::api::error::ProviderError;
use super::Tab;

/// Bracket prefix on the status line (§20.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiErrorCategory {
    Net,
    Api,
    Rate,
    Parse,
    Cfg,
    Int,
}

impl UiErrorCategory {
    pub fn as_prefix(self) -> &'static str {
        match self {
            UiErrorCategory::Net => "[net] ",
            UiErrorCategory::Api => "[api] ",
            UiErrorCategory::Rate => "[rate] ",
            UiErrorCategory::Parse => "[parse] ",
            UiErrorCategory::Cfg => "[cfg] ",
            UiErrorCategory::Int => "[int] ",
        }
    }
}

/// Application-level error (§20.2).
#[derive(Debug, Clone)]
pub enum AppError {
    Provider(ProviderError),
    ConfigSave(String),
    Internal(String),
}

impl AppError {
    pub fn category(&self) -> UiErrorCategory {
        match self {
            AppError::Provider(pe) => category_from_provider(pe),
            AppError::ConfigSave(_) => UiErrorCategory::Cfg,
            AppError::Internal(s) => {
                if s.starts_with("Polygon provider requires") {
                    UiErrorCategory::Cfg
                } else {
                    UiErrorCategory::Int
                }
            }
        }
    }

    /// Single-line status / banner text with prefix (§20.3).
    pub fn status_line(&self) -> String {
        let cat = self.category();
        let prefix = cat.as_prefix();
        let body = self.body_for_line();
        let hint = self.retry_hint_suffix();
        let mut out = format!("{prefix}{body}");
        if let Some(h) = hint {
            out.push(' ');
            out.push_str(&h);
        }
        truncate_line_utf8(out, 512)
    }

    fn body_for_line(&self) -> String {
        match self {
            AppError::Provider(pe) => match pe {
                ProviderError::Transport(msg) => {
                    // Avoid "[net] Network error: foo" doubling (§20.3).
                    msg.strip_prefix("Network error: ")
                        .unwrap_or(msg)
                        .trim()
                        .to_string()
                }
                _ => pe.to_string(),
            },
            AppError::ConfigSave(s) | AppError::Internal(s) => s.clone(),
        }
    }

    fn retry_hint_suffix(&self) -> Option<String> {
        match self {
            AppError::Provider(ProviderError::RateLimited { retry_after }) => {
                retry_after.and_then(|d| {
                    if d.is_zero() {
                        return None;
                    }
                    let secs = d.as_secs().max(1);
                    Some(format!("retry in {secs}s"))
                })
            }
            _ => None,
        }
    }
}

pub fn category_from_provider(pe: &ProviderError) -> UiErrorCategory {
    match pe {
        ProviderError::Timeout | ProviderError::Transport(_) => UiErrorCategory::Net,
        ProviderError::RateLimited { .. } => UiErrorCategory::Rate,
        ProviderError::Http { status, .. } => {
            if (500..600).contains(status) {
                UiErrorCategory::Net
            } else {
                UiErrorCategory::Api
            }
        }
        ProviderError::Json(_) => UiErrorCategory::Parse,
        ProviderError::ApiMessage(_) => UiErrorCategory::Api,
    }
}

/// Transient errors auto-clear after [`ERROR_TRANSIENT_TTL`]; sticky persist (§20.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPersistence {
    Transient,
    Sticky,
}

pub fn persistence_for_app_error(err: &AppError) -> ErrorPersistence {
    match err {
        AppError::Provider(pe) => persistence_for_provider(pe),
        AppError::ConfigSave(_) => ErrorPersistence::Sticky,
        AppError::Internal(s) => {
            if s.starts_with("Polygon provider requires") {
                ErrorPersistence::Sticky
            } else {
                ErrorPersistence::Sticky
            }
        }
    }
}

fn persistence_for_provider(pe: &ProviderError) -> ErrorPersistence {
    match pe {
        ProviderError::Timeout
        | ProviderError::Transport(_)
        | ProviderError::RateLimited { .. } => ErrorPersistence::Transient,
        ProviderError::Http { status, .. } => {
            if (500..600).contains(status) {
                ErrorPersistence::Transient
            } else {
                ErrorPersistence::Sticky
            }
        }
        ProviderError::Json(_) | ProviderError::ApiMessage(_) => ErrorPersistence::Sticky,
    }
}

/// Which fetch / subsystem produced the active error (for TTL + success clearing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSourceDomain {
    Stock,
    Charts,
    News,
    Search,
    Settings,
    Portfolio,
    Alerts,
    NewsOpenUrl,
    #[allow(dead_code)]
    Other,
}

/// One row in the error log ring buffer (§20.4).
#[derive(Debug, Clone)]
pub struct ErrorLogEntry {
    pub when: DateTime<Local>,
    pub tab: Tab,
    pub category: UiErrorCategory,
    pub line: String,
}

pub const ERROR_LOG_CAP: usize = 20;

pub const ERROR_TRANSIENT_TTL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct ActiveErrorState {
    pub error: AppError,
    pub persistence: ErrorPersistence,
    pub shown_since: std::time::Instant,
    pub source_domain: ErrorSourceDomain,
}

impl ActiveErrorState {
    pub fn new(
        error: AppError,
        persistence: ErrorPersistence,
        shown_since: std::time::Instant,
        source_domain: ErrorSourceDomain,
    ) -> Self {
        Self {
            error,
            persistence,
            shown_since,
            source_domain,
        }
    }

    pub fn display_line(&self) -> String {
        self.error.status_line()
    }
}

#[derive(Debug, Clone)]
pub enum LastFailedFetch {
    StockQuoteBatch,
    Historical,
    News { symbol: String },
    Search { query: String, generation: u64 },
    None,
}

pub fn push_error_log(
    deque: &mut VecDeque<ErrorLogEntry>,
    tab: Tab,
    category: UiErrorCategory,
    line: String,
) {
    let line = truncate_line_utf8(line, 256);
    deque.push_back(ErrorLogEntry {
        when: Local::now(),
        tab,
        category,
        line,
    });
    while deque.len() > ERROR_LOG_CAP {
        deque.pop_front();
    }
}

pub fn startup_banner_style() -> Style {
    Style::default().fg(Color::Red).bg(Color::DarkGray)
}

fn truncate_line_utf8(mut s: String, max_chars: usize) -> String {
    let n = s.chars().count();
    if n <= max_chars {
        return s;
    }
    s = s.chars().take(max_chars.saturating_sub(1)).collect();
    s.push('…');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rate_limited_status_includes_rate_prefix_and_retry_hint() {
        let e = AppError::Provider(ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(10)),
        });
        let s = e.status_line();
        assert!(s.starts_with("[rate]"));
        assert!(s.contains("retry in 10s"), "got {s:?}");
    }

    #[test]
    fn transport_uses_net_without_double_network() {
        let e = AppError::Provider(ProviderError::Transport(
            "Network error: connection refused".into(),
        ));
        let s = e.status_line();
        assert!(s.starts_with("[net]"));
        assert!(s.contains("connection refused"));
        assert!(!s.contains("Network error: Network"));
    }

    #[test]
    fn error_log_evicts_oldest_at_21() {
        let mut d = VecDeque::new();
        for i in 0..21 {
            push_error_log(
                &mut d,
                Tab::StockView,
                UiErrorCategory::Int,
                format!("msg{i}"),
            );
        }
        assert_eq!(d.len(), ERROR_LOG_CAP);
        assert_eq!(d.front().unwrap().line, "msg1");
        assert_eq!(d.back().unwrap().line, "msg20");
    }
}
