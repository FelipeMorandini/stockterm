//! Parameters for [`MarketDataProvider::get_historical`](crate::api::provider::MarketDataProvider).

/// Provider-agnostic historical chart request (M4 / Issue #9).
#[derive(Debug, Clone, Copy)]
pub struct HistoricalQuery<'a> {
    pub from: &'a str,
    pub to: &'a str,
    /// Yahoo v8 `interval=` (e.g. `5m`, `1d`, `1wk`).
    pub bar_interval: &'a str,
    /// When set, Yahoo uses `range=` + `interval=` and ignores calendar `from`/`to`.
    pub yahoo_range: Option<&'static str>,
    pub polygon_multiplier: u32,
    pub polygon_timespan: &'a str,
    /// Polygon aggregates `limit=` cap for this request (Yahoo ignores this field).
    pub polygon_limit: u32,
}

/// Returns `t` verbatim when already in ms (≥ §65 threshold), else `t * 1_000`.
///
/// Centralizes the §65 invariant. Yahoo callers pass `t_sec`; Polygon callers
/// pass the upstream `t` (already ms).
///
/// Issue #200 / SPEC §65.1
#[inline]
pub(crate) fn normalize_bar_timestamp_to_ms(t_raw: u64) -> u64 {
    // Threshold pinned to 2001-09-09T01:46:40Z (10^12 ms). Anything ≥ this is ms.
    const MS_EPOCH_FLOOR: u64 = 1_000_000_000_000;
    if t_raw >= MS_EPOCH_FLOOR {
        t_raw
    } else {
        t_raw.saturating_mul(1_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_bar_timestamp_to_ms_passthrough_for_ms() {
        assert_eq!(normalize_bar_timestamp_to_ms(1_700_000_000_000), 1_700_000_000_000);
        assert_eq!(normalize_bar_timestamp_to_ms(1_000_000_000_000), 1_000_000_000_000);
    }

    #[test]
    fn normalize_bar_timestamp_to_ms_upscales_seconds() {
        assert_eq!(normalize_bar_timestamp_to_ms(1_700_000_000), 1_700_000_000_000);
        assert_eq!(normalize_bar_timestamp_to_ms(1), 1_000);
    }

    #[test]
    fn normalize_bar_timestamp_to_ms_saturates() {
        assert_eq!(normalize_bar_timestamp_to_ms(u64::MAX), u64::MAX);
    }

    #[test]
    fn normalize_bar_timestamp_to_ms_zero() {
        assert_eq!(normalize_bar_timestamp_to_ms(0), 0);
    }
}
