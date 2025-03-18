use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::models::portfolio::PortfolioItem;
use crate::models::alerts::Alert;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub portfolio: Vec<PortfolioItem>,
    pub refresh_rate: u64,
    pub api_key: String,
    pub alerts: Vec<Alert>,
    pub default_symbol: String,
    pub theme: Option<Theme>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Config::get_config_path();
        if let Ok(config_str) = fs::read_to_string(config_path) {
            serde_json::from_str(&config_str).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) {
        let config_path = Config::get_config_path();
        let config_str = serde_json::to_string_pretty(self).unwrap();
        fs::write(config_path, config_str).unwrap();
    }

    fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap();
        path.push(".stockterm.json");
        path
    }
}