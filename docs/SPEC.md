# SPEC — Issues #30, #37, #38: Alerts loop + table layout

**Sources:**

- [GitHub Issue #30](https://github.com/FelipeMorandini/stockterm/issues/30) — wire `check_alerts` into main loop / tick path without blocking the UI.
- [GitHub Issue #37](https://github.com/FelipeMorandini/stockterm/issues/37) — fix `draw_alerts` table: header/row column count vs `Constraint` count mismatch.
- [GitHub Issue #38](https://github.com/FelipeMorandini/stockterm/issues/38) — invoke `check_alerts` from `App::run` at a sensible cadence; document in SPEC/QA.

**Overlap:** #30 and #38 describe the same functional gap (`App::check_alerts` never called from `App::run`). This SPEC treats them as **one implementation** with both issues closed by the same change set.

**Prerequisite:** [Issue #27](https://github.com/FelipeMorandini/stockterm/issues/27) — `save_alerts` → `Config::try_save` and `check_alerts` persistence on `triggered` transitions are already implemented in `src/app/alerts.rs`.

---

## 1. Current gaps (verified in tree)

| ID | Location | Problem |
|----|----------|---------|
| #37 | `src/app/alerts.rs` — `draw_alerts` | Header and rows expose **five** columns (`Symbol`, `Condition`, `Price`, `Current`, `Status`), but `Table::new` passes **seven** `Constraint::Length` entries with portfolio-style comments (Shares, Avg, Value, P/L, etc.). Ratatui may mis-layout or panic depending on version; layout is wrong regardless. |
| #30 / #38 | `src/app/app.rs` — `App::run` / `fetch_ticker_data` | `check_alerts` is never called, so `Alert.triggered` never flips from `false` → `true` in the running app and `save_alerts` never runs for that transition. |
| #30 / #38 | `App::run` — `Event::Tick` | Throttled `fetch_ticker_data` runs only when `active_tab == Tab::StockView`. If the user sits on **Alerts**, quotes (and thus prices used by `get_current_price`) may go stale unless they switch back to Stock View. |

**Non-goals for this milestone:** OS notifications / terminal bell, alert creation dialog, multi-symbol batch quotes for arbitrary alert symbols, changing `get_current_price` semantics beyond documenting them.

---

## 2. Crate & module layout

- **Single package:** `stockterm` (no new crates).
- **`src/app/alerts.rs`:** `draw_alerts` constraint fix; no signature change to `check_alerts` unless needed.
- **`src/app/app.rs`:** Call `check_alerts` from the quote-update path; adjust tick routing so Alerts tab keeps receiving throttled quote updates (shared throttle with Stock View).

---

## 3. Implementation plan (Rust)

### 3.1 Issue #37 — `draw_alerts` column constraints

- Replace the seven `Constraint::Length(...)` entries with **exactly five**, aligned to the header/row order:
  - Symbol — moderate width (e.g. `Length(8)` or `Min(6)`).
  - Condition — short (`Length(8)` or `Length(10)` for `"Above"`/`"Below"`).
  - Price / Current — numeric (`Length(10)`–`Length(12)`).
  - Status — `"TRIGGERED"` / `"Waiting"` (`Length(12)`–`Length(14)` or `Min(10)`).
- Prefer **mixed constraints** if readability on narrow terminals matters: e.g. `Min` for Symbol/Status and `Length` for numeric columns, matching patterns used elsewhere (e.g. portfolio table).
- **Acceptance:** constraint slice length equals **5**; table renders without layout glitches on a typical 80-column terminal.

### 3.2 Issues #30 & #38 — When to call `check_alerts`

**Semantics (existing code, do not break):**

- `check_alerts` uses `get_current_price(symbol)`, which returns a price only when:
  - `ticker_data` matches that symbol (usually the **active** `App.symbol` after a successful fetch), or
  - a **portfolio** row for that symbol has `current_price` set (back-filled when that symbol was last fetched).
- Alerts for symbols with **no** cached price are skipped until a price exists — document in QA.

**Call sites:**

1. **`fetch_ticker_data` — success path**  
   After updating `self.ticker_data` and after the existing portfolio back-fill for the active symbol (see `src/app/app.rs` today), call **`self.check_alerts()`**.  
   - Ensures every successful quote refresh re-evaluates conditions.  
   - Covers explicit refetch (`should_fetch_ticker`), initial `run()` fetch, and tick-driven fetches.

2. **`App::run` — `Event::Tick` throttled fetch**  
   Extend the condition that triggers `fetch_ticker_data` so it runs when `active_tab` is **`Tab::StockView` OR `Tab::Alerts`**, still gated by `last_stock_network_poll` and `data_poll_interval()`.  
   - Reuse **`last_stock_network_poll`** (same clock as Stock View) so we do not double API traffic when switching tabs.  
   - Rationale: users on Alerts should see `triggered` / persisted state update without forcing a tab switch.

**Async / blocking:**

- `check_alerts` remains **synchronous**, O(n) in alert count, no I/O except `save_alerts` on transition (already acceptable per #27).  
- No new `await` inside `check_alerts`; “without blocking UI” means we do **not** add extra network calls here — only hook into the existing fetch cadence.

### 3.3 Edge cases

- **Empty symbol / missing API key:** `fetch_ticker_data` returns early; `check_alerts` is not required on those paths (no new price).  
- **Fetch error:** Do not call `check_alerts` on failure if prices are cleared; optional conservative call is unnecessary — **SPEC:** invoke only on the successful branch where `ticker_data` and portfolio back-fill have been updated.  
- **Selection / table state:** `check_alerts` does not mutate `alerts_state`; no change expected.

---

## 4. Verification targets (automated)

- `cargo build --release` and `cargo clippy -- -D warnings` — clean.

---

## 5. Out of scope

- [Issue #39](https://github.com/FelipeMorandini/stockterm/issues/39) / [Issue #40](https://github.com/FelipeMorandini/stockterm/issues/40) — portfolio `try_save`, async config I/O.  
- Deduping GitHub #30 vs #38 as separate PRs — one PR may close both.

---

## 6. Approval

After maintainer approval of this SPEC, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and `docs/QA_PLAN.md`.

---

## 7. Shipment

- **Status:** Implemented; closes [Issue #30](https://github.com/FelipeMorandini/stockterm/issues/30), [#37](https://github.com/FelipeMorandini/stockterm/issues/37), [#38](https://github.com/FelipeMorandini/stockterm/issues/38). Manual QA per [`docs/QA_PLAN.md`](QA_PLAN.md) (with known limits: symbol entry, hard-coded alert add).
- **Also in branch:** Polygon aggregate `v` (volume) deserialized as `f64` (`TickerResult`, `HistoricalData`) after live API returned fractional volume; display uses rounded whole numbers.
- **Deferred:** [#42](https://github.com/FelipeMorandini/stockterm/issues/42) (Status vs `triggered`), [#43](https://github.com/FelipeMorandini/stockterm/issues/43) (block titles), [#44](https://github.com/FelipeMorandini/stockterm/issues/44) (Shift/symbol keys). Non-blocking UI: [#17](https://github.com/FelipeMorandini/stockterm/issues/17); alert dialog / UX: [#10](https://github.com/FelipeMorandini/stockterm/issues/10).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/45
