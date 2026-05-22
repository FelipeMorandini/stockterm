//! Yahoo Finance `v7/finance/options` adapter (Issue #22 / SPEC §48.2).

use chrono::{TimeZone, Utc};
use serde::Deserialize;
use urlencoding::encode;

use crate::api::error::{ProviderError, ProviderResult};
use crate::api::yahoo::fetch_text_query1_or_query2_on_404;
use crate::api::symbol::resolve_provider_symbol;
use crate::config::MarketProviderKind;
use crate::models::options::{
    Expiration, OptionContract, OptionGreeks, OptionRight, OptionsChain, OptionsChainSlice,
};

const QUERY1: &str = "https://query1.finance.yahoo.com";
const QUERY2: &str = "https://query2.finance.yahoo.com";

/// Fetches and parses an options chain for `symbol` (optional `expiration_ts` unix seconds).
pub async fn yahoo_options_chain(
    symbol: &str,
    expiration_ts: Option<u64>,
) -> ProviderResult<OptionsChain> {
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
    let chain = parse_yahoo_options_response(&text, symbol, expiration_ts)?;
    if std::env::var("STOCKTERM_DEBUG_YAHOO_OPTIONS").as_deref() == Ok("1") {
        tracing::info!(
            target: "stockterm::yahoo_options",
            symbol = %chain.underlying,
            expirations = chain.expirations.len(),
            calls = chain.slice.calls.len(),
            puts = chain.slice.puts.len(),
            selected_ts = chain.selected_expiration_ts,
            "parsed options chain"
        );
    }
    Ok(chain)
}

/// Parses Yahoo `optionChain` JSON into [`OptionsChain`].
pub fn parse_yahoo_options_response(
    text: &str,
    requested_symbol: &str,
    requested_expiration_ts: Option<u64>,
) -> ProviderResult<OptionsChain> {
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

    let selected_ts = match requested_expiration_ts {
        Some(ts) => {
            if !expirations.iter().any(|e| e.ts == ts) {
                return Err(ProviderError::ApiMessage(format!(
                    "No options for expiration {ts}"
                )));
            }
            ts
        }
        None => result
            .options
            .first()
            .map(|o| o.expiration_date as u64)
            .or_else(|| expirations.first().map(|e| e.ts))
            .ok_or_else(|| ProviderError::ApiMessage("No options available".into()))?,
    };

    let block = result
        .options
        .iter()
        .find(|o| o.expiration_date as u64 == selected_ts)
        .or_else(|| result.options.first())
        .ok_or_else(|| ProviderError::ApiMessage("No options available".into()))?;

    let expiration = expirations
        .iter()
        .find(|e| e.ts == selected_ts)
        .cloned()
        .unwrap_or_else(|| expiration_from_ts(selected_ts));

    let underlying = result
        .underlying_symbol
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| requested_symbol.to_string());

    let mut calls: Vec<OptionContract> = block
        .calls
        .iter()
        .map(|leg| map_leg(leg, OptionRight::Call, selected_ts))
        .collect();
    let mut puts: Vec<OptionContract> = block
        .puts
        .iter()
        .map(|leg| map_leg(leg, OptionRight::Put, selected_ts))
        .collect();
    sort_contracts(&mut calls);
    sort_contracts(&mut puts);

    if calls.is_empty() && puts.is_empty() {
        return Err(ProviderError::ApiMessage("No options available".into()));
    }

    Ok(OptionsChain {
        underlying,
        expirations,
        selected_expiration_ts: selected_ts,
        slice: OptionsChainSlice {
            underlying: requested_symbol.to_string(),
            expiration,
            calls,
            puts,
        },
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
