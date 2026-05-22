//! Yahoo Finance `v7/finance/options` adapter (Issue #22 / SPEC §48.2, #168 / §49.2).

use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde::Deserialize;
use urlencoding::encode;

use crate::api::error::{ProviderError, ProviderResult};
use crate::api::symbol::resolve_provider_symbol;
use crate::api::yahoo::fetch_text_query1_or_query2_on_404;
use crate::config::MarketProviderKind;
use crate::models::options::{
    Expiration, OptionContract, OptionGreeks, OptionRight, OptionsChain, OptionsChainSlice,
};

const QUERY1: &str = "https://query1.finance.yahoo.com";
const QUERY2: &str = "https://query2.finance.yahoo.com";

/// Parsed options chain plus all inline expiration slices from one HTTP response (Issue #168).
#[derive(Debug, Clone)]
pub struct YahooOptionsParseResult {
    /// Active chain view for the requested/selected expiration.
    pub chain: OptionsChain,
    /// Every expiration block present in this response, keyed by unix seconds.
    pub slices_by_ts: HashMap<u64, OptionsChainSlice>,
}

/// Fetches and parses an options chain for `symbol` (optional `expiration_ts` unix seconds).
pub async fn yahoo_options_chain(
    symbol: &str,
    expiration_ts: Option<u64>,
) -> ProviderResult<OptionsChain> {
    Ok(yahoo_options_chain_with_slices(symbol, expiration_ts)
        .await?
        .chain)
}

/// Fetches options and returns all inline expiration slices from the payload (Issue #168).
pub async fn yahoo_options_chain_with_slices(
    symbol: &str,
    expiration_ts: Option<u64>,
) -> ProviderResult<YahooOptionsParseResult> {
    let wire = resolve_provider_symbol(MarketProviderKind::Yahoo, symbol);
    let enc = encode(wire.as_str());
    let mut url = format!("{QUERY1}/v7/finance/options/{enc}");
    if let Some(ts) = expiration_ts {
        url.push_str(&format!("?date={ts}"));
    }
    let q2 = match expiration_ts {
        Some(ts) => format!("{QUERY2}/v7/finance/options/{enc}?date={ts}"),
        None => format!("{QUERY2}/v7/finance/options/{enc}"),
    };
    let text = fetch_text_query1_or_query2_on_404(&url, &q2).await?;
    let parsed = parse_yahoo_options_with_slices(&text, symbol, expiration_ts)?;
    if std::env::var("STOCKTERM_DEBUG_YAHOO_OPTIONS").as_deref() == Ok("1") {
        tracing::info!(
            target: "stockterm::yahoo_options",
            symbol = %parsed.chain.underlying,
            expirations = parsed.chain.expirations.len(),
            cached_slices = parsed.slices_by_ts.len(),
            calls = parsed.chain.slice.calls.len(),
            puts = parsed.chain.slice.puts.len(),
            selected_ts = parsed.chain.selected_expiration_ts,
            "parsed options chain"
        );
    }
    Ok(parsed)
}

/// Parses Yahoo `optionChain` JSON into [`OptionsChain`] (primary slice only).
pub fn parse_yahoo_options_response(
    text: &str,
    requested_symbol: &str,
    requested_expiration_ts: Option<u64>,
) -> ProviderResult<OptionsChain> {
    Ok(parse_yahoo_options_with_slices(text, requested_symbol, requested_expiration_ts)?.chain)
}

/// Parses Yahoo `optionChain` JSON and extracts every inline expiration block (Issue #168).
pub fn parse_yahoo_options_with_slices(
    text: &str,
    requested_symbol: &str,
    requested_expiration_ts: Option<u64>,
) -> ProviderResult<YahooOptionsParseResult> {
    let envelope: OptionsEnvelope = serde_json::from_str(text)?;
    let result = envelope
        .option_chain
        .result
        .into_iter()
        .next()
        .ok_or_else(|| ProviderError::ApiMessage("No options available".into()))?;

    let expirations = build_expirations(&result.expiration_dates);
    if expirations.is_empty() {
        return Err(ProviderError::ApiMessage("No options available".into()));
    }

    let underlying = result
        .underlying_symbol
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| requested_symbol.to_string());

    let mut slices_by_ts: HashMap<u64, OptionsChainSlice> = HashMap::new();
    for block in &result.options {
        let ts = block.expiration_date as u64;
        if let Some(slice) =
            block_to_slice(block, requested_symbol, &expirations, ts)
        {
            slices_by_ts.insert(ts, slice);
        }
    }

    if slices_by_ts.is_empty() {
        return Err(ProviderError::ApiMessage("No options available".into()));
    }

    let default_ts = result
        .options
        .first()
        .map(|o| o.expiration_date as u64)
        .or_else(|| expirations.first().map(|e| e.ts))
        .expect("non-empty slices");

    let selected_ts = match requested_expiration_ts {
        Some(ts) => {
            if !expirations.iter().any(|e| e.ts == ts) {
                return Err(ProviderError::ApiMessage(format!(
                    "No options for expiration {ts}"
                )));
            }
            ts
        }
        None => default_ts,
    };

    let slice = slices_by_ts.get(&selected_ts).cloned().ok_or_else(|| {
        ProviderError::ApiMessage(format!("No options for expiration {selected_ts}"))
    })?;

    let chain = OptionsChain {
        underlying,
        expirations,
        selected_expiration_ts: selected_ts,
        slice,
    };

    Ok(YahooOptionsParseResult {
        chain,
        slices_by_ts,
    })
}

