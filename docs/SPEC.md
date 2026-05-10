# SPEC — Issue #3: Multi-symbol watchlist & multi-row quote table

**Sources:**

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

- **Status:** Implemented in tree for [Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44). Manual verification: [`docs/QA_PLAN.md`](QA_PLAN.md) (Issue #44 section).
- **Code:** `src/app/keyboard.rs` (`letter_key_plain`), updates to [`src/app/handlers.rs`](../src/app/handlers.rs) and [`src/app/alerts.rs`](../src/app/alerts.rs).
