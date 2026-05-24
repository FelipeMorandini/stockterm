//! Resolved ratatui colors from [`crate::config::theme`] (Issue #14 / SPEC §21).

use ratatui::style::{Color, Style};

use crate::config::theme::PaletteRgb;

/// Cheap fingerprint of a resolved [`PaletteRgb`] for cache invalidation (Issues #195 / §61).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThemeStamp(u64);

impl ThemeStamp {
    /// Deterministic FNV-1a hash of all eight RGB slots (24 bytes).
    pub fn from_palette(p: &PaletteRgb) -> Self {
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        let slots = [
            p.background,
            p.foreground,
            p.accent,
            p.positive,
            p.negative,
            p.border,
            p.selection,
            p.muted,
        ];
        let mut hash = FNV_OFFSET;
        for slot in slots {
            for byte in slot {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
        Self(hash)
    }
}

/// Ratatui colors for one frame (derived from `Config.theme` + Settings preview).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedTheme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub positive: Color,
    pub negative: Color,
    pub border: Color,
    pub selection: Color,
    pub muted: Color,
}

impl ResolvedTheme {
    pub fn from_palette(p: PaletteRgb) -> Self {
        Self {
            background: Color::Rgb(p.background[0], p.background[1], p.background[2]),
            foreground: Color::Rgb(p.foreground[0], p.foreground[1], p.foreground[2]),
            accent: Color::Rgb(p.accent[0], p.accent[1], p.accent[2]),
            positive: Color::Rgb(p.positive[0], p.positive[1], p.positive[2]),
            negative: Color::Rgb(p.negative[0], p.negative[1], p.negative[2]),
            border: Color::Rgb(p.border[0], p.border[1], p.border[2]),
            selection: Color::Rgb(p.selection[0], p.selection[1], p.selection[2]),
            muted: Color::Rgb(p.muted[0], p.muted[1], p.muted[2]),
        }
    }

    /// Arbitrary foreground on the theme background (e.g. P/L column colors).
    pub fn fg_color(self, c: Color) -> Style {
        Style::default().fg(c).bg(self.background)
    }

    /// Default fill + primary text (Issue #14 — terminals do not auto-paint `background`; we do).
    pub fn canvas(self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }

    pub fn fg_accent(self) -> Style {
        Style::default().fg(self.accent).bg(self.background)
    }

    pub fn fg_foreground(self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    pub fn fg_positive(self) -> Style {
        Style::default().fg(self.positive).bg(self.background)
    }

    pub fn fg_negative(self) -> Style {
        Style::default().fg(self.negative).bg(self.background)
    }

    pub fn fg_muted(self) -> Style {
        Style::default().fg(self.muted).bg(self.background)
    }

    pub fn fg_border(self) -> Style {
        Style::default().fg(self.border).bg(self.background)
    }

    pub fn startup_banner(self) -> Style {
        Style::default().fg(self.negative).bg(self.selection)
    }

    pub fn error_text(self) -> Style {
        Style::default().fg(self.negative).bg(self.background)
    }

    pub fn success_text(self) -> Style {
        Style::default().fg(self.positive).bg(self.background)
    }

    pub fn warning_text(self) -> Style {
        Style::default().fg(self.border).bg(self.background)
    }

    pub fn highlight_symbol(self) -> Style {
        Style::default().fg(self.border).bg(self.background)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::theme::ThemePreset;

    #[test]
    fn theme_stamp_from_palette_differs_by_preset() {
        let dark = ThemePreset::Dark.base_rgb();
        let light = ThemePreset::Light.base_rgb();
        assert_ne!(
            ThemeStamp::from_palette(&dark),
            ThemeStamp::from_palette(&light)
        );
    }

    #[test]
    fn theme_stamp_from_palette_stable() {
        let p = ThemePreset::Dark.base_rgb();
        assert_eq!(ThemeStamp::from_palette(&p), ThemeStamp::from_palette(&p));
    }
}
