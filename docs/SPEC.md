# SPEC — StockTerm (Issue #3 baseline + follow-ons)

**Issue #3** — Multi-symbol watchlist & multi-row quote table (§§1–7). **Issue #44** — Stock View & Alerts keyboard modifiers (§8, shipped). **Issues #48 / #6** — Portfolio tab: keyboard parity (§12, shipped); add dialog, confirm remove, quote coverage (§13, shipped). **Issue #31** — Yahoo Finance default provider & Polygon fallback (§9, shipped). **Issues #29 / #5 / #11 / #12** — Search typeahead, News list, Settings editor (§10, shipped — see §10.9 PR). **Issues #9 / #8 / #7** — Historical time ranges, chart viewport (zoom/pan), real candlestick widget (§11, shipped — see §11.10 PR). **Issues #62 / #63 / #64** — Charts polish: symbol/series coherence, Yahoo W1 empty fallback, historical fetch resilience (§11.11, shipped — see §11.11.7). **Issues #71 / #72 / #73 / #74** — Charts/async hardening: inflight recovery on channel send failure, remove dead sync historical fetch, Yahoo W1 unit tests, watchlist add without spurious chart clear (§11.12, shipped — see §11.12.8). **Issues #43 / #49 / #50 / #67 / #69** — Alerts titles & copy, Stock View watchlist typing hint, Portfolio dialog Tab/Shift+Tab field focus, commit inline errors and optional numeric caps (§15, shipped — see §15.8). **Issues #17 / #46 / #77** — Non-blocking loop completion, quote-batch panic-safety, and `stock_refresh_pending` on stock inflight recovery (§16, shipped — see §16.8). **Issue #2** — Latest-session stock quotes via provider adapters (§17, shipped — see §17.9). **Issues #10 / #42** — Alerts: add dialog + bell/desktop notify + Settings toggle; Status column from latched `triggered` (§18, shipped — see §18.12).

