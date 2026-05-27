use serde::{Deserialize, Serialize};

/// Upper bound on saved-filter display name length (§69.3.2).
pub const MAX_SAVED_FILTER_NAME_LEN: usize = 32;

/// Upper bound on persisted saved-filter entries (§69.3.2).
pub const MAX_SAVED_FILTERS: usize = 32;

/// Upper bound on saved-filter pattern length (same as §23.5 `MAX_FILTER_QUERY_LEN`).
pub const MAX_SAVED_FILTER_PATTERN_LEN: usize = 64;

/// Named table filter persisted in `~/.stockterm.json` (Issue #194 / SPEC §69).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedFilter {
    /// User-visible label for recall and save-as (not used in matching).
    pub name: String,
    /// Substring or regex pattern applied to the symbol column.
    pub pattern: String,
    /// When true, [`pattern`](Self::pattern) is compiled as a regex; otherwise substring match.
    pub regex: bool,
}

impl SavedFilter {
    /// Returns a copy trimmed to spec limits; `None` when name or pattern is empty after trim.
    pub fn sanitized(&self) -> Option<Self> {
        let name: String = self
            .name
            .trim()
            .chars()
            .take(MAX_SAVED_FILTER_NAME_LEN)
            .collect();
        let pattern: String = self
            .pattern
            .chars()
            .take(MAX_SAVED_FILTER_PATTERN_LEN)
            .collect();
        if name.is_empty() || pattern.is_empty() {
            return None;
        }
        Some(Self {
            name,
            pattern,
            regex: self.regex,
        })
    }
}

/// Drop invalid rows and cap list length after config load (§69.3.2).
pub fn sanitize_saved_filters(filters: &mut Vec<SavedFilter>) {
    let before = filters.len();
    filters.retain_mut(|row| {
        if let Some(s) = row.sanitized() {
            *row = s;
            true
        } else {
            false
        }
    });
    let dropped = before.saturating_sub(filters.len());
    if dropped > 0 {
        tracing::warn!(
            dropped,
            "removed invalid saved_filters rows while loading config"
        );
    }
    if filters.len() > MAX_SAVED_FILTERS {
        let excess = filters.len() - MAX_SAVED_FILTERS;
        tracing::warn!(
            excess,
            max = MAX_SAVED_FILTERS,
            "truncated saved_filters list while loading config"
        );
        filters.truncate(MAX_SAVED_FILTERS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let f = SavedFilter {
            name: "mega".to_string(),
            pattern: "^A".to_string(),
            regex: true,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: SavedFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn sanitize_rejects_empty_name() {
        let f = SavedFilter {
            name: "   ".to_string(),
            pattern: "a".to_string(),
            regex: false,
        };
        assert!(f.sanitized().is_none());
    }

    #[test]
    fn sanitize_drops_invalid_rows() {
        let mut filters = vec![
            SavedFilter {
                name: "ok".to_string(),
                pattern: "a".to_string(),
                regex: false,
            },
            SavedFilter {
                name: "  ".to_string(),
                pattern: "b".to_string(),
                regex: false,
            },
        ];
        sanitize_saved_filters(&mut filters);
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "ok");
    }
}
