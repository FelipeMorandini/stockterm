//! Options tab: chain display cache and draw (Issue #22 / SPEC §48.4).

use crate::app::format::format_usd_price;
use crate::app::layout::centered_rect;
use crate::app::styles::ResolvedTheme;
use crate::app::App;
use crate::config::MarketProviderKind;
use crate::models::options::{OptionContract, OptionsChain, OptionsChainSlice};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use std::cmp::Ordering;

const EM_DASH: &str = "—";
const STRIKE_MATCH_EPS: f64 = 1e-6;

/// Preformatted row for one calls/puts table line (built in Update, not draw).
#[derive(Debug, Clone)]
pub struct OptionsRowDisplay {
    pub strike: f64,
    pub strike_label: String,
    pub bid_label: String,
    pub ask_label: String,
    pub last_label: String,
    pub iv_label: String,
    pub greek_labels: Option<[String; 5]>,
}

/// Cached strings and table widgets for the Options tab (built in Update — draw is style-only).
#[derive(Debug, Clone, Default)]
pub struct OptionsDisplayCache {
    pub pane_title: String,
    pub empty_hint: String,
    pub header_primary: String,
    pub header_muted: String,
    pub calls: Vec<OptionsRowDisplay>,
    pub puts: Vec<OptionsRowDisplay>,
    pub call_table_rows: Vec<Row<'static>>,
    pub put_table_rows: Vec<Row<'static>>,
    pub table_header: Row<'static>,
    pub table_col_widths: Vec<Constraint>,
}

/// Merges inline expiration blocks from one HTTP response into the session cache (Issue #168).
pub fn merge_options_inline_slices(
    cache: &mut std::collections::HashMap<u64, crate::models::options::OptionsChainSlice>,
    chain: &OptionsChain,
    extra_slices: &std::collections::HashMap<u64, OptionsChainSlice>,
) {
    for (ts, slice) in extra_slices {
        cache.insert(*ts, slice.clone());
    }
    cache.insert(chain.selected_expiration_ts, chain.slice.clone());
}

/// Clears per-expiration slice cache and Polygon expiration list before **`r`** refresh (Issue #171 / §51).
pub fn invalidate_options_refresh_caches(app: &mut App) {
    app.options_slices_by_ts.clear();
    if app.config.provider == MarketProviderKind::Polygon {
        app.options_polygon_expirations_cache = None;
    }
}

/// Clears session options data (symbol change, provider switch).
pub fn clear_options_session(app: &mut App) {
    app.options_chain = None;
    app.options_slices_by_ts.clear();
    app.options_polygon_expirations_cache = None;
    app.options_no_listed = false;
    app.options_selected_strike = None;
    app.options_display.calls.clear();
    app.options_display.puts.clear();
    app.options_display.call_table_rows.clear();
    app.options_display.put_table_rows.clear();
    app.options_display.table_col_widths.clear();
    app.options_display.header_primary.clear();
    app.options_display.header_muted.clear();
    sync_options_chrome(app);
}

/// Updates title / empty-hint strings when symbol changes (no chain required).
pub fn sync_options_chrome(app: &mut App) {
    app.options_display.pane_title = format!(" Options — {} ", app.symbol);
    app.options_display.empty_hint = if app.symbol.is_empty() {
        "Set a symbol on Stock View first".to_string()
    } else {
        "Press r to load options chain".to_string()
    };
}

fn format_opt_price(v: Option<f64>) -> String {
    match v {
        Some(p) if p.is_finite() => format_usd_price(p),
        _ => EM_DASH.to_string(),
    }
}

fn format_iv(v: Option<f64>) -> String {
    match v {
        Some(iv) if iv.is_finite() => format!("{:.1}%", iv * 100.0),
        _ => EM_DASH.to_string(),
    }
}

fn format_greek(v: Option<f64>) -> String {
    match v {
        Some(g) if g.is_finite() => format!("{:.3}", g),
        _ => EM_DASH.to_string(),
    }
}

fn row_from_contract(c: &OptionContract, show_greeks: bool) -> OptionsRowDisplay {
    let greek_labels = if show_greeks {
        c.greeks.as_ref().map(|g| {
            [
                format_greek(g.delta),
                format_greek(g.gamma),
                format_greek(g.theta),
                format_greek(g.vega),
                format_greek(g.rho),
            ]
        })
    } else {
        None
    };
    OptionsRowDisplay {
        strike: c.strike,
        strike_label: format_usd_price(c.strike),
        bid_label: format_opt_price(c.bid),
        ask_label: format_opt_price(c.ask),
        last_label: format_opt_price(c.last),
        iv_label: format_iv(c.implied_volatility),
        greek_labels,
    }
}

