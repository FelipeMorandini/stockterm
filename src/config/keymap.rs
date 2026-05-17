//! User-configurable keyboard shortcuts (GitHub Issue #13 / SPEC §24; Issue #134 / §25;
//! Issue #136 / SPEC §26 — dialog digit + settings edit buffer actions).
//!
//! ## JSON shape
//!
//! Optional on [`crate::config::Config`]: `"keymap": { "<chord>": "<ActionName>", ... }`.
//! Chords combine modifiers with `+` (see README). Each **action** has a primary
//! [`BindingLayer`] via [`action_binding_layer`]; user overrides propagate to every layer where
//! [`default_bindings`] registers that action (Issue #134).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use thiserror::Error;

/// Tab / modal slice used for lookup (same physical key may bind to different [`Action`] values).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindingLayer {
    Global,
    ErrorOverlay,
    StockView,
    Charts,
    Search,
    News,
    SettingsBrowse,
    SettingsEdit,
    Portfolio,
    PortfolioRemoveArmed,
    PortfolioDialog,
    Alerts,
    AlertDialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Chord {
    #[inline]
    pub fn matches_key_event(&self, key: &KeyEvent) -> bool {
        key.code == self.code && key.modifiers == self.modifiers
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    Quit,
    OpenErrorLog,
    ForceRefresh,
    GlobalTab,
    GlobalBackTab,
    OverlayClose,
    OverlayScrollDown,
    OverlayScrollUp,
    OverlayPageDown,
    OverlayPageUp,
    StockFilterToggle,
    WatchlistAdd,
    WatchlistRemove,
    WatchlistRemoveShift,
    StockRowDown,
    StockRowUp,
    StockBackspace,
    StockEnter,
    ChartRangeD1,
    ChartRangeW1,
    ChartRangeM1,
    ChartRangeY1,
    ChartResetViewport,
    ChartZoomIn,
    ChartZoomOut,
    ChartPanLeft,
    ChartPanRight,
    ChartToggleCandle,
    SearchEsc,
    SearchBackspace,
    SearchEnter,
    SearchRowDown,
    SearchRowUp,
    NewsRowDown,
    NewsRowUp,
    NewsEnter,
    /// Copy selected article URL to clipboard (Issue #58 / SPEC §27).
    NewsCopyUrl,
    SettingsEscThemeDraft,
    SettingsThemePrev,
    SettingsThemeNext,
    SettingsRowDown,
    SettingsRowUp,
    SettingsEnter,
    SettingsEditEsc,
    SettingsEditEnter,
    SettingsEditBackspace,
    /// Refresh-rate row: digits `0`–`9` (Issue #136 / SPEC §26).
    SettingsEditDigit,
    /// Default-symbol row: letters, digits, `.`, `-` via explicit chords (§26).
    SettingsEditSymbolChar,
    PortfolioFilterToggle,
    PortfolioAdd,
    PortfolioRemoveArm,
    PortfolioRowDown,
    PortfolioRowUp,
    PortfolioEnterStock,
    PortfolioRemoveCancel,
    PortfolioRemoveDecline,
    PortfolioRemoveConfirm,
    PortfolioDialogEsc,
    PortfolioDialogFocusNext,
    PortfolioDialogBackspace,
    PortfolioDialogEnter,
    PortfolioDialogDigitOrDot,
    AlertAdd,
    AlertRemove,
    AlertRowUp,
    AlertRowDown,
    AlertDialogEsc,
    AlertDialogTab,
    AlertDialogShiftTab,
    AlertDialogLeft,
    AlertDialogRight,
    AlertDialogConditionCycleOrFocusNext,
    AlertDialogEnter,
    AlertDialogBackspace,
    /// Alert add dialog: default chords for digits and `.` on symbol/threshold fields (focus
    /// selects append rules). Symbol letters and condition `a`/`b` stay wildcard — Issue #136 / §26.
    AlertDialogDigitOrDot,
}

#[inline]
pub fn action_binding_layer(a: Action) -> BindingLayer {
    use Action::*;
    match a {
        Quit | OpenErrorLog | ForceRefresh | GlobalTab | GlobalBackTab => BindingLayer::Global,
        OverlayClose | OverlayScrollDown | OverlayScrollUp | OverlayPageDown | OverlayPageUp => {
            BindingLayer::ErrorOverlay
        }
        StockFilterToggle | WatchlistAdd | WatchlistRemove | WatchlistRemoveShift | StockRowDown
        | StockRowUp | StockBackspace | StockEnter => BindingLayer::StockView,
        ChartRangeD1 | ChartRangeW1 | ChartRangeM1 | ChartRangeY1 | ChartResetViewport
        | ChartZoomIn | ChartZoomOut | ChartPanLeft | ChartPanRight | ChartToggleCandle => {
            BindingLayer::Charts
        }
        SearchEsc | SearchBackspace | SearchEnter | SearchRowDown | SearchRowUp => {
            BindingLayer::Search
        }
        NewsRowDown | NewsRowUp | NewsEnter | NewsCopyUrl => BindingLayer::News,
        SettingsEscThemeDraft | SettingsThemePrev | SettingsThemeNext | SettingsRowDown
        | SettingsRowUp | SettingsEnter => BindingLayer::SettingsBrowse,
        SettingsEditEsc | SettingsEditEnter | SettingsEditBackspace | SettingsEditDigit
        | SettingsEditSymbolChar => BindingLayer::SettingsEdit,
        PortfolioFilterToggle | PortfolioAdd | PortfolioRemoveArm | PortfolioRowDown
        | PortfolioRowUp | PortfolioEnterStock => BindingLayer::Portfolio,
        PortfolioRemoveCancel | PortfolioRemoveDecline | PortfolioRemoveConfirm => {
            BindingLayer::PortfolioRemoveArmed
        }
        PortfolioDialogEsc | PortfolioDialogFocusNext | PortfolioDialogBackspace
        | PortfolioDialogEnter | PortfolioDialogDigitOrDot => BindingLayer::PortfolioDialog,
        AlertAdd | AlertRemove | AlertRowUp | AlertRowDown => BindingLayer::Alerts,
        AlertDialogEsc | AlertDialogTab | AlertDialogShiftTab | AlertDialogLeft
        | AlertDialogRight | AlertDialogConditionCycleOrFocusNext | AlertDialogEnter
        | AlertDialogBackspace | AlertDialogDigitOrDot => BindingLayer::AlertDialog,
    }
}

/// Layers that receive user `keymap` updates for `action` (all layers where [`default_bindings`]
/// registers that action). Issue #134 / SPEC §25.
#[inline]
pub(crate) fn action_overlay_layers(action: Action) -> &'static [BindingLayer] {
    static EMPTY: &[BindingLayer] = &[];
    overlay_layer_index()
        .get(&action)
        .map(|v| v.as_slice())
        .unwrap_or(EMPTY)
}

