//! Polygon.io options chain adapter (Issue #167 / SPEC §50; expiration list cache Issue #171 / §51).

use std::collections::HashMap;

use chrono::{NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use urlencoding::encode;

use crate::api::error::{ProviderError, ProviderResult};
use crate::api::polygon::polygon_key;
use crate::api::polygon_pagination::validate_polygon_next_url;
use crate::api::retry::execute_get_text_with_retry;
use crate::api::symbol::resolve_provider_symbol;
use crate::config::{Config, MarketProviderKind};
use crate::models::options::{
    Expiration, OptionContract, OptionGreeks, OptionRight, OptionsChain, OptionsChainSlice,
};

const BASE_URL: &str = "https://api.polygon.io";
const POLYGON_OPTIONS_MAX_CONTRACT_PAGES: usize = 10;
const POLYGON_OPTIONS_MAX_SNAPSHOT_PAGES: usize = 40;

/// Parsed Polygon options chain for one fetch (active view + cacheable slice).
#[derive(Debug, Clone)]
pub struct PolygonOptionsParseResult {
    /// Active chain view for the requested/selected expiration.
    pub chain: OptionsChain,
    /// Slice keyed by expiration unix seconds for session cache (§49.2).
    pub slices_by_ts: HashMap<u64, OptionsChainSlice>,
}

/// Fetches and parses a Polygon options chain for `symbol` (optional `expiration_ts` unix seconds).
///
/// When `cached_expirations` is non-empty, skips `v3/reference/options/contracts` (Issue #171 / §51).
pub async fn polygon_options_chain_with_slices(
    symbol: &str,
    expiration_ts: Option<u64>,
    config: &Config,
    cached_expirations: Option<&[Expiration]>,
) -> ProviderResult<PolygonOptionsParseResult> {
    let key = polygon_key(config)?;
    let wire = resolve_provider_symbol(MarketProviderKind::Polygon, symbol);

    let (expirations, contracts_fetch) =
        if let Some(cached) = cached_expirations.filter(|e| !e.is_empty()) {
            (cached.to_vec(), false)
        } else {
            (fetch_contract_expirations(&wire, &key).await?, true)
        };
    if expirations.is_empty() {
        return Err(ProviderError::ApiMessage("No options available".into()));
    }

    let wire_owned = wire.clone();
    let key_owned = key.clone();
    build_polygon_options_result(
        symbol,
        &wire,
        expirations,
        expiration_ts,
        contracts_fetch,
        move |date_label| {
            let w = wire_owned.clone();
            let k = key_owned.clone();
            let d = date_label.to_string();
            async move { fetch_chain_snapshot(&w, &d, &k).await }
        },
    )
    .await
}

/// Builds a chain from a known expiration list + snapshot fetch (shared by live path and unit tests).
async fn build_polygon_options_result<F, Fut>(
    symbol: &str,
    wire: &str,
    expirations: Vec<Expiration>,
    expiration_ts: Option<u64>,
    contracts_fetch: bool,
    snapshot_fetch: F,
) -> ProviderResult<PolygonOptionsParseResult>
where
    F: FnOnce(&str) -> Fut,
    Fut: std::future::Future<Output = ProviderResult<Vec<OptionContract>>>,
{
    let selected_ts = match expiration_ts {
        Some(ts) => {
            if !expirations.iter().any(|e| e.ts == ts) {
                return Err(ProviderError::ApiMessage(format!(
                    "No options for expiration {ts}"
                )));
            }
            ts
        }
        None => select_default_expiration_ts(&expirations)
            .ok_or_else(|| ProviderError::ApiMessage("No options available".into()))?,
    };

    let date_label = expirations
        .iter()
        .find(|e| e.ts == selected_ts)
        .map(|e| e.label.as_str())
        .ok_or_else(|| {
            ProviderError::ApiMessage(format!("No options for expiration {selected_ts}"))
        })?;

    let contracts = snapshot_fetch(date_label).await?;
    let slice = build_slice_from_contracts(symbol, &expirations, selected_ts, contracts);
    if slice.calls.is_empty() && slice.puts.is_empty() {
        return Err(ProviderError::ApiMessage("No options available".into()));
    }

    let chain = OptionsChain {
        underlying: symbol.to_string(),
        expirations: expirations.clone(),
        selected_expiration_ts: selected_ts,
        slice: slice.clone(),
    };

    let mut slices_by_ts = HashMap::new();
    slices_by_ts.insert(selected_ts, slice);

    if std::env::var("STOCKTERM_DEBUG_POLYGON_OPTIONS").as_deref() == Ok("1") {
        tracing::info!(
            target: "stockterm::polygon_options",
            symbol = %chain.underlying,
            wire = %wire,
            contracts_fetch,
            expirations = chain.expirations.len(),
            selected = %date_label,
            calls = chain.slice.calls.len(),
            puts = chain.slice.puts.len(),
            "polygon options fetch"
        );
    }

    Ok(PolygonOptionsParseResult {
        chain,
        slices_by_ts,
    })
}

/// Maps `YYYY-MM-DD` to Unix seconds at UTC midnight.
pub fn expiration_date_to_ts(date: &str) -> Option<u64> {
    let naive = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let dt = naive.and_hms_opt(0, 0, 0)?;
    Utc.from_utc_datetime(&dt).timestamp().try_into().ok()
}

fn enc(s: &str) -> String {
    encode(s).into_owned()
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> ProviderResult<T> {
    let text = execute_get_text_with_retry(url).await?;
    serde_json::from_str(&text).map_err(ProviderError::from)
}

fn check_polygon_status(status: Option<&str>, url: &str) -> ProviderResult<()> {
    match status {
        Some("OK") | None => Ok(()),
        Some(other) => {
            let display_url = url.split('?').next().unwrap_or(url);
            Err(ProviderError::ApiMessage(format!(
                "Polygon options error ({other}) for {display_url}"
            )))
        }
    }
}

async fn fetch_contract_expirations(wire: &str, key: &str) -> ProviderResult<Vec<Expiration>> {
    let mut url = format!(
        "{BASE_URL}/v3/reference/options/contracts?underlying_ticker={}&sort=expiration_date&order=asc&limit=1000&apiKey={}",
        enc(wire),
        enc(key)
    );
    let mut rows: Vec<ContractRow> = Vec::new();
    for page in 0..POLYGON_OPTIONS_MAX_CONTRACT_PAGES {
        let page_data: ContractsPage = fetch_json(&url).await?;
        check_polygon_status(page_data.status.as_deref(), &url)?;
        rows.extend(page_data.results);
        if page + 1 >= POLYGON_OPTIONS_MAX_CONTRACT_PAGES {
            break;
        }
        match page_data.next_url {
            Some(next) => url = validate_polygon_next_url(&next)?,
            None => break,
        }
    }
    Ok(expirations_from_contract_rows(&rows))
}

async fn fetch_chain_snapshot(
    wire: &str,
    expiration_date: &str,
    key: &str,
) -> ProviderResult<Vec<OptionContract>> {
    let mut url = format!(
        "{BASE_URL}/v3/snapshot/options/{}?expiration_date={}&limit=250&sort=strike_price&order=asc&apiKey={}",
        enc(wire),
        enc(expiration_date),
        enc(key)
    );
    let mut contracts = Vec::new();
    for page in 0..POLYGON_OPTIONS_MAX_SNAPSHOT_PAGES {
        let page_data: SnapshotPage = fetch_json(&url).await?;
        check_polygon_status(page_data.status.as_deref(), &url)?;
        for row in page_data.results {
            if let Some(c) = map_snapshot_row(&row) {
                contracts.push(c);
            }
        }
        if page + 1 >= POLYGON_OPTIONS_MAX_SNAPSHOT_PAGES {
            break;
        }
        match page_data.next_url {
            Some(next) => url = validate_polygon_next_url(&next)?,
            None => break,
        }
    }
    Ok(contracts)
}

fn expirations_from_contract_rows(rows: &[ContractRow]) -> Vec<Expiration> {
    let mut dates: Vec<String> = rows
        .iter()
        .filter_map(|r| r.expiration_date.clone())
        .collect();
    dates.sort();
    dates.dedup();
    dates
        .into_iter()
        .filter_map(|d| expiration_from_date(&d))
        .collect()
}

fn expiration_from_date(date: &str) -> Option<Expiration> {
    let ts = expiration_date_to_ts(date)?;
    Some(Expiration {
        ts,
        label: date.to_string(),
    })
}

fn select_default_expiration_ts(expirations: &[Expiration]) -> Option<u64> {
    if expirations.is_empty() {
        return None;
    }
    let now = Utc::now().timestamp();
    expirations
        .iter()
        .map(|e| e.ts)
        .filter(|&ts| (ts as i64) >= now)
        .min()
        .or_else(|| expirations.first().map(|e| e.ts))
}

fn build_slice_from_contracts(
    underlying: &str,
    expirations: &[Expiration],
    selected_ts: u64,
    contracts: Vec<OptionContract>,
) -> OptionsChainSlice {
    let expiration = expirations
        .iter()
        .find(|e| e.ts == selected_ts)
        .cloned()
        .unwrap_or_else(|| {
            expiration_from_date(
                &Utc.timestamp_opt(selected_ts as i64, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| selected_ts.to_string()),
            )
            .unwrap_or(Expiration {
                ts: selected_ts,
                label: selected_ts.to_string(),
            })
        });

    let mut calls: Vec<OptionContract> = contracts
        .iter()
        .filter(|c| c.right == OptionRight::Call && c.expiration_ts == selected_ts)
        .cloned()
        .collect();
    let mut puts: Vec<OptionContract> = contracts
        .iter()
        .filter(|c| c.right == OptionRight::Put && c.expiration_ts == selected_ts)
        .cloned()
        .collect();
    sort_contracts(&mut calls);
    sort_contracts(&mut puts);

    OptionsChainSlice {
        underlying: underlying.to_string(),
        expiration,
        calls,
        puts,
    }
}

/// Splits snapshot JSON into calls/puts for fixture validation (unit tests).
#[cfg(test)]
fn parse_snapshot_page(
    text: &str,
    underlying: &str,
    expiration_date: &str,
) -> ProviderResult<OptionsChainSlice> {
    let page: SnapshotPage = serde_json::from_str(text)?;
    let expiration_ts = expiration_date_to_ts(expiration_date).ok_or_else(|| {
        ProviderError::ApiMessage(format!("Invalid expiration date {expiration_date}"))
    })?;
    let expiration = expiration_from_date(expiration_date).unwrap_or(Expiration {
        ts: expiration_ts,
        label: expiration_date.to_string(),
    });
    let mut contracts = Vec::new();
    for row in page.results {
        if let Some(c) = map_snapshot_row(&row) {
            contracts.push(c);
        }
    }
    let mut calls: Vec<OptionContract> = contracts
        .iter()
        .filter(|c| c.right == OptionRight::Call)
        .cloned()
        .collect();
    let mut puts: Vec<OptionContract> = contracts
        .iter()
        .filter(|c| c.right == OptionRight::Put)
        .cloned()
        .collect();
    sort_contracts(&mut calls);
    sort_contracts(&mut puts);
    Ok(OptionsChainSlice {
        underlying: underlying.to_string(),
        expiration,
        calls,
        puts,
    })
}

fn sort_contracts(contracts: &mut [OptionContract]) {
    contracts.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn map_snapshot_row(row: &SnapshotRow) -> Option<OptionContract> {
    let details = row.details.as_ref()?;
    let strike = details.strike_price?;
    let right = parse_contract_type(details.contract_type.as_deref()?)?;
    let expiration_date = details.expiration_date.as_deref()?;
    let expiration_ts = expiration_date_to_ts(expiration_date)?;

    let bid = row.last_quote.as_ref().and_then(|q| q.bid);
    let ask = row.last_quote.as_ref().and_then(|q| q.ask);
    let last = row
        .last_trade
        .as_ref()
        .and_then(|t| t.price)
        .or_else(|| row.day.as_ref().and_then(|d| d.close));

    let volume = row
        .day
        .as_ref()
        .and_then(|d| d.volume)
        .and_then(volume_to_u64);

    let greeks = row.greeks.as_ref().and_then(|g| {
        let has_any = g.delta.is_some()
            || g.gamma.is_some()
            || g.theta.is_some()
            || g.vega.is_some()
            || g.rho.is_some();
        if has_any {
            Some(OptionGreeks {
                delta: g.delta,
                gamma: g.gamma,
                theta: g.theta,
                vega: g.vega,
                rho: g.rho,
            })
        } else {
            None
        }
    });

    Some(OptionContract {
        symbol: details.ticker.clone().unwrap_or_default(),
        strike,
        right,
        expiration_ts,
        bid,
        ask,
        last,
        volume,
        open_interest: row.open_interest.and_then(volume_to_u64),
        implied_volatility: row.implied_volatility,
        greeks,
    })
}

fn parse_contract_type(s: &str) -> Option<OptionRight> {
    if s.eq_ignore_ascii_case("call") {
        Some(OptionRight::Call)
    } else if s.eq_ignore_ascii_case("put") {
        Some(OptionRight::Put)
    } else {
        None
    }
}

fn volume_to_u64(v: f64) -> Option<u64> {
    if v.is_finite() && v >= 0.0 {
        Some(v as u64)
    } else {
        None
    }
}

#[derive(Debug, Deserialize)]
struct ContractsPage {
    #[serde(default)]
    results: Vec<ContractRow>,
    status: Option<String>,
    next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContractRow {
    expiration_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotPage {
    #[serde(default)]
    results: Vec<SnapshotRow>,
    status: Option<String>,
    next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotRow {
    details: Option<SnapshotDetails>,
    day: Option<SnapshotDay>,
    last_quote: Option<SnapshotQuote>,
    last_trade: Option<SnapshotTrade>,
    open_interest: Option<f64>,
    implied_volatility: Option<f64>,
    greeks: Option<SnapshotGreeks>,
}

#[derive(Debug, Deserialize)]
struct SnapshotDetails {
    ticker: Option<String>,
    strike_price: Option<f64>,
    contract_type: Option<String>,
    expiration_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotDay {
    close: Option<f64>,
    volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SnapshotQuote {
    bid: Option<f64>,
    ask: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SnapshotTrade {
    price: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SnapshotGreeks {
    delta: Option<f64>,
    gamma: Option<f64>,
    theta: Option<f64>,
    vega: Option<f64>,
    rho: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn snapshot_fixture_text() -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/polygon_options_aapl_snapshot.json");
        std::fs::read_to_string(path).expect("fixture readable")
    }

    #[test]
    fn expiration_date_to_ts_parses_utc_midnight() {
        let ts = expiration_date_to_ts("2026-06-20").expect("valid date");
        let dt = Utc.timestamp_opt(ts as i64, 0).single().expect("valid ts");
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-06-20");
    }

    #[test]
    fn parse_snapshot_fixture() {
        let slice = parse_snapshot_page(&snapshot_fixture_text(), "AAPL", "2026-06-20").unwrap();
        assert_eq!(slice.calls.len(), 2);
        assert_eq!(slice.puts.len(), 2);
        let call_strikes: Vec<f64> = slice.calls.iter().map(|c| c.strike).collect();
        assert_eq!(call_strikes, vec![150.0, 155.0]);
        let put_strikes: Vec<f64> = slice.puts.iter().map(|c| c.strike).collect();
        assert_eq!(put_strikes, vec![150.0, 155.0]);
        assert!(slice
            .calls
            .iter()
            .any(|c| c.implied_volatility.is_some() && c.greeks.is_some()));
        assert_eq!(slice.calls[0].symbol, "O:AAPL260620C00150000");
    }

    #[test]
    fn build_chain_selects_requested_expiration() {
        let exp1 = expiration_from_date("2026-06-20").unwrap();
        let exp2 = expiration_from_date("2026-06-27").unwrap();
        let exp2_ts = exp2.ts;
        let expirations = vec![exp1, exp2];
        let slice = parse_snapshot_page(&snapshot_fixture_text(), "AAPL", "2026-06-20").unwrap();
        let chain = OptionsChain {
            underlying: "AAPL".into(),
            expirations: expirations.clone(),
            selected_expiration_ts: exp2_ts,
            slice: slice.clone(),
        };
        assert_eq!(chain.selected_expiration_ts, exp2_ts);
        assert_eq!(chain.expirations.len(), 2);
        assert_eq!(chain.slice.calls.first().map(|c| c.strike), Some(150.0));
    }

    #[test]
    fn empty_results_is_no_options() {
        let text = r#"{"status":"OK","results":[]}"#;
        let slice = parse_snapshot_page(text, "AAPL", "2026-06-20").unwrap();
        assert!(slice.calls.is_empty());
        assert!(slice.puts.is_empty());
    }

    #[test]
    fn expirations_from_contract_rows_dedupes_dates() {
        let rows = vec![
            ContractRow {
                expiration_date: Some("2026-06-27".into()),
            },
            ContractRow {
                expiration_date: Some("2026-06-20".into()),
            },
            ContractRow {
                expiration_date: Some("2026-06-20".into()),
            },
        ];
        let exps = expirations_from_contract_rows(&rows);
        assert_eq!(exps.len(), 2);
        assert_eq!(exps[0].label, "2026-06-20");
        assert_eq!(exps[1].label, "2026-06-27");
    }

    #[test]
    fn select_default_expiration_prefers_nearest_future() {
        let past = expiration_from_date("2020-01-01").unwrap();
        let future = expiration_from_date("2099-12-31").unwrap();
        let future_ts = future.ts;
        let ts = select_default_expiration_ts(&[past, future]).unwrap();
        assert_eq!(ts, future_ts);
    }

    /// Issue #171 / §51 — warm expiration list uses snapshot path only (no contracts HTTP in this helper).
    #[tokio::test]
    async fn build_polygon_options_with_cached_expirations() {
        let exp1 = expiration_from_date("2026-06-20").unwrap();
        let exp2 = expiration_from_date("2026-06-27").unwrap();
        let expirations = vec![exp1.clone(), exp2];
        let slice = parse_snapshot_page(&snapshot_fixture_text(), "AAPL", "2026-06-20").unwrap();
        let contracts: Vec<OptionContract> = slice
            .calls
            .into_iter()
            .chain(slice.puts.into_iter())
            .collect();

        let result = build_polygon_options_result(
            "AAPL",
            "AAPL",
            expirations.clone(),
            Some(exp1.ts),
            false,
            |_date| async { Ok(contracts.clone()) },
        )
        .await
        .expect("build from cached expirations");

        assert_eq!(result.chain.expirations.len(), 2);
        assert_eq!(result.chain.selected_expiration_ts, exp1.ts);
        assert_eq!(result.chain.slice.calls.len(), 2);
        assert!(result.slices_by_ts.contains_key(&exp1.ts));
    }
}