fn block_to_slice(
    block: &OptionsBlock,
    requested_symbol: &str,
    expirations: &[Expiration],
    ts: u64,
) -> Option<OptionsChainSlice> {
    let expiration = expirations
        .iter()
        .find(|e| e.ts == ts)
        .cloned()
        .unwrap_or_else(|| expiration_from_ts(ts));

    let mut calls: Vec<OptionContract> = block
        .calls
        .iter()
        .map(|leg| map_leg(leg, OptionRight::Call, ts))
        .collect();
    let mut puts: Vec<OptionContract> = block
        .puts
        .iter()
        .map(|leg| map_leg(leg, OptionRight::Put, ts))
        .collect();
    sort_contracts(&mut calls);
    sort_contracts(&mut puts);

    if calls.is_empty() && puts.is_empty() {
        return None;
    }

    Some(OptionsChainSlice {
        underlying: requested_symbol.to_string(),
        expiration,
        calls,
        puts,
    })
}

fn build_expirations(dates: &[i64]) -> Vec<Expiration> {
    let mut out: Vec<Expiration> = dates
        .iter()
        .map(|&ts| expiration_from_ts(ts as u64))
        .collect();
    out.sort_by_key(|e| e.ts);
    out.dedup_by_key(|e| e.ts);
    out
}

fn expiration_from_ts(ts: u64) -> Expiration {
    let label = Utc
        .timestamp_opt(ts as i64, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| ts.to_string());
    Expiration { ts, label }
}

fn sort_contracts(contracts: &mut [OptionContract]) {
    contracts.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn non_negative_u64(v: i64) -> Option<u64> {
    u64::try_from(v).ok()
}

fn map_leg(leg: &YahooOptionLeg, right: OptionRight, expiration_ts: u64) -> OptionContract {
    OptionContract {
        symbol: leg.contract_symbol.clone().unwrap_or_default(),
        strike: leg.strike.unwrap_or(0.0),
        right,
        expiration_ts,
        bid: leg.bid,
        ask: leg.ask,
        last: leg.last_price,
        volume: leg.volume.and_then(non_negative_u64),
        open_interest: leg.open_interest.and_then(non_negative_u64),
        implied_volatility: leg.implied_volatility,
        greeks: leg.greeks.as_ref().map(|g| OptionGreeks {
            delta: g.delta,
            gamma: g.gamma,
            theta: g.theta,
            vega: g.vega,
            rho: g.rho,
        }),
    }
}

#[derive(Debug, Deserialize)]
struct OptionsEnvelope {
    #[serde(rename = "optionChain")]
    option_chain: OptionChainBody,
}

#[derive(Debug, Deserialize)]
struct OptionChainBody {
    #[serde(default)]
    result: Vec<OptionChainResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionChainResult {
    #[serde(default)]
    expiration_dates: Vec<i64>,
    #[serde(default)]
    options: Vec<OptionsBlock>,
    underlying_symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionsBlock {
    expiration_date: i64,
    #[serde(default)]
    calls: Vec<YahooOptionLeg>,
    #[serde(default)]
    puts: Vec<YahooOptionLeg>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YahooOptionLeg {
    contract_symbol: Option<String>,
    strike: Option<f64>,
    bid: Option<f64>,
    ask: Option<f64>,
    last_price: Option<f64>,
    volume: Option<i64>,
    open_interest: Option<i64>,
    implied_volatility: Option<f64>,
    greeks: Option<YahooGreeks>,
}

#[derive(Debug, Deserialize)]
struct YahooGreeks {
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

    fn fixture_text() -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/yahoo_options_aapl.json");
        std::fs::read_to_string(path).expect("fixture readable")
    }

    #[test]
    fn parse_yahoo_options_aapl_fixture() {
        let chain = parse_yahoo_options_response(&fixture_text(), "AAPL", None).unwrap();
        assert!(!chain.expirations.is_empty());
        assert_eq!(chain.expirations.len(), 2);
        assert!(!chain.slice.calls.is_empty());
        assert!(!chain.slice.puts.is_empty());
        let strikes: Vec<f64> = chain.slice.calls.iter().map(|c| c.strike).collect();
        assert_eq!(strikes, vec![220.0, 225.0]);
        assert!(chain
            .slice
            .calls
            .iter()
            .any(|c| c.implied_volatility.is_some()));
    }

    #[test]
    fn parse_multi_expiration_inline_blocks() {
        let parsed = parse_yahoo_options_with_slices(&fixture_text(), "AAPL", None).unwrap();
        assert_eq!(parsed.slices_by_ts.len(), 2);
        assert!(parsed.slices_by_ts.contains_key(&1732147200));
        assert!(parsed.slices_by_ts.contains_key(&1732752000));
        let later = parsed.slices_by_ts.get(&1732752000).unwrap();
        let strikes: Vec<f64> = later.calls.iter().map(|c| c.strike).collect();
        assert_eq!(strikes, vec![230.0, 235.0]);
    }

    #[test]
    fn parse_selects_requested_expiration_slice() {
        let parsed =
            parse_yahoo_options_with_slices(&fixture_text(), "AAPL", Some(1732752000)).unwrap();
        assert_eq!(parsed.chain.selected_expiration_ts, 1732752000);
        assert_eq!(
            parsed.chain.slice.calls.first().map(|c| c.strike),
            Some(230.0)
        );
    }

    #[test]
    fn parse_empty_result_is_not_found() {
        let text = r#"{"optionChain":{"result":[]}}"#;
        let err = parse_yahoo_options_response(text, "AAPL", None).unwrap_err();
        assert!(matches!(err, ProviderError::ApiMessage(_)));
    }

    #[test]
    fn calls_sorted_by_strike() {
        let chain = parse_yahoo_options_response(&fixture_text(), "AAPL", None).unwrap();
        let mut prev = f64::NEG_INFINITY;
        for c in &chain.slice.calls {
            assert!(c.strike >= prev);
            prev = c.strike;
        }
    }
}
