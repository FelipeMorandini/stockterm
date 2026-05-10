# SPEC — Issue #1: Stabilize build & critical API wiring

**Source:** [GitHub Issue #1](https://github.com/FelipeMorandini/stockterm/issues/1) (MVP milestone).  
**Goal:** The binary compiles with no errors, Clippy is clean (`-D warnings`), Polygon calls use one configured key, tab navigation and per-tab keyboard handlers are reachable, and async fetch paths are correct.

**Context:** `docs/REQUIREMENTS.md` is not present in the repo; requirements are taken from Issue #1 and `docs/ROADMAP.md` (§4.x / §5).

---

## 1. Current gaps (verified in tree)

| Area | Problem |
|------|---------|
| `src/config/config.rs` | `theme: Option<Theme>` with no `Theme` type or import. |
| `src/api/polygon.rs` | `get_ticker_data(symbol, config)` uses `config.api_key`, but `get_historical_data`, `search_symbols`, `get_news` use a module-level `API_KEY` placeholder string. |
| `src/app/app.rs` | `fetch_ticker_data` calls `get_ticker_data(&self.symbol).await` — wrong arity / missing `&self.config`. |
| `src/app/handlers.rs` | Only global quit / symbol editing / Enter; does not call `next_tab` / `prev_tab`, `handle_portfolio_events`, or `handle_alerts_events`. |
| `src/app/portfolio.rs` | `handle_portfolio_events`: on Enter, calls `app.fetch_ticker_data()` without `.await` inside a synchronous function. |
| `src/app/alerts.rs` | Stray merge on line 87 (`}pub fn handle_alerts_events`) — syntax break; `impl App` in this file is otherwise valid. |
| `src/app/ui.rs` | `draw_stock_view` and related use `Frame` without importing it (e.g. `ratatui::Frame`). |
| Tabs vs acceptance | Issue acceptance refers to an “Alerts tab”; `Tab` today has no `Alerts` variant — alerts UI is only embedded under Stock View. A dedicated tab is required to satisfy “On the Alerts tab, the alerts handler runs.” |

---

## 2. Crate & module layout (no new crates)

- **Single package:** `stockterm` (`Cargo.toml` as today).
- **Modules:** Keep `config`, `api::polygon`, `app::{app, handlers, ui, event, portfolio, alerts, charts}`.
- **Async runtime:** `tokio` full (already). Main loop stays in `App::run`: `draw` → `events.next()` → dispatch input → optional tick-driven fetches.

---

## 3. Implementation plan (Rust)

### 3.1 `Theme` (`src/config/`)

- Add a minimal `Theme` type (new file `src/config/theme.rs` or inline in `config.rs`) with `Serialize` / `Deserialize` / `Clone` / `Debug` / `Default`.
- Fields can be placeholder palette values (e.g. optional hex strings or simple enum) sufficient for `serde_json` round-trip; full theming is deferred to roadmap / #14.
- Export from `config/mod.rs` and ensure `Config.theme: Option<Theme>` compiles.

### 3.2 Polygon API key (`src/api/polygon.rs`)

- Remove the `const API_KEY` literal and any `YOUR_POLYGON_API_KEY` substring from `src/`.
- Thread `&Config` (or `&str` key) into **`get_historical_data`**, **`search_symbols`**, and **`get_news`** so all four public functions use the same source.
- **Resolution order** for the key string (document in code comments):
  1. `config.api_key` when non-empty (from `~/.stockterm.json` after load).
  2. Else `std::env::var("STOCKTERM_API_KEY")` when `Ok` and non-empty.
  3. Else empty string (callers may surface API errors — acceptable for MVP; optional: log in `fetch_*` paths).

Implement resolution in `Config::load()` (merge env into default/loaded struct) **or** a small `Config::effective_api_key(&self) -> &str` used by all API calls — pick one approach and use it consistently.

### 3.3 `get_ticker_data` call site (`src/app/app.rs`)

- Standardize on `get_ticker_data(&self.symbol, &self.config).await` (parameter order may be `(symbol, config)` as today in `polygon.rs`; keep a single convention across the module).
- Update any other call sites if added later.

### 3.4 Tab navigation (`src/app/handlers.rs`)

- In `handle_event`, match:
  - `KeyCode::Tab` → `app.next_tab()`
  - `KeyCode::BackTab` → `app.prev_tab()` (Shift+Tab)
- Ensure these do not conflict with symbol input; typically only apply when **not** in a raw “typing” mode, or always take precedence over `Char` — Issue #1 implies global tab switching; **recommendation:** process Tab / BackTab **before** the generic `Char` branch so tab switching works from any tab.

### 3.5 Per-tab dispatch (`src/app/handlers.rs`)

- After global keys (quit, tab switch), branch on `app.active_tab`:
  - `Tab::Portfolio` → `handle_portfolio_events(app, key)` (and return / do not fall through to stock symbol keys if portfolio consumes the event — match issue intent: portfolio `a`/`d` work on that tab).
  - `Tab::Alerts` → `handle_alerts_events(app, key)` (see §3.6).
  - Other tabs: retain or extend behavior (e.g. Stock View symbol keys, Enter → `should_fetch_ticker`).
- Import handlers from `crate::app::portfolio` and `crate::app::alerts`.

**Portfolio vs symbol input:** On `Tab::StockView`, uppercase letters append to `app.symbol` (current behavior). On `Tab::Portfolio`, `a`/`d` must hit `handle_portfolio_events` and must **not** append to `symbol`. Order branches accordingly.

### 3.6 `Tab::Alerts` (`src/app/app.rs`, `src/app/ui.rs`)

- Add `Alerts` to the `Tab` enum.
- Update `next_tab` / `prev_tab` cycle order to include Alerts (consistent with tab bar order).
- Update `ui.rs` tab titles and `.select` index mapping so the bar matches `active_tab`.
- In `draw`, add `Tab::Alerts => draw_alerts(f, app, chunks[1])`.
- Remove or reduce duplication: if Stock View currently embeds `draw_alerts` in a split layout, either remove that embed so alerts live only on `Tab::Alerts`, or keep both — **SPEC preference:** single primary surface on `Tab::Alerts` to match acceptance; drop the embedded alerts panel from Stock View unless product wants both (Issue #1 favors a clear “Alerts tab”).

### 3.7 Async portfolio handler (`src/app/portfolio.rs`, `src/app/handlers.rs`, `src/app/app.rs`)

- Change `handle_portfolio_events` to `pub async fn handle_portfolio_events(app: &mut App, key: KeyEvent)`.
- On Enter (jump to stock + refresh): set `app.symbol`, then `app.fetch_ticker_data().await`, then switch tab — **or** set `app.should_fetch_ticker = true` and avoid duplicate fetch if the main loop already handles it; prefer **one** clear path. Recommended: `await fetch_ticker_data` inside the async handler for immediate consistency with Issue #1.
- Change `handle_event` to `pub async fn handle_event(app: &mut App, key: KeyEvent)` and `App::run` to `handle_event(self, input).await`.
- `handle_alerts_events` can remain sync unless it needs await; if `handle_event` is async, call sync subhandler without `.await`.

### 3.8 UI compile fix (`src/app/ui.rs`)

- Add `use ratatui::Frame` (or fully qualify) for `draw_stock_view`, `draw_search`, `draw_news`, `draw_settings` signatures.

### 3.9 `src/app/alerts.rs` hygiene

- Fix the merged `}` / `pub fn` line so the file parses.
- Optionally implement `save_alerts` to persist `config.alerts` (ROADMAP notes no-op); **out of scope for Issue #1** unless needed for build.

---

## 4. Tokio / control flow

- No new spawned tasks required. All network I/O stays on the main async runtime driven from `App::run`.
- Tick handler (`Event::Tick`) continues periodic fetches for Stock / Charts / News as today; portfolio Enter-triggered fetch uses direct `await` in async `handle_portfolio_events` or the existing `should_fetch_ticker` flag — document the chosen approach in the PR.

---

## 5. Verification targets

- `cargo build --release` — zero errors, zero warnings.
- `cargo clippy -- -D warnings` — clean.
- `rg 'YOUR_POLYGON_API_KEY' src/` — no matches.
- **Polygon key:** If `effective_api_key()` is empty, do not call Polygon; show a single clear error (avoid `apiKey=` on the wire and confusing 401s). Yahoo or other providers remain out of scope for Issue #1.
- **Portfolio / Alerts tables:** `TableState` must have a defined selection after load (non-empty lists), after add, and after delete (clamped). Arrow **Up**/**Down** must match regardless of crossterm modifier bits on the key event; first **Down** from `selected == None` selects row `0`.
- **Audit hardening:** Polygon URLs must percent-encode user/config-derived path and query parts (`urlencoding` or equivalent). Config load/save and the crossterm poll thread must not panic on I/O or missing HOME (`Config::try_load` / `try_save`, non-panicking `event::poll`/`read`).

---

## 6. Out of scope (Issue #1)

- Search / News / Settings pane UX beyond what is needed to compile.
- Candlestick widget, watchlist persistence, full theme system (#14).
- Fixing `save_alerts` persistence unless required for compile.

---

## 7. Approval

After maintainer approval of this SPEC, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and `docs/QA_PLAN.md`.
