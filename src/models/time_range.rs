//! Chart time window selection (Issues #9 / M4).

use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};

/// User-selected historical window on the Charts tab (`1`–`4` keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    D1,
    W1,
    #[default]
    M1,
    Y1,
}

impl TimeRange {
    pub const fn label(self) -> &'static str {
        match self {
            TimeRange::D1 => "1D",
            TimeRange::W1 => "1W",
            TimeRange::M1 => "1M",
            TimeRange::Y1 => "1Y",
        }
    }

    /// Maps to Yahoo v8 `range=` + `interval=` (see `HistoricalQueryParams::yahoo_range`).
    ///
    /// | Range | Yahoo `range` | `interval` | Polygon window | Polygon bars |
    /// |-------|---------------|------------|----------------|--------------|
    /// | D1 | 1d | 5m | 5 trading days back → today | 5 × minute |
    /// | W1 | 5d | 30m | 8 days → today | 30 × minute |
    /// | M1 | 1mo | 1d | 32 days → today | 1 × day |
    /// | Y1 | 1y | 1wk | 400 days → today | 1 × week |
    pub fn historical_params(self, now: DateTime<Local>) -> HistoricalQueryParams {
        let today = now.date_naive();
        match self {
            TimeRange::D1 => HistoricalQueryParams {
                from: (today - Duration::days(5))
                    .format("%Y-%m-%d")
                    .to_string(),
                to: today.format("%Y-%m-%d").to_string(),
                bar_interval: "5m",
                yahoo_range: Some("1d"),
                polygon_multiplier: 5,
                polygon_timespan: "minute",
                polygon_limit: polygon_historical_limit(TimeRange::D1),
            },
            TimeRange::W1 => HistoricalQueryParams {
                from: (today - Duration::days(8))
                    .format("%Y-%m-%d")
                    .to_string(),
                to: today.format("%Y-%m-%d").to_string(),
                bar_interval: "30m",
                yahoo_range: Some("5d"),
                polygon_multiplier: 30,
                polygon_timespan: "minute",
                polygon_limit: polygon_historical_limit(TimeRange::W1),
            },
            TimeRange::M1 => HistoricalQueryParams {
                from: (today - Duration::days(32))
                    .format("%Y-%m-%d")
                    .to_string(),
                to: today.format("%Y-%m-%d").to_string(),
                bar_interval: "1d",
                yahoo_range: Some("1mo"),
                polygon_multiplier: 1,
                polygon_timespan: "day",
                polygon_limit: polygon_historical_limit(TimeRange::M1),
            },
            TimeRange::Y1 => HistoricalQueryParams {
                from: (today - Duration::days(400))
                    .format("%Y-%m-%d")
                    .to_string(),
                to: today.format("%Y-%m-%d").to_string(),
                bar_interval: "1wk",
                yahoo_range: Some("1y"),
                polygon_multiplier: 1,
                polygon_timespan: "week",
                polygon_limit: polygon_historical_limit(TimeRange::Y1),
            },
        }
    }
}

/// Hard ceiling for any single Polygon aggregates request (Issue #65 / §52.1.1).
pub const POLYGON_AGG_LIMIT_CEILING: u32 = 5_000;

/// Per-`TimeRange` bar caps for Polygon `limit=` (sort=asc).
pub fn polygon_historical_limit(tr: TimeRange) -> u32 {
    let n = match tr {
        TimeRange::D1 => 500,
        TimeRange::W1 => 400,
        TimeRange::M1 => 45,
        TimeRange::Y1 => 60,
    };
    n.min(POLYGON_AGG_LIMIT_CEILING)
}

/// Owned parameters for a historical request; converted to [`crate::api::HistoricalQuery`] at the call site.
#[derive(Debug, Clone)]
pub struct HistoricalQueryParams {
    pub from: String,
    pub to: String,
    pub bar_interval: &'static str,
    pub yahoo_range: Option<&'static str>,
    pub polygon_multiplier: u32,
    pub polygon_timespan: &'static str,
    pub polygon_limit: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_time_range_is_m1() {
        assert_eq!(TimeRange::default(), TimeRange::M1);
    }

    #[test]
    fn historical_params_non_empty_dates() {
        let now = Local::now();
        for tr in [TimeRange::D1, TimeRange::W1, TimeRange::M1, TimeRange::Y1] {
            let p = tr.historical_params(now);
            assert!(!p.from.is_empty());
            assert!(!p.to.is_empty());
            assert!(p.yahoo_range.is_some());
        }
    }

    #[test]
    fn historical_params_includes_polygon_limit() {
        let now = Local::now();
        assert_eq!(
            TimeRange::D1.historical_params(now).polygon_limit,
            500
        );
        for tr in [TimeRange::D1, TimeRange::W1, TimeRange::M1, TimeRange::Y1] {
            assert!(tr.historical_params(now).polygon_limit <= POLYGON_AGG_LIMIT_CEILING);
        }
        assert!(polygon_historical_limit(TimeRange::D1) < 50_000);
    }
}
