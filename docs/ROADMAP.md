# StockTerm — Product Roadmap

_A living gap analysis between the current codebase and the StockTerm product
requirements. Source of truth for the next round of `docs/SPEC.md` work._

Last updated: 2026-05-10

---

## 1. Project Snapshot

**StockTerm** is a Rust-based, terminal UI (TUI) stock-tracking application.

Stack (from `Cargo.toml`):

| Concern              | Crate / Version                 |
| -------------------- | ------------------------------- |
| Async runtime        | `tokio = "1"` (full features)   |
| HTTP client          | `reqwest = "0.11"` (json, rustls-tls) |
| TUI framework        | `ratatui = "0.25.0"`            |
| Terminal backend     | `crossterm = "0.27.0"`          |
| Serialization        | `serde = "1"` + `serde_json = "1"` |
| CLI parsing          | `clap = "4"` (derive)           |
| Time / dates         | `chrono = "0.4.40"`             |
| Config dirs          | `dirs = "6.0.0"`                |
| Edition              | `2021`                          |

Crate layout (from `src/`):

- `main.rs` — terminal bootstrap (raw mode, alt screen, `App::run`).
- `lib.rs` — re-exports `app`, `api`, `config`, `models`.
- `api/polygon.rs` — single Polygon.io REST client.
- `app/` — `app.rs` (state machine), `ui.rs`, `event.rs`, `handlers.rs`,
  `charts.rs`, `portfolio.rs`, `alerts.rs`.
- `config/config.rs` — JSON-backed config at `~/.stockterm.json`.
- `models/` — `ticker`, `historical`, `search`, `news`, `portfolio`, `alerts`.
- `tests/` — unit tests in `src/` (`config`, `models::ticker`); no `tests/` integration suite yet.

See `docs/SPEC.md`, `docs/QA_PLAN.md`, and this roadmap for product/engineering docs.

---

## 2. GitHub Issues

Queried via the GitHub MCP `list_issues` tool against
`FelipeMorandini/stockterm` (no state filter, both `OPEN` and `CLOSED`).

- Issues are tracked on GitHub (`FelipeMorandini/stockterm`); M0 was Issue **#1**.
  Tech-debt follow-ups from the ship phase are filed as separate issues.

This roadmap remains the de-facto starting backlog. The "Recommended next milestones" section below is a
suggested seed for issues to file.

---

## 3. Process / SDD Gap

Workspace rule `.cursor/rules/sdd_workflow.mdc` requires Spec-Driven Development:
> No feature code changes are permitted unless the `docs/SPEC.md` is updated
> first... `engineer` must verify implementation against the `QA_PLAN.md`.

Current state:

- `docs/SPEC.md` — **missing**.
- `docs/QA_PLAN.md` — **missing**.
- `docs/ROADMAP.md` — created by this pass.

**Action**: before any further feature code is merged, the architect must
produce a finalized `docs/SPEC.md` (and matching `docs/QA_PLAN.md`) for at
least the next milestone. This roadmap does not invent SPEC content; it only
identifies the gap and frames the work that the SPEC must cover.

---

## 4. Requirement Coverage

Legend: **Implemented** = working end-to-end; **Partial** = code exists but
incomplete, broken, or unwired; **Missing** = no code path.

### 4.1 Core — Real-time quotes

- **Partial — quotes (daily aggregates, not streaming)**
  - Evidence: `get_ticker_data` (`api/polygon.rs`) uses Polygon daily aggregates
    over a rolling window; `draw_stock_detail` / watchlist table (`src/app/ui.rs`)
    show OHLCV from `TickerResult`.
  - Values remain **EOD-style / daily**, not true streaming real-time.
