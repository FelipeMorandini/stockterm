//! Precomputed watchlist table rows for draw paths (Issue #24 / §70 audit).

use crate::app::format::{format_signed_usd_delta, format_usd_price, symbol_kind_label};
use crate::app::ui::watchlist_quote_for_symbol;
use crate::app::App;
use crate::models::ticker::TickerResponse;
use ratatui::style::Color;
use ratatui::widgets::{Cell, Row};
use std::collections::HashMap;

/// Price direction for theme coloring at draw time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchlistPriceTrend {
    Up,
    Down,
    Flat,
}

/// One watchlist row precomputed off the 60fps render path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistDisplayRow {
    pub sym: String,
    pub kind: String,
    pub last_s: String,
    pub chg_s: String,
    pub pct_s: String,
    pub vol_s: String,
    pub trend: WatchlistPriceTrend,
}

/// Rebuild [`App::watchlist_display_rows_cache`] from filter indices and quote cache.
pub fn rebuild_watchlist_display_rows(app: &mut App) {
    app.watchlist_display_rows_cache = app
        .watchlist_filter_indices_cache
        .iter()
        .map(|&idx| {
            let sym = app.watchlist[idx].clone();
            let kind = symbol_kind_label(app.symbol_kind_for_display(&sym)).to_string();
            let (last_s, chg_s, pct_s, vol_s, trend) =
                quote_cells_for_symbol(&app.watchlist_quotes, &sym);
            WatchlistDisplayRow {
                sym,
                kind,
                last_s,
                chg_s,
                pct_s,
                vol_s,
                trend,
            }
        })
        .collect();
}

fn quote_cells_for_symbol(
    quotes: &HashMap<String, TickerResponse>,
    sym: &str,
) -> (String, String, String, String, WatchlistPriceTrend) {
    match watchlist_quote_for_symbol(quotes, sym).and_then(|r| r.latest_result()) {
        Some(bar) => {
            let ref_price = bar.change_reference();
            let price_change = bar.c - ref_price;
            let pct = if ref_price.abs() > f64::EPSILON {
                (price_change / ref_price) * 100.0
            } else {
                0.0
            };
            let trend = if price_change > 0.0 {
                WatchlistPriceTrend::Up
            } else if price_change < 0.0 {
                WatchlistPriceTrend::Down
            } else {
                WatchlistPriceTrend::Flat
            };
            (
                format_usd_price(bar.c),
                format_signed_usd_delta(price_change),
                format!("{}{:.2}%", if price_change >= 0.0 { "+" } else { "" }, pct),
                format!("{:.0}", bar.v),
                trend,
            )
        }
        _ => (
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            WatchlistPriceTrend::Flat,
        ),
    }
}

fn trend_color(
    trend: WatchlistPriceTrend,
    positive: Color,
    negative: Color,
    muted: Color,
) -> Color {
    match trend {
        WatchlistPriceTrend::Up => positive,
        WatchlistPriceTrend::Down => negative,
        WatchlistPriceTrend::Flat => muted,
    }
}

/// Build ratatui table rows from precomputed cache (no per-row `format!` on draw).
pub fn watchlist_display_rows_to_ratatui(
    rows: &[WatchlistDisplayRow],
    show_kind: bool,
    row_fg: Color,
    row_bg: Color,
    positive: Color,
    negative: Color,
    muted: Color,
) -> Vec<Row<'_>> {
    let row_style = ratatui::style::Style::default().fg(row_fg).bg(row_bg);
    rows.iter()
        .map(|r| {
            let chg_color = trend_color(r.trend, positive, negative, muted);
            let mut cells = vec![Cell::from(r.sym.as_str())];
            if show_kind {
                cells.push(Cell::from(r.kind.as_str()).style(if r.kind.is_empty() {
                    row_style
                } else {
                    row_style.fg(muted)
                }));
            }
            cells.extend([
                Cell::from(r.last_s.as_str()),
                Cell::from(r.chg_s.as_str()).style(row_style.fg(chg_color)),
                Cell::from(r.pct_s.as_str()).style(row_style.fg(chg_color)),
                Cell::from(r.vol_s.as_str()),
            ]);
            Row::new(cells).height(1).style(row_style)
        })
        .collect()
}
