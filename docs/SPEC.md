# SPEC — StockTerm (Issue #3 baseline + follow-ons)

**Issue #3** — Multi-symbol watchlist & multi-row quote table (§§1–7). **Issue #44** — Stock View & Alerts keyboard modifiers (§8, shipped). **Issues #48 / #6** — Portfolio tab: keyboard parity (§12, shipped); add dialog, confirm remove, quote coverage (§13, shipped). **Issue #31** — Yahoo Finance default provider & Polygon fallback (§9, shipped). **Issues #29 / #5 / #11 / #12** — Search typeahead, News list, Settings editor (§10, shipped — see §10.9 PR). **Issues #9 / #8 / #7** — Historical time ranges, chart viewport (zoom/pan), real candlestick widget (§11, shipped — see §11.10 PR).

**Sources (Issue #3):**

- [GitHub Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3) — `Watchlist` in config, fan-out quotes, Stock View table, navigation, persistence, bounded concurrency, non-blocking refresh.

**Related issues (dependencies / alignment):**

- [#4](https://github.com/FelipeMorandini/stockterm/issues/4) — `Config.refresh_rate` drives quote refresh cadence (seconds); UI tick stays fast (~200 ms).
- [#17](https://github.com/FelipeMorandini/stockterm/issues/17) — Network I/O must not sit inline between redraws; input stays responsive during slow API.
- [#18](https://github.com/FelipeMorandini/stockterm/issues/18) — Shared HTTP client, timeouts, 429/backoff, concurrency cap (this SPEC adopts a **minimal** cap for watchlist fan-out; full `ProviderError` work can extend #18).
- [#19](https://github.com/FelipeMorandini/stockterm/issues/19) — Surface `Config::try_save` failures via `App.error_message`; avoid silent persistence loss for watchlist edits.

**Overlap note:** Issue #3 acceptance says refresh must respect `refresh_rate` and not block input. Today `App::run` still `await`s `fetch_*` on the main path ([`src/app/app.rs`](../src/app/app.rs)), so **meeting the full acceptance bar implies completing or partially landing #17 in the same delivery as #3** (or immediately before). This SPEC describes the target architecture assuming that constraint is satisfied.

---

## 1. Current gaps (verified in tree)

| Area | Location | Problem |
|------|----------|---------|
| Single symbol | `App::symbol` only | No persisted list; Stock View is a single-symbol paragraph ([`src/app/ui.rs`](../src/app/ui.rs) `draw_stock_view`). |
| Config | [`src/config/config.rs`](../src/config/config.rs) | No `watchlist` field; older JSON files must still deserialize after adding the field (`serde(default)`). |
| Quote cache | `App::ticker_data: Option<TickerResponse>` | Only one response; watchlist needs per-symbol quote cache for the table **and** for `get_current_price` ([`src/app/alerts.rs`](../src/app/alerts.rs)) for non-active alert symbols. |
| Keys | [`src/app/handlers.rs`](../src/app/handlers.rs) `handle_stock_view_keys` | No `w` / remove / table navigation. |
| Fan-out | `fetch_ticker_data` | Single `get_ticker_data(&self.symbol, …)` only. |
| Blocking | `App::run` | Await on fetch between `draw` and `events.next()`; multi-symbol makes stalls worse without #17. |

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

- **Requirement:** No `await` on `get_ticker_data` (or other HTTP) on the synchronous path between `terminal.draw` and processing the next user input event.
- **Target pattern:** Spawn fetch work via `tokio::spawn` (or a dedicated worker task); send results through `tokio::sync::mpsc` (or `watch` channel); `App::run` uses `tokio::select!` between **input**, **tick**, and **fetch result** messages. Bridge crossterm input from a thread or blocking task into async channels as described in #17.
- **Loading:** Status bar shows a short “Refreshing…” or spinner tick while a fan-out is in flight; redraws continue.

If #17 is not ready, document **interim** blocking behavior in QA as **fail** for the non-blocking checklist until fixed.

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
- **Follow-ups:** [Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) — specified in **§8** below (Stock View / Alerts modifier keys). [Issue #46](https://github.com/FelipeMorandini/stockterm/issues/46) (quote batch panic-safety), [#17](https://github.com/FelipeMorandini/stockterm/issues/17) / [#18](https://github.com/FelipeMorandini/stockterm/issues/18) (full non-blocking + API hardening).

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
| Gating | [`src/app/app.rs`](../src/app/app.rs) | **`polygon_key_configured()`** blocks **`spawn_stock_fetch_task`**, **`try_spawn_historical_fetch`**, **`try_spawn_news_fetch`**, and sync **`fetch_historical_data` / `search_symbols` / `fetch_news`** — unusable without a key. |
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

**Date inputs:** Call sites today pass **`from_date`**, **`to_date`** as **`YYYY-MM-DD`** strings ([`App::fetch_historical_data`](../src/app/app.rs), **`try_spawn_historical_fetch`**). Parse with **`chrono::NaiveDate`**, convert to UTC midnight timestamps **consistently** (document: use **UTC** boundary **or** US market calendar — pick **UTC midnight** for simplicity; note intraday drift in comments).

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

**`try_spawn_historical_fetch`**, **`try_spawn_news_fetch`**, **`fetch_historical_data`**, **`search_symbols`**, **`fetch_news`:**

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
| Historical window | [`try_spawn_historical_fetch`](../src/app/app.rs), [`fetch_historical_data`](../src/app/app.rs) | Hard-coded **30 days**, **`"day"`** only. |
| Yahoo history | [`YahooProvider::get_historical`](../src/api/yahoo.rs) | Rejects **`timespan != "day"`**; URL uses **`interval=1d`** only. |
| Polygon history | [`PolygonProvider::get_historical`](../src/api/polygon.rs) | **`range/1/{timespan}/`** — supports Polygon **`minute` / `hour` / `day`** (etc.) per API; today call site always passes **`"day"`**. |
| Charts keys | [`handlers.rs`](../src/app/handlers.rs) `Tab::Charts` | **No** tab-local handler — must add `handle_charts_events`. |
| Line chart | [`draw_charts`](../src/app/charts.rs) | Full-series min/max x/y; no viewport. |
| Candlestick | [`draw_candlestick`](../src/app/charts.rs) | OHLC **text table**; unused from [`ui.rs`](../src/app/ui.rs). |

---

### 11.1 Recommended delivery order

1. **#9 (data contract)** — Introduce `TimeRange`, map to `(from, to, bar_resolution)` per provider, extend **`get_historical`** (or add a parallel method) so Yahoo can request **`interval=1m`** / **`5m`** / **`1d`** / **`1wk`** via v8 chart. Wire `try_spawn_historical_fetch` / `fetch_historical_data` to use `App.time_range`. Add Charts tab range keys and on-range-change **invalidate / refit** viewport (step 2).
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
- **Follow-ups:** [#62](https://github.com/FelipeMorandini/stockterm/issues/62) (stale chart vs symbol), [#63](https://github.com/FelipeMorandini/stockterm/issues/63) (Yahoo W1 empty fallback), [#64](https://github.com/FelipeMorandini/stockterm/issues/64) (fetch resilience), [#65](https://github.com/FelipeMorandini/stockterm/issues/65) (Polygon limits / payload size).
- **Behavior note (post-audit):** Periodic historical refresh preserves zoom/pan via `chart_viewport_after_refresh` unless the view was full-range or the ticker changed; see `src/app/charts.rs`.

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

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #48 section); closes [#48](https://github.com/FelipeMorandini/stockterm/issues/48) (same PR as §13).
- **Code:** [`src/app/portfolio.rs`](../src/app/portfolio.rs) — `letter_key_plain` on Portfolio `a`/`d`/armed keys + **`j`**/**`k`** navigation.
- **Follow-ups:** [#67](https://github.com/FelipeMorandini/stockterm/issues/67) (Tab cycles dialog fields vs global tab bar).

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
- **`;`** (semicolon, no modifiers) cycles **focus** between Shares and Price fields (**`Tab`** stays global for the app tab bar — see §13.4); **Enter** on **Shares** moves to **Price**; **Enter** on **Price** **commits**; **Esc** **cancels** (clear dialog state, no mutation).
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
- **`src/app/handlers.rs`:** No change if Portfolio stays self-contained; ensure **global** keys (`q`, Tab) still behave — dialog should **not** swallow **Tab** if Tab switches app tabs; prefer **Esc** to close dialog rather than overloading Tab (match Settings pattern).

### 13.3 Quote batch — include all portfolio symbols

Extend **`collect_symbols_for_quote_fetch`** to iterate **`self.portfolio`** and push **normalized** `item.symbol` into the same **deduped** list as watchlist + `symbol`. Order: existing watchlist order, then `symbol`, then portfolio symbols not yet seen (stable order aids debugging). Keeps **`MAX_CONCURRENT_QUOTES`** behavior unchanged.

### 13.4 Input routing precedence

When `portfolio_dialog.is_some()`:

1. **Esc** → cancel dialog, clear buffers.
2. **Field cycle:** **`;`** (semicolon, no modifiers) cycles `focused` between Shares and Price. **`Tab`** is reserved for the global tab bar (handled before tab-local input); do not rely on Tab inside the dialog.
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

- **Status:** Shipped — manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #6 section); closes [#6](https://github.com/FelipeMorandini/stockterm/issues/6).
- **Code:** [`src/app/portfolio.rs`](../src/app/portfolio.rs) (`PortfolioAddDialog`, overlay, two-step remove); [`src/app/app.rs`](../src/app/app.rs) (`collect_symbols_for_quote_fetch` includes portfolio symbols; `add_to_portfolio` / `remove_from_portfolio` + **`try_save`**).
- **Related closure:** [#39](https://github.com/FelipeMorandini/stockterm/issues/39) (portfolio **`try_save`** parity — addressed in same delivery).
- **Follow-ups:** [#67](https://github.com/FelipeMorandini/stockterm/issues/67) (Tab in dialog), [#68](https://github.com/FelipeMorandini/stockterm/issues/68) (decimal money), [#69](https://github.com/FelipeMorandini/stockterm/issues/69) (commit edge cases / input caps).

---

## 14. Issue #44 — reference (shipped)

**Issue #44** is **closed**; behavior is specified in **§8** and verified in [`docs/QA_PLAN.md`](QA_PLAN.md). **§12** and **§13** must stay consistent with §8 for modifier semantics on letter keys.
