# StockTerm — Product Roadmap

_A living gap analysis between the current codebase and the StockTerm product
requirements. Source of truth for the next round of `docs/SPEC.md` work._

Last updated: 2026-05-12 — **§18.14 ([#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98))** — implemented in tree: alerts save-failure banner + quote-batch **`save_alerts`** retry, one coalesced desktop toast per crossing batch, sanitized notify `body` (see [`docs/SPEC.md`](SPEC.md) §18.14.9, [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#96–#98** — **manual QA sign-off pending**). **Shipped same day:** **§18.13 ([#93](https://github.com/FelipeMorandini/stockterm/issues/93)–[#95](https://github.com/FelipeMorandini/stockterm/issues/95))** [PR #102](https://github.com/FelipeMorandini/stockterm/pull/102) — shared `app::layout::centered_rect`, alert add dialog **Condition** **←/→** keys, optional **`STOCKTERM_DEBUG_ALERT_NOTIFY`** stderr for `Notification::show()` (§18.13.8). **Open follow-ups:** ship triage [#100](https://github.com/FelipeMorandini/stockterm/issues/100), [#101](https://github.com/FelipeMorandini/stockterm/issues/101). **Earlier:** **Alerts (#10 / #42)** [PR #99](https://github.com/FelipeMorandini/stockterm/pull/99) / §18; Issue #2 [PR #92](https://github.com/FelipeMorandini/stockterm/pull/92) / §17; scratch [#89](https://github.com/FelipeMorandini/stockterm/issues/89)–[#91](https://github.com/FelipeMorandini/stockterm/issues/91); §16 [PR #88](https://github.com/FelipeMorandini/stockterm/pull/88); audit [#85](https://github.com/FelipeMorandini/stockterm/issues/85)–[#87](https://github.com/FelipeMorandini/stockterm/issues/87); §15 (#43, #49, #50, #67, #69); [#81](https://github.com/FelipeMorandini/stockterm/issues/81)–[#83](https://github.com/FelipeMorandini/stockterm/issues/83); charts [#76](https://github.com/FelipeMorandini/stockterm/issues/76)–[#79](https://github.com/FelipeMorandini/stockterm/issues/79).

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
  `charts.rs`, `layout.rs` (modal `centered_rect`), `portfolio.rs`, `alerts.rs`.
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

- `docs/SPEC.md` — maintained (SDD baseline + milestones; latest shipped slices §11.12 / [#71](https://github.com/FelipeMorandini/stockterm/issues/71)–[#74](https://github.com/FelipeMorandini/stockterm/issues/74), §15 / [#43](https://github.com/FelipeMorandini/stockterm/issues/43) [#49](https://github.com/FelipeMorandini/stockterm/issues/49) [#50](https://github.com/FelipeMorandini/stockterm/issues/50) [#67](https://github.com/FelipeMorandini/stockterm/issues/67) [#69](https://github.com/FelipeMorandini/stockterm/issues/69), **§18 / [#10](https://github.com/FelipeMorandini/stockterm/issues/10) [#42](https://github.com/FelipeMorandini/stockterm/issues/42)** — alerts dialog, notifications, latched Status, **§18.13 / [#93](https://github.com/FelipeMorandini/stockterm/issues/93)–[#95](https://github.com/FelipeMorandini/stockterm/issues/95)** [PR #102](https://github.com/FelipeMorandini/stockterm/pull/102), and **§18.14 / [#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98)** — alerts persistence banner + retry, notify batching, symbol sanitize — **§18.14.9** code landed; **manual QA pending**).
- `docs/QA_PLAN.md` — maintained (manual steps per milestone).
- `docs/ROADMAP.md` — this file (gap analysis vs product goals).

**Process:** new feature code follows `.cursor/rules/sdd_workflow.mdc` — update SPEC first, then implement, then verify against QA_PLAN.

---

## 4. Requirement Coverage

Legend: **Implemented** = working end-to-end; **Partial** = code exists but
incomplete, broken, or unwired; **Missing** = no code path.

### 4.1 Core — Real-time quotes

- **Implemented — latest-session quotes via REST ([Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2), [`docs/SPEC.md`](SPEC.md) §17)** — not streaming / not Level-2.
  - Evidence: **`MarketDataProvider::get_quote`** — **Yahoo:** **`v7/finance/quote`** primary, **`v8/finance/chart`** `range=1d&interval=1d` fallback (`yahoo_latest_quote` in `src/api/yahoo.rs`); maps into **`TickerResult`**. **Polygon:** `PolygonProvider::get_quote` — daily aggregates, rolling window, **`sort=desc`** + **`limit=5`** + `latest_result()` (`src/api/polygon.rs`). Batched in **`run_stock_quote_batch`** (`src/app/app.rs`). **`draw_stock_detail`** / watchlist (`src/app/ui.rs`) unchanged at **`TickerResult`**.
  - **Follow-ups:** [#89](https://github.com/FelipeMorandini/stockterm/issues/89) (integration test v7→v8), [#90](https://github.com/FelipeMorandini/stockterm/issues/90) (fallback observability), [#91](https://github.com/FelipeMorandini/stockterm/issues/91) (v7 row symbol match); batching [#53](https://github.com/FelipeMorandini/stockterm/issues/53); rate limits [#18](https://github.com/FelipeMorandini/stockterm/issues/18).
- **Implemented — watchlist + multi-row table (Issue #3)**
  - Evidence: `Config.watchlist`, `App.watchlist` / `watchlist_quotes`,
    `run_stock_quote_batch` + bounded concurrency (`src/app/app.rs`); Stock View
    table + detail pane; persist via `Config::try_save`.
- **Partial — configurable refresh**
  - Evidence: `data_poll_interval()` uses `Config.refresh_rate` (seconds, min 5)
    for throttled quote / charts / news fetches. UI tick remains ~200 ms via
    `spawn_event_thread` (`src/app/event.rs`).

### 4.2 Core — Symbol search with typeahead

- **Implemented (Issues #5 / #29)**
  - Evidence: `draw_search` + `handle_search_events` (`src/app/ui.rs`,
    `handlers.rs`); debounced `FetchDone::Search` + `spawn_search_task`
    (`src/app/app.rs`); provider `search_symbols` via Yahoo/Polygon.

### 4.3 Core — Portfolio (CRUD, totals, P/L, share counts)

- **Implemented (Issues [#6](https://github.com/FelipeMorandini/stockterm/issues/6) / [#48](https://github.com/FelipeMorandini/stockterm/issues/48))**
  - Evidence: `models/portfolio.rs::PortfolioItem`; `App::add_to_portfolio` /
    `remove_from_portfolio` with **`Config::try_save`**; weighted-average cost;
    totals helpers; `draw_portfolio` + add dialog / two-step remove /
    `letter_key_plain` (`src/app/portfolio.rs`).
  - Quote batch includes **watchlist + active symbol + all portfolio tickers**
    (`collect_symbols_for_quote_fetch`); `apply_stock_fetch_done` back-fills
    `current_price` from `watchlist_quotes`.
  - `handle_portfolio_events` from `handlers.rs` on `Tab::Portfolio`; Enter → Stock
    View + `request_immediate_stock_poll`.
- **Implemented (Issues [#43](https://github.com/FelipeMorandini/stockterm/issues/43) / [#49](https://github.com/FelipeMorandini/stockterm/issues/49) / [#50](https://github.com/FelipeMorandini/stockterm/issues/50) / [#67](https://github.com/FelipeMorandini/stockterm/issues/67) / [#69](https://github.com/FelipeMorandini/stockterm/issues/69), `docs/SPEC.md` §15)** — Alerts **Price Alerts** title parity + empty-state **a/A** copy; Stock View status **A–Z** + **w/x/j/k** Shift hint; portfolio add dialog **Tab**/**Shift+Tab** field focus; **`inline_error`** on commit when `add_to_portfolio` fails without **`try_save`**; **`validate_holding_limits`** (shares/price caps).
- **Partial — further polish** — optional decimal money ([#68](https://github.com/FelipeMorandini/stockterm/issues/68)); row edit UI not implemented; narrow-terminal status bar ([#81](https://github.com/FelipeMorandini/stockterm/issues/81)); plain-Tab-only dialog cycle ([#82](https://github.com/FelipeMorandini/stockterm/issues/82)); **`add_to_portfolio`** error-path docs ([#83](https://github.com/FelipeMorandini/stockterm/issues/83)).

### 4.4 Core — Historical charts in terminal

- **Implemented (Issues #7 / #8 / #9, M4)** — line + candlestick widget, viewport zoom/pan, `TimeRange` keys; see `docs/SPEC.md` §11.
- **Implemented (Issues [#62](https://github.com/FelipeMorandini/stockterm/issues/62) / [#63](https://github.com/FelipeMorandini/stockterm/issues/63) / [#64](https://github.com/FelipeMorandini/stockterm/issues/64), §11.11)** — symbol change clears stale `historical_data`; Yahoo W1 intraday empty → daily retry; transient historical errors keep last-good series; viewport ticker uses requested symbol when response `ticker` is empty; see `docs/SPEC.md` §11.11.7.
- **Implemented (Issues [#71](https://github.com/FelipeMorandini/stockterm/issues/71)–[#74](https://github.com/FelipeMorandini/stockterm/issues/74), §11.12)** — `InflightRecovery` + second channel when `FetchDone` send fails; removed dead **`fetch_historical_data`**; **`yahoo_w1_daily_fallback_interval`** + tests; watchlist add skips chart clear on case-only normalization — see [`docs/SPEC.md`](SPEC.md) §11.12.8.
- **Partial — further polish** — dense candle layout vs web charts; optional follow-ups [#76](https://github.com/FelipeMorandini/stockterm/issues/76)–[#79](https://github.com/FelipeMorandini/stockterm/issues/79) (tracing, pending-flag edge case, recovery hardening, Unicode tickers).

### 4.5 Core — Time ranges (1D/1W/1M/1Y)

- **Implemented (Issue #9 / M4)** — `TimeRange`, provider mapping, Charts keys `1`–`4`; see `docs/SPEC.md` §11.

### 4.6 Core — Price alerts and notifications

- **Implemented ([Issues #10](https://github.com/FelipeMorandini/stockterm/issues/10) / [#42](https://github.com/FelipeMorandini/stockterm/issues/42), [`docs/SPEC.md`](SPEC.md) §18)**
  - Evidence: `models/alerts.rs` (`Alert`, `AlertCondition`, `process_alert_crossings`); `App::{add_alert, remove_alert, check_alerts, get_current_price}`; `draw_alerts` / `AlertAddDialog` / `handle_alerts_events` (`src/app/alerts.rs`); `save_alerts` → `Config::try_save`; `check_alerts` after `apply_stock_fetch_done`; terminal **BEL** + optional **`notify-rust`** (Cargo feature **`desktop-notify`**, default on) when `notifications_enabled`; Settings row **Desktop alert toasts**; **Status** uses latched **`triggered`** (**TRIGGERED** / **Armed** / **No quote**). Shipped: [PR #99](https://github.com/FelipeMorandini/stockterm/pull/99).
  - **Follow-ups:** [#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98) — **code landed** per [`docs/SPEC.md`](SPEC.md) **§18.14.9** (manual QA in [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#96–#98** pending); [#100](https://github.com/FelipeMorandini/stockterm/issues/100) / [#101](https://github.com/FelipeMorandini/stockterm/issues/101) (§18.13 ship triage); [#19](https://github.com/FelipeMorandini/stockterm/issues/19) (persistence UX overlap for failed saves). **§18.13 (#93–#95)** shipped 2026-05-12 — `src/app/layout.rs`, Condition **←/→**, `STOCKTERM_DEBUG_ALERT_NOTIFY`.

### 4.7 Core — News headlines

- **Implemented (Issues #11 / #29); clipboard copy follow-up #58**
  - Evidence: `draw_news`, `news_list_state`, `handle_news_events`; throttled
    `try_spawn_news_fetch` + `FetchDone::News`. Yahoo path uses `query1`
    search `news` + RSS fallback before legacy `query2` (`src/api/yahoo.rs`).
  - Gap: optional clipboard copy (#58); non-blocking `open` tracked in #59.

### 4.8 TUI — Layout, color, formatting

- **Implemented — base layout**
  - Evidence: `ui.rs::draw` builds a top tab bar + content + status bar with
    `ratatui::Layout`, `Tabs`, `Block::borders`, color spans for change/P/L.
- **Partial — empty tabs** — Charts-focused stubs only; Search, News, Settings
  implemented (Issues #29 / #5 / #11 / #12).

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

- **Partial — §16 slice shipped (Issues [#17](https://github.com/FelipeMorandini/stockterm/issues/17) / [#46](https://github.com/FelipeMorandini/stockterm/issues/46) / [#77](https://github.com/FelipeMorandini/stockterm/issues/77); [`docs/SPEC.md`](SPEC.md) §16.8); optional cancel token remains**
  - Evidence: `App::run` uses `tokio::select!` over async event + `FetchDone` + `InflightRecovery`; `event.rs` bridges crossterm from a std thread; stock / historical / news / search HTTP runs in `tokio::spawn` (Issue #3, §11.12).
  - **Shipped (2026-05-11):** **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** before quote fan-out; stock batch **`catch_unwind`** + synthetic `FetchDone::Stock` on panic; **`apply_inflight_recovery(Stock)`** drains **`stock_refresh_pending`**; recovery **`send`** failures logged. **Optional follow-up:** **`CancellationToken`** if overlapping batches are introduced; clippy lock hygiene on future refactors.

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

1. Charts / candlestick / time-range UX remain partial; Search/News/Settings
   are implemented (M3).
2. Test coverage is thin (a few unit tests only); expand per milestone M7.
3. **Optional:** **`CancellationToken`** for quote overlap — [`docs/SPEC.md`](SPEC.md) §16.1 item 2 (if product adds overlapping batches).

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
   - **Remaining:** intraday / "latest quote" feel (likely **M1** Yahoo `quote` or `chart?range=1d&interval=1m`); **#17** artificial-delay smoke + optional cancel semantics — [`docs/SPEC.md`](SPEC.md) §16.1.
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