fn strikes_match(a: f64, b: f64) -> bool {
    (a - b).abs() < STRIKE_MATCH_EPS
}

fn row_style(highlighted: bool, rt: &ResolvedTheme) -> Style {
    if highlighted {
        Style::default()
            .fg(rt.foreground)
            .bg(rt.selection)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(rt.foreground).bg(rt.background)
    }
}

fn table_row_from_display(
    r: &OptionsRowDisplay,
    show_greeks: bool,
    selected_strike: Option<f64>,
    rt: &ResolvedTheme,
) -> Row<'static> {
    let mut cells = vec![
        Cell::from(r.strike_label.clone()),
        Cell::from(r.bid_label.clone()),
        Cell::from(r.ask_label.clone()),
        Cell::from(r.last_label.clone()),
        Cell::from(r.iv_label.clone()),
    ];
    if show_greeks {
        if let Some(g) = &r.greek_labels {
            for label in g {
                cells.push(Cell::from(label.clone()));
            }
        } else {
            for _ in 0..5 {
                cells.push(Cell::from(EM_DASH));
            }
        }
    }
    let highlighted = selected_strike.is_some_and(|s| strikes_match(r.strike, s));
    Row::new(cells).style(row_style(highlighted, rt))
}

fn header_row(show_greeks: bool) -> Row<'static> {
    let mut headers = vec!["Strike", "Bid", "Ask", "Last", "IV"];
    if show_greeks {
        headers.extend(["Δ", "Γ", "Θ", "ν", "ρ"]);
    }
    Row::new(headers.into_iter().map(Cell::from))
}

fn col_widths(show_greeks: bool) -> Vec<Constraint> {
    if show_greeks {
        vec![
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ]
    } else {
        vec![
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(4),
        ]
    }
}

fn rebuild_options_table_rows(app: &mut App, rt: &ResolvedTheme) {
    let show_greeks = app.options_show_greeks;
    let selected = app.options_selected_strike;
    let cache = &mut app.options_display;
    cache.call_table_rows = cache
        .calls
        .iter()
        .map(|d| table_row_from_display(d, show_greeks, selected, rt))
        .collect();
    cache.put_table_rows = cache
        .puts
        .iter()
        .map(|d| table_row_from_display(d, show_greeks, selected, rt))
        .collect();
    cache.table_header = header_row(show_greeks);
    cache.table_col_widths = col_widths(show_greeks);
}

/// Rebuilds cached table rows after strike highlight changes (j/k only).
pub fn refresh_options_row_highlights(app: &mut App) {
    if app.options_display.calls.is_empty() && app.options_display.puts.is_empty() {
        return;
    }
    let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
    rebuild_options_table_rows(app, &rt);
}

/// Rebuilds [`App::options_display`] from [`App::options_chain`].
pub fn rebuild_options_display_cache(app: &mut App) {
    sync_options_chrome(app);
    let Some(chain) = &app.options_chain else {
        app.options_display.header_primary.clear();
        app.options_display.header_muted.clear();
        app.options_display.calls.clear();
        app.options_display.puts.clear();
        app.options_display.call_table_rows.clear();
        app.options_display.put_table_rows.clear();
        app.options_display.table_col_widths.clear();
        return;
    };
    let show_greeks = app.options_show_greeks;
    let calls: Vec<OptionsRowDisplay> = chain
        .slice
        .calls
        .iter()
        .map(|c| row_from_contract(c, show_greeks))
        .collect();
    let puts: Vec<OptionsRowDisplay> = chain
        .slice
        .puts
        .iter()
        .map(|c| row_from_contract(c, show_greeks))
        .collect();

    let exp_idx = chain
        .expirations
        .iter()
        .position(|e| e.ts == chain.selected_expiration_ts)
        .map(|i| i + 1)
        .unwrap_or(1);
    let exp_total = chain.expirations.len().max(1);
    let spot = app
        .get_current_price(&app.symbol)
        .filter(|p| p.is_finite())
        .map(format_usd_price)
        .unwrap_or_else(|| EM_DASH.to_string());

    let expiration_banner = format!(
        "Exp {} / {} — {}",
        exp_idx, exp_total, chain.slice.expiration.label
    );
    let spot_label = format!("Spot: {spot}");
    let greeks_hint = if show_greeks {
        "Greeks: on (g)"
    } else {
        "Greeks: off (g)"
    };

    app.options_display.header_primary =
        format!("{expiration_banner} │ {spot_label} │ ");
    app.options_display.header_muted = greeks_hint.to_string();
    app.options_display.calls = calls;
    app.options_display.puts = puts;

    let rt = ResolvedTheme::from_palette(app.theme_palette_for_render());
    rebuild_options_table_rows(app, &rt);
}