fn overlay_layer_index() -> &'static HashMap<Action, Vec<BindingLayer>> {
    static INDEX: OnceLock<HashMap<Action, Vec<BindingLayer>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map: HashMap<Action, Vec<BindingLayer>> = HashMap::new();
        for &(layer, _, action) in default_bindings() {
            let layers = map.entry(action).or_default();
            if !layers.contains(&layer) {
                layers.push(layer);
            }
        }
        map
    })
}

#[derive(Debug, Error)]
pub enum KeymapParseError {
    #[error("empty chord")]
    EmptyChord,
    #[error("unknown chord token: {0}")]
    UnknownToken(String),
    #[error("invalid chord: {0}")]
    InvalidChord(String),
}

/// Parse chord text from JSON (ASCII; modifiers + key tokens).
pub fn parse_chord(s: &str) -> Result<Chord, KeymapParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(KeymapParseError::EmptyChord);
    }
    let lower = s.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split('+').map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Err(KeymapParseError::EmptyChord);
    }

    let mut mods = KeyModifiers::NONE;
    let mut key_tokens: Vec<&str> = Vec::new();
    for p in &parts {
        match *p {
            "shift" => mods |= KeyModifiers::SHIFT,
            "ctrl" | "control" => mods |= KeyModifiers::CONTROL,
            "alt" => mods |= KeyModifiers::ALT,
            _ => key_tokens.push(p),
        }
    }
    if key_tokens.len() != 1 {
        return Err(KeymapParseError::InvalidChord(s.to_string()));
    }
    let tail = key_tokens[0];
    let (code, tail_mods) = parse_key_token(tail)?;
    Ok(Chord {
        code,
        modifiers: mods | tail_mods,
    })
}