**Sources (Issue #3):**

- [GitHub Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3) — `Watchlist` in config, fan-out quotes, Stock View table, navigation, persistence, bounded concurrency, non-blocking refresh.

**Related issues (dependencies / alignment):**

- [#4](https://github.com/FelipeMorandini/stockterm/issues/4) — `Config.refresh_rate` drives quote refresh cadence (seconds); UI tick stays fast (~200 ms).
- [#17](https://github.com/FelipeMorandini/stockterm/issues/17) — Network I/O must not sit inline between redraws; input stays responsive during slow API.
- [#18](https://github.com/FelipeMorandini/stockterm/issues/18) — Shared HTTP client, timeouts, 429/backoff, concurrency cap (this SPEC adopts a **minimal** cap for watchlist fan-out; full `ProviderError` work can extend #18).
- [#19](https://github.com/FelipeMorandini/stockterm/issues/19) — Surface `Config::try_save` failures via `App.error_message`; avoid silent persistence loss for watchlist edits.

**Overlap note:** Issue #3 acceptance requires refresh to respect `refresh_rate` and not block input. **As of the §11.12 tree**, [`App::run`](../src/app/app.rs) uses **`tokio::select!`** over **`tokio::sync::mpsc`** event / `FetchDone` / `InflightRecovery` channels, and quote / historical / news / search HTTP runs inside **`tokio::spawn`** tasks — **no HTTP `await` on the path between `draw` and the next `select!` branch**. Remaining **#17** work is **acceptance polish** (documented smoke delay, optional `CancellationToken`, clippy lock hygiene) — see **§16**.

---

## 1. Current gaps (verified in tree)

| Area | Location | Problem |
|------|----------|---------|
| Single symbol | `App::symbol` only | No persisted list; Stock View is a single-symbol paragraph ([`src/app/ui.rs`](../src/app/ui.rs) `draw_stock_view`). |
| Config | [`src/config/config.rs`](../src/config/config.rs) | No `watchlist` field; older JSON files must still deserialize after adding the field (`serde(default)`). |
| Quote cache | `App::ticker_data: Option<TickerResponse>` | Only one response; watchlist needs per-symbol quote cache for the table **and** for `get_current_price` ([`src/app/alerts.rs`](../src/app/alerts.rs)) for non-active alert symbols. |
| Keys | [`src/app/handlers.rs`](../src/app/handlers.rs) `handle_stock_view_keys` | No `w` / remove / table navigation. |
| Fan-out | `fetch_ticker_data` | Single `get_ticker_data(&self.symbol, …)` only. |
| Non-blocking (#17) | `App::run` | **Shipped baseline:** async `select!` + background fetches (see §16.1). **Remaining:** smoke harness, optional cancel token, clippy `await_holding_lock` gate. |

**Already helpful in tree:** `data_poll_interval()` uses `config.refresh_rate` with a minimum of 5 seconds ([`src/app/app.rs`](../src/app/app.rs)); `Config::try_save` exists for safe persistence.

---

## 2. Crate & module layout

- **Single package:** `stockterm` (no new crate).
- **`src/config/config.rs`:** Add `watchlist: Vec<String>` with `#[serde(default)]`; document default (empty). Optionally coordinate `default_symbol` with #19 — out of scope for #3 unless the same PR touches `App::new`.
- **`src/app/app.rs`:** Watchlist state, fan-out fetch orchestration, throttle integration, `symbol` / selection invariants, portfolio back-fill from cached quotes where applicable.
- **`src/app/ui.rs`:** Split Stock View into a **watchlist table** + **detail** region (or dedicated `draw_watchlist` in `src/app/stock_view.rs` if the module grows — optional file split).
- **`src/app/handlers.rs`:** Stock View key bindings: add/remove/list navigation; avoid conflicting with existing `A`–`Z` symbol typing (see §3.5).
- **`src/app/alerts.rs`:** Extend `get_current_price` to consult the watchlist quote cache before returning `None`.
- **`src/api/polygon.rs`:** No schema change to `TickerResponse`; reuse `get_ticker_data`. Concurrency limiting may use a small helper or `tokio::sync::Semaphore` in `api` or `app` (prefer one shared semaphore for all Polygon quote calls if #18 lands later).

---

## 3. Implementation plan (Rust)

### 3.1 Config & migration

- Add `pub watchlist: Vec<String>` to `Config` with `#[serde(default)]` so missing field → empty vec on `try_load`.
- Normalize symbols when persisting: uppercase, trim, reject empty strings; dedupe on add.
- After any add/remove/reorder that should persist, assign `self.config.watchlist = self.watchlist.clone()` (or use config as single source of truth) and call **`self.config.try_save()`**; on `Err`, set `self.error_message` (align with #19 / `save_alerts` pattern).

### 3.2 Application state

- **`watchlist: Vec<String>`** — Loaded from `config.watchlist` in `App::new`; kept in sync with `config` on save.
- **`watchlist_state: ratatui::widgets::TableState`** — Selection index into `watchlist` (same pattern as `portfolio_state` / `alerts_state`).
- **`watchlist_quotes: std::collections::HashMap<String, TickerResponse>`** (or `HashMap<String, TickerResult>` if only the latest bar is needed) — Last successful quote per symbol; clear or mark stale per product decision (recommended: update in place on each successful fan-out; on per-symbol error keep previous bar and optionally store a side-channel error map or a single aggregated status string).

**Active symbol (`App::symbol`):**

- Continues to drive Charts, News, alerts add (`'a'`), portfolio context, and the **detail** pane on Stock View.
- **Invariant:** When the user moves the watchlist selection (`j`/`k` or arrows), set `self.symbol` to `watchlist[i]` so the rest of the app tracks the highlighted row.
- **Typing buffer:** Today uppercase letters append to `symbol` and Backspace pops ([`handlers.rs`](../src/app/handlers.rs)). With a table, either:
  - **Recommended:** Treat typing as editing the “pending” ticker: still mutate `symbol`; when the user confirms with **Enter**, fetch and optionally move selection to that symbol if it exists in the watchlist; **or**
  - Keep selection and typed string in sync only when navigating rows (simpler UX: row change overwrites `symbol`).

Document the chosen behavior in QA steps.

### 3.3 Fan-out fetch & throttle

- Replace or extend the single-symbol path with **`fetch_watchlist_quotes`** (name flexible) that:
  1. Builds the distinct set of symbols to refresh: **all `watchlist` entries** plus **`symbol`** if it is non-empty and not already in the set (so the typed ticker still gets a quote before `w` adds it).
  2. Respects **bounded concurrency**: e.g. `const MAX_CONCURRENT_QUOTES: usize = 2` (tunable; Polygon free tier is 5 req/min — sequential or 2-wide fan-out is safer than unbounded `join_all`).
  3. Uses `futures::stream::FuturesUnordered` + `buffer_unordered(N)`, or chunks of `N` with `futures::future::join_all`, or a `Semaphore` with `acquire_owned` around each `get_ticker_data` — all acceptable; pick one style and use it consistently.
  4. Merges successes into `watchlist_quotes` and updates `ticker_data` for **`self.symbol`** from the cached map (or last fetch result) so existing code that reads `ticker_data` for the detail pane keeps working.
  5. After successful updates, **portfolio `current_price` back-fill**: for each portfolio row whose symbol has a fresh quote in the map, update `current_price` (same idea as today’s single-symbol path in `fetch_ticker_data`).
  6. Calls **`check_alerts()`** once after the batch (prices for multiple symbols may now exist via `watchlist_quotes` — see §3.4).

- **Throttle:** Reuse `last_stock_network_poll` + `data_poll_interval()` so watchlist refresh runs on the same cadence as today’s stock poll for `Tab::StockView | Tab::Alerts` (and any other tab that the implementation decides needs fresh quotes — keep parity with current behavior unless SPEC is extended).

- **In-flight guard (#4):** If a watchlist fetch is still running, do not start another full fan-out; optionally set a flag or use a generation counter so only the **latest** completed batch applies (pairs with #17 cancellation semantics).

### 3.4 `get_current_price` & alerts

Extend `get_current_price` order roughly to:

1. If `ticker_data` matches the requested symbol (existing logic) → use it.
2. Else if `watchlist_quotes.get(symbol)` has a latest bar → `Some(bar.c)`.
3. Else portfolio `current_price` (existing).

Then `check_alerts` can evaluate alerts for watchlist symbols without requiring that symbol to be the single global `ticker_data` row.

### 3.5 UI — Stock View

- **Layout:** Vertical split (e.g. `Layout`) — **top:** `Table` with columns **Symbol | Last | Change | % Change | Volume** (values from latest daily bar: `c`, `c-o`, percent vs `o`, `v` rounded).
- **Bottom:** Existing detail block (open/high/low/volume narrative) for **`symbol`**, driven by `ticker_data` or by row lookup in `watchlist_quotes`.
- **Highlight:** `TableState` selection; highlight style consistent with portfolio/alerts tables.
- **Empty watchlist:** Show empty-state hint (“Press `w` to add current symbol”) and still allow typing a symbol and Enter to fetch detail.

### 3.6 Key bindings (Stock View)

| Key | Action |
|-----|--------|
| `w` | Add current `symbol` (normalized) to `watchlist` if not duplicate; persist with `try_save`. |
| `x` or `Shift+d` (`D`) | Remove selected watchlist row; adjust selection; set `symbol` to new selection or first remaining; persist. |
| `j` / `k` or `Up` / `Down` | Move selection; update `symbol` to selected ticker. |

**Conflict check:** Lowercase `a`–`z` are not used today for symbol input (only uppercase). `w`, `x`, `j`, `k` are safe. Use `Shift+d` for delete if `d` would collide with future bindings.

### 3.7 Non-blocking UI (#17)

- **Requirement:** No `await` on `get_quote` / other HTTP on the path between `terminal.draw(…)` and the next **input-capable** turn of the main loop.
- **Shipped pattern (tree):** [`spawn_event_thread`](../src/app/event.rs) bridges crossterm into **`tokio::sync::mpsc::unbounded_channel`**. **`App::run`** uses **`tokio::select!`** over **input/tick**, **`FetchDone`**, and **`InflightRecovery`**. Stock batch (**`run_stock_quote_batch`**), historical, news, and search use **`tokio::spawn`** + **`FetchDone`** variants.
- **Loading:** While **`stock_refresh_inflight`** (or other inflight flags) is true, status UI may show a short “Refreshing…” / busy hint; **ticks keep firing** (~200 ms).
- **Remaining acceptance (#17 / §16):** Artificial-delay smoke test, optional **`tokio_util::sync::CancellationToken`** (or stricter generation docs) for superseded work, **`cargo clippy`** without **`await_holding_lock`** (and similar) on touched code.

If the §16 checklist is not satisfied, QA keeps marking the **#17 smoke** row **fail** until fixed.

### 3.8 API robustness (#18) — minimal slice for #3

- Prefer reusing a single `reqwest::Client` with timeouts once introduced for #18; until then, document that watchlist multiplies call volume and testers should use conservative `refresh_rate` on Polygon free tier.
- Concurrency cap (§3.3) is mandatory for #3 even before full `ProviderError` types.

---

## 4. Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- Optional: unit test for watchlist normalization / dedupe if pure functions are extracted.

---

## 5. Out of scope

- Yahoo migration / `MarketDataProvider` trait (ROADMAP §7).
- Settings UI to edit watchlist (#12 / M3).
- Full `ProviderError` and 429 backoff (#18) — unless merged in the same PR.
- Watchlist ordering UI (drag/sort) — not required; optional stable sort by symbol.

---

## 6. Approval

After maintainer approval of this SPEC, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md).

---

## 7. Shipment

- **Status:** Implemented; closes [Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3). Manual verification: [`docs/QA_PLAN.md`](QA_PLAN.md).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/47
- **Follow-ups:** [Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) — specified in **§8** below (Stock View / Alerts modifier keys). **§16** — [#17](https://github.com/FelipeMorandini/stockterm/issues/17) / [#46](https://github.com/FelipeMorandini/stockterm/issues/46) / [#77](https://github.com/FelipeMorandini/stockterm/issues/77). [#18](https://github.com/FelipeMorandini/stockterm/issues/18) (429/backoff / richer `ProviderError`).

### Prior reference

Alerts loop + table layout (Issues #30 / #37 / #38): [PR #45](https://github.com/FelipeMorandini/stockterm/pull/45).

---

## 8. Next milestone — Issue #44: Stock View & Alerts keyboard modifiers

**Sources:**

- [GitHub Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) — accept `SHIFT` with letter keys, accept lowercase `a`–`z` for symbol typing and Alerts hotkeys, normalize tickers to uppercase, reject Ctrl/Alt/Meta/Super/Hyper chords.

**Related:**

- [#19](https://github.com/FelipeMorandini/stockterm/issues/19) — `default_symbol` at startup (separate).

### 8.1 Problem (verified in tree)

[`handle_stock_view_keys`](../src/app/handlers.rs) and [`handle_alerts_events`](../src/app/alerts.rs) match `KeyModifiers::NONE` for most `KeyCode::Char` arms. Many terminals report **Shift+letter** with `KeyModifiers::SHIFT` set (and sometimes an uppercase `Char`). Symbol entry only accepts `c.is_ascii_uppercase()` with `NONE`, so **lowercase** and **Shift-held** typing fail. Alerts **`a`** / **`d`** similarly ignore Shift-only and mixed case.

### 8.2 Acceptance

- **Stock View:** Watchlist actions (`w`, `x`, `j`, `k`), symbol buffer input, **Enter**, and **Backspace** behave consistently when the user types with **Shift** or **Caps Lock** (within normal terminal variance): letters append as **uppercase** ticker characters. **Hotkeys stay the lowercase letters** `w`/`x`/`j`/`k` (Issue #3 convention): uppercase `W`/`X`/`J`/`K` are **symbol input**, not shortcuts. Shifted uppercase may still carry `KeyModifiers::SHIFT`; that is allowed for the generic letter arm as long as meta keys are clear.
- **Alerts:** **`a`** (add) and **`d`** (delete selected) work with the same modifier rule and case normalization (`a`/`A`, `d`/`D`).
- **Safety:** Combinations with **Control, Alt, Meta, Hyper, or Super** (as exposed by `crossterm::event::KeyModifiers`) must **not** trigger these letter bindings or append to the symbol buffer.
- **No new crate** — logic stays in `stockterm` binary.

### 8.3 Crate & module layout

- **`src/app/handlers.rs`:** Refactor `handle_stock_view_keys` to use a shared predicate for “plain letter key” (Shift allowed, meta disallowed). Optionally move the predicate to a tiny `src/app/keyboard.rs` or `handlers` private `fn` if it is shared with alerts.
- **`src/app/alerts.rs`:** Update `handle_alerts_events` to use the same predicate and case-insensitive `Char` matching for `a`/`d`.

### 8.4 Implementation plan (Rust)

1. **Modifier predicate**  
   Define a `const` mask of disallowed modifiers, e.g.  
   `KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::META | KeyModifiers::HYPER | KeyModifiers::SUPER`  
   (verify against `crossterm 0.27` `KeyModifiers` — include every non-Shift flag that indicates a chord).  
   **`letter_key_plain(m: KeyModifiers) -> bool`:** `!m.intersects(DISALLOWED_MODIFIERS)` (and optionally document that **Shift may or may not** be set for uppercase letters depending on terminal).

2. **Watchlist / navigation keys (Stock View)** — **before** the generic letter arm  
   Match **`Char('w')`, `Char('x')`, `Char('j')`, `Char('k')`** explicitly with `letter_key_plain(modifiers)` (same behavior as today). **Do not** treat uppercase `W`/`X`/`J`/`K` as these shortcuts — they belong to the symbol buffer (preserves tickers like **WMT**, **XOM**, etc., and matches pre–#44 behavior where only uppercase was typed).  
   **Remove row:** **`x`** = `Char('x')` + plain modifiers; **`Shift+d`** = `Char(c)` where `c.eq_ignore_ascii_case('d') && modifiers.contains(KeyModifiers::SHIFT) && letter_key_plain(modifiers)` so terminals that emit `'D'` vs `'d'` both work.

3. **Symbol buffer (Stock View)**  
   **After** the hotkey arms, match:  
   `KeyCode::Char(c) if c.is_ascii_alphabetic() && letter_key_plain(modifiers)` → `app.symbol.push(c.to_ascii_uppercase())`.  
   **Edge case:** An all-lowercase ticker that **starts** with `w`, `x`, `j`, or `k` (e.g. `wmt`) cannot be entered with a leading lowercase `w`/`x`/`j`/`k` because those keys are shortcuts; use **Shift** for the first letter (**`Wmt`** → **WMT**) or type in uppercase. Document in QA.

4. **Alerts**  
   For add/remove, match `Char(c)` with `c.eq_ignore_ascii_case('a')` / `eq_ignore_ascii_case('d')` and `letter_key_plain(modifiers)`.

5. **Enter / Backspace**  
   Leave **`KeyModifiers::NONE`** (or equivalent “no meta chord”) for **Enter** and **Backspace** so `Ctrl+Enter` / `Alt+Backspace` do not trigger app actions unintentionally. If the product later wants Shift+Enter, extend in a separate issue.

6. **Async / channels**  
   No change — pure input-path refactor.

### 8.5 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- **Unit tests** (in `handlers.rs` or `keyboard.rs`): `letter_key_plain(KeyModifiers::NONE)` and `letter_key_plain(KeyModifiers::SHIFT)` are true; false when `CONTROL`, `ALT`, or `SUPER` (etc.) are set alone or combined with `SHIFT`.

### 8.6 Out of scope

- **Portfolio** tab (`handle_portfolio_events` in [`src/app/portfolio.rs`](../src/app/portfolio.rs)) — same pattern may be applied later for parity; not required by Issue #44.
- Tab switching, arrow keys, or mouse — unchanged.
- Remapping keys in `Config` (ROADMAP M6).

### 8.7 Approval

After maintainer approval of §8, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) Issue #44 section.

### 8.8 Shipment

- **Status:** Implemented; closes [Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44). Manual verification: [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #44 section).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/52
- **Code:** `src/app/keyboard.rs` (`letter_key_plain`), updates to [`src/app/handlers.rs`](../src/app/handlers.rs) and [`src/app/alerts.rs`](../src/app/alerts.rs).
- **Follow-ups:** [#48](https://github.com/FelipeMorandini/stockterm/issues/48) (Portfolio keyboard parity), [#49](https://github.com/FelipeMorandini/stockterm/issues/49) (Stock View hints), [#50](https://github.com/FelipeMorandini/stockterm/issues/50) (Alerts copy), [#51](https://github.com/FelipeMorandini/stockterm/issues/51) (global quit/tab modifiers).

---

## 9. Issue #31 — Yahoo Finance default provider (engineer migration playbook)

**Product decision (locked):** **`provider` defaults to `yahoo`**. Existing configs **without** a `provider` field deserialize as **`yahoo`** via `serde(default)` so users are **not** required to obtain a Polygon key to run the app. Polygon remains an **explicit opt-in** (`"provider": "polygon"` + API key).

**Sources:**

- [GitHub Issue #31](https://github.com/FelipeMorandini/stockterm/issues/31)
- [`docs/ROADMAP.md`](ROADMAP.md) §7 — API strategy

**Related:** [#18](https://github.com/FelipeMorandini/stockterm/issues/18) (429/backoff — follow-up), [#17](https://github.com/FelipeMorandini/stockterm/issues/17) (non-blocking UI — already landed; only swap call sites).

---

### 9.1 Problem inventory (verified in tree)

| Area | Location | Issue |
|------|----------|--------|
| HTTP | [`src/api/polygon.rs`](../src/api/polygon.rs) | `reqwest::get` — **no** connect/request timeout; errors are raw **`reqwest::Error`**. |
| Gating | [`src/app/app.rs`](../src/app/app.rs) | **`polygon_key_configured()`** blocks **`spawn_stock_fetch_task`**, **`try_spawn_historical_fetch`**, **`try_spawn_news_fetch`**, and sync **`search_symbols` / `fetch_news`** (if present) — unusable without a key. |
| Batch quotes | [`run_stock_quote_batch`](../src/app/app.rs) | Calls **`get_ticker_data`** from Polygon only. |
| Models | [`src/models/`](../src/models/) | **`TickerResponse`**, **`HistoricalResponse`**, **`SymbolSearchResponse`**, **`NewsResponse`** are **app-internal contracts**; adapters **construct** these types (they need not `Deserialize` Yahoo JSON directly into them — prefer **wire structs + mapping fns**). |

---

### 9.2 Acceptance criteria (closure checklist)

- [x] **`Config`** exposes **`provider: MarketProviderKind`** (or equivalent) with serde **`"yahoo"` \| `"polygon"`**, **`Default`** = **`Yahoo`**. Missing JSON field → Yahoo.
- [x] **Single shared `reqwest::Client`** (timeouts + User-Agent). **No** `reqwest::get` in provider code paths.
- [x] **`ProviderError`** enum + **`Display`**; HTTP non-2xx, JSON parse failures, and empty/invalid Yahoo payloads surfaced clearly on **`App.error_message`**.
- [x] **Yahoo** implements **quote**, **historical (daily)**, **symbol search**, **news** (see §9.10–9.13); maps into **existing** model types without breaking UI.
- [x] **Polygon** path preserved: same models, refactored to shared client + **`ProviderError`**; **`api_key`** required only when **`provider == Polygon`**.
- [x] **`provider_ready()`** replaces **`polygon_key_configured()`**: returns **`true`** for Yahoo always; for Polygon requires **`effective_api_key()`** non-empty.
- [x] **`cargo build --release`**, **`cargo clippy -- -D warnings`**, **`cargo test`** pass; unit tests for Yahoo mapping fixtures + error classification per §9.18.
- [x] **`docs/QA_PLAN.md`** Issue #31 manual verification (see sign-off in QA Plan).

---

### 9.3 Configuration (`src/config/config.rs`)

**New type (recommended):**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketProviderKind {
    Yahoo,
    Polygon,
}

impl Default for MarketProviderKind {
    fn default() -> Self {
        Self::Yahoo
    }
}
```

**On `Config`:**

- Add **`#[serde(default)] pub provider: MarketProviderKind`**.
- Keep **`api_key: String`** as today; document that **`effective_api_key()`** is used **only for Polygon** network calls.
- Optional doc comment: **`STOCKTERM_API_KEY`** env still overrides empty file key for Polygon users (existing behavior).

**Migration:** Users with old JSON **without** `provider` get **Yahoo** — may change behavior vs former Polygon-only workflow; acceptable per product decision above.

---

### 9.4 Dependencies (`Cargo.toml`)

- **`async-trait = "0.1"`** — if using **`dyn MarketDataProvider`** + trait objects (**recommended** for clarity and testing with mock providers later).
- **No** extra HTTP crate required; reuse **`reqwest`** with shared **`Client`**.
- Optional: **`once_cell`** only if **`std::sync::OnceLock`** is avoided for MSRV/readability — otherwise prefer **`OnceLock`** (Rust 1.70+) for the global client.

---

### 9.5 Module layout & exports

| Path | Responsibility |
|------|----------------|
| [`src/api/mod.rs`](../src/api/mod.rs) | `pub mod error; pub mod http; pub mod provider; pub mod yahoo; pub mod polygon;` + re-export **`ProviderError`**, **`market_provider_for(config)`** (name flexible). |
| `src/api/http.rs` | **`fn shared_client() -> &'static reqwest::Client`** built with **`OnceLock`**, timeouts, User-Agent. |
| `src/api/error.rs` | **`ProviderError`** + **`type ProviderResult<T>`**. |
| `src/api/provider.rs` | **`#[async_trait::async_trait] pub trait MarketDataProvider`** with four methods below; **`pub fn market_provider_for(kind: MarketProviderKind) -> Arc<dyn MarketDataProvider + Send + Sync>`** (or **`Box`** — prefer **`Arc`** if sharing across spawned tasks without cloning config-heavy state). |
| `src/api/yahoo.rs` | Wire **`Deserialize`** structs (private), **`pub async fn`** impl methods, **pure** `map_*` into `models::*`. |
| `src/api/polygon.rs` | Refactor existing URLs to use **`shared_client()`**, return **`ProviderError`**, implement **`MarketDataProvider`**. |

**Trait surface (exact signatures):**

```rust
async fn get_quote(&self, symbol: &str, config: &Config) -> ProviderResult<TickerResponse>;
async fn get_historical(&self, symbol: &str, from: &str, to: &str, timespan: &str, config: &Config) -> ProviderResult<HistoricalResponse>;
async fn search_symbols(&self, query: &str, config: &Config) -> ProviderResult<SymbolSearchResponse>;
async fn get_news(&self, symbol: &str, config: &Config) -> ProviderResult<NewsResponse>;
```

**Note:** `config` may be ignored for Yahoo (`get_quote` does not need a key) but keep the parameter for a uniform trait and future provider options.

---

### 9.6 `ProviderError` design (`src/api/error.rs`)

Define variants sufficient for debugging **and** user-visible strings:

| Variant | When |
|---------|------|
| **`Timeout`** | `reqwest::Error::is_timeout()` or equivalent |
| **`Http { status: u16, url: String }`** | `status()` after **`error_for_status()`** or manual check — **do not** dump full body in UI; optional **`body_preview: Option<String>`** truncated ≤120 chars for logs/tests only |
| **`Json`** | `serde_json::Error` / wrong schema |
| **`ApiMessage(String)`** | HTTP 200 but Yahoo/Polygon logical error, empty quote list, or `chart.error` in Yahoo payload |
| **`Transport(String)`** | Other **`reqwest::Error`** (DNS, connection reset) — **`Display`** = short message |

Implement **`impl Display for ProviderError`** with stable, copy-pastable English phrases (the TUI shows **`error_message`**).

**`From` impls:** `reqwest::Error`, `serde_json::Error` where convenient.

---

### 9.7 Shared HTTP client (`src/api/http.rs`)

**Constants (starting point):**

- **Connect timeout:** `Duration::from_secs(10)`
- **Pool idle / overall request:** use **`reqwest::ClientBuilder::timeout(Duration::from_secs(30))`** as **total per request** (covers connect + transfer).

**User-Agent (required):** set a non-empty string, e.g. **`stockterm/<crate_version> (+https://github.com/FelipeMorandini/stockterm)`** — reduces anonymous blocking.

**TLS:** keep **`rustls-tls`** feature on **`reqwest`** as today.

**Pattern:**

```rust
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn shared_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("stockterm/{} (...)", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("reqwest Client builder")
    })
}
```

Every provider **`get`** / **`post`** uses **`shared_client()`**.

---

### 9.8 Yahoo Finance — general rules

**Hosts:** Primary **`https://query1.finance.yahoo.com`**. Some secondary routes use **`query2.finance.yahoo.com`** (e.g. news). **Verify URLs with `curl` during implementation** — unofficial endpoints change.

**Symbol encoding:** Path segments must be **URL-encoded** (e.g. **`BRK-B`** → **`BRK%2FB`** depending on Yahoo symbol format — use Yahoo’s convention: often **`BRK-B`** in path; **test two tickers with `-` and `.`**).

**Timestamps:** Yahoo **chart** endpoints use Unix seconds in **`timestamp`** arrays. Internal **`TickerResult.t`** / **`HistoricalData.t`** are **`u64`** and used with **`latest_result()`** by **max timestamp** — Polygon uses **milliseconds**. **Standardize on milliseconds** in mapped output: **`t_yahoo_secs * 1000`**.

**Null bars:** Chart arrays may contain **`null`** in OHLCV — **skip** indices where **`close`** is null or pair-wise invalid.

---

### 9.9 Yahoo — quotes / watchlist (`get_quote` → `TickerResponse`)

**Endpoint:**

`GET https://query1.finance.yahoo.com/v7/finance/quote?symbols={SYMBOL}`

For multiple symbols in one HTTP request (optimization): comma-separated, URL-encoded list — see §9.16.

**Wire JSON (conceptual):** root **`quoteResponse.result`** = array of quote objects; **`quoteResponse.error`** may exist.

**Mapping into [`TickerResponse`](../src/models/ticker.rs) / [`TickerResult`](../src/models/ticker.rs):**

Build **`results: vec![TickerResult { ... }]`** with **one row** representing the **latest regular session snapshot** (sufficient for Stock View “Last” and **`latest_result()`**):

| `TickerResult` field | Yahoo source (typical field names) | Notes |
|----------------------|-----------------------------------|--------|
| **`o`** | `regularMarketOpen` | If missing, fallback **`regularMarketPreviousClose`** or **`postMarketPrice`** — document chosen precedence in code comment |
| **`h`** | `regularMarketDayHigh` | |
| **`l`** | `regularMarketDayLow` | |
| **`c`** | `regularMarketPrice` | Primary “last” |
| **`v`** | `regularMarketVolume` | Default **`0.0`** if null |
| **`t`** | `regularMarketTime` | Unix **seconds** → **multiply by 1000** |

Set **`TickerResponse.ticker`** from Yahoo **`symbol`** string (fallback: requested symbol uppercase). **`status`** = **`"OK"`**; **`error`** = **`None`** on success.

**Empty result:** If **`result`** empty or symbol unknown → **`ProviderError::ApiMessage`** with text like **`Unknown symbol: AAPL`** (use requested symbol in message).

---

### 9.10 Yahoo — historical / Charts (`get_historical` → `HistoricalResponse`)

**Endpoint:**

`GET https://query1.finance.yahoo.com/v8/finance/chart/{SYMBOL}?period1={START_UNIX}&period2={END_UNIX}&interval={INTERVAL}`

**Parameters:**

- **`period1` / `period2`**: Unix **seconds** (inclusive/exclusive semantics per Yahoo — align **period2** to **end-of-day** for daily range).
- **`interval`**: For **`timespan == "day"`** (only case required for parity with current app): **`1d`**.

**Date inputs:** Call sites today pass **`from_date`**, **`to_date`** as **`YYYY-MM-DD`** strings via **`try_spawn_historical_fetch`** ([`src/app/app.rs`](../src/app/app.rs)). Parse with **`chrono::NaiveDate`**, convert to UTC midnight timestamps **consistently** (document: use **UTC** boundary **or** US market calendar — pick **UTC midnight** for simplicity; note intraday drift in comments).

**Wire JSON (conceptual):** **`chart.result[0]`** contains **`timestamp`** (Vec of seconds), **`indicators.quote[0]`** with parallel arrays **`open`**, **`high`**, **`low`**, **`close`**, **`volume`**. Handle **`chart.error`**.

**Mapping into [`HistoricalResponse`](../src/models/historical.rs) / [`HistoricalData`](../src/models/historical.rs):**

| Field | Source |
|-------|--------|
| **`HistoricalResponse.ticker`** | `chart.result[0].meta.symbol` or requested symbol |
| **`HistoricalResponse.status`** | **`"OK"`** if successful |
| **`HistoricalResponse.request_id`** | **`""`** |
| **`HistoricalResponse.count`** | number of valid bars |
| **`HistoricalData.o/h/l/c/v`** | aligned arrays index **`i`** |
| **`HistoricalData.t`** | **`timestamp[i] * 1000`** |
| **`HistoricalData.vw`** | use **`close`** as VWAP proxy **or** **`(o+h+l+c)/4`** — document (Polygon supplies VWAP; Yahoo chart includes separate adjclose — optional improvement) |
| **`HistoricalData.n`** | **`None`** |

**Order:** Preserve **chronological order** ascending (charts may assume order — match existing Polygon ordering if any code depends on it).

---

### 9.11 Yahoo — symbol search (`search_symbols` → `SymbolSearchResponse`)

**Endpoint:**

`GET https://query1.finance.yahoo.com/v1/finance/search?q={QUERY}&quotesCount=10`

**Wire:** **`quotes`** array (and optionally **`news`**, **`mutualfunds`** — ignore for MVP).

**Mapping into [`SymbolSearchResponse`](../src/models/search.rs):**

- **`status`**: **`"OK"`**
- **`count`**: **`quotes.len()` as u32**
- For each Yahoo quote row, build **`SymbolResult`**:

| `SymbolResult` | Yahoo / fallback |
|----------------|------------------|
| **`ticker`** | `symbol` |
| **`name`** | `shortname` **or** `longname` |
| **`market`** | `exchDisp` **or** `exchange` **or** `""` |
| **`locale`** | **`"us"`** if absent |
| **`primary_exchange`** | `exchDisp` **or** `""` |
| **`type_`** | `quoteType` **or** `typeDisp` **or** **`"EQUITY"`** |
| **`active`** | **`true`** |
| **`currency_name`** | `currency` **or** **`"USD"`** |
| **`cik`**, **`composite_figi`**, **`share_class_figi`** | **`None`** |
| **`last_updated_utc`** | **`""`** |

---

### 9.12 Yahoo — news (`get_news` → `NewsResponse`)

**Goal:** Populate [`NewsResponse`](../src/models/news.rs) / [`NewsItem`](../src/models/news.rs) without Polygon.

**Approach (implementation order):** `query2` **`/v2/finance/news`** often returns **HTTP 500**; the provider therefore tries, in order:

1. **`GET https://query1.finance.yahoo.com/v1/finance/search?q={SYMBOL}&newsCount=20&quotesCount=0`** — JSON **`news`** array (`title`, `publisher`, `link`, `providerPublishTime`).
2. **RSS:** `GET https://feeds.finance.yahoo.com/rss/2.0/headline?s={SYMBOL}&region=US&lang=en-US` — parse `<item>` / `<title>` / `<link>` / `<pubDate>`.
3. **Legacy:** `GET https://query2.finance.yahoo.com/v2/finance/news?symbols={SYMBOL}` — existing stream JSON mapper.

If endpoints shift: fix parsers + fixtures; on HTTP success with empty content, **`Ok`** with zero results is acceptable per empty-news UX — still surface **`ProviderError`** on hard HTTP/parse failure after all attempts.

**Mapping highlights:**

- **`NewsItem.id`**: hash URL or use Yahoo id if present.
- **`publisher`**: map nested **`name`**, **`homepage_url`**, **`logo_url`**, **`favicon_url`** — use **`""`** for unknown URLs.
- **`published_utc`**: RFC3339 string from Yahoo field **`providerPublishTime`** / **`pubDate`** / equivalent — normalize to **ISO-8601** string as today’s UI expects.

---

### 9.13 Polygon adapter refactor (`src/api/polygon.rs`)

- Replace **`reqwest::get`** with **`shared_client().get(url)`** + **`.send().await`** + **`error_for_status()`**.
- Map **`reqwest::Error`** → **`ProviderError`**.
- Deserialize JSON as today, then if **`TickerResponse.api_error_message()`** returns **`Some`**, convert to **`ProviderError::ApiMessage`** **or** keep legacy behavior by letting **`App`** layers handle **`TickerResponse`** errors — **preferred:** return **`Ok(TickerResponse)`** only when logically OK; otherwise **`Err(ApiMessage(...))`** for consistency.
- Implement **`MarketDataProvider`** for **`struct PolygonProvider`** (zero-sized or holds nothing).

---

### 9.14 Application wiring (`src/app/app.rs`) — mechanical checklist

**Imports:** Remove direct **`crate::api::polygon::*`**. Import **`market_provider_for`** (or equivalent) + **`MarketProviderKind`**.

**`run_stock_quote_batch`:**

- Accept **`MarketProviderKind`** or **`Arc<dyn MarketDataProvider>`** — simplest: **`clone `** `config` already has **`provider`**; inside batch, **`let p = market_provider_for(cfg.provider);`** then **`p.get_quote(&sym, &cfg).await`**.
- Map **`Err(e)`** → **`errors.push(format!("{sym}: {e}"))`** (same as today).

**`spawn_stock_fetch_task` (~L259):**

- Replace **`if !self.polygon_key_configured()`** with **`if !self.provider_ready()`** where **`provider_ready`** is **`false`** only for **Polygon + empty key**.
- For **Yahoo**, **never** short-circuit with “missing API key”.

**`try_spawn_historical_fetch`**, **`try_spawn_news_fetch`**, **`search_symbols`**, **`fetch_news`:**

- Same gating: **`provider_ready()`** instead of Polygon-only.
- Replace **`get_historical_data` / `get_news` / `search_symbols`** calls with **`market_provider_for(self.config.provider)`** trait methods.
- Spawns already pass **`Config`** — ensure **`provider`** is included in **`clone`**.

**Constants / messages:**

- Rename **`MISSING_POLYGON_KEY_MSG`** → e.g. **`MISSING_API_KEY_FOR_POLYGON_MSG`** and show **only** when **`provider == Polygon`** and key missing.

**`lib.rs`:** Re-export nothing new unless tests need it.

---

### 9.15 Optional performance — batched Yahoo quotes

Yahoo **`v7/finance/quote`** accepts **comma-separated `symbols`**. Current **`JoinSet`** issues **N** requests — acceptable for MVP.

**Follow-up (not blocking #31):** If **`provider == Yahoo`**, collapse **`collect_symbols_for_quote_fetch()`** into **one** HTTP call, parse multiple quotes, fill **`HashMap`**. Keep **`JoinSet`** path for Polygon or mixed future providers.

---

### 9.16 Edge cases & QA hints

- **International tickers:** Yahoo suffix conventions (**`7203.T`**, **`SAP.DE`**) — user types symbol as today; **do not** second-guess beyond encoding.
- **`TickerResponse.latest_result()`** assumes **`t`** comparable — ms timestamps required.
- **Charts empty:** If no bars (delisted window) → **`ApiMessage`** or **`Ok`** with empty **`results`** — pick one and ensure **`draw_charts`** doesn’t panic (existing code paths).
- **Rate limits:** Yahoo may throttle abusive IPs; respectful **`refresh_rate`** still matters.

---

### 9.17 Implementation phases (recommended order)

1. **`http.rs` + `error.rs`** — shared client + **`ProviderError`**.
2. **`Config` + `MarketProviderKind`** — default Yahoo; **`serde`** round-trip test / manual JSON sample.
3. **`provider` trait + `PolygonProvider`** wrapping old logic — prove parity with **`cargo test`** / manual Polygon still works.
4. **`yahoo.rs`** — **`get_quote`** + fixtures → **`TickerResponse`**; wire **`run_stock_quote_batch`** + **`spawn_stock_fetch_task`** gating.
5. **`get_historical`** (chart) + Charts tab smoke.
6. **`search_symbols`** + Search tab.
7. **`get_news`** + News tab.
8. Cleanup strings, clippy, **`docs/QA_PLAN.md`** run.

---

### 9.18 Automated testing expectations

- **Fixture tests** (stored `&str` JSON snippets in `yahoo.rs` **`#[cfg(test)]`**): quote mapping, chart mapping (include **null** volume row), search mapping.
- **`ProviderError::Display`** smoke test.
- **Optional:** **`wiremock`** integration test — out of scope for #31 unless quick — prefer fixtures first.

---

### 9.19 Out of scope

- Exponential backoff / 429 ([#18](https://github.com/FelipeMorandini/stockterm/issues/18)).
- Settings UI for provider.
- New providers beyond Yahoo + Polygon.
- Intraday intervals and multi-range charts — **Issue #31** shipped daily-only Yahoo history; intraday + **1D/1W/1M/1Y** switching is specified in **§11** (Issues [#9](https://github.com/FelipeMorandini/stockterm/issues/9) / [#8](https://github.com/FelipeMorandini/stockterm/issues/8) / [#7](https://github.com/FelipeMorandini/stockterm/issues/7)).

---

### 9.20 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) Issue #31; PR [#57](https://github.com/FelipeMorandini/stockterm/pull/57).
- **Issue:** https://github.com/FelipeMorandini/stockterm/issues/31
- **Dependencies:** `async-trait` **0.1.89** (see `Cargo.lock`).
- **Code:** `src/api/{http,error,provider,yahoo}.rs`, refactored [`src/api/polygon.rs`](../src/api/polygon.rs); [`src/config/config.rs`](../src/config/config.rs) `MarketProviderKind`; [`src/app/app.rs`](../src/app/app.rs) `provider_ready` / `market_provider_for`; fixtures under [`tests/fixtures/`](../tests/fixtures/).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/57

---

## 10. M3 — Search, News, Settings tabs (Issues #29, #5, #11, #12)

**Umbrella:** [Issue #29](https://github.com/FelipeMorandini/stockterm/issues/29) — replace stub panes with real UIs and tab-local key handling.

**Child issues (acceptance detail):**

- [Issue #5](https://github.com/FelipeMorandini/stockterm/issues/5) — Search: typeahead, debounce, list navigation, Enter → Stock View.
- [Issue #11](https://github.com/FelipeMorandini/stockterm/issues/11) — News: scrollable headlines, loading/empty states, Enter → open URL or copy (best-effort).
- [Issue #12](https://github.com/FelipeMorandini/stockterm/issues/12) — Settings: edit `refresh_rate` / `default_symbol`, placeholders for theme/keymap, persist via `Config::try_save`.

**Related:** [#17](https://github.com/FelipeMorandini/stockterm/issues/17) — search/news/settings fetches must stay off the draw/input hot path (extend existing `FetchDone` + `tokio::spawn` pattern). [#19](https://github.com/FelipeMorandini/stockterm/issues/19) — surface `try_save` failures on `App.error_message`. Keyboard parity: reuse [`letter_key_plain`](../src/app/keyboard.rs) where letter keys must not fire under Ctrl/Alt/Meta chords.

**Verified baseline (tree):**

| Area | Location | State |
|------|----------|--------|
| Search UI | [`src/app/ui.rs`](../src/app/ui.rs) `draw_search` | Empty stub. |
| News UI | `draw_news` | Empty stub. |
| Settings UI | `draw_settings` | Empty stub. |
| Search API | [`FetchDone::Search`](../src/app/app.rs) + `spawn_search_task` | Debounced tick on `Tab::Search`; stale guard on generation + query string. |
| News fetch | `try_spawn_news_fetch`, `FetchDone::News` | Background fetch on `Tab::News` only; data never rendered. |
| State | `App` | `search_query`, `search_results`, `news_data`, `news_refresh_inflight` exist; `selected_index` is **unused** — replace or repurpose for list selection. |

---

### 10.1 Crate & module layout

- **Single package** `stockterm`; no new crate unless clipboard/open requires a tiny helper crate (prefer **no** new dependency: shell out to `open` / `xdg-open` / `cmd.exe /c start` for URLs).
- **`src/app/ui.rs`:** Implement `draw_search`, `draw_news`, `draw_settings` (layout: `Block`, `Paragraph`, `Table` or `List`, `Layout`, consistent with Stock/Portfolio panes).
- **`src/app/handlers.rs`:** Dispatch `Tab::Search`, `Tab::News`, `Tab::Settings` to new `handle_search_events`, `handle_news_events`, `handle_settings_events` (mirror `handle_portfolio_events` style).
- **Optional file split:** If `handlers.rs` grows, add `src/app/search_tab.rs`, `news_tab.rs`, `settings_tab.rs` exporting only the `handle_*` + small helpers — optional; keep diff focused.
- **`src/app/app.rs`:**
  - Extend **`FetchDone`** with **`Search { generation: u64, query: String, result: Result<SymbolSearchResponse, String> }`** (or `Err` maps to same string pattern as `News`).
  - Add search-specific fields: e.g. **`search_list_state: ratatui::widgets::ListState`**, **`search_request_generation: u64`**, **`search_refresh_inflight: bool`**, **`search_debounce_deadline: Option<Instant>`** (or a single **`search_pending_query: Option<String>`** + deadline).
  - Add **`news_list_state: ListState`** for News selection (do **not** overload `watchlist_state`).
  - Settings: **`settings_row: usize`**, **`settings_editing: Option<SettingsEdit>`** enum (`RefreshRate`, `DefaultSymbol`) with **`edit_buffer: String`**, optional **`settings_saved_flash_until: Option<Instant>`** for a short “Saved” hint.
- **`src/config/`:** No schema change required for MVP beyond existing `refresh_rate`, `default_symbol`, `theme: Option<Theme>`, `provider`. Settings screen may show **`provider`** as **read-only** text (editing provider belongs to a later issue unless explicitly extended).

---

### 10.2 Search tab (Issue #5) — behavior & async

**UI:**

- Top: single-line **query** bound to `App.search_query` (prefix with label e.g. `Query:`).
- Below: **results table** from `search_results` — columns **Symbol | Name | Type | Exchange** (map `SymbolResult`: `ticker`, `name`, `type_`, `primary_exchange` or `market`).
- Footer/status: **`Searching…`** when `search_refresh_inflight`; **`No results`** when response is success with empty `results`; provider error on `error_message` line.

**Keys (Search tab only):**

- Printable ASCII that belongs in company/ticker search: **letters, digits, space, `-`, `.`** — append to `search_query` when `letter_key_plain` allows, with an explicit arm for **digits and punctuation** that still requires **no** Ctrl/Alt/Meta (same safety as Stock View).
- **Backspace** — pop char (modifiers: **NONE** only for Backspace/Enter/Esc, matching Issue #44 §8.5).
- **Esc** — clear `search_query`, clear results, reset list selection, cancel pending debounced request (bump generation so stale responses drop).
- **Enter** — if results non-empty, take **highlighted** row’s ticker: `normalize_symbol`, set `app.symbol`, set `active_tab = Tab::StockView`, clear or keep query per UX (recommend **keep** query for repeat searches), call **`request_immediate_stock_poll()`** (same as Stock View Enter path).
- **Up/Down** or **j/k** (with `letter_key_plain` for `j`/`k`) — move `search_list_state` selection within bounds.

**Debounce & concurrency:**

- **Debounce interval:** **250 ms** from last mutation to `search_query` (character add/remove/clear).
- On each qualifying tick (`Event::Tick`) while `active_tab == Tab::Search`, if deadline elapsed and query non-empty and `provider_ready()`:
  - If `search_refresh_inflight`, **do not** stack another request; optionally set a **“pending retry”** flag when the in-flight query ≠ current query (when current completes, if query changed, schedule again).
  - Else increment **`search_request_generation`**, spawn **`tokio::spawn`** that calls `provider.search_symbols(&query, &cfg).await`, send **`FetchDone::Search { generation, query, result }`**.
- **`apply_fetch_done`:** For `Search`, clear `search_refresh_inflight`. Apply result **only if** `generation == search_request_generation` **and** `query == search_query` (stale guard). On success, replace `search_results`; clamp `search_list_state` selection; on error, set `error_message` and clear or keep last results (recommend **clear** results on error to avoid misleading rows).

**Empty query:** Do not call API; set `search_results = None` and show hint text.

**Polygon gate:** If `!provider_ready()`, mirror existing `MISSING_API_KEY_FOR_POLYGON_MSG` on `error_message` and skip spawn.

---

### 10.3 News tab (Issue #11) — behavior

**UI:**

- **`List`** (or table) of items from `news_data.results`: **publisher name** (truncate), **title** (truncate with ellipsis), **published_utc** (short form), optional **URL** column or footer line for selection.
- **`news_list_state`** for highlight.
- While **`news_refresh_inflight`:** show **Loading…** (reuse pattern from Stock refresh if any).
- **Empty:** `news_data` present with `results.is_empty()` or count 0 → **No news available** message; distinguish from “not yet loaded”.

**Keys:**

- **Up/Down**, **j/k** — navigate list (`letter_key_plain` for `j`/`k`).
- **Enter** — **best-effort** open article:
  - **macOS:** `Command::new("open").arg(url)`  
  - **Windows:** `cmd /C start "" <url>` (or `start` pattern that avoids injection — use single arg).  
  - **Unix (non-mac):** `xdg-open <url>` if desired, else skip.  
  - If spawn fails, set a short `error_message` (“Could not open URL”).  
  - **Optional:** If open fails or user prefers copy, try clipboard via **`pbcopy`** / **`wl-copy`** / **`xclip -selection clipboard`** when `which` succeeds — document in QA as platform-dependent; **not** required for closure if open works on primary dev OS.

**Refresh semantics:**

- Keep **`try_spawn_news_fetch`** on **`Tab::News`** tick with existing throttle (`data_poll_interval`).
- **When `symbol` changes** while user is on News (e.g. after returning from Search): stale responses are already ignored in `apply_fetch_done` by symbol match — additionally **reset `news_list_state`**, and either **clear `news_data`** until next fetch or **force** immediate news poll when `symbol` changes and `active_tab == News` (recommend **clear + reset `last_news_network_poll` to None** for instant refresh on next tick, or call a small `request_immediate_news_poll` helper).

---

### 10.4 Settings tab (Issue #12) — behavior

**UI:**

- Menu of rows (numbered or plain list): **Refresh interval (seconds)**, **Default symbol**, **Theme** (show JSON-ish summary or “Not configured” / accent keys from [`Theme`](../src/config/theme.rs)), **Provider** (read-only: `yahoo` / `polygon`), **Keymap** (placeholder: “Coming later” / issue reference).
- **Enter** on editable row enters **edit mode** (`settings_editing`). In edit mode, typing fills **`edit_buffer`**; **Enter** commits, **Esc** cancels edit.
- **Refresh rate editor:** numeric only; validate **integer ≥ 1** (document interaction with existing **`data_poll_interval`** minimum of **5** seconds in [`App::data_poll_interval`](../src/app/app.rs) — UI may allow typing `3` but effective poll remains 5; show inline note “Minimum effective: 5s” or clamp on commit with message).
- **Default symbol:** `normalize_symbol` on commit; reject empty after trim with inline error.
- **Persist:** On successful commit, assign `self.config.refresh_rate` / `self.config.default_symbol`, call **`Config::try_save()`**; on `Err`, set **`error_message`** (Issue #19). On success, set **`settings_saved_flash_until = now + 2s`** (tunable).
- **Live default symbol:** Changing `default_symbol` updates config only; **current session** `symbol` unchanged until next app launch — matches Issue #12 acceptance (“on next launch, `App::new` uses it”). Optionally document in QA.

**Theme row:** No editing in this milestone — display only (Issue #12 checklist).

---

### 10.5 Keyboard & global keys

- **Tab / Shift+Tab** — already switch tabs; ensure Search/News/Settings do not consume these.
- **`q`** — global quit unchanged (**NONE** only).
- Reuse **`letter_key_plain`** for Search/News letter keys consistent with Issues #3/#44.

---

### 10.6 Out of scope

- Editing **`provider`** or **`api_key`** in Settings (security / validation — separate issue).
- Full **theme** application across widgets ([#14](https://github.com/FelipeMorandini/stockterm/issues/14) or roadmap M6).
- Custom **keymap** editing ([#13](https://github.com/FelipeMorandini/stockterm/issues/13)).
- Watchlist management from Settings (Issue #3 / `w` only).
- Changing Yahoo batch quote N→1 ([#53](https://github.com/FelipeMorandini/stockterm/issues/53)).

---

### 10.7 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- `cargo test`
- **Unit tests (recommended):** stale-search generation helper (pure fn), optional `normalize_symbol` / settings validation if extracted.

---

### 10.8 Approval

After maintainer approval of §10, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (M3 / Issues #29, #5, #11, #12 section).

### 10.9 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (M3 sign-off, 2026-05-10).
- **Tracking:** [Issue #29](https://github.com/FelipeMorandini/stockterm/issues/29), [#5](https://github.com/FelipeMorandini/stockterm/issues/5), [#11](https://github.com/FelipeMorandini/stockterm/issues/11), [#12](https://github.com/FelipeMorandini/stockterm/issues/12).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/61
- **Code:** `src/app/{app,handlers,ui,open_url}.rs`; `FetchDone::Search`; Settings via `Config::try_save`; Yahoo `get_news` uses `query1` search + RSS before `query2` (`src/api/yahoo.rs`).
- **Follow-up issues:** [#58](https://github.com/FelipeMorandini/stockterm/issues/58) (clipboard), [#59](https://github.com/FelipeMorandini/stockterm/issues/59) (non-blocking open), [#60](https://github.com/FelipeMorandini/stockterm/issues/60) (Search Esc vs global error).

---

## 11. M4 — Charts: time ranges (#9), viewport (#8), candlesticks (#7)

**Tracking (GitHub):**

- [Issue #9](https://github.com/FelipeMorandini/stockterm/issues/9) — `TimeRange` (1D / 1W / 1M / 1Y), provider window + bar granularity, Charts tab keys `1`–`4`, title/status reflects range.
- [Issue #8](https://github.com/FelipeMorandini/stockterm/issues/8) — `ChartViewport` indices, zoom `+`/`-`, pan `h`/`l` (and/or arrows), reset `0`, y-axis from visible window, visible date range in UI.
- [Issue #7](https://github.com/FelipeMorandini/stockterm/issues/7) — Custom `ratatui` candlestick `Widget`, green/red bodies + wicks, toggle vs line (`c`), remove or demote text-table `draw_candlestick`.

**Related:** [#17](https://github.com/FelipeMorandini/stockterm/issues/17) — historical fetch stays on `tokio::spawn` + `FetchDone::Historical` (no change to hot-path blocking). [#18](https://github.com/FelipeMorandini/stockterm/issues/18) — intraday may increase request volume; respect `refresh_rate` / provider limits.

**Verified baseline (tree):**

| Area | Location | State |
|------|----------|--------|
| Historical window | [`try_spawn_historical_fetch`](../src/app/app.rs) | Hard-coded **30 days**, **`"day"`** only (pre–`TimeRange`; superseded by §11). |
| Yahoo history | [`YahooProvider::get_historical`](../src/api/yahoo.rs) | Rejects **`timespan != "day"`**; URL uses **`interval=1d`** only. |
| Polygon history | [`PolygonProvider::get_historical`](../src/api/polygon.rs) | **`range/1/{timespan}/`** — supports Polygon **`minute` / `hour` / `day`** (etc.) per API; today call site always passes **`"day"`**. |
| Charts keys | [`handlers.rs`](../src/app/handlers.rs) `Tab::Charts` | **No** tab-local handler — must add `handle_charts_events`. |
| Line chart | [`draw_charts`](../src/app/charts.rs) | Full-series min/max x/y; no viewport. |
| Candlestick | [`draw_candlestick`](../src/app/charts.rs) | OHLC **text table**; unused from [`ui.rs`](../src/app/ui.rs). |

---

### 11.1 Recommended delivery order

1. **#9 (data contract)** — Introduce `TimeRange`, map to `(from, to, bar_resolution)` per provider, extend **`get_historical`** (or add a parallel method) so Yahoo can request **`interval=1m`** / **`5m`** / **`1d`** / **`1wk`** via v8 chart. Wire **`try_spawn_historical_fetch`** to use `App.time_range`. Add Charts tab range keys and on-range-change **invalidate / refit** viewport (step 2).
2. **#8 (viewport)** — Add `ChartViewport`, slice `historical_data.results` for drawing, key bindings, dynamic y-bounds, visible-range label. Works for **line** mode first; candlestick reuses the same slice.
3. **#7 (rendering)** — Implement `CandlestickChart` widget consuming the **viewport-sliced** `&[HistoricalData]`, wire **`c`** toggle, delete or gate the old text-table helper.

This order avoids building a candlestick widget twice (full series vs windowed).

---

### 11.2 Crate & module layout

- **`src/models/time_range.rs`** (or `src/app/chart_state.rs` if you prefer app-only):  
  `#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)] pub enum TimeRange { D1, W1, M1, Y1 }` with **`Default = M1`** (parity with today’s ~30-day daily habit). Optionally reserve variants **`M3`, `M6`, `Ytd`, `Y5`** behind `#[non_exhaustive]` for growth without breaking match exhaustiveness in `non_exhaustive` style — only **`D1`–`Y1`** required for closure.
- **`src/app/app.rs`:** `time_range: TimeRange`, `chart_viewport: ChartViewport`, `chart_mode: ChartDisplayMode` (`Line` | `Candlestick`). On successful `FetchDone::Historical`, **reset viewport** to full range (`0..results.len()`); on **`time_range` change** before fetch completes, clear or keep stale data per existing Historical stale-guard pattern.
- **`src/app/charts.rs`:** `draw_charts` takes **`&ChartViewport`**, **`ChartDisplayMode`**, **`TimeRange`** (for title), slices data, dispatches to line `Chart` or candlestick widget. Extract **`visible_slice(results, viewport) -> &[HistoricalData]`** (empty-safe).
- **`src/app/handlers.rs`:** `handle_charts_events` — range keys, viewport keys, mode toggle; use **`letter_key_plain`** for **`h`/`l`/`c`** where applicable; **`+`/`-`/`0`/`1`–`4`** typically **`KeyModifiers::NONE`** only (avoid `Ctrl++` collisions — document).
- **`src/api/provider.rs`:** Extend historical API so providers receive enough to fetch intraday + daily windows. **Recommended shape:**

```rust
/// Bar size for chart history (Yahoo `interval` string; Polygon multiplier+timespan derived in adapter).
pub struct HistoricalQuery<'a> {
    pub from: &'a str, // YYYY-MM-DD and/or document when intraday uses same-day bounds
    pub to: &'a str,
    pub bar_interval: &'a str, // e.g. "1m", "5m", "1d", "1wk" — provider maps
}
```

Replace the loose **`timespan: &str`** argument in **`get_historical`** with **`HistoricalQuery`** **or** add an overload `get_historical_v2` and migrate call sites in one PR — pick one to avoid dual paths. This SPEC assumes a **single** trait method taking **`HistoricalQuery`** (or equivalent **`interval: &str`** + date pair) after refactor.

- **`src/api/yahoo.rs`:** Remove the **`timespan != "day"`** guard; build chart URL with **`interval={bar_interval}`** from query; keep **`period1`/`period2`** as Unix seconds (extend helpers for “start of session” vs calendar midnight where needed for **D1**).
- **`src/api/polygon.rs`:** Map **`HistoricalQuery.bar_interval`** to Polygon **`multiplier` + `timespan`** (`minute`/`hour`/`day`/`week`) per [Polygon aggregates docs](https://polygon.io/docs/stocks/get_v2_aggs_ticker__stocksticker__range__multiplier___timespan___from___to); validate free-tier limits in comments.

---

### 11.3 TimeRange → provider mapping (#9)

**Goal:** Keys **`1`/`2`/`3`/`4`** set **`D1` / `W1` / `M1` / `Y1`** respectively. Show active range in chart **block title** or **status** line (e.g. **`M1 · daily · 2026-04-10 → 2026-05-10`**).

**Suggested mapping (tune during implementation; document final table in code comments):**

| `TimeRange` | Calendar window (anchor: local `now`) | Yahoo `interval` (v8 chart) | Notes |
|-------------|--------------------------------------|-----------------------------|--------|
| **D1** | Current session window: `period1` ≈ start of **current trading day** (US **Eastern** recommended for US equities) through `period2` = now | **`1m`** or **`5m`** | Yahoo may cap intraday points; clamp or subsample if payload huge. Acceptance: **intraday bars** visible. |
| **W1** | ~7 calendar days ending today | **`30m`** or **`1h`** | Coarser bars reduce noise; if empty, fall back to **`1d`** for the same window. |
| **M1** | ~30 calendar days (match old behavior) | **`1d`** | Parity with pre–M4 default. |
| **Y1** | ~365 calendar days | **`1d`** or **`1wk`** | **`1wk`** reduces point count for line/candles; pick one and keep axis labels honest. |

**Polygon:** For each row, choose **`multiplier`** and **`timespan`** to approximate the same bar count (e.g. D1 → `1`/`minute` or `5`/`minute` over ISO date range). **Empty / illiquid** responses: return **`Ok`** with empty **`results`** where appropriate; UI shows existing “No historical data” copy — **no panic**.

**Stale fetch:** If `FetchDone::Historical` arrives after **`symbol`** or **`time_range`** changed, drop result (mirror **`FetchDone::News`** / **`Search`** generation pattern) — add **`hist_request_epoch`** or compare **`(symbol, time_range)`** tuple in **`apply_fetch_done`**.

---

### 11.4 ChartViewport (#8)

**State:**

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct ChartViewport {
    /// Inclusive start index into the **sorted** `historical_data.results` vector.
    pub start: usize,
    /// Exclusive end index (Rust half-open range: `start..end`).
    pub end: usize,
}
```

- **Invariant:** `start < end` when `results.len() >= 2`; if `results.len() <= 1`, viewport equals `0..len` or full range; drawing shows message for fewer than 2 points when zoom/pan is meaningless.
- **`+` zoom:** Shrink window around **center** of current `start..end` (e.g. new width = max(2, (end-start)/2)); clamp to `0..len`.
- **`-` zoom:** Grow window symmetrically; cap at full `0..len`.
- **`h` / `l`** (and optionally **Left/Right**): shift window by **one bar** (or **N** bars); clamp at dataset edges — **no wrap**, **no panic**.
- **`0` reset:** `start = 0`, `end = results.len()` after each successful load and when user presses **`0`**.
- **Y-axis:** **`min`/`max`** price computed **only** from visible OHLC (use **low**/**high** per bar, not close-only) with small padding (reuse ~10% padding from current `draw_charts`).
- **X-axis labels:** Derive from **first/last/mid** visible bar timestamps (format adapts to intraday vs daily).
- **Title / status:** Append visible date range from first/last visible bar (timezone: **UTC** or **local** — pick one, document in QA).

**On `time_range` change (#9):** After user presses **`1`–`4`**, set `time_range`, bump stale token, **reset viewport** to full range (or `0..0` until data arrives), clear `last_charts_network_poll` / force refresh so new range fetches immediately (same pattern as “immediate poll” helpers elsewhere).

---

### 11.5 Candlestick widget (#7)

- Implement **`struct CandlestickChart<'a>`** implementing **`Widget`** (or **`StatefulWidget`** if selection is needed later). Input: **draw `Rect`**, **visible `&[HistoricalData]`**, **x as bar index 0..n-1** mapped to pixel columns (or Braille blocks), **y** from price scale (viewport y-bounds).
- **Body:** vertical segment from **`open` → `close`** (thick column or two cells); **wick:** **`low` → `high`** (thin). **Green** if **`close >= open`**, **red** otherwise (reuse `Color::Green` / `Color::Red` or theme later).
- **Toggle:** **`c`** cycles **`Line` ↔ `Candlestick`**; persist only in-memory unless a follow-up adds `Config` (out of scope).
- **Line chart polish (optional in same PR):** Improve axis labels when viewport is active; ensure line dataset uses **same** slice as candles.
- **Remove** unused import of **`draw_candlestick`** from **`ui.rs`** or replace call path so dead code is eliminated.

---

### 11.6 Keyboard summary (Charts tab only)

| Key | Action |
|-----|--------|
| `1`–`4` | Set **`TimeRange`** **D1** / **W1** / **M1** / **Y1**; trigger refetch + viewport reset. |
| `+` / `-` | Zoom in / out. |
| `h` / `l` | Pan left / right ( **`letter_key_plain`** ). |
| `0` | Full range. |
| `c` | Toggle line / candlestick. |
| Arrows | Optional alias for pan (recommended for accessibility). |

**Global:** **`Tab` / Shift+Tab**, **`q`** unchanged. Do not bind **`1`–`4`** on other tabs (Charts-only dispatch).

---

### 11.7 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- `cargo test`
- **Unit tests:** `visible_slice` / viewport clamping (pure fn); `TimeRange` → `HistoricalQuery` mapping (table-driven); optional Yahoo URL builder test with fixed clock (if injectable).

---

### 11.8 Out of scope

- Persisting **`time_range`** / **`chart_mode`** in **`~/.stockterm.json`** (follow-up).
- Touch/mouse drag on chart.
- Volume histogram pane, MACD, indicators.
- Changing **`MarketDataProvider`** trait without migrating both Yahoo and Polygon in the same change (avoid Yahoo-only intraday).

---

### 11.9 Approval

After maintainer approval of §11, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #7, #8, #9 section).

### 11.10 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (M4 / Issues #7, #8, #9); closes [#7](https://github.com/FelipeMorandini/stockterm/issues/7), [#8](https://github.com/FelipeMorandini/stockterm/issues/8), [#9](https://github.com/FelipeMorandini/stockterm/issues/9).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/66
- **Code:** `src/models/time_range.rs`, `src/api/historical_query.rs`, `src/api/{yahoo,polygon,provider}.rs`, `src/app/{app,charts,handlers}.rs`.
- **Follow-ups:** [#62](https://github.com/FelipeMorandini/stockterm/issues/62), [#63](https://github.com/FelipeMorandini/stockterm/issues/63), [#64](https://github.com/FelipeMorandini/stockterm/issues/64) — specified in **§11.11**. [#65](https://github.com/FelipeMorandini/stockterm/issues/65) (Polygon limits / payload size).
- **Behavior note (post-audit):** Periodic historical refresh preserves zoom/pan via `chart_viewport_after_refresh` unless the view was full-range or the ticker changed; see `src/app/charts.rs`.

---

### 11.11 M4 follow-ups — Issues #62, #63, #64 (Charts polish)

**Tracking (GitHub):**

- [Issue #62](https://github.com/FelipeMorandini/stockterm/issues/62) — Clear or gate stale `historical_data` when `App.symbol` changes so chart chrome and OHLC series never disagree.
- [Issue #63](https://github.com/FelipeMorandini/stockterm/issues/63) — Yahoo **W1**: if primary intraday request returns **zero** bars, retry same window with **daily** bars (§11.3 already suggested this).
- [Issue #64](https://github.com/FelipeMorandini/stockterm/issues/64) — Transient historical errors, empty `HistoricalResponse.ticker` in viewport logic, and `mpsc` send failure vs `hist_refresh_inflight`.

**Verified baseline (symbol vs charts):**

| Area | Location | Problem |
|------|----------|---------|
| Symbol changes | `search_pick_symbol_go_stock`, `add_current_to_watchlist`, `remove_selected_watchlist_row`, `watchlist_select_*`, Portfolio **Enter** → Stock (`portfolio.rs`) | These paths call `notify_symbol_changed_for_news()` but do **not** clear `historical_data` / `chart_viewport`. |
| Charts draw | `draw_charts` (`charts.rs`) | Title uses `app.symbol`; series comes from `app.historical_data` — mismatch until `FetchDone::Historical` applies. |
| W1 Yahoo | `TimeRange::W1` → `yahoo_range: "5d"`, `bar_interval: "30m"` (`time_range.rs`) | Illiquid symbols may get **empty** intraday series; no second request today. |
| Historical error | `apply_fetch_done` / `FetchDone::Historical` (`app.rs`) | On `Err`, clears **`historical_data`** and viewport — user loses last-good chart during transient failures. |
| Viewport refresh | `chart_viewport_after_refresh` (`charts.rs`) | Compares `prev.ticker` to `new_data.ticker` with `eq_ignore_ascii_case`; if Yahoo leaves **`ticker` empty**, comparison fails and viewport resets to full range unnecessarily. |

---

#### 11.11.1 Issue #62 — Symbol / series coherence

**Goal:** After any **effective** change to the active ticker (`App.symbol`), the Charts tab must not render OHLC from a **different** ticker until a fetch for the new symbol succeeds.

**Recommended approach (single helper):**

- Add **`App::on_active_symbol_changed_for_charts(&mut self)`** (name flexible) that:
  - Sets **`historical_data = None`**
  - Sets **`chart_viewport = ChartViewport::default()`** (or `full(0)` equivalent — match existing “empty” conventions in `draw_charts`)
  - Sets **`last_charts_network_poll = None`** so the next Charts poll schedules immediately when the user lands on Charts (optional but aligns with “loading” state)
  - Does **not** alone flip **`hist_refresh_inflight`** — in-flight tasks still complete; **`apply_fetch_done`** already drops stale responses when `symbol != self.symbol` or `time_range` mismatches.

**Call sites (audit each `self.symbol = …` in `app.rs`, `portfolio.rs`, and any future navigators):**

- After **`search_pick_symbol_go_stock`** assigns `self.symbol`
- After **`add_current_to_watchlist`** / **`remove_selected_watchlist_row`** / **`watchlist_select_prev`** / **`watchlist_select_next`** when `symbol` changes
- Portfolio **Enter** path when jumping to Stock View with a new holding symbol

**Alternative (not preferred unless profiling demands it):** In **`draw_charts`**, render the chart body **only if** `historical_data.as_ref().map(|h| effective_ticker_for_draw(h, &app.symbol))` matches **`normalize_symbol` / case-insensitive** `app.symbol`; otherwise show **Loading…** / empty-state. The helper approach avoids duplicating match logic in the widget layer.

**Keys / typing:** Character-by-character edits to `symbol` on Stock View without confirming **Enter** may keep old series until fetch — acceptable if chrome shows the **typed** buffer consistently; if product wants “clear as soon as buffer diverges,” extend the helper to partial clears — **out of scope** unless Issue #62 acceptance is expanded.

---

#### 11.11.2 Issue #63 — Yahoo W1 empty intraday fallback

**Goal:** For **`TimeRange::W1`**, when the primary Yahoo request (`range=5d`, `interval=30m`) returns **`Ok`** with **`results.is_empty()`**, issue a **second** request for the **same** rolling window with **`interval=1d`** (daily bars for ~the same calendar span). If the second response has bars, return that **`HistoricalResponse`**; if still empty, return empty **`Ok`** (same as today — UI shows “no data”). **No panic.**

**Implementation placement (pick one, avoid dual call sites):**

- **`src/api/yahoo.rs`:** Inside the **`yahoo_historical_range`** path (or a small private **`yahoo_historical_range_with_empty_fallback`** used only from **`get_historical`** when `query.yahoo_range == Some("5d")` and `query.bar_interval == "30m"`), after parsing the first envelope:
  - If `results.len() == 0`, call **`yahoo_historical_range(symbol, "5d", "1d")`** (or build URL twice without duplicating fetch helpers).
- **Polygon:** No change required for #63 (issue scope is Yahoo); if Polygon W1 returns empty, existing empty-state UI applies.

**Tests:** Unit-test URL builder or injectable fetch seam if present; otherwise table-driven test that **`chart_to_historical` empty → second interval** is invoked (mock provider or internal fn).

---

#### 11.11.3 Issue #64 — Historical fetch resilience

**1) Transient errors vs last-good series**

- **Chosen behavior:** On **`FetchDone::Historical` with `Err(err)`**, **do not** clear **`historical_data`** or **`chart_viewport`** if **`historical_data` is already `Some`** for the **current** `(symbol, time_range)` (i.e. we previously had a successful load for this selection). Set **`error_message`** to a short prefix + provider error (reuse existing string style).
- **First load failure** (no prior series for this selection): keep **`historical_data = None`** and default viewport — same as today.
- **Success after error:** Clear **`error_message`** for this path (already done on Ok branch).
- Rationale: matches Issue #64 acceptance (“keep last-good series and surface error until retry succeeds”) without hiding stale **symbol** data — combined with **§11.11.1**, after a symbol change the series is already cleared, so “last-good” is always for the **current** symbol.

**2) Empty `HistoricalResponse.ticker` in `chart_viewport_after_refresh`**

- Extend **`chart_viewport_after_refresh`** (or a thin wrapper) to accept **`requested_symbol: &str`** (the **`FetchDone::Historical.symbol`** / spawn capture).
- **Effective ticker** for comparison: `if new_data.ticker.is_empty() { requested_symbol } else { new_data.ticker.as_str() }` (trim if needed). Use that for **`eq_ignore_ascii_case`** against **`prev.ticker`** when deciding ticker-change vs append-only refresh.
- Optionally normalize **`HistoricalResponse.ticker`** in **`chart_to_historical`** to **`requested.to_uppercase()`** when meta symbol missing — only if it does not break Polygon payloads; otherwise rely on **requested_symbol** at call site.

**3) `hist_refresh_inflight` when `tx.send` fails**

- Background tasks use **`let _ = tx.send(FetchDone::Historical { … })`**. If the **`UnboundedSender`** is disconnected (shutting down or abnormal), **`hist_refresh_inflight` stays `true`** forever.
- **Minimal mitigation:** In the **`tokio::spawn`** block, **`match tx.send(...)`** — on **`Err`**, **do not** rely on `App` mutation; document that shutdown drops the receiver. Optional: **`eprintln!`** / **`tracing::warn!`** if tracing is added later.
- **Stronger (optional):** Send a synthetic **`FetchDone::Historical { result: Err("disconnected") }`** is impossible without a live sender — instead, ensure **`App::run`** sets **`fetch_done_tx = None`** only on exit after draining — **out of scope** unless reproducible stuck state appears in production.

---

#### 11.11.4 Crate & module layout

| Item | Module | Change |
|------|--------|--------|
| #62 | `src/app/app.rs` | New **`on_active_symbol_changed_for_charts`** (or merged **`on_active_symbol_changed`** that also calls **`notify_symbol_changed_for_news`** pattern — avoid double-clear). Wire from every **`symbol`** mutation that affects the active ticker. |
| #62 | `src/app/portfolio.rs` | Portfolio **Enter** → call the same helper after **`symbol`** assignment. |
| #63 | `src/api/yahoo.rs` | W1 empty → retry **`5d`/`1d`**; keep **`ProviderResult`** semantics. |
| #64 | `src/app/app.rs` | Adjust **`apply_fetch_done`** Historical **`Err`** branch per §11.11.3.1. |
| #64 | `src/app/charts.rs` | **`chart_viewport_after_refresh(prev_vp, new_data, requested_symbol)`** signature update + tests in same file `#[cfg(test)]`. |

---

#### 11.11.5 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- `cargo test`
- **Unit tests:** `chart_viewport_after_refresh` with **empty `new_data.ticker`** and non-empty **`requested_symbol`**; optional Yahoo fallback test seam.

---

#### 11.11.6 Approval

After maintainer approval of §11.11, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #62 / #63 / #64 section).

### 11.11.7 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #62 / #63 / #64 section, 2026-05-11).
- **Tracking:** Closes [#62](https://github.com/FelipeMorandini/stockterm/issues/62), [#63](https://github.com/FelipeMorandini/stockterm/issues/63), [#64](https://github.com/FelipeMorandini/stockterm/issues/64).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/75
- **Code:** `src/app/{app,charts,portfolio}.rs`, `src/api/yahoo.rs`.
- **Follow-ups (shipped in §11.12):** [#71](https://github.com/FelipeMorandini/stockterm/issues/71)–[#74](https://github.com/FelipeMorandini/stockterm/issues/74) — see **§11.12.8**. New polish backlog: [#76](https://github.com/FelipeMorandini/stockterm/issues/76)–[#79](https://github.com/FelipeMorandini/stockterm/issues/79).

---

### 11.12 M4 follow-ups — Issues #71, #72, #73, #74 (async hardening, tests, UX)

**Tracking (GitHub):**

- [Issue #71](https://github.com/FelipeMorandini/stockterm/issues/71) — When `FetchDone` (or stock batch completion) **`send`** fails, matching **`*_inflight`** flags must not stay stuck; unify logging vs silent `let _ = tx.send`.
- [Issue #72](https://github.com/FelipeMorandini/stockterm/issues/72) — Remove dead **`App::fetch_historical_data`** (or isolate behind **`#[cfg(test)]`**) so only **`try_spawn_historical_fetch`** + **`FetchDone::Historical`** define production historical loads.
- [Issue #73](https://github.com/FelipeMorandini/stockterm/issues/73) — Unit tests for Yahoo **W1** empty intraday → **daily** retry (**#63**) without live HTTP.
- [Issue #74](https://github.com/FelipeMorandini/stockterm/issues/74) — **`add_current_to_watchlist`**: if normalization only changes **case**, skip **`on_active_symbol_changed_for_charts`** to avoid chart flicker; preserve **#62** behavior for real symbol changes.

**Related:** [#17](https://github.com/FelipeMorandini/stockterm/issues/17) (async UX), [#63](https://github.com/FelipeMorandini/stockterm/issues/63) / §11.11.2 (W1 fallback under test), [#62](https://github.com/FelipeMorandini/stockterm/issues/62) / §11.11.1 (symbol/chart coherence).

---

#### 11.12.1 Issue #71 — Inflight flags vs `mpsc` send failures

**Problem (verified in tree):** `try_spawn_historical_fetch` sets **`hist_refresh_inflight = true`**, then **`tokio::spawn`** runs HTTP and **`tx.send(FetchDone::Historical { ... })`**. On **`Err(SendError)`**, the task logs to **stderr** but **`apply_fetch_done`** never runs, so **`hist_refresh_inflight`** can remain **`true`** and block further chart fetches. **`spawn_stock_fetch_task`**, **`try_spawn_news_fetch`**, and **`spawn_search_task`** use **`let _ = tx.send(...)`** with **no** inflight recovery and **no** logging.

**Acceptance:**

- Every background path that sets **`hist_refresh_inflight`**, **`stock_refresh_inflight`**, **`news_refresh_inflight`**, or **`search_refresh_inflight`** must **clear** that flag on the **main** async loop if the result cannot be delivered via **`FetchDone`** (same semantics: user can retry on next tick).
- Replace ad-hoc **`eprintln!`** with a **single** style: **`tracing::warn!`** if the crate adds **`tracing`** (optional per issue); otherwise keep **`eprintln!`** with a consistent **`stockterm:`** prefix.

**Implementation plan (Rust):**

1. **Recovery channel (recommended):** Introduce **`#[derive(Debug, Clone, Copy)] enum InflightRecovery { Historical, News, Search, Stock }`** and a second **`tokio::sync::mpsc::unbounded_channel<InflightRecovery>`** — **`inflight_recovery_rx`** merged into **`App::run`**’s **`tokio::select!`** alongside **`fetch_rx`**. Store **`Option<UnboundedSender<InflightRecovery>>`** on **`App`**, cloned into each fetch **`tokio::spawn`** alongside **`fetch_done_tx`**. After **`fetch_tx.send(...).map_err(|e| { warn!(...); let _ = recovery_tx.send(InflightRecovery::Historical); })`** — the **`select!`** arm **`Some(InflightRecovery::Historical) => { self.hist_refresh_inflight = false; }`** (mirror for **`Stock`**, **`News`**, **`Search`**). Optionally set a one-line **`error_message`** (“Fetch result dropped — retrying”) if product wants visible feedback; **default:** clear flag only, same as a no-op completion for throttle purposes.
2. **Alternative:** **Stale-inflight watchdog** in **`on_background_tick`** (e.g. clear if inflight and **no** progress for **N** seconds). Prefer only if a second channel is unacceptable; document **N** and false-positive risk on slow networks.
3. **Stock batch:** Apply the **same** **`send` + recovery** pattern to **`spawn_stock_fetch_task`** (today **`stock_refresh_inflight`** can stick like historical).
4. **Tests:** **`#[cfg(test)]`** can expose a helper **`send_fetch_done_or_recover`**; optional integration test with **dropped receiver** is **out of scope** unless trivial.

**Modules:** **`src/app/app.rs`** (primary); optional **`src/app/fetch_channels.rs`** if **`App::run`** grows too large.

---

#### 11.12.2 Issue #72 — Remove or isolate `App::fetch_historical_data`

**Problem:** **`pub async fn fetch_historical_data`** ([`src/app/app.rs`](../src/app/app.rs)) duplicates **`try_spawn_historical_fetch`** + **`apply_fetch_done`** semantics and is **not** called from **`App::run`**.

**Acceptance:** No second production entry point for historical loads; **`cargo clippy -- -D warnings`** passes (no unjustified **`dead_code`**).

**Implementation plan:**

1. Confirm **no** callers (**`rg fetch_historical_data`**) across the workspace.
2. **Preferred:** **Delete** the method; keep a **single** pipeline: **`try_spawn_historical_fetch`** → **`FetchDone::Historical`** → **`apply_fetch_done`**.
3. **Alternative:** If tests need inline history, add **`#[cfg(test)]`** helpers that call **`MarketDataProvider::get_historical`** directly **without** mutating **`App`** through a parallel code path.

**Docs:** Legacy SPEC bullets that named **`fetch_historical_data`** are updated in this revision to reference **`try_spawn_historical_fetch`** only.

---

#### 11.12.3 Issue #73 — Unit tests for Yahoo W1 empty intraday → daily fallback

**Goal:** Lock **#63** / §11.11.2 behavior: primary **`5d` / `30m`** response with **zero** bars triggers a **second** request with **`5d` / `1d`**.

**Implementation plan:**

1. **Extract** a **pure** decision function (name flexible), e.g. **`fn yahoo_w1_daily_fallback_interval(yahoo_range: Option<&str>, bar_interval: &str, first_result_count: usize) -> Option<&'static str>`** returning **`Some("1d")`** only when **`yahoo_range == Some("5d")`**, **`bar_interval == "30m"`**, and **`first_result_count == 0`**; otherwise **`None`**.
2. **`YahooProvider::get_historical`** (or inner helper) calls this after parsing the first envelope; on **`Some("1d")`**, issue the follow-up fetch using existing URL builders.
3. **`#[cfg(test)] mod tests`** in **`src/api/yahoo.rs`**: table-driven tests for **(range, interval, len) →** expected next interval / no retry.

**Automated:** **`cargo test`** includes these cases; **no** live Yahoo HTTP.

---

#### 11.12.4 Issue #74 — Watchlist add: skip chart invalidation on case-only normalization

**Problem:** **`add_current_to_watchlist`** assigns **`self.symbol = sym`** (normalized) and always calls **`on_active_symbol_changed_for_charts()`**, which clears **`historical_data`** / viewport. If the buffer was already the same ticker in different case (**`aapl`** → **`AAPL`**), the chart clears unnecessarily (**minor flicker**).

**Acceptance:** If the **effective** ticker is unchanged under **ASCII case-insensitive** equality, **do not** call **`on_active_symbol_changed_for_charts`**. If the ticker **actually** changes, keep **#62** / §11.11.1 behavior (clear stale series).

**Implementation plan (Rust):**

1. At entry, **`let prev_effective = self.symbol.clone();`**
2. After **`let Some(sym) = normalize_symbol(...)`**, if **`prev_effective.eq_ignore_ascii_case(&sym)`**, **skip** **`on_active_symbol_changed_for_charts`**; otherwise call it **after** state updates as today.
3. Still **`push`**, **`try_save`**, update **`watchlist_state`**, **`notify_symbol_changed_for_news`**, and set **`self.symbol = sym`** for consistent casing.

**Module:** **`src/app/app.rs`** — **`add_current_to_watchlist`**.

---

#### 11.12.5 Crate & module layout (summary)

| Issue | Module(s) |
|-------|-----------|
| #71 | `src/app/app.rs` (+ optional `fetch_channels.rs`) |
| #72 | `src/app/app.rs` |
| #73 | `src/api/yahoo.rs` |
| #74 | `src/app/app.rs` |

---

#### 11.12.6 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- `cargo test` (includes **#73** table tests)

---

#### 11.12.7 Approval

After maintainer approval of §11.12, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #71 / #72 / #73 / #74 section).

### 11.12.8 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #71–#74 section, 2026-05-11).
- **Tracking:** Closes [#71](https://github.com/FelipeMorandini/stockterm/issues/71), [#72](https://github.com/FelipeMorandini/stockterm/issues/72), [#73](https://github.com/FelipeMorandini/stockterm/issues/73), [#74](https://github.com/FelipeMorandini/stockterm/issues/74).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/80
- **Code:** [`src/app/app.rs`](../src/app/app.rs) (`InflightRecovery`, fetch send + recovery channel, `add_current_to_watchlist` case-only skip), [`src/api/yahoo.rs`](../src/api/yahoo.rs) (`yahoo_w1_daily_fallback_interval` + unit tests).
- **Deferred (scratchpad → issues):** [#76](https://github.com/FelipeMorandini/stockterm/issues/76) (tracing), **[#77](https://github.com/FelipeMorandini/stockterm/issues/77) → §16.3** (`stock_refresh_pending` vs `InflightRecovery::Stock`), [#78](https://github.com/FelipeMorandini/stockterm/issues/78) (recovery channel hardening), [#79](https://github.com/FelipeMorandini/stockterm/issues/79) (Unicode tickers).

---

## 12. Issue #48 — Portfolio tab keyboard parity (Issue #44 follow-up)

**Sources:**

- [GitHub Issue #48](https://github.com/FelipeMorandini/stockterm/issues/48) — reuse `letter_key_plain` for Portfolio `a` / `d` (and any future letter hotkeys); same modifier rules as Stock View / Alerts.
- **Baseline:** [Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) (shipped, §8) — `src/app/keyboard.rs::letter_key_plain`.

**Related:** [Issue #6](https://github.com/FelipeMorandini/stockterm/issues/6) — broader portfolio UX; may land in the same PR or after #48.

### 12.1 Problem (verified in tree)

[`handle_portfolio_events`](../src/app/portfolio.rs) matches `KeyCode::Char('a')` / `Char('d')` with **`KeyModifiers::NONE` only**. Terminals that report **Shift+letter** with `KeyModifiers::SHIFT`, or lowercase **`a`** / **`d`**, do not match — add/remove feel broken compared to Alerts.

**Already correct:** `Up` / `Down` use `..` for modifiers (arrow parity with other tables).

### 12.2 Acceptance

- **`a` (add)** and **`d` (delete)** accept the same modifier surface as §8.4: `letter_key_plain(modifiers)` is **true**, and character match is **ASCII case-insensitive** (`eq_ignore_ascii_case('a')`, `eq_ignore_ascii_case('d')`).
- **Chord safety:** Control / Alt / Meta / Hyper / Super (per `crossterm`) must **not** trigger `a` / `d` actions.
- **Reuse** `crate::app::keyboard::letter_key_plain` — no duplicated bitmask logic.
- **`Enter`** (jump to Stock View for highlighted row): keep **`KeyModifiers::NONE` only** (parity with §8.5 / Settings — avoid accidental `Ctrl+Enter`).
- **No async / HTTP changes** for #48 alone.

### 12.3 Implementation plan (Rust)

1. In [`src/app/portfolio.rs`](../src/app/portfolio.rs), `use crate::app::keyboard::letter_key_plain`.
2. Replace the two `KeyEvent { code: Char('a'|'d'), modifiers: NONE, .. }` arms with `Char(c)` patterns gated by `letter_key_plain(key.modifiers)` and `c.eq_ignore_ascii_case('a')` / `eq_ignore_ascii_case('d')`.
3. When **§13** lands (portfolio add dialog), **`a`** while a dialog is open should be handled by the dialog first (see §13.4) — do not double-add.

### 12.4 Automated verification

- `cargo build --release`, `cargo clippy -- -D warnings`, `cargo test`.
- Extend **unit tests** only if new helpers are introduced; otherwise rely on existing `keyboard.rs` tests + manual QA.

### 12.5 Out of scope

- **`j` / `k`** row navigation (optional parity with Stock View — track under §13 or a follow-up).
- Global tab switching / quit modifier rules ([#51](https://github.com/FelipeMorandini/stockterm/issues/51)).

### 12.6 Approval

After maintainer approval of §12, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #48 section).

### 12.7 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #48 section); closes [#48](https://github.com/FelipeMorandini/stockterm/issues/48) (PR [#70](https://github.com/FelipeMorandini/stockterm/pull/70), same as §13).
- **Code:** [`src/app/portfolio.rs`](../src/app/portfolio.rs) — `letter_key_plain` on Portfolio `a`/`d`/armed keys + **`j`**/**`k`** navigation.
- **Follow-ups:** [#67](https://github.com/FelipeMorandini/stockterm/issues/67) — **§15.4** (Tab / BackTab in add dialog).

---

## 13. Issue #6 — Portfolio UX: add dialog, confirm remove, quote coverage

**Sources:**

- [GitHub Issue #6](https://github.com/FelipeMorandini/stockterm/issues/6) — replace hard-coded `(1.0, 100.0)` add; confirm-before-remove; refresh prices after add; navigation.

**Supersedes outdated bullets in the GitHub issue body** (as of 2026-05-10 tree audit):

- **`handle_portfolio_events` is wired** from [`handlers.rs`](../src/app/handlers.rs) when `active_tab == Tab::Portfolio`.
- **No `fetch_ticker_data().await` in the handler** — quotes use **`spawn_stock_fetch_task`** + `FetchDone::Stock` ([#17](https://github.com/FelipeMorandini/stockterm/issues/17) pattern); [`apply_stock_fetch_done`](../src/app/app.rs) already back-fills **`portfolio[].current_price`** from **`watchlist_quotes`**.
- **Remaining gap:** [`collect_symbols_for_quote_fetch`](../src/app/app.rs) includes **watchlist + `symbol` only** — **not** every **portfolio** symbol. Holdings whose tickers are neither on the watchlist nor the active `symbol` can stay stale until the user selects that ticker. §13 requires unioning **all distinct portfolio symbols** into the quote batch (deduped with watchlist / `symbol`).

**Related:** [#19](https://github.com/FelipeMorandini/stockterm/issues/19) — surface `Config::try_save` errors via `App.error_message` (today `add_to_portfolio` / `remove_from_portfolio` call `Config::save()` which can panic on I/O — align with `try_save` when touching these paths). [#48](https://github.com/FelipeMorandini/stockterm/issues/48) / §12 — keyboard parity (land before or with §13).

### 13.1 Acceptance criteria

- **`a` on Portfolio** opens an **in-app input flow** (modal / overlay), **not** an immediate `add_to_portfolio(1.0, 100.0)`.
- **Symbol** shown in the dialog is the **active** `App.symbol` (read-only label), **normalized** (uppercase); if `symbol` is empty or invalid, show an inline error and do not open numeric fields (or open with disabled commit until Stock View sets a symbol — pick one and document in QA).
- **Shares** and **purchase price** are user-entered **positive floats** (digits + one `.`, Backspace, reasonable max length).
- **Field focus:** **`;`** (semicolon, no modifiers) cycles **Shares** ↔ **Price**. **`Tab`** / **`Shift+Tab`** (`BackTab`) do the same **when the add dialog is open**, without switching app tabs (**Issue #67**, §15.4). When the dialog is closed, **Tab** / **BackTab** keep switching app tabs as today. **Enter** on **Shares** moves to **Price**; **Enter** on **Price** **commits**; **Esc** **cancels** (clear dialog state, no mutation).
- On **commit:** call existing **`add_to_portfolio(shares, price)`** logic (weighted average when symbol already exists); persist via **`Config::try_save`**; on `Err`, set **`error_message`** and keep dialog open or close per UX choice (document in QA).
- After successful add: call **`request_immediate_stock_poll()`** so a quote batch runs soon and **`apply_stock_fetch_done`** updates the new row’s **`current_price`** (with §13.3 ensuring the symbol is in the batch).
- **`d` remove:** **two-step confirm** — first `d` arms removal for the **selected** row (status hint); second `d` **or** **`y`** confirms; **`n`** or **`Esc`** cancels the armed state. While armed, other keys are ignored or only safety keys work (document). **Chord / case rules** for `d` / `y` / `n` follow §12 (`letter_key_plain` + case-insensitive where applicable).
- **Row navigation:** keep **Up/Down**; add **`j` / `k`** with `letter_key_plain` (optional but recommended for parity with Stock View / Search).
- **Totals** in `draw_portfolio` reflect new data immediately after commit (same frame after state update; price may fill on next `FetchDone::Stock`).

### 13.2 Crate & module layout

- **`src/app/app.rs`:** New fields on `App`, for example:
  - `portfolio_dialog: Option<PortfolioAddDialog>` where `PortfolioAddDialog` holds `shares_buffer: String`, `price_buffer: String`, `focused: PortfolioAddField` (`Shares` | `Price`), and optionally `inline_error: Option<String>`.
  - `portfolio_remove_armed: bool` (or `Option<usize>` if selection must be snapshotted — prefer bool if arm always targets **current** `portfolio_state.selected()`).
- **`src/app/portfolio.rs`:** `draw_portfolio` draws an **overlay** (centered `Block` or extra `Layout` split) when `portfolio_dialog` is `Some` or `portfolio_remove_armed`; `handle_portfolio_events` dispatches to **`handle_portfolio_dialog_keys`** / **`App` methods** when dialog active or remove armed.
- **`src/app/handlers.rs`:** **Issue #67** (§15.4) requires a **narrow** change: when **`Tab`/`BackTab`** would switch tabs, **guard** with `active_tab == Tab::Portfolio && portfolio_dialog.is_some()` and route to field-cycle instead of `next_tab`/`prev_tab`. **`q`** and other globals unchanged.

### 13.3 Quote batch — include all portfolio symbols

Extend **`collect_symbols_for_quote_fetch`** to iterate **`self.portfolio`** and push **normalized** `item.symbol` into the same **deduped** list as watchlist + `symbol`. Order: existing watchlist order, then `symbol`, then portfolio symbols not yet seen (stable order aids debugging). Keeps **`MAX_CONCURRENT_QUOTES`** behavior unchanged.

### 13.4 Input routing precedence

When `portfolio_dialog.is_some()`:

1. **Esc** → cancel dialog, clear buffers.
2. **Field cycle:** **`;`** (semicolon, no modifiers) cycles `focused` between Shares and Price. **`Tab`** / **`BackTab`:** when the dialog is open, **`handle_event`** must cycle fields instead of app tabs (**§15.4**, Issue #67). When the dialog is closed, Tab / BackTab switch app tabs as today.
3. **Digits / `.`** → append to active buffer (validate no multiple `.`).
4. **Backspace** → pop from active buffer (`KeyModifiers::NONE` only recommended).
5. **Enter** → if focus is **Price**, parse both buffers and commit; if focus is **Shares`, move focus to **Price** (alternative: Enter always advances field — document one behavior in QA).

When `portfolio_remove_armed` and no dialog:

- **Esc** / **`n`** → disarm.
- **`d`** or **`y`** → confirm remove for selected index, then `remove_from_portfolio`, `try_save`, disarm.

**Letter `a` while armed:** Either disarm first or ignore — pick one (recommend **ignore** until user clears arm, to avoid accidental add).

### 13.5 Persistence

- Replace **`Config::save()`** in **`add_to_portfolio`** / **`remove_from_portfolio`** with **`try_save`**, matching **`save_alerts`** / watchlist patterns: on failure set **`error_message`**, do not panic.

### 13.6 Automated verification

- `cargo build --release`, `cargo clippy -- -D warnings`, `cargo test`.
- **Unit tests (recommended):** pure fn for **parsing** shares/price strings; optional test that **`collect_symbols_for_quote_fetch`** includes a portfolio-only symbol fixture (if extracted for testability).

### 13.7 Out of scope

- Full **editing** of existing rows (shares/price) — new issue.
- **OS dialogs** or external TUI crates — stay **ratatui** + existing patterns.
- **Portfolio** symbol different from `App.symbol` in the add dialog (Issue #6 text mentions symbol in dialog; this SPEC pins symbol to **active `App.symbol`** — user switches symbol on Stock View first, or via Enter from portfolio row).

### 13.8 Approval

After maintainer approval of §13, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #6 section).

### 13.9 Shipment record

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #6 section); closes [#6](https://github.com/FelipeMorandini/stockterm/issues/6). **PR:** [#70](https://github.com/FelipeMorandini/stockterm/pull/70).
- **Code:** [`src/app/portfolio.rs`](../src/app/portfolio.rs) (`PortfolioAddDialog`, overlay, two-step remove); [`src/app/app.rs`](../src/app/app.rs) (`collect_symbols_for_quote_fetch` includes portfolio symbols; `add_to_portfolio` / `remove_from_portfolio` + **`try_save`**).
- **Related closure:** [#39](https://github.com/FelipeMorandini/stockterm/issues/39) (portfolio **`try_save`** parity — addressed in same delivery).
- **Follow-ups:** [#67](https://github.com/FelipeMorandini/stockterm/issues/67) / [#69](https://github.com/FelipeMorandini/stockterm/issues/69) — **§15**. [#68](https://github.com/FelipeMorandini/stockterm/issues/68) — optional decimal money display (out of §15 scope).

---

## 14. Issue #44 — reference (shipped)

**Issue #44** is **closed**; behavior is specified in **§8** and verified in [`docs/QA_PLAN.md`](QA_PLAN.md). **§12** and **§13** must stay consistent with §8 for modifier semantics on letter keys.

---

## 15. Issues #43, #49, #50, #67, #69 — Alerts polish, Stock View hints, Portfolio dialog input

**Sources:**

- [Issue #43](https://github.com/FelipeMorandini/stockterm/issues/43) — unify **`draw_alerts`** block titles (empty vs table).
- [Issue #49](https://github.com/FelipeMorandini/stockterm/issues/49) — Stock View status/footer: watchlist hotkeys + **A–Z symbol typing** + §8.4 edge case (leading `w`/`x`/`j`/`k`).
- [Issue #50](https://github.com/FelipeMorandini/stockterm/issues/50) — Alerts empty-state copy: **`a` / `A`** (Shift-friendly) for add.
- [Issue #67](https://github.com/FelipeMorandini/stockterm/issues/67) — Portfolio add dialog: **Tab** / **Shift+Tab** cycle Shares/Price; precedence over global tab bar when dialog open.
- [Issue #69](https://github.com/FelipeMorandini/stockterm/issues/69) — Portfolio add: **inline_error** on commit when `add_to_portfolio` fails for non–`try_save` reasons; optional **max shares / max price** caps.

**Non-goals:** No API/provider changes; no new async tasks; no OS notifications.

### 15.1 Issue #43 — Alerts block titles

- **Current:** [`src/app/alerts.rs`](../src/app/alerts.rs) — empty branch wraps content in `Block::title("Price Alerts")`; non-empty branch renders `Table` with inner `Block::title("Alerts")`.
- **Target:** One consistent user-visible title on both branches (recommended: **"Price Alerts"** on both, or a single outer `Block` title and inner blocks without conflicting titles). If two nested titles remain, add a **short code comment** documenting the hierarchy.
- **Verification:** Visual only; no handler changes.

### 15.2 Issue #50 — Alerts empty-state copy

- Update the yellow helper line so users know add matches **`a`** and **`A`** / Shift-friendly input (same semantics as `letter_key_plain` in [`handlers.rs`](../src/app/handlers.rs) / [`alerts.rs`](../src/app/alerts.rs)).
- **Verification:** Empty `app.alerts` on **Alerts** tab.

### 15.3 Issue #49 — Stock View status bar

- **Location:** [`src/app/ui.rs`](../src/app/ui.rs) **`draw_status_bar`**, `Tab::StockView` branch (today: `w` add, `x`/`D` remove, `j`/`k` move, Enter fetch).
- **Add:** Explicit note that **ticker symbols use A–Z** (and link visually to existing hotkey spans). One-line reminder of **§8.4**: symbols starting with **`w`**, **`x`**, **`j`**, or **`k`** — type the first letter with **Shift** when using lowercase (`Wmt` → WMT), because those keys are watchlist shortcuts.
- **Layout:** Prefer a **single** `Line` of `Span`s; if width is tight on small terminals, use **DarkGray** for the edge-case clause or truncate responsibly — record the chosen UX in QA.

### 15.4 Issue #67 — Tab / BackTab in Portfolio add dialog (sync routing)

**Problem:** [`handle_event`](../src/app/handlers.rs) matches **`KeyCode::Tab`** and **`BackTab`** before the `match app.active_tab` dispatch, so [`handle_portfolio_dialog_keys`](../src/app/portfolio.rs) never receives Tab.

**Algorithm:**

1. In **`handle_event`**, replace the unconditional `Tab` → `next_tab` / `BackTab` → `prev_tab` arms with:
   - If **`app.active_tab == Tab::Portfolio`** && **`app.portfolio_dialog.is_some()`**:
     - **Tab** (any modifiers policy: match existing global Tab arm — today unrestricted): cycle **`PortfolioAddField`** forward (Shares → Price → Shares).
     - **BackTab:** cycle backward.
     - Clear **`inline_error`** on cycle (same as **`;`** handler).
   - Else: **`app.next_tab()`** / **`app.prev_tab()`** unchanged.
2. Keep **`;`** in **`handle_portfolio_dialog_keys`** as an alternate cycle (shipped §13 behavior).
3. Update dialog overlay help text in **`draw_portfolio`** to mention **Tab** / **Shift+Tab** and **`;`**.

**Crates / types:** No new dependencies. Optional **`fn cycle_portfolio_dialog_focus(app: &mut App, forward: bool)`** in `portfolio.rs` (or **`App`** impl in `app.rs`) to share logic between **`;`** and Tab.

**Async:** None.

### 15.5 Issue #69 — Commit failures and optional caps

**Commit path:** [`try_commit_portfolio_dialog`](../src/app/portfolio.rs) — after **`parse_holding_decimal`** succeeds for both fields and **`add_to_portfolio(shares, price)`** returns **`false`**:

| Condition | Action |
|-----------|--------|
| **`app.error_message.is_some()`** | **`try_save`** failed inside `add_to_portfolio`; message already set; **keep dialog open**; do not clear **`error_message`**. |
| **`error_message` is `None`** | e.g. **`normalize_symbol(&app.symbol)`** is **`None`** at commit time — set **`portfolio_dialog.inline_error`** with a clear, user-facing string (dialog must not **no-op** silently). |

**Optional caps (recommended in same delivery):** After parse, before `add_to_portfolio`, reject if **shares** or **price** exceed **`const`** ceilings (pick conservative values, e.g. `1e9` shares and `1e12` USD per share — tune for realism). On violation set **`inline_error`** only (no **`error_message`**). Document constants in QA.

**Tests:** Unit tests in **`portfolio.rs`** (or extracted pure **`fn`**) for cap boundaries and for “`add_to_portfolio` false + no error_message ⇒ caller sets inline error” if testable without full **`App`** (otherwise manual QA emphasis).

### 15.6 Automated verification

- `cargo build --release`, `cargo clippy -- -D warnings`, `cargo test`.

### 15.7 Approval

After maintainer approval of §15, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #43, #49, #50, #67, #69 section).

### 15.8 Shipment record

- **Status:** Shipped (implementation 2026-05-11). **PR:** [#84](https://github.com/FelipeMorandini/stockterm/pull/84). Manual QA: [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #43, #49, #50, #67, #69 section).
- **Issues:** [#43](https://github.com/FelipeMorandini/stockterm/issues/43), [#49](https://github.com/FelipeMorandini/stockterm/issues/49), [#50](https://github.com/FelipeMorandini/stockterm/issues/50), [#67](https://github.com/FelipeMorandini/stockterm/issues/67), [#69](https://github.com/FelipeMorandini/stockterm/issues/69).
- **Code:** [`src/app/alerts.rs`](../src/app/alerts.rs) (#43, #50), [`src/app/ui.rs`](../src/app/ui.rs) (#49 status bar), [`src/app/handlers.rs`](../src/app/handlers.rs) + [`src/app/portfolio.rs`](../src/app/portfolio.rs) (#67, #69 — `cycle_portfolio_dialog_focus`, `validate_holding_limits`, `try_commit_portfolio_dialog`), [`src/app/app.rs`](../src/app/app.rs) (unit test for failed add without `try_save`).

---

## 16. Issues #17, #46, #77 — Async main loop polish (non-blocking completion, quote robustness, pending coalescing)

**Sources:**

- [Issue #17](https://github.com/FelipeMorandini/stockterm/issues/17) — Non-blocking UI: decouple network fetch from input loop.
- [Issue #46](https://github.com/FelipeMorandini/stockterm/issues/46) — Watchlist quote batch: panic-safety and inflight flag cleanup.
- [Issue #77](https://github.com/FelipeMorandini/stockterm/issues/77) — Clear or drain **`stock_refresh_pending`** when stock **`FetchDone`** send fails (**`InflightRecovery::Stock`**).

**Related:** [#71](https://github.com/FelipeMorandini/stockterm/issues/71) / §11.12 (recovery channel), [#3](https://github.com/FelipeMorandini/stockterm/issues/3) / §3.3 (generation + single-flight), [#4](https://github.com/FelipeMorandini/stockterm/issues/4) (throttle).

### 16.1 Issue #17 — Current tree vs GitHub acceptance

**Already implemented (verify during implementation; do not regress):**

| Item | Location | Notes |
|------|----------|--------|
| Async event channel | [`src/app/event.rs`](../src/app/event.rs) | `tokio::sync::mpsc::UnboundedSender<Event>`; blocking **`event::poll` / `read`** on a **std thread**, not on the async runtime worker that runs **`draw`**. |
| `tokio::select!` | [`src/app/app.rs`](../src/app/app.rs) **`App::run`** | Arms: **`event_rx`**, **`fetch_rx`**, **`recovery_rx`**. |
| HTTP off hot path | **`app.rs`** | **`run_stock_quote_batch`**, historical / news / search tasks: **`tokio::spawn`** + **`FetchDone`**; **`apply_fetch_done`** on receive. |
| Stale quote results | **`apply_stock_fetch_done`** | **`generation != stock_fetch_generation`** → ignore payload; **do not** apply stale quotes to **`watchlist_quotes`**. |
| Coalesced refresh | **`request_immediate_stock_poll`** | Sets **`stock_refresh_pending`** when a batch is already in flight; **`apply_stock_fetch_done`** tail may spawn a follow-up. |

**Remaining / explicit close-out for #17:**

1. **Smoke test (mandatory for closing #17):** **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** — non-negative integer read once per process (via **`std::sync::OnceLock`**). When **> 0**, **`maybe_debug_http_delay`** (**`src/api/http.rs`**) **`tokio::time::sleep`** s that long **once per quote batch** at the start of **`run_stock_quote_batch`** (before per-symbol fan-out). Default when unset or invalid: **0**. With **≥ 5000** ms, confirm **rapid keypresses** (tab switch, watchlist **`j`/`k`**, symbol typing) keep updating the TUI and **`select!`** keeps receiving **Tick** / **Input** while quotes are in flight.
2. **Cancellation / supersede (product minimum today):** Document that **`stock_fetch_generation`** + ignore-stale-result is the **supported** supersede model for overlapping quote batches (**single flight** per **`spawn_stock_fetch_task`**). Optional follow-up: **`tokio_util::sync::CancellationToken`** passed into **`run_stock_quote_batch`** and cancelled when **`stock_fetch_generation`** bumps — only if we introduce **true** overlap; not required if single-flight invariant is preserved.
3. **Clippy:** **`cargo clippy -- -D warnings`** on touched modules; fix any **`await_holding_lock`** / **`mutex_lock`** across **`await`** if introduced during refactors.
4. **GitHub issue body:** After ship, update Issue #17 checklist to point at **`event.rs` + `App::run`** so future readers are not misled by the original “sync `mpsc`” wording.

### 16.2 Issue #46 — Panic-safety and inflight invariants

**Problems (from issue + code audit):**

1. A **panic** inside the spawned stock task **after** `run_stock_quote_batch` returns but **before** **`tx.send`** — rare — or a panic that **aborts** the task without hitting **`send`**, leaves **`stock_refresh_inflight == true`** until **`InflightRecovery::Stock`** (only if recovery **`send`** succeeds) or restart.
2. **`apply_stock_fetch_done`** early-return on **`generation != stock_fetch_generation`** intentionally **does not** clear **`stock_refresh_inflight`** — correct **only** while a **newer** batch is still in flight. Document this invariant in a **short comment** on **`apply_stock_fetch_done`** and in §16.2.1.

#### 16.2.1 Single-flight invariant (document)

- At most **one** quote batch task is “authoritative” for clearing **`stock_refresh_inflight`** via **`apply_stock_fetch_done`** for a given **`stock_fetch_generation`**.
- When **`generation`** is stale, either a **newer** batch is in flight (**inflight stays `true`**) or the app incremented generation without spawning (should not happen — audit **`spawn_stock_fetch_task`** guards).

#### 16.2.2 Implementation options (pick one in PR)

**A (recommended):** Inside the **`tokio::spawn`** closure, structure the **`async move { ... }`** so **`run_stock_quote_batch(...).await`** is followed by **`send`** in all non-abort paths. Add **`std::panic::AssertUnwindSafe`** + **`std::panic::catch_unwind`** around a **`pin!`**’d boxed future (or a small **`async fn`** shim) if needed so a **panic** in the batch still reaches a **`send(FetchDone::Stock { … empty quotes, errors: ["…"] })`** or **`InflightRecovery::Stock`** tail — **avoid** new dependencies unless the chosen pattern already matches a transitive crate (e.g. **`futures`** only if added deliberately).

**B:** Rely on **`JoinSet::join_next`** **`Err(JoinError)`** for per-symbol panics (already pushes to **`errors`**) **plus** an outer guard that guarantees **`send`** after the **`while let Some(joined)`** loop completes; document that panics **outside** that loop require **A** or a **`finally`**-equivalent.

**Tests:** **`#[cfg(test)]`** — unit test a small **`async fn`** helper that panics mid-batch and assert the completion path clears **`stock_refresh_inflight`** when wired through a test **`UnboundedChannel`** (optional if too heavy — then **manual QA** + code review sign-off).

### 16.3 Issue #77 — `stock_refresh_pending` vs `InflightRecovery::Stock`

**Bug:** [`apply_inflight_recovery`](../src/app/app.rs) for **`InflightRecovery::Stock`** clears **`stock_refresh_inflight`** but **not** **`stock_refresh_pending`**. If the user coalesced a refresh (**`stock_refresh_pending = true`**) and the background task’s **`FetchDone::Stock`** **`send`** fails, recovery clears inflight but **pending stays `true`** until a later **`apply_stock_fetch_done`** — **no follow-up spawn** if no other completion arrives.

**Target behavior (choose one, document in QA):**

| Option | Behavior |
|--------|----------|
| **A (recommended)** | In **`apply_inflight_recovery(Stock)`**, after **`stock_refresh_inflight = false`**, if **`stock_refresh_pending`**, set it **`false`** and call **`request_immediate_stock_poll()`** (or inline the same tail as **`apply_stock_fetch_done`**) so coalesced user intent becomes a **new** spawn now that the channel is healthy again. |
| **B** | Clear **`stock_refresh_pending`** without spawning; rely on **`on_background_tick`** + throttle for the next refresh. Simpler but **may delay** an explicit user-driven coalesced refresh. |

**Implementation:** **`src/app/app.rs`** only — extend **`apply_inflight_recovery`** (or a tiny **`fn reconcile_stock_refresh_after_recovery(&mut self)`** called from there).

### 16.4 Crate & module layout (summary)

| Issue | Primary module(s) | Optional |
|-------|-------------------|----------|
| #17 | `src/app/app.rs`, `src/app/event.rs`, `src/api/*` (debug delay behind cfg/env) | `Cargo.toml` feature **`slow-network`** |
| #46 | `src/app/app.rs` (`spawn_stock_fetch_task` closure, `run_stock_quote_batch`) | `src/app/app.rs` **`#[cfg(test)]`** |
| #77 | `src/app/app.rs` (`apply_inflight_recovery`) | — |

### 16.5 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- `cargo test` (include new §16.2 tests if added)

### 16.6 Out of scope

- **#18** rate-limit / backoff taxonomy (separate milestone).
- Replacing **`UnboundedChannel`** with bounded back-pressure (**#78**).

### 16.7 Approval

After maintainer approval of §16, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #17 / #46 / #77 section).

### 16.8 Shipment record

- **Status:** Shipped (implementation 2026-05-11) — **`STOCKTERM_DEBUG_HTTP_DELAY_MS`**, quote-batch **`catch_unwind`** + synthetic **`FetchDone::Stock`** on panic, **`apply_inflight_recovery(Stock)`** drains **`stock_refresh_pending`** into **`spawn_stock_fetch_task`**, stale-generation comment on **`apply_stock_fetch_done`**.
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/88
- **Tracking:** Closes [#17](https://github.com/FelipeMorandini/stockterm/issues/17), [#46](https://github.com/FelipeMorandini/stockterm/issues/46), [#77](https://github.com/FelipeMorandini/stockterm/issues/77) after merge; manual QA: [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #17 / #46 / #77 section).
- **Follow-ups (audit):** [#85](https://github.com/FelipeMorandini/stockterm/issues/85) (cap **`STOCKTERM_DEBUG_HTTP_DELAY_MS`**), [#86](https://github.com/FelipeMorandini/stockterm/issues/86) (dev panic logging), [#87](https://github.com/FelipeMorandini/stockterm/issues/87) (bounded channels / back-pressure).

---

## 17. Issue #2 — Latest-session quotes (provider adapters; no UI schema change)

**Sources:**

- [GitHub Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2) — replace stale / EOD-only quote semantics with **latest trading-session** prices for Stock View + watchlist batch; map into existing **`TickerResult`**; eliminate fixed historical calendar windows in **`src/api/`**; document Yahoo field mapping at the adapter.

**Related:** [#31](https://github.com/FelipeMorandini/stockterm/issues/31) (**`MarketDataProvider`** — quote path is **`get_quote`**), [#3](https://github.com/FelipeMorandini/stockterm/issues/3) (**`run_stock_quote_batch`** / **`watchlist_quotes`**), [#18](https://github.com/FelipeMorandini/stockterm/issues/18) (429/backoff — out of scope unless merged here).

### 17.1 Tree audit vs GitHub issue body (supersedes outdated bullets)

| Issue #2 text (historical) | Current tree (2026-05-11) |
|----------------------------|---------------------------|
| Polygon pinned to **`2023-01-01..2023-12-31`** | **`PolygonProvider::get_quote`** uses a **rolling ~30 calendar days** of **1/day** aggregates anchored to **`chrono::Local::now()`** ([`src/api/polygon.rs`](../src/api/polygon.rs)). |
| `App::fetch_ticker_data` | Quotes flow through **`run_stock_quote_batch`** → **`Arc<dyn MarketDataProvider>::get_quote`** ([`src/app/app.rs`](../src/app/app.rs)); no separate **`fetch_ticker_data`** symbol. |
| Yahoo **`v7/finance/quote`** | Yahoo default quote uses **`v8/finance/chart`** with **`range=1d&interval=1d`**, then **`chart_to_ticker`** maps **chart meta** → one **`TickerResult`** ([`src/api/yahoo.rs`](../src/api/yahoo.rs)). |

**Conclusion:** Much of #2 is **already satisfied** for the default Yahoo path (session fields from chart meta). This §17 defines **explicit acceptance**, **optional v7 primary**, **Polygon tightening/docs**, and **tests** so #2 can be **closed with evidence** without changing **`TickerResult`** call sites in **`ui.rs`** / **`alerts.rs`**.

### 17.2 Product acceptance (unchanged public types)

1. **`models/ticker.rs`** — **`TickerResponse`** / **`TickerResult`** field names and meaning at **UI** boundaries stay **`o` / `h` / `l` / `c` / `v` / `t`** (ms since epoch for bar timestamp, consistent with Polygon). **Do not** change **`draw_stock_detail`** / watchlist row math to require new fields; adapters absorb provider differences.
2. **No hard-coded multi-year quote windows** in **`src/api/`** (e.g. no fixed `2023-..` range literals for **live** quotes). Rolling **`Local::now()`** / **UTC-relative** windows are allowed. **Regression:** `rg '20[0-9]{2}-[0-9]{2}-[0-9]{2}.*20[0-9]{2}-[0-9]{2}-[0-9]{2}' src/api` should stay **empty** for quote URLs (historical calendar **`period1`/`period2`** built from **`NaiveDate`** args are fine).
3. **Semantics:** For liquid US equities during market hours, **`latest_result()`**’s **`c`** reflects **Yahoo regular market price** (or Polygon **latest daily bar close** for the most recent session bar), not a years-old frozen snapshot.
4. **Symbol change:** Changing **`App.symbol`** or watchlist selection triggers the **existing** batch path; Open/High/Low/Volume update from the **new** symbol’s adapter output without code changes outside **`api/`**.

### 17.3 Yahoo — implementation plan (Rust)

**Files:** [`src/api/yahoo.rs`](../src/api/yahoo.rs) only (plus tests in the same module’s **`#[cfg(test)]`** block).

1. **Primary quote path (recommended for #2 closure):** Implement **`yahoo_quote_v7(symbol) -> ProviderResult<TickerResponse>`** calling **`GET {QUERY1}/v7/finance/quote?symbols={enc(symbol)}`**. Deserialize into **private** structs (e.g. `QuoteEnvelope { quote_response: QuoteResponse }` with **`result: Option<Vec<QuoteItem>>`** — match real Yahoo JSON; camelCase via **`serde(rename)`** as needed).
2. **Field mapping** (adapter boundary — document in **`///`** on the mapper fn):

   | Yahoo (typical v7 field) | `TickerResult` |
   |--------------------------|----------------|
   | `regularMarketOpen` | **`o`** |
   | `regularMarketDayHigh` | **`h`** |
   | `regularMarketDayLow` | **`l`** |
   | `regularMarketPrice` | **`c`** |
   | `regularMarketVolume` | **`v`** (as **`f64`**) |
   | `regularMarketTime` (Unix **seconds**) | **`t`** = **`secs.saturating_mul(1000)`** (ms) |

   If any OHLC leg is missing, use the same **fallback** rules as today’s **`chart_to_ticker`** (e.g. high/low default to **`c`**, open fall back to **`chartPreviousClose`** / **`c`**).

3. **Orchestration:** Rename or wrap the public async path used by **`YahooProvider::get_quote`** as **`yahoo_latest_quote(symbol)`**: **try v7** first; on **`ProviderError`** or empty **`result`**, **fall back** to existing **`yahoo_quote`** (v8 chart **`chart_to_ticker`**). Keeps resilience if Yahoo changes v7 behavior.
4. **Async:** Single **`reqwest`** GET per attempt; reuse **`fetch_text`** / **`shared_client`**; no **`tokio::spawn`** inside the provider (callers already spawn batch work).

### 17.4 Polygon — implementation plan (Rust)

**Files:** [`src/api/polygon.rs`](../src/api/polygon.rs).

1. **Correctness:** Keep **daily** aggregates as today; ensure **`latest_result()`** (max **`t`**) is the canonical “display bar” — document in **`///`** on **`PolygonProvider::get_quote`** that **`c`** is the **close of the most recent returned bar** (typically last **US session** trading day in the window, depending on Polygon calendar).
2. **Optional optimization:** If the REST API allows, prefer **`sort=desc`** + **`limit=1`** (or smallest **`limit`** that guarantees at least one bar when the market is open) to shrink JSON; otherwise keep current **`limit=120`** + **`latest_result`** — product-neutral.
3. **Out of scope for #2:** Polygon **WebSocket** / **real-time** trades (#2 stays **REST latest-session**, not streaming).

### 17.5 Application layer

**No change required** for #2 if adapters meet §17.2 — **`run_stock_quote_batch`**, **`apply_stock_fetch_done`**, **`resolve_quote`**, and **`get_current_price`** already consume **`TickerResponse`**.

### 17.6 Automated verification

- `cargo build --release`
- `cargo clippy -- -D warnings`
- **`cargo test`:** add **`#[cfg(test)]`** fixtures in **`yahoo.rs`**:
  - v7 JSON snippet → mapped **`TickerResult`** matches expected floats and **`t`** scaling.
  - v7 empty / error-shaped body → fallback path returns same shape as v8 success **or** returns the same error variant as today’s chart path (pick one and assert).

### 17.7 Out of scope

- WebSocket / true streaming quotes.
- Changing **`Config.refresh_rate`** throttle (#4).
- Batch **multi-symbol** v7 (`symbols=AAPL,MSFT`) — optional future optimization; current **`get_quote`** per symbol is acceptable.

### 17.8 Approval

After maintainer approval of §17, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #2 section).

### 17.9 Shipment record

- **Status:** Shipped (code + manual QA 2026-05-11) — closes [Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2). **PR:** https://github.com/FelipeMorandini/stockterm/pull/92
- **Code:** [`src/api/yahoo.rs`](../src/api/yahoo.rs) — **`yahoo_quote_v7`**, **`v7_envelope_to_ticker`**, **`yahoo_latest_quote`** (v7 then v8 **`yahoo_quote`**); unit tests for v7 JSON mapping / empty / error. [`src/api/polygon.rs`](../src/api/polygon.rs) — **`get_quote`** doc + **`limit=5`** with **`sort=desc`**.
- **Manual QA:** [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #2 section — sign-off 2026-05-11).

---

## 18. Issues #10, #42 — Alerts: add dialog, notifications, latched Status

**Sources:**

- [GitHub Issue #10](https://github.com/FelipeMorandini/stockterm/issues/10) — persistence, evaluation on refresh, input UX, optional OS notification (issue body predates several fixes; see §18.1).
- [GitHub Issue #42](https://github.com/FelipeMorandini/stockterm/issues/42) — **`draw_alerts`** Status must match persisted **`Alert.triggered`** (latched fire), not live price vs threshold.

**Related:** [#27](https://github.com/FelipeMorandini/stockterm/issues/27) / [#30](https://github.com/FelipeMorandini/stockterm/issues/30) / [#38](https://github.com/FelipeMorandini/stockterm/issues/38) / [#3](https://github.com/FelipeMorandini/stockterm/issues/3) (quote batch + **`check_alerts`** wiring — already in tree). **§15** shipped title/copy only (no notifications).

### 18.1 Tree audit vs Issue #10 (supersedes outdated checklist)

| #10 task (GitHub) | Current tree (2026-05-11 audit) | §18 action |
|-------------------|-----------------------------------|------------|
| Implement **`save_alerts`** | **`save_alerts`** assigns **`config.alerts`** and **`try_save`**, sets **`error_message`** on failure ([`src/app/alerts.rs`](../src/app/alerts.rs)). | None (verify no regressions). |
| Call **`save_alerts`** after add/remove | **`add_alert`** / **`remove_alert`** call it. | None. |
| Drive **`check_alerts`** on refresh | **`apply_stock_fetch_done`** calls **`check_alerts()`** after **`watchlist_quotes`** / portfolio price updates ([`src/app/app.rs`](../src/app/app.rs)). | None. |
| Dispatch **`handle_alerts_events`** | **`handlers.rs`** routes **`Tab::Alerts`**. | None. |
| Replace hard-coded **`(Above, 100.0)`** | Still **`add_alert(app.symbol.clone(), Above, 100.0)`** on **`a`**. | **Implement** add dialog (§18.4). |
| Bell + optional **`notify-rust`** | Not present. | **Implement** (§18.5–18.6). |
| **`Config.notifications_enabled`** | Missing. | **Add** field + Settings row (§18.3, §18.7). |
| Visually distinguish triggered vs armed | **`draw_alerts`** derives Status from **live** price vs threshold, not **`triggered`**. | **Fix** per #42 (§18.2). |

**Latch semantics (unchanged):** **`check_alerts`** remains the **only** writer that flips **`triggered`** from **`false` → `true`** when the threshold is crossed with a known quote. There is **no** “reset when price uncrosses” unless a future issue explicitly requests it.

### 18.2 Issue #42 — Status column and styling

**Problem:** [`draw_alerts`](../src/app/alerts.rs) sets **`is_triggered`** from **`current_price`** vs **`alert.price`**, while **`check_alerts`** sets **`alert.triggered`** once on first crossing and persists. After a crossing, price can move back so live comparison shows “Waiting” while JSON still has **`triggered: true`**.

**Target:**

1. **Primary Status text** — If **`alert.triggered`**: show **`TRIGGERED`** (same red emphasis as today). If **not** **`triggered`** and **`get_current_price`** returns **Some**: show **`Armed`**. If **not** **`triggered`** and **`get_current_price`** is **`None`**: show **`No quote`** in **DarkGray**.
2. **Do not** use live **`current_price > alert.price`** (or Below mirror) for the **main** Status label; optional **secondary** hint is allowed: e.g. a trailing DarkGray parenthetical **`(live)`** only for debugging — default build should keep the row to **five** columns without clutter; prefer **no** live-derived label for “fired” semantics.
3. **`models::Alert::is_triggered(price)`** may remain for tests or future “preview” UI; **`draw_alerts`** must not contradict **`alert.triggered`**.

**Files:** [`src/app/alerts.rs`](../src/app/alerts.rs) (**`draw_alerts`** only for #42; **`check_alerts`** touch only if notification hooks share the transition site).

### 18.3 Config — `notifications_enabled`

**Schema:** Add to [`Config`](../src/config/config.rs):

```rust
#[serde(default = "default_notifications_enabled")]
pub notifications_enabled: bool,
```

with **`fn default_notifications_enabled() -> bool { true }`** so existing **`~/.stockterm.json`** files deserialize without migration.

**Persistence:** Toggle commits via **`Config::try_save`** with the same **`error_message`** pattern as other settings.

### 18.4 Alert add dialog (replaces hard-coded add)

**Pattern:** Reuse the **modal overlay** approach from **`PortfolioAddDialog`** ([`src/app/portfolio.rs`](../src/app/portfolio.rs)): a small struct on **`App`** (e.g. **`Option<AlertAddDialog>`**) with **`AlertAddField`** enum **`Symbol | Condition | Threshold`**, **`inline_error`**, and **`settings_row`-style** focus cycling.

| Field | Behavior |
|-------|----------|
| **Symbol** | Initial buffer = **`normalize_symbol(&app.symbol).unwrap_or_default()`** (or empty); commit requires **`normalize_symbol`** **Some**; store uppercase in **`Alert.symbol`**. |
| **Condition** | Cycle **Above / Below** with **`;`** (and **Tab** / **Shift+Tab** if aligned with §15 portfolio dialog — same **`letter_key_plain`** / global Tab rules: if a dialog is open, tab bar must not steal Tab). |
| **Threshold** | Parse as **`f64`** \> **0** (reject NaN / inf); reuse a local parse helper or mirror **`parse_holding_decimal`** semantics where sensible. |
| **Keys** | **`Esc`** cancel (clear dialog, no mutation). **`Enter`** on last field or global “commit” key: validate → **`add_alert(symbol, condition, price)`** (existing fn sets **`triggered: false`**). |

**Handler split:** In **`handle_alerts_events`**, if **`alert_add_dialog.is_some()`**, delegate to **`handle_alert_dialog_keys`** (new **`fn`** in **`alerts.rs`**); else **`a`**/**`A`** opens dialog (instead of calling **`add_alert`** immediately). **`d`** delete behavior unchanged when dialog closed.

**Drawing:** Add **`draw_alert_add_overlay`** (or inline in **`draw_alerts`**) — bounded **`Rect`** centered or upper-third; show field labels + buffer + helper line (**`Esc`** cancel, **`Tab`** / **`;`** cycle, **`Enter`** commit).

### 18.5 Terminal bell on first fire

When **`check_alerts`** transitions **`alert.triggered`** from **`false` → `true`** (same **`updated`** batch where **`save_alerts`** runs):

- Emit **BEL** (**`\x07`**) once **per newly triggered alert** in that batch (not per tick while already true).
- Implementation: **`use std::io::{self, Write};`** **`let _ = io::stdout().write_all(b"\x07");`** **`let _ = io::stdout().flush();`** or **`crossterm::queue!`/`execute!`** with a bell-capable command — prefer **minimal** deps; BEL on raw-mode TTY is acceptable on macOS/Linux.

**Tests:** Optional unit test on a pure **`fn`** that computes “newly triggered indices” from before/after slices; bell itself is **manual QA**.

### 18.6 Desktop notification (`notify-rust`)

**Dependency:** Add **`notify-rust`** to **[`Cargo.toml`](../Cargo.toml)** (pin a current **4.x** release). **Optional:** gate behind **`[features] desktop-notify`** default **`true`** so headless/CI can **`--no-default-features`** if desktop crates cause pain — document in QA.

**Call site:** Same **`check_alerts`** transition as §18.5, **only if** **`self.config.notifications_enabled`**:

- **`Notification::new()`** (or builder) with **`summary("StockTerm")`** and **`body`** including **symbol**, **Above/Below**, **threshold**, and **last price** if known.
- **`show()`** errors: swallow or **`tracing`/eprintln!** — do **not** block the TUI loop indefinitely; if **`show()`** is synchronous and slow, run in **`std::thread::spawn`** with **`Clone`** data (symbol strings only).

**Platform note:** macOS may require terminal permissions for notifications; QA documents “allow if prompted”.

### 18.7 Settings tab — toggle row

Extend **[`SETTINGS_ROW_COUNT`](../src/app/app.rs)** and **[`draw_settings`](../src/app/ui.rs)** with a new row (recommended index **2**, renumber **Theme → 3**, **Provider → 4**, **Keymap → 5**):

- Label: **`Desktop alert toasts`** (or equivalent).
- Display **`on`/`off`** from **`config.notifications_enabled`**.
- **`settings_begin_edit` / commit:** For this row, **`Enter`** **toggles** the bool and **`try_save`** immediately (no multi-char buffer), or treat **`Enter`** as “edit mode” that flips on second **Enter** — prefer **single Enter toggles** when row selected and not in text-edit mode for consistency with boolean UX.

Update **`settings_row_prev`/`next`** bounds and **`settings_try_enter_row`** match arms.

### 18.8 Crate & module layout (Rust)

| Area | Module / type | Notes |
|------|----------------|-------|
| #42 UI | **`src/app/alerts.rs`** | **`draw_alerts`** Status from **`alert.triggered`** + quote presence. |
| Dialog | **`src/app/alerts.rs`** + **`App`** fields in **`app.rs`** | **`AlertAddDialog`**, overlay draw, key routing. |
| **`check_alerts`** | **`src/app/alerts.rs`** (`impl App`) | Bell + optional **`notify`** after mutating **`triggered`**. |
| Config | **`src/config/config.rs`** | **`notifications_enabled`** + default. |
| Settings UI | **`src/app/ui.rs`**, **`src/app/app.rs`**, **`src/app/handlers.rs`** | Row count, toggle, **`SettingsEdit`** only if text rows need enum extension — bool row may skip **`SettingsEdit`**. |

**Async:** No new **`tokio::spawn`** for alerts logic; quote batch already async. Desktop notify may use **`std::thread`** only to avoid blocking **`apply_stock_fetch_done`** for hundreds of ms.

### 18.9 Automated verification

- **`cargo build --release`**, **`cargo clippy -- -D warnings`**, **`cargo test`**.
- **Unit tests (recommended):** **`check_alerts`** — mock **`get_current_price`** via **`App`** test harness or extract a small **`fn evaluate_alerts(prices: &[(String,f64)], alerts: &mut [Alert]) -> Vec<usize>`** returning indices newly triggered for bell/notify assertions.

### 18.10 Out of scope

- Clearing **`triggered`** when price returns below/above threshold (explicit product change).
- Watchlist / quote batching / **#18** rate limits.
- Replacing **BEL** with configurable sound file.

### 18.11 Approval

After maintainer approval of §18, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #10 / #42 section).

### 18.12 Shipment record

- **Status:** Shipped (implementation + manual QA 2026-05-11) — [Issue #10](https://github.com/FelipeMorandini/stockterm/issues/10), [Issue #42](https://github.com/FelipeMorandini/stockterm/issues/42). **PR:** https://github.com/FelipeMorandini/stockterm/pull/99 — manual QA: [`docs/QA_PLAN.md`](QA_PLAN.md) (Issues #10 / #42 section).
- **Code:** [`src/app/alerts.rs`](../src/app/alerts.rs) — latched **Status**, **`AlertAddDialog`**, **`check_alerts`** bell + optional **`notify-rust`** (feature **`desktop-notify`**); [`src/app/app.rs`](../src/app/app.rs) — dialog state, **`settings_toggle_notifications`**, **`SETTINGS_ROW_COUNT`**, Tab routing; [`src/app/handlers.rs`](../src/app/handlers.rs) — **`cycle_alert_dialog_focus`** on Tab when dialog open; [`src/config/config.rs`](../src/config/config.rs) — **`notifications_enabled`**; [`src/app/ui.rs`](../src/app/ui.rs) — Settings row **2**; [`src/models/alerts.rs`](../src/models/alerts.rs) — **`process_alert_crossings`** + unit test; [`Cargo.toml`](../Cargo.toml) — optional **`notify-rust`** behind default feature.
