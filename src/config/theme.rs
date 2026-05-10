use serde::{Deserialize, Serialize};

/// Placeholder palette for future theming (see roadmap). Serialized in `~/.stockterm.json`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Theme {
    pub accent_hex: Option<String>,
    pub background_hex: Option<String>,
}
