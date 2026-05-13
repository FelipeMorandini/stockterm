//! Issue #14 / [`docs/SPEC.md`](../../docs/SPEC.md) §21 — JSON theme + presets.

use serde::{Deserialize, Serialize};

/// Built-in color schemes (serde snake_case; `"default"` is a reserved Rust keyword).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    #[default]
    #[serde(rename = "default")]
    BuiltinDefault,
    Dark,
    Light,
    #[serde(rename = "high_contrast")]
    HighContrast,
}

impl ThemePreset {
    pub const ALL: [ThemePreset; 4] = [
        ThemePreset::BuiltinDefault,
        ThemePreset::Dark,
        ThemePreset::Light,
        ThemePreset::HighContrast,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemePreset::BuiltinDefault => "Default",
            ThemePreset::Dark => "Dark",
            ThemePreset::Light => "Light",
            ThemePreset::HighContrast => "High contrast",
        }
    }

    pub fn next(self) -> ThemePreset {
        let i = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> ThemePreset {
        let i = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    /// Full palette for this preset (RGB triples).
    pub fn base_rgb(self) -> PaletteRgb {
        match self {
            ThemePreset::BuiltinDefault => PaletteRgb {
                background: [0, 0, 0],
                foreground: [220, 220, 220],
                accent: [86, 182, 194],
                positive: [80, 200, 120],
                negative: [224, 108, 117],
                border: [184, 160, 0],
                selection: [128, 128, 128],
                muted: [100, 100, 100],
            },
            ThemePreset::Dark => PaletteRgb {
                background: [18, 18, 24],
                foreground: [230, 230, 235],
                accent: [130, 200, 255],
                positive: [90, 210, 130],
                negative: [255, 120, 130],
                border: [90, 90, 120],
                selection: [70, 70, 95],
                muted: [110, 110, 130],
            },
            ThemePreset::Light => PaletteRgb {
                background: [250, 250, 252],
                foreground: [28, 28, 32],
                accent: [0, 110, 150],
                positive: [0, 130, 70],
                negative: [180, 40, 50],
                border: [160, 160, 175],
                selection: [210, 210, 225],
                muted: [120, 120, 135],
            },
            ThemePreset::HighContrast => PaletteRgb {
                background: [0, 0, 0],
                foreground: [255, 255, 255],
                accent: [255, 255, 0],
                positive: [0, 255, 0],
                negative: [255, 0, 0],
                border: [255, 255, 255],
                selection: [85, 85, 85],
                muted: [192, 192, 192],
            },
        }
    }
}

/// RGB palette after merging preset + overrides (Issue #14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteRgb {
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub accent: [u8; 3],
    pub positive: [u8; 3],
    pub negative: [u8; 3],
    pub border: [u8; 3],
    pub selection: [u8; 3],
    pub muted: [u8; 3],
}

/// Optional per-slot hex overrides (`#rgb` / `#rrggbb`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemePalette {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub positive: Option<String>,
    #[serde(default)]
    pub negative: Option<String>,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub selection: Option<String>,
    #[serde(default)]
    pub muted: Option<String>,
}

impl ThemePalette {
    pub fn is_empty(&self) -> bool {
        self.background.is_none()
            && self.foreground.is_none()
            && self.accent.is_none()
            && self.positive.is_none()
            && self.negative.is_none()
            && self.border.is_none()
            && self.selection.is_none()
            && self.muted.is_none()
    }
}

/// On-disk theme (`~/.stockterm.json`). Legacy `accent_hex` / `background_hex` remain supported.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<ThemePreset>,
    #[serde(default, skip_serializing_if = "ThemePalette::is_empty")]
    pub overrides: ThemePalette,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_hex: Option<String>,
}

impl Theme {
    pub fn from_preset(preset: ThemePreset) -> Self {
        Self {
            preset: Some(preset),
            overrides: ThemePalette::default(),
            accent_hex: None,
            background_hex: None,
        }
    }

    /// Effective preset when `preset` is omitted (custom-only JSON uses default base).
    pub fn effective_preset(&self) -> ThemePreset {
        self.preset.unwrap_or(ThemePreset::BuiltinDefault)
    }