fn parse_key_token(tail: &str) -> Result<(KeyCode, KeyModifiers), KeymapParseError> {
    if let Some(rest) = tail.strip_prefix("char:") {
        let mut it = rest.chars();
        let c = it.next().ok_or_else(|| KeymapParseError::InvalidChord(tail.to_string()))?;
        if it.next().is_some() {
            return Err(KeymapParseError::InvalidChord(tail.to_string()));
        }
        return Ok((KeyCode::Char(c), KeyModifiers::NONE));
    }
    match tail {
        "tab" => Ok((KeyCode::Tab, KeyModifiers::NONE)),
        "backtab" => Ok((KeyCode::BackTab, KeyModifiers::NONE)),
        "esc" | "escape" => Ok((KeyCode::Esc, KeyModifiers::NONE)),
        "enter" | "return" => Ok((KeyCode::Enter, KeyModifiers::NONE)),
        "backspace" => Ok((KeyCode::Backspace, KeyModifiers::NONE)),
        "up" => Ok((KeyCode::Up, KeyModifiers::NONE)),
        "down" => Ok((KeyCode::Down, KeyModifiers::NONE)),
        "left" => Ok((KeyCode::Left, KeyModifiers::NONE)),
        "right" => Ok((KeyCode::Right, KeyModifiers::NONE)),
        "pageup" => Ok((KeyCode::PageUp, KeyModifiers::NONE)),
        "pagedown" => Ok((KeyCode::PageDown, KeyModifiers::NONE)),
        "colon" => Ok((KeyCode::Char(':'), KeyModifiers::NONE)),
        "semicolon" => Ok((KeyCode::Char(';'), KeyModifiers::NONE)),
        "slash" => Ok((KeyCode::Char('/'), KeyModifiers::NONE)),
        "plus" => Ok((KeyCode::Char('+'), KeyModifiers::NONE)),
        "minus" => Ok((KeyCode::Char('-'), KeyModifiers::NONE)),
        _ if tail.len() == 1 => {
            let c = tail.chars().next().unwrap();
            if c.is_ascii_graphic() && c != '+' {
                Ok((KeyCode::Char(c), KeyModifiers::NONE))
            } else {
                Err(KeymapParseError::UnknownToken(tail.to_string()))
            }
        }
        _ => Err(KeymapParseError::UnknownToken(tail.to_string())),
    }
}

type LayerMap = HashMap<Chord, Action>;

#[derive(Debug, Clone)]
pub struct ResolvedKeymap {
    layers: HashMap<BindingLayer, LayerMap>,
}

impl ResolvedKeymap {
    pub fn build(raw: Option<&HashMap<String, String>>) -> (Self, Option<String>) {
        let mut layers: HashMap<BindingLayer, LayerMap> = HashMap::new();
        if let Err(e) = insert_defaults(&mut layers) {
            debug_assert!(false, "builtin defaults: {e}");
        }
        let Some(user) = raw else {
            return (Self { layers }, None);
        };
        if user.is_empty() {
            return (Self { layers }, None);
        }
        match apply_user_overlay(&layers, user) {
            Ok(next) => (Self { layers: next }, None),
            Err(e) => {
                let mut fresh: HashMap<BindingLayer, LayerMap> = HashMap::new();
                let _ = insert_defaults(&mut fresh);
                (
                    Self { layers: fresh },
                    Some(format!("keymap: {e}")),
                )
            }
        }
    }

