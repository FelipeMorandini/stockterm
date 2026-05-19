//! Keyboard modifier helpers (Issue #44): allow Shift with letters, reject meta chords.

use crossterm::event::KeyModifiers;

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

#[cfg(test)]
mod tests {
    use super::letter_key_plain;
    use crossterm::event::KeyModifiers;

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
}
