use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::theme::Theme;
use crate::models::alerts::Alert;
use crate::models::portfolio::PortfolioItem;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub portfolio: Vec<PortfolioItem>,
    pub refresh_rate: u64,
    pub api_key: String,
    pub alerts: Vec<Alert>,
    pub default_symbol: String,
    pub theme: Option<Theme>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("home directory not found")]
    NoHomeDir,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Config {
    /// Polygon API key resolution:
    /// 1. Non-empty `api_key` from config file (`~/.stockterm.json`).
    /// 2. Else non-empty `STOCKTERM_API_KEY` environment variable.
    /// 3. Else empty string (API calls may fail until configured).
    pub fn effective_api_key(&self) -> Cow<'_, str> {
        if !self.api_key.is_empty() {
            return Cow::Borrowed(self.api_key.as_str());
        }
        match std::env::var("STOCKTERM_API_KEY") {
            Ok(s) if !s.is_empty() => Cow::Owned(s),
            _ => Cow::Borrowed(""),
        }
    }

    /// Load config from disk, or defaults. Never panics; I/O/JSON errors fall back to default.
    pub fn load() -> Self {
        Self::try_load().unwrap_or_default()
    }

    pub fn try_load() -> Result<Self, ConfigError> {
        let path = match Self::config_file_path() {
            Ok(p) => p,
            Err(ConfigError::NoHomeDir) => return Ok(Config::default()),
            Err(e) => return Err(e),
        };

        match fs::read_to_string(&path) {
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(ConfigError::Io(e)),
            Ok(s) => serde_json::from_str(&s).map_err(ConfigError::Serde),
        }
    }

    /// Persist config. Errors are dropped (TUI has no logger); use [`try_save`](Self::try_save) to handle them.
    pub fn save(&self) {
        let _ = self.try_save();
    }

    pub fn try_save(&self) -> Result<(), ConfigError> {
        let path = Self::config_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let config_str = serde_json::to_string_pretty(self).map_err(ConfigError::Serde)?;
        fs::write(path, config_str).map_err(ConfigError::Io)?;
        Ok(())
    }

    fn config_file_path() -> Result<PathBuf, ConfigError> {
        let mut path = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        path.push(".stockterm.json");
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_api_key_prefers_config_file_value() {
        let mut c = Config::default();
        c.api_key = "from-config".to_string();
        assert_eq!(c.effective_api_key().as_ref(), "from-config");
    }

}