    /// Merge preset, `overrides`, and legacy hex fields into a single RGB palette.
    pub fn resolve_rgb(&self) -> PaletteRgb {
        let mut p = self.effective_preset().base_rgb();
        let o = &self.overrides;
        if let Some(c) = o.background.as_deref().and_then(parse_hex_rgb) {
            p.background = c;
        } else if let Some(c) = self.background_hex.as_deref().and_then(parse_hex_rgb) {
            p.background = c;
        }
        if let Some(c) = o.foreground.as_deref().and_then(parse_hex_rgb) {
            p.foreground = c;
        }
        if let Some(c) = o.accent.as_deref().and_then(parse_hex_rgb) {
            p.accent = c;
        } else if let Some(c) = self.accent_hex.as_deref().and_then(parse_hex_rgb) {
            p.accent = c;
        }
        if let Some(c) = o.positive.as_deref().and_then(parse_hex_rgb) {
            p.positive = c;
        }
        if let Some(c) = o.negative.as_deref().and_then(parse_hex_rgb) {
            p.negative = c;
        }
        if let Some(c) = o.border.as_deref().and_then(parse_hex_rgb) {
            p.border = c;
        }
        if let Some(c) = o.selection.as_deref().and_then(parse_hex_rgb) {
            p.selection = c;
        }
        if let Some(c) = o.muted.as_deref().and_then(parse_hex_rgb) {
            p.muted = c;
        }
        p
    }
}

/// Parse `#rgb` or `#rrggbb` (ASCII hex, case-insensitive). Leading/trailing ASCII whitespace trimmed.
pub fn parse_hex_rgb(s: &str) -> Option<[u8; 3]> {
    let t = s.trim();
    let t = t.strip_prefix('#')?;
    let t = t.trim();
    match t.len() {
        3 => {
            let r = u8::from_str_radix(&t[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&t[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&t[2..3].repeat(2), 16).ok()?;
            Some([r, g, b])
        }
        6 => {
            let r = u8::from_str_radix(&t[0..2], 16).ok()?;
            let g = u8::from_str_radix(&t[2..4], 16).ok()?;
            let b = u8::from_str_radix(&t[4..6], 16).ok()?;
            Some([r, g, b])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_accepts_shorthand_and_long() {
        assert_eq!(parse_hex_rgb("#0f0"), Some([0, 255, 0]));
        assert_eq!(parse_hex_rgb(" #00Ff00 "), Some([0, 255, 0]));
        assert_eq!(parse_hex_rgb("#aabbcc"), Some([0xaa, 0xbb, 0xcc]));
    }

    #[test]
    fn parse_hex_rejects_garbage() {
        assert_eq!(parse_hex_rgb("not-a-color"), None);
        assert_eq!(parse_hex_rgb("#gg0000"), None);
        assert_eq!(parse_hex_rgb("#00"), None);
    }

    #[test]
    fn resolve_preset_only_non_default_slots() {
        let t = Theme::from_preset(ThemePreset::Dark);
        let p = t.resolve_rgb();
        assert_eq!(p, ThemePreset::Dark.base_rgb());
    }

    #[test]
    fn partial_override_replaces_only_accent() {
        let t = Theme {
            preset: Some(ThemePreset::Dark),
            overrides: ThemePalette {
                accent: Some("#ffcc00".into()),
                ..Default::default()
            },
            accent_hex: None,
            background_hex: None,
        };
        let mut want = ThemePreset::Dark.base_rgb();
        want.accent = [255, 204, 0];
        assert_eq!(t.resolve_rgb(), want);
    }

    #[test]
    fn legacy_accent_hex_migrates_when_override_missing() {
        let t = Theme {
            preset: None,
            overrides: ThemePalette::default(),
            accent_hex: Some("#112233".into()),
            background_hex: None,
        };
        let mut want = ThemePreset::BuiltinDefault.base_rgb();
        want.accent = [0x11, 0x22, 0x33];
        assert_eq!(t.resolve_rgb(), want);
    }

    #[test]
    fn serde_roundtrip_legacy_json() {
        let json = r##"{"accent_hex":"#aabbcc","background_hex":null}"##;
        let t: Theme = serde_json::from_str(json).expect("legacy theme");
        assert_eq!(t.accent_hex.as_deref(), Some("#aabbcc"));
        assert_eq!(t.background_hex, None);
        let p = t.resolve_rgb();
        assert_eq!(p.accent, [0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn serde_preset_with_overrides() {
        let json = r##"{"preset":"dark","overrides":{"accent":"#ffcc00"}}"##;
        let t: Theme = serde_json::from_str(json).expect("preset theme");
        assert_eq!(t.preset, Some(ThemePreset::Dark));
        let p = t.resolve_rgb();
        assert_eq!(p.accent, [255, 204, 0]);
        assert_eq!(p.background, ThemePreset::Dark.base_rgb().background);
    }

    #[test]
    fn invalid_override_hex_falls_back_to_preset() {
        let t = Theme {
            preset: Some(ThemePreset::Light),
            overrides: ThemePalette {
                accent: Some("bad".into()),
                ..Default::default()
            },
            accent_hex: None,
            background_hex: None,
        };
        assert_eq!(
            t.resolve_rgb().accent,
            ThemePreset::Light.base_rgb().accent
        );
    }
}