/// Default strike nearest to spot (union of calls/puts strikes).
pub fn default_selected_strike(chain: &OptionsChain, spot: Option<f64>) -> f64 {
    let strikes = canonical_strikes(chain);
    if strikes.is_empty() {
        return 0.0;
    }
    let spot = spot.unwrap_or(f64::NAN);
    if !spot.is_finite() {
        return strikes[0];
    }
    strikes
        .iter()
        .copied()
        .min_by(|a, b| {
            (a - spot)
                .abs()
                .partial_cmp(&(b - spot).abs())
                .unwrap_or(Ordering::Equal)
        })
        .unwrap_or(strikes[0])
}

/// Sorted unique strikes from calls and puts (canonical for scroll).
pub fn canonical_strikes(chain: &OptionsChain) -> Vec<f64> {
    let mut strikes: Vec<f64> = chain
        .slice
        .calls
        .iter()
        .map(|c| c.strike)
        .chain(chain.slice.puts.iter().map(|p| p.strike))
        .collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    strikes.dedup_by(|a, b| strikes_match(*a, *b));
    strikes
}

/// Draws the Options tab (pure render; table rows from [`OptionsDisplayCache`]).
pub fn draw_options(f: &mut Frame, app: &App, area: Rect, rt: &ResolvedTheme) {
    let cache = &app.options_display;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(cache.pane_title.as_str())
        .style(rt.canvas())
        .border_style(Style::default().fg(rt.border).bg(rt.background));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.options_inflight {
        let msg = Paragraph::new("Loading options…")
            .style(Style::default().fg(rt.muted));
        f.render_widget(msg, inner);
        return;
    }

    if app.options_no_listed {
        let popup = centered_rect(inner, 60, 30);
        let msg = Paragraph::new("No options available")
            .style(Style::default().fg(rt.foreground));
        f.render_widget(msg, popup);
        return;
    }

    if app.options_chain.is_none() {
        let popup = centered_rect(inner, 70, 30);
        let msg = Paragraph::new(cache.empty_hint.as_str())
            .style(Style::default().fg(rt.muted));
        f.render_widget(msg, popup);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let header_line = Line::from(vec![
        Span::styled(&cache.header_primary, Style::default().fg(rt.foreground)),
        Span::styled(&cache.header_muted, Style::default().fg(rt.muted)),
        Span::raw(" │ [ ] h/l exp · j/k strike · r refresh"),
    ]);
    f.render_widget(Paragraph::new(header_line), chunks[0]);

    let body = chunks[1];
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);

    let calls_block = Block::default()
        .borders(Borders::ALL)
        .title(" CALLS ")
        .style(rt.canvas());
    let puts_block = Block::default()
        .borders(Borders::ALL)
        .title(" PUTS ")
        .style(rt.canvas());

    let calls_table = Table::new(cache.call_table_rows.clone(), cache.table_col_widths.clone())
        .header(cache.table_header.clone())
        .block(calls_block);
    let puts_table = Table::new(cache.put_table_rows.clone(), cache.table_col_widths.clone())
        .header(cache.table_header.clone())
        .block(puts_block);

    f.render_widget(calls_table, cols[0]);
    f.render_widget(puts_table, cols[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::options::{
        Expiration, OptionContract, OptionRight, OptionsChain, OptionsChainSlice,
    };
    use std::collections::HashMap;

    fn sample_chain() -> OptionsChain {
        OptionsChain {
            underlying: "AAPL".into(),
            expirations: vec![Expiration {
                ts: 1,
                label: "2026-06-20".into(),
            }],
            selected_expiration_ts: 1,
            slice: OptionsChainSlice {
                underlying: "AAPL".into(),
                expiration: Expiration {
                    ts: 1,
                    label: "2026-06-20".into(),
                },
                calls: vec![
                    OptionContract {
                        symbol: "C220".into(),
                        strike: 220.0,
                        right: OptionRight::Call,
                        expiration_ts: 1,
                        bid: None,
                        ask: None,
                        last: None,
                        volume: None,
                        open_interest: None,
                        implied_volatility: None,
                        greeks: None,
                    },
                    OptionContract {
                        symbol: "C225".into(),
                        strike: 225.0,
                        right: OptionRight::Call,
                        expiration_ts: 1,
                        bid: None,
                        ask: None,
                        last: None,
                        volume: None,
                        open_interest: None,
                        implied_volatility: None,
                        greeks: None,
                    },
                ],
                puts: vec![OptionContract {
                    symbol: "P220".into(),
                    strike: 220.0,
                    right: OptionRight::Put,
                    expiration_ts: 1,
                    bid: None,
                    ask: None,
                    last: None,
                    volume: None,
                    open_interest: None,
                    implied_volatility: None,
                    greeks: None,
                }],
            },
        }
    }

    #[test]
    fn default_selected_strike_picks_nearest() {
        let chain = sample_chain();
        assert_eq!(default_selected_strike(&chain, Some(223.0)), 225.0);
    }

    #[test]
    fn canonical_strikes_merges_calls_and_puts() {
        let chain = sample_chain();
        assert_eq!(canonical_strikes(&chain), vec![220.0, 225.0]);
    }

    #[test]
    fn merge_options_inline_slices_inserts_all_keys() {
        let chain = sample_chain();
        let mut extra = HashMap::new();
        extra.insert(
            2,
            OptionsChainSlice {
                underlying: "AAPL".into(),
                expiration: Expiration {
                    ts: 2,
                    label: "2026-06-27".into(),
                },
                calls: vec![],
                puts: vec![],
            },
        );
        let mut cache = HashMap::new();
        merge_options_inline_slices(&mut cache, &chain, &extra);
        assert!(cache.contains_key(&1));
        assert!(cache.contains_key(&2));
    }

    #[test]
    fn clear_options_session_clears_polygon_expiration_cache() {
        use crate::models::options::Expiration;

        let mut app = App::new();
        app.options_polygon_expirations_cache = Some((
            "AAPL".into(),
            vec![Expiration {
                ts: 1,
                label: "2026-06-20".into(),
            }],
        ));
        clear_options_session(&mut app);
        assert!(app.options_polygon_expirations_cache.is_none());
    }

    #[test]
    fn invalidate_options_refresh_caches_clears_slices_and_polygon_list() {
        use crate::config::MarketProviderKind;
        use crate::models::options::{Expiration, OptionsChainSlice};

        let mut app = App::new();
        app.config.provider = MarketProviderKind::Polygon;
        app.options_slices_by_ts.insert(
            1,
            OptionsChainSlice {
                underlying: "AAPL".into(),
                expiration: Expiration {
                    ts: 1,
                    label: "2026-06-20".into(),
                },
                calls: vec![],
                puts: vec![],
            },
        );
        app.options_polygon_expirations_cache = Some((
            "AAPL".into(),
            vec![Expiration {
                ts: 1,
                label: "2026-06-20".into(),
            }],
        ));
        invalidate_options_refresh_caches(&mut app);
        assert!(app.options_slices_by_ts.is_empty());
        assert!(app.options_polygon_expirations_cache.is_none());
    }

    #[test]
    fn canonical_strikes_puts_only() {
        let mut chain = sample_chain();
        chain.slice.calls.clear();
        chain.slice.puts = vec![
            OptionContract {
                symbol: "P200".into(),
                strike: 200.0,
                right: OptionRight::Put,
                expiration_ts: 1,
                bid: None,
                ask: None,
                last: None,
                volume: None,
                open_interest: None,
                implied_volatility: None,
                greeks: None,
            },
            OptionContract {
                symbol: "P210".into(),
                strike: 210.0,
                right: OptionRight::Put,
                expiration_ts: 1,
                bid: None,
                ask: None,
                last: None,
                volume: None,
                open_interest: None,
                implied_volatility: None,
                greeks: None,
            },
        ];
        assert_eq!(canonical_strikes(&chain), vec![200.0, 210.0]);
    }
}