- **Implemented — watchlist + multi-row table (Issue #3)**
  - Evidence: `Config.watchlist`, `App.watchlist` / `watchlist_quotes`,
    `run_stock_quote_batch` + bounded concurrency (`src/app/app.rs`); Stock View
    table + detail pane; persist via `Config::try_save`.
- **Partial — configurable refresh**
  - Evidence: `data_poll_interval()` uses `Config.refresh_rate` (seconds, min 5)
    for throttled quote / charts / news fetches. UI tick remains ~200 ms via
    `spawn_event_thread` (`src/app/event.rs`).

### 4.2 Core — Symbol search with typeahead

- **Partial — search backend only**
  - Evidence: `api/polygon.rs::search_symbols`, `models/search.rs`
    (`SymbolSearchResponse`), `App::search_symbols`, `App.search_query`,
    `App.search_results`.
  - Gaps: `draw_search` in `src/app/ui.rs` is an empty stub; no debounce, no
    typeahead suggestion list, no key handling for the Search tab in
    `app/handlers.rs`.

### 4.3 Core — Portfolio (CRUD, totals, P/L, share counts)

- **Implemented (logic)**
  - Evidence: `models/portfolio.rs::PortfolioItem` with `shares`,
    `purchase_price`, `current_price`, `market_value`, `cost_basis`,
    `profit_loss`, `profit_loss_percent`.
  - `App::add_to_portfolio` does weighted-average cost on add;
    `App::remove_from_portfolio`; `calculate_portfolio_value`,
    `calculate_portfolio_cost`, `calculate_portfolio_profit_loss`.
  - `draw_portfolio` (`src/app/portfolio.rs`) renders summary + holdings table
    (Symbol/Shares/Avg/Current/Value/P/L/P/L %).
  - `current_price` is back-filled from the watchlist quote batch
    (`watchlist_quotes`) when symbols match.
- **Partial — UX wiring**
  - Add/remove are bound to `'a'`/`'d'` in `handle_portfolio_events` but with
    **hard-coded** `(1.0, 100.0)` shares/price — no input dialog.
  - `handle_portfolio_events` is dispatched from `handlers.rs` on `Tab::Portfolio`;
    Enter jumps to Stock View and triggers `request_immediate_stock_poll`.

### 4.4 Core — Historical charts in terminal

- **Partial — line chart**
  - Evidence: `draw_charts` in `src/app/charts.rs` builds a `ratatui::Chart`
    with `GraphType::Line` over `historical_data.results`.
- **Partial — "candlestick"**
  - Evidence: `draw_candlestick` in `src/app/charts.rs` renders only an OHLC
    text table colored green/red — not a real candlestick widget.
  - `draw_candlestick` is imported by `ui.rs` but never called from any tab.
- **Missing — interactivity (zoom/pan)** — no key bindings, no viewport state.

### 4.5 Core — Time ranges (1D/1W/1M/1Y)

- **Missing**
  - Evidence: `App::fetch_historical_data` hard-codes `from_date = now − 30
    days`, `timespan = "day"`. There is no `TimeRange` enum, no key binding,
    no UI selector.

### 4.6 Core — Price alerts and notifications

- **Partial**
  - Evidence: `models/alerts.rs::{Alert, AlertCondition::{Above,Below}}`,
    `App::{add_alert, remove_alert, check_alerts, get_current_price}` and
    `draw_alerts` (`src/app/alerts.rs`).
  - `Config.alerts` field exists; `check_alerts` writes to it on transition.
  - Gaps:
    - **Resolved (Issue #27):** `save_alerts` persists `alerts` to
      `~/.stockterm.json` via `Config::try_save`, with errors in
      `App.error_message`.
    - **Resolved (Issues #30 / #38 / #3):** `check_alerts` runs after each
      successful quote batch update; throttled quote fetch on `Tab::StockView` and
      `Tab::Alerts` (shared throttle).
    - `handle_alerts_events` is dispatched from `handlers.rs` on `Tab::Alerts`.
    - No OS-level notification (no `notify-rust` / bell / toast); only an
      in-pane "TRIGGERED" label.
    - Alert add UX is hard-coded to `(Above, $100)` with no dialog.

### 4.7 Core — News headlines

- **Partial — backend only**
  - Evidence: `api/polygon.rs::get_news`, `models/news.rs`,
    `App::fetch_news`, tick-driven on `Tab::News`.
  - Gap: `draw_news` in `src/app/ui.rs` is an empty stub — headlines never
    render even when fetched.

### 4.8 TUI — Layout, color, formatting

- **Implemented — base layout**
  - Evidence: `ui.rs::draw` builds a top tab bar + content + status bar with
    `ratatui::Layout`, `Tabs`, `Block::borders`, color spans for change/P/L.
- **Partial — empty tabs** — Search, News, Settings panes are stubs.

### 4.9 TUI — Interactive charts (zoom/pan)

- **Missing** — chart viewport is fixed to data min/max bounds; no key handling
  modifies it.

### 4.10 TUI — Keyboard navigation & customizable shortcuts

- **Partial — minimal nav**
  - Evidence: `handle_event` in `src/app/handlers.rs` handles `q` (quit),
    A–Z (append to symbol), Backspace (pop), Enter (refetch).
  - `App::next_tab`/`prev_tab` exist but are **not bound** to any key in the
    handler (Tab/Shift-Tab unhandled).
  - Per-tab handlers (`handle_portfolio_events`, `handle_alerts_events`) are
    not dispatched from the main handler.
- **Missing — customizable shortcuts** — no keymap structure in `Config`.

### 4.11 TUI — Configurable display / layout / theme

- **Partial / broken**
  - `Config.theme: Option<Theme>` is declared in `src/config/config.rs` but
    **`Theme` is not defined or imported anywhere** — this is a compile error
    until a `Theme` type is added (or the field removed).
  - No theme is read or applied in any draw fn.
- **Missing — layout customization** — the layout is hard-coded in `ui.rs`.

### 4.12 TUI — Filter stocks

- **Missing** — no filter input, no predicate over portfolio/watchlist.

### 4.13 Technical — Async fetching, non-blocking UI

- **Partial**
  - Evidence: `App::run` uses `tokio::select!` over async event + `FetchDone`
    channels; stock / historical / news HTTP runs in `tokio::spawn` (Issue #3).
  - Gap: [Issue #17](https://github.com/FelipeMorandini/stockterm/issues/17)
    remains for cancellation semantics, `search_symbols` when Search tab ships,
    and full acceptance criteria (smoke test with artificial delay).

### 4.14 Technical — Stock API integration with rate limits & errors

- **Partial**
  - Evidence: [Issue #31](https://github.com/FelipeMorandini/stockterm/issues/31)
    landed **`MarketDataProvider`** (`yahoo` default, `polygon` opt-in),
    shared **`reqwest::Client`** with connect/request timeouts (`src/api/http.rs`),
    and **`ProviderError`** (`src/api/error.rs`). Polygon requests use the same
    client; HTTP errors shown to the user omit query strings (no `apiKey` leak).
  - Gaps:
    - Full rate-limit handling (429 / `Retry-After`), exponential backoff,
      and richer **`ProviderError`** taxonomy — [Issue #18](https://github.com/FelipeMorandini/stockterm/issues/18).
    - Watchlist fan-out still uses **N** Yahoo quote requests — batching follow-up
      [Issue #53](https://github.com/FelipeMorandini/stockterm/issues/53).

### 4.15 Technical — Config file for prefs / portfolio

- **Implemented (basic)**
  - Evidence: `Config::{load, save, get_config_path}` in
    `src/config/config.rs` reads/writes JSON at
    `$HOME/.stockterm.json`. Persists portfolio and (intent) alerts.
  - Gaps: `Config::save` `unwrap`s I/O errors; `theme` field unusable
    (see 4.11); `refresh_rate` ignored; `default_symbol` set in struct but
    not consumed in `App::new` (hard-codes `"AAPL"`).

### 4.16 Technical — Clear errors

- **Partial** — string-formatted `App.error_message` shown in status bar /
  stock view. No error categorization (network vs API vs parse), no error
  log, no retry UX.

### 4.17 Technical — Cross-platform

- **Implemented (by virtue of stack)** — `ratatui` + `crossterm` cover
  Linux/macOS/Windows; no platform-specific code present.

### 4.18 Technical — Persistence between sessions

- **Partial**
  - Portfolio persists via `Config.save` after add/remove.
    - Alerts persist on add/remove via `save_alerts` → `Config::try_save` (Issue
      #27); `triggered` transitions run via `check_alerts` after quote refresh
      (Issues #30 / #38 / #3).
  - **Watchlist persists** (`Config.watchlist`, Issue #3).
  - Last-selected tab, last symbol (beyond watchlist default), and theme do not persist.

### 4.19 Advanced / optional

- **Missing** — Technical indicators (SMA/EMA/RSI/MACD), options chains,
  crypto, custom widgets, backtesting. None of these have any code, types, or
  modules.

---

## 5. Code-quality / Stability Gaps

_Many pre–M0 items (Theme, Polygon key plumbing, tab handlers, async portfolio
Enter) were fixed in Issue #1 ([PR #26](https://github.com/FelipeMorandini/stockterm/pull/26)).
Alert persistence landed in Issue #27._

Open gaps worth tracking:

1. `draw_search`, `draw_news`, `draw_settings` remain minimal or stub UIs.
2. Test coverage is thin (a few unit tests only); expand per milestone M7.
3. Main loop still awaits network I/O inline (UI can stall on slow API) — see §4.13; [Issue #17](https://github.com/FelipeMorandini/stockterm/issues/17).

_Recent follow-ups from ship:_ [Issue #39](https://github.com/FelipeMorandini/stockterm/issues/39)
(portfolio `try_save` parity), [Issue #40](https://github.com/FelipeMorandini/stockterm/issues/40)
(non-blocking config I/O). Issues [#30](https://github.com/FelipeMorandini/stockterm/issues/30)/[#37](https://github.com/FelipeMorandini/stockterm/issues/37)/[#38](https://github.com/FelipeMorandini/stockterm/issues/38) (alerts loop + table) shipped in the PR linked from `docs/SPEC.md` §7.
[Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) (keyboard modifiers for Stock View / Alerts) shipped in [PR #52](https://github.com/FelipeMorandini/stockterm/pull/52); deferred polish → [#48](https://github.com/FelipeMorandini/stockterm/issues/48)–[#51](https://github.com/FelipeMorandini/stockterm/issues/51).

---

## 6. Recommended Next Milestones

Suggested ordering (each should land its own `docs/SPEC.md` update + GitHub
issue before code):

1. **M0 — Stabilize build & SDD baseline** ✅ **Delivered** (GitHub Issue #1)
   - Fix `Theme`, `get_ticker_data` signature, hard-coded API key.
   - Wire `next_tab`/`prev_tab` and per-tab handlers into `handle_event`.
   - Author initial `docs/SPEC.md` + `docs/QA_PLAN.md` covering the existing
     tabs.
   - **Merge:** https://github.com/FelipeMorandini/stockterm/pull/26 — manual verification: `docs/QA_PLAN.md`. Follow-up tech debt → GitHub issues filed at ship.
2. **M1 — Swap data source to Yahoo Finance**
   - Replace Polygon client with a Yahoo-Finance-backed module
     (see §7). Keep model layer (`TickerResult`, `HistoricalData`, etc.) as
     an internal contract; add an adapter from the Yahoo response.
   - Add request timeout, non-2xx handling, structured errors.
3. **M2 — Real-time-ish quotes & multi-symbol watchlist**
   - **Partial — delivered:** [Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3) — `Watchlist` in `Config`, multi-row table on Stock View, bounded concurrent Polygon quotes, `refresh_rate` throttle, background fetch via `tokio::select!` (see `docs/SPEC.md`).
   - **Remaining:** intraday / "latest quote" feel (likely **M1** Yahoo `quote` or `chart?range=1d&interval=1m`), full #17 cancellation smoke test.
4. **M3 — Search typeahead + News + Settings UI**
   - Implement `draw_search` with debounced typeahead suggestions.
   - Implement `draw_news` listing headlines (publisher, title, date, link).
   - Implement `draw_settings` to edit `refresh_rate`, `default_symbol`,
     theme, and (later) keymap.
5. **M4 — Time ranges & interactive charts**
   - Add `TimeRange::{D1, W1, M1, Y1}` selector (e.g. `1`, `2`, `3`, `4`).
   - Implement zoom/pan via `+`/`-`/`h`/`l`.
   - Replace text-table candlestick with a real candlestick widget
     (custom `ratatui::Widget` impl).
6. **M5 — Alerts polish**
   - Persist alerts — **done** (Issue #27: `save_alerts` → `Config::try_save`).
   - Drive `check_alerts` after quote refresh — **done** (Issues #30 / #38); table constraints — **done** (#37).
   - Add OS notification (e.g. `notify-rust`) and terminal bell.
   - Add input dialog for symbol/condition/price.
7. **M6 — Filters, customizable shortcuts, themes**
   - Add `Filter` predicate over watchlist/portfolio.
   - Define `Keymap` in `Config`, look up actions via map.
   - Define `Theme` (palette) and apply via a `Style`-builder helper.
8. **M7 — Tests & CI**
   - Unit tests for `models::portfolio` math, `models::alerts::is_triggered`.
   - Snapshot tests for `draw_*` using `ratatui::backend::TestBackend`.
   - Integration test against a mocked HTTP server (`wiremock`).
9. **M8 — Optional / advanced**
   - Indicators (SMA/EMA/RSI/MACD), crypto symbols, options, backtesting,
     custom widgets.

---

## 7. API Strategy Note (Yahoo vs Polygon vs Alpha Vantage / IEX)

The codebase currently targets **Polygon.io** (`src/api/polygon.rs`). The user
prefers a free / cheap source and has accepted **Yahoo Finance** as the default.

Recommendation: **migrate to Yahoo Finance as the primary source**, but
abstract the call sites behind a trait so we can swap providers later.

- **Yahoo Finance (recommended)**
  - Pros: free, no API key, broad coverage (US + international tickers,
    crypto, FX), supports search (`v1/finance/search`), quote
    (`v7/finance/quote`), and historical OHLC (`v8/finance/chart`).
  - Cons: unofficial / undocumented endpoints, can rate-limit by IP, occasional
    schema drift; news endpoint requires scraping or a feed.
  - Rust options: use `reqwest` directly against the public endpoints, or
    adopt a maintained crate (e.g. `yahoo_finance_api`) — pin and vendor
    types into `models/` to insulate the rest of the app.
- **Polygon.io (current code)**
  - Pros: clean REST + docs, official, supports tickers/news/aggregates.
  - Cons: free tier is **5 requests/minute**, end-of-day data only on free
    tier, requires an API key. Real-time and intraday require a paid plan.
- **Alpha Vantage**
  - Pros: free key, simple REST.
  - Cons: free tier is **5 req/min, 500/day**; tighter than even Polygon.
- **IEX Cloud**
  - Pros: low-cost paid tiers, real-time US equities.
  - Cons: paid; 2024+ migration to "IEX Cloud retired" / new platform — risk.

Concrete next step: introduce `src/api/mod.rs` with a `MarketDataProvider`
trait (`get_quote`, `get_history(range)`, `search`, `get_news`), implement
`YahooProvider`, keep `PolygonProvider` as an opt-in alternative wired through
`Config` (e.g. `provider: "yahoo" | "polygon"`, plus optional `api_key`). This
satisfies "use Yahoo Finance for free" while keeping the door open to a paid
provider without rewriting the app layer.

---

## 8. Deliverables checklist for this pass

- [x] `docs/ROADMAP.md` (this file)
- [x] `docs/SPEC.md` — Issue #3 SPEC + shipment section (SDD)
- [x] `docs/QA_PLAN.md` — manual steps for Issue #3
- [x] GitHub issues — backlog tracked in repo (see Issues)