    /// Resolve `key` to an [`Action`] in `layer`, accounting for terminal variance on **Shift+Tab**
    /// (see [`chord_lookup_candidates`]).
    #[inline]
    pub fn action(&self, layer: BindingLayer, key: &KeyEvent) -> Option<Action> {
        let map = self.layers.get(&layer)?;
        for ch in chord_lookup_candidates(key) {
            if let Some(a) = map.get(&ch) {
                return Some(*a);
            }
        }
        None
    }
}

/// Crossterm reports **Shift+Tab** inconsistently: `BackTab` + `SHIFT`, `BackTab` + `NONE`, or
/// `Tab` + `SHIFT` only. Default bindings use `backtab` → [`KeyCode::BackTab`] + [`KeyModifiers::NONE`];
/// we try exact `key` first, then these aliases so `GlobalBackTab` / dialog `BackTab` paths keep working.
fn chord_lookup_candidates(key: &KeyEvent) -> impl Iterator<Item = Chord> {
    let mut out = Vec::with_capacity(4);
    out.push(Chord {
        code: key.code,
        modifiers: key.modifiers,
    });

    let meta = key.modifiers.intersects(
        KeyModifiers::CONTROL
            | KeyModifiers::ALT
            | KeyModifiers::META
            | KeyModifiers::SUPER
            | KeyModifiers::HYPER,
    );
    if !meta {
        if key.code == KeyCode::BackTab && key.modifiers.contains(KeyModifiers::SHIFT) {
            out.push(Chord {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE,
            });
        }
        // `contains(SHIFT)` is the idiomatic check for `KeyModifiers` bitflags (vs `== SHIFT`).
        if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
            out.push(Chord {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE,
            });
            out.push(Chord {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::SHIFT,
            });
        }
    }

    out.into_iter()
}

fn insert_defaults(layers: &mut HashMap<BindingLayer, LayerMap>) -> Result<(), String> {
    let mut seen: HashSet<(BindingLayer, Chord)> = HashSet::new();
    for &(layer, chord_s, action) in default_bindings() {
        let ch = parse_chord(chord_s).map_err(|e| format!("default {chord_s}: {e}"))?;
        if !seen.insert((layer, ch)) {
            return Err(format!("duplicate default chord {chord_s:?} in {layer:?}"));
        }
        layers.entry(layer).or_default().insert(ch, action);
    }
    Ok(())
}

fn apply_user_remap(
    out: &mut HashMap<BindingLayer, LayerMap>,
    chord_s: &str,
    chord: Chord,
    action: Action,
    targets: &[BindingLayer],
) -> Result<(), String> {
    if targets.is_empty() {
        return Err(format!("action {action:?} has no default bindings"));
    }
    for &layer in targets {
        let map = out.entry(layer).or_default();
        if let Some(&existing) = map.get(&chord) {
            if existing != action {
                return Err(format!(
                    "chord {chord_s:?} already maps to {existing:?} (cannot map to {action:?})"
                ));
            }
        }
    }
    for &layer in targets {
        let map = out.entry(layer).or_default();
        map.retain(|c, a| !(*a == action && *c != chord));
        map.insert(chord, action);
    }
    Ok(())
}

