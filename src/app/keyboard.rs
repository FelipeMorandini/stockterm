//! Keyboard modifier helpers (Issue #44): allow Shift with letters, reject meta chords.

use crate::config::keymap::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// True if `m` has no Control / Alt / Meta / Hyper / Super (Shift is allowed).
pub fn letter_key_plain(m: KeyModifiers) -> bool {
    !m.contains(KeyModifiers::CONTROL)
        && !m.contains(KeyModifiers::ALT)
        && !m.contains(KeyModifiers::META)
        && !m.contains(KeyModifiers::HYPER)
        && !m.contains(KeyModifiers::SUPER)
}

/// True for unmodified **Tab** / **BackTab** routing (Issue #82 / SPEC §36.2) — same meta rules as
/// [`letter_key_plain`].
#[inline]
pub fn tab_key_plain(m: KeyModifiers) -> bool {
    letter_key_plain(m)
}

/// True when this key should trigger global quit (Issue #51 / SPEC §42.1).
///
/// Accepts **`q`** / **`Q`** with [`letter_key_plain`] modifiers. Custom keymap remaps
/// (non-`q` chords) are handled separately via exact chord match in [`handle_event`](crate::app::handlers::handle_event).
pub fn global_quit_key(key: &KeyEvent) -> bool {
    if let KeyCode::Char(c) = key.code {
        return c.eq_ignore_ascii_case(&'q') && letter_key_plain(key.modifiers);
    }
    false
}

/// Whether this key should quit the app (Issue #51 / SPEC §42.1).
///
/// Honors exact **`Action::Quit`** chords (including remaps like **`colon`**). **`q`/`Q`**
/// wildcards apply only when **`q`** is unbound on **Global** or still mapped to
/// **`Quit`** — not when remapped to another global action (§24).
pub fn should_global_quit(key: &KeyEvent, global_action: Option<Action>) -> bool {
    matches!(global_action, Some(Action::Quit))
        || (global_quit_key(key) && !matches!(global_action, Some(a) if a != Action::Quit))
}

#[cfg(test)]
mod tests {
    use super::{global_quit_key, letter_key_plain, should_global_quit};
    use crate::config::keymap::Action;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventState, KeyModifiers};

    #[test]
    fn letter_key_plain_allows_none_and_shift() {
        assert!(letter_key_plain(KeyModifiers::NONE));
        assert!(letter_key_plain(KeyModifiers::SHIFT));
    }

    #[test]
    fn letter_key_plain_rejects_meta_chords() {
        assert!(!letter_key_plain(KeyModifiers::CONTROL));
        assert!(!letter_key_plain(KeyModifiers::ALT));
        assert!(!letter_key_plain(KeyModifiers::SUPER));
        assert!(!letter_key_plain(KeyModifiers::HYPER));
        assert!(!letter_key_plain(KeyModifiers::META));
        assert!(!letter_key_plain(KeyModifiers::CONTROL | KeyModifiers::SHIFT));
        assert!(!letter_key_plain(KeyModifiers::ALT | KeyModifiers::SHIFT));
    }

    #[test]
    fn tab_key_plain_matches_letter_key_plain() {
        use super::tab_key_plain;
        assert!(tab_key_plain(KeyModifiers::NONE));
        assert!(tab_key_plain(KeyModifiers::SHIFT));
        assert!(!tab_key_plain(KeyModifiers::CONTROL));
        assert!(!tab_key_plain(KeyModifiers::CONTROL | KeyModifiers::SHIFT));
    }

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: crossterm::event::KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn global_quit_key_accepts_q_and_shift_q() {
        assert!(global_quit_key(&key_event(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        )));
        assert!(global_quit_key(&key_event(
            KeyCode::Char('Q'),
            KeyModifiers::SHIFT,
        )));
    }

    #[test]
    fn global_quit_key_rejects_meta_chords() {
        assert!(!global_quit_key(&key_event(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL,
        )));
        assert!(!global_quit_key(&key_event(
            KeyCode::Char('Q'),
            KeyModifiers::ALT | KeyModifiers::SHIFT,
        )));
    }

    #[test]
    fn should_global_quit_shift_q_when_global_unbound() {
        assert!(should_global_quit(
            &key_event(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            None,
        ));
    }

    #[test]
    fn should_global_quit_not_when_q_remapped_on_global() {
        assert!(!should_global_quit(
            &key_event(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(Action::GlobalTab),
        ));
        assert!(!should_global_quit(
            &key_event(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            Some(Action::GlobalTab),
        ));
    }

    #[test]
    fn should_global_quit_exact_quit_chord() {
        assert!(should_global_quit(
            &key_event(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(Action::Quit),
        ));
    }
}