fn apply_user_overlay(
    base: &HashMap<BindingLayer, LayerMap>,
    user: &HashMap<String, String>,
) -> Result<HashMap<BindingLayer, LayerMap>, String> {
    let mut out = base.clone();
    for (chord_s, action_s) in user {
        let chord = parse_chord(chord_s).map_err(|e| format!("{e} (chord {chord_s:?})"))?;
        let action: Action = serde_json::from_value(serde_json::Value::String(action_s.clone()))
            .map_err(|_| format!("unknown action name {action_s:?}"))?;
        apply_user_remap(
            &mut out,
            chord_s,
            chord,
            action,
            action_overlay_layers(action),
        )?;
    }
    Ok(out)
}

/// Built-in rows before Issue #136 programmatic digit/symbol chords (see [`default_bindings`]).
#[allow(clippy::redundant_static_lifetimes)]
const CORE_DEFAULTS: &[(BindingLayer, &'static str, Action)] = {
    use Action::*;
    use BindingLayer::*;
    &[
        (Global, "q", Quit),
        (Global, "ctrl+e", OpenErrorLog),
        (Global, "ctrl+r", ForceRefresh),
        (Global, "tab", GlobalTab),
        (Global, "backtab", GlobalBackTab),
        (ErrorOverlay, "esc", OverlayClose),
        (ErrorOverlay, "char:j", OverlayScrollDown),
        (ErrorOverlay, "down", OverlayScrollDown),
        (ErrorOverlay, "char:k", OverlayScrollUp),
        (ErrorOverlay, "up", OverlayScrollUp),
        (ErrorOverlay, "pagedown", OverlayPageDown),
        (ErrorOverlay, "pageup", OverlayPageUp),
        (StockView, "slash", StockFilterToggle),
        (StockView, "char:w", WatchlistAdd),
        (StockView, "char:x", WatchlistRemove),
        (StockView, "shift+d", WatchlistRemoveShift),
        (StockView, "char:j", StockRowDown),
        (StockView, "down", StockRowDown),
        (StockView, "char:k", StockRowUp),
        (StockView, "up", StockRowUp),
        (StockView, "backspace", StockBackspace),
        (StockView, "enter", StockEnter),
        (Charts, "1", ChartRangeD1),
        (Charts, "2", ChartRangeW1),
        (Charts, "3", ChartRangeM1),
        (Charts, "4", ChartRangeY1),
        (Charts, "0", ChartResetViewport),
        (Charts, "plus", ChartZoomIn),
        (Charts, "shift+=", ChartZoomIn),
        (Charts, "minus", ChartZoomOut),
        (Charts, "char:h", ChartPanLeft),
        (Charts, "char:l", ChartPanRight),
        (Charts, "left", ChartPanLeft),
        (Charts, "right", ChartPanRight),
        (Charts, "char:c", ChartToggleCandle),
        (Search, "esc", SearchEsc),
        (Search, "backspace", SearchBackspace),
        (Search, "enter", SearchEnter),
        (Search, "char:j", SearchRowDown),
        (Search, "down", SearchRowDown),
        (Search, "char:k", SearchRowUp),
        (Search, "up", SearchRowUp),
        (News, "char:j", NewsRowDown),
        (News, "down", NewsRowDown),
        (News, "char:k", NewsRowUp),
        (News, "up", NewsRowUp),
        (News, "enter", NewsEnter),
        (News, "char:c", NewsCopyUrl),
        (SettingsBrowse, "esc", SettingsEscThemeDraft),
        (SettingsBrowse, "char:h", SettingsThemePrev),
        (SettingsBrowse, "char:l", SettingsThemeNext),
        (SettingsBrowse, "left", SettingsThemePrev),
        (SettingsBrowse, "right", SettingsThemeNext),
        (SettingsBrowse, "char:j", SettingsRowDown),
        (SettingsBrowse, "down", SettingsRowDown),
        (SettingsBrowse, "char:k", SettingsRowUp),
        (SettingsBrowse, "up", SettingsRowUp),
        (SettingsBrowse, "enter", SettingsEnter),
        (SettingsEdit, "esc", SettingsEditEsc),
        (SettingsEdit, "enter", SettingsEditEnter),
        (SettingsEdit, "backspace", SettingsEditBackspace),
        (Portfolio, "slash", PortfolioFilterToggle),
        (Portfolio, "char:a", PortfolioAdd),
        (Portfolio, "char:d", PortfolioRemoveArm),
        (Portfolio, "char:j", PortfolioRowDown),
        (Portfolio, "down", PortfolioRowDown),
        (Portfolio, "char:k", PortfolioRowUp),
        (Portfolio, "up", PortfolioRowUp),
        (Portfolio, "enter", PortfolioEnterStock),
        (PortfolioRemoveArmed, "esc", PortfolioRemoveCancel),
        (PortfolioRemoveArmed, "char:n", PortfolioRemoveDecline),
        (PortfolioRemoveArmed, "char:d", PortfolioRemoveConfirm),
        (PortfolioRemoveArmed, "char:y", PortfolioRemoveConfirm),
        (PortfolioRemoveArmed, "up", PortfolioRowUp),
        (PortfolioRemoveArmed, "down", PortfolioRowDown),
        (PortfolioRemoveArmed, "char:j", PortfolioRowDown),
        (PortfolioRemoveArmed, "char:k", PortfolioRowUp),
        (PortfolioDialog, "esc", PortfolioDialogEsc),
        (PortfolioDialog, "semicolon", PortfolioDialogFocusNext),
        (PortfolioDialog, "backspace", PortfolioDialogBackspace),
        (PortfolioDialog, "enter", PortfolioDialogEnter),
        (Alerts, "char:a", AlertAdd),
        (Alerts, "char:d", AlertRemove),
        (Alerts, "up", AlertRowUp),
        (Alerts, "down", AlertRowDown),
        (AlertDialog, "esc", AlertDialogEsc),
        (AlertDialog, "tab", AlertDialogTab),
        (AlertDialog, "backtab", AlertDialogShiftTab),
        (AlertDialog, "left", AlertDialogLeft),
        (AlertDialog, "right", AlertDialogRight),
        (AlertDialog, "char:;", AlertDialogConditionCycleOrFocusNext),
        (AlertDialog, "enter", AlertDialogEnter),
        (AlertDialog, "backspace", AlertDialogBackspace),
    ]
};

fn build_default_bindings_with_issue136_rows() -> &'static [(BindingLayer, &'static str, Action)] {
    use Action::*;
    use BindingLayer::*;
    let mut v: Vec<(BindingLayer, &'static str, Action)> = CORE_DEFAULTS.to_vec();
    for d in '0'..='9' {
        let s: &'static str = Box::leak(format!("char:{d}").into_boxed_str());
        v.push((PortfolioDialog, s, PortfolioDialogDigitOrDot));
        v.push((AlertDialog, s, AlertDialogDigitOrDot));
        v.push((SettingsEdit, s, SettingsEditDigit));
    }
    let dot: &'static str = Box::leak(Box::from("char:."));
    v.push((PortfolioDialog, dot, PortfolioDialogDigitOrDot));
    v.push((AlertDialog, dot, AlertDialogDigitOrDot));
    for c in 'a'..='z' {
        let s: &'static str = Box::leak(format!("char:{c}").into_boxed_str());
        v.push((SettingsEdit, s, SettingsEditSymbolChar));
    }
    for sym in ['.', '-'] {
        let s: &'static str = Box::leak(format!("char:{sym}").into_boxed_str());
        v.push((SettingsEdit, s, SettingsEditSymbolChar));
    }
    Box::leak(v.into_boxed_slice())
}

fn default_bindings() -> &'static [(BindingLayer, &'static str, Action)] {
    static ALL: OnceLock<&'static [(BindingLayer, &'static str, Action)]> = OnceLock::new();
    ALL.get_or_init(build_default_bindings_with_issue136_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    #[test]
    fn parse_chord_q_and_colon() {
        let q = parse_chord("q").unwrap();
        assert_eq!(q.code, KeyCode::Char('q'));
        assert_eq!(q.modifiers, KeyModifiers::NONE);
        let colon = parse_chord("colon").unwrap();
        assert_eq!(colon.code, KeyCode::Char(':'));
        let c2 = parse_chord(":").unwrap();
        assert_eq!(c2.code, KeyCode::Char(':'));
    }

    #[test]
    fn parse_shift_d() {
        let ch = parse_chord("shift+d").unwrap();
        assert_eq!(ch.code, KeyCode::Char('d'));
        assert!(ch.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn overlay_remap_quit_to_colon() {
        let mut m = HashMap::new();
        m.insert("colon".to_string(), "Quit".to_string());
        let (km, err) = ResolvedKeymap::build(Some(&m));
        assert!(err.is_none());
        let quit_key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
        assert_eq!(km.action(BindingLayer::Global, &quit_key), Some(Action::Quit));
        let old = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(km.action(BindingLayer::Global, &old), None);
    }

    #[test]
    fn invalid_action_rejects_overlay() {
        let mut m = HashMap::new();
        m.insert("q".to_string(), "NotAnAction".to_string());
        let (km, err) = ResolvedKeymap::build(Some(&m));
        assert!(err.is_some());
        let q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(km.action(BindingLayer::Global, &q), Some(Action::Quit));
    }

    #[test]
    fn default_stock_j_is_row_down() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::StockView, &j),
            Some(Action::StockRowDown)
        );
        assert_eq!(
            km.action(BindingLayer::Search, &j),
            Some(Action::SearchRowDown)
        );
    }

    #[test]
    fn shift_tab_variants_resolve_global_back_tab() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let tab_shift = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
        assert_eq!(
            km.action(BindingLayer::Global, &tab_shift),
            Some(Action::GlobalBackTab)
        );
        let backtab_shift = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(
            km.action(BindingLayer::Global, &backtab_shift),
            Some(Action::GlobalBackTab)
        );
        let backtab_plain = KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::Global, &backtab_plain),
            Some(Action::GlobalBackTab)
        );
    }

    #[test]
    fn shift_tab_alert_dialog_matches_backtab_binding() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let tab_shift = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
        assert_eq!(
            km.action(BindingLayer::AlertDialog, &tab_shift),
            Some(Action::AlertDialogShiftTab)
        );
    }

    #[test]
    fn shift_tab_tab_with_super_does_not_alias_to_backtab() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let tab_shift_super = KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::SHIFT | KeyModifiers::SUPER,
        );
        assert_eq!(km.action(BindingLayer::Global, &tab_shift_super), None);
    }

    #[test]
    fn primary_binding_layer_is_in_overlay_layers_for_defaults() {
        let mut seen = HashSet::new();
        for &(_, _, action) in default_bindings() {
            if !seen.insert(action) {
                continue;
            }
            let primary = action_binding_layer(action);
            let layers = action_overlay_layers(action);
            assert!(
                layers.contains(&primary),
                "{action:?}: primary {primary:?} not in overlay layers {layers:?}"
            );
        }
    }

    #[test]
    fn overlay_layers_include_portfolio_and_armed_for_row_actions() {
        let layers = action_overlay_layers(Action::PortfolioRowDown);
        assert!(layers.contains(&BindingLayer::Portfolio));
        assert!(layers.contains(&BindingLayer::PortfolioRemoveArmed));
        let up = action_overlay_layers(Action::PortfolioRowUp);
        assert!(up.contains(&BindingLayer::Portfolio));
        assert!(up.contains(&BindingLayer::PortfolioRemoveArmed));
    }

    #[test]
    fn remap_portfolio_row_down_propagates_to_armed_layer() {
        let mut m = HashMap::new();
        // `char:u` is free on both portfolio layers (`char:n` is decline on armed).
        m.insert("char:u".to_string(), "PortfolioRowDown".to_string());
        let (km, err) = ResolvedKeymap::build(Some(&m));
        assert!(err.is_none(), "unexpected overlay error: {err:?}");
        let u = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::Portfolio, &u),
            Some(Action::PortfolioRowDown)
        );
        assert_eq!(
            km.action(BindingLayer::PortfolioRemoveArmed, &u),
            Some(Action::PortfolioRowDown)
        );
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(km.action(BindingLayer::Portfolio, &j), None);
        assert_eq!(km.action(BindingLayer::PortfolioRemoveArmed, &j), None);
    }

    #[test]
    fn remap_portfolio_remove_confirm_only_armed_layer() {
        let mut m = HashMap::new();
        m.insert("char:z".to_string(), "PortfolioRemoveConfirm".to_string());
        let (km, err) = ResolvedKeymap::build(Some(&m));
        assert!(err.is_none());
        let z = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::PortfolioRemoveArmed, &z),
            Some(Action::PortfolioRemoveConfirm)
        );
        assert_eq!(km.action(BindingLayer::Portfolio, &z), None);
        let d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::Portfolio, &d),
            Some(Action::PortfolioRemoveArm)
        );
    }

    #[test]
    fn overlay_rejects_action_without_default_bindings() {
        let mut base: HashMap<BindingLayer, LayerMap> = HashMap::new();
        insert_defaults(&mut base).unwrap();
        let chord = parse_chord("q").unwrap();
        let err = apply_user_remap(
            &mut base,
            "q",
            chord,
            Action::Quit,
            &[],
        );
        assert!(err.is_err());
        assert!(
            err.unwrap_err()
                .contains("has no default bindings")
        );
    }

    #[test]
    fn issue136_portfolio_dialog_digit_or_dot_defaults() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let five = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::PortfolioDialog, &five),
            Some(Action::PortfolioDialogDigitOrDot)
        );
        let dot = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::PortfolioDialog, &dot),
            Some(Action::PortfolioDialogDigitOrDot)
        );
        let n = default_bindings()
            .iter()
            .filter(|&&(_, _, a)| a == Action::PortfolioDialogDigitOrDot)
            .count();
        assert_eq!(n, 11);
    }

    #[test]
    fn issue136_alert_dialog_digit_or_dot_defaults() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let three = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::AlertDialog, &three),
            Some(Action::AlertDialogDigitOrDot)
        );
        let n = default_bindings()
            .iter()
            .filter(|&&(_, _, a)| a == Action::AlertDialogDigitOrDot)
            .count();
        assert_eq!(n, 11);
    }

    #[test]
    fn issue136_settings_edit_digit_and_symbol_defaults() {
        let (km, err) = ResolvedKeymap::build(None);
        assert!(err.is_none());
        let d = KeyEvent::new(KeyCode::Char('7'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::SettingsEdit, &d),
            Some(Action::SettingsEditDigit)
        );
        let z = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::SettingsEdit, &z),
            Some(Action::SettingsEditSymbolChar)
        );
        let n_digit = default_bindings()
            .iter()
            .filter(|&&(_, _, a)| a == Action::SettingsEditDigit)
            .count();
        assert_eq!(n_digit, 10);
    }

    #[test]
    fn default_map_binds_news_copy_url() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let km = ResolvedKeymap::build(None).0;
        let c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(km.action(BindingLayer::News, &c), Some(Action::NewsCopyUrl));
    }

    #[test]
    fn propagation_rejects_on_sibling_chord_conflict() {
        let mut m = HashMap::new();
        m.insert("char:d".to_string(), "PortfolioRowDown".to_string());
        let (km, err) = ResolvedKeymap::build(Some(&m));
        assert!(err.is_some());
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(
            km.action(BindingLayer::Portfolio, &j),
            Some(Action::PortfolioRowDown)
        );
        assert_eq!(
            km.action(BindingLayer::PortfolioRemoveArmed, &j),
            Some(Action::PortfolioRowDown)
        );
    }
}
