# QA Plan — Manual verification

Use the sections below per milestone. **Issue #3** remains the regression baseline for the watchlist; **Issue #44** adds keyboard modifier behavior (Stock View / Alerts). **Issues #48 / #6** extend modifier parity and portfolio add/remove UX on the Portfolio tab (see [`docs/SPEC.md`](SPEC.md) §§12–13). **Issue #31** covers the Yahoo/Polygon provider adapter and structured errors. **Issues #29 / #5 / #11 / #12** cover the Search, News, and Settings tabs (M3). **Issues #9, #8, #7** cover Charts time ranges, zoom/pan, and candlesticks (M4 — see [`docs/SPEC.md`](SPEC.md) §11). **Issues #62, #63, #64** cover M4 Charts polish (symbol/series coherence, Yahoo W1 fallback, fetch resilience — see [`docs/SPEC.md`](SPEC.md) §11.11). **Issues #71, #72, #73, #74** cover M4 follow-up hardening (inflight/channel parity, dead historical helper removal, W1 unit tests, watchlist chart flicker — see [`docs/SPEC.md`](SPEC.md) §11.12). **Issues #43, #49, #50, #67, #69** cover Alerts title/copy, Stock View typing hint, Portfolio dialog Tab focus, and commit validation (see [`docs/SPEC.md`](SPEC.md) §15). **Issues #17, #46, #77** cover async loop close-out, quote-batch panic hardening, and pending-flag behavior on stock recovery (see [`docs/SPEC.md`](SPEC.md) §16). **Issue #2** covers latest-session quote adapters (Yahoo v7 primary + v8 fallback, Polygon daily latest bar — see [`docs/SPEC.md`](SPEC.md) §17). **Issues #10, #42** cover Alerts add dialog, bell + desktop notifications, Settings toggle, and latched Status vs `triggered` (see [`docs/SPEC.md`](SPEC.md) §18). **Issues #93, #94, #95** cover shared modal `centered_rect`, alert Condition **←/→** keys, and optional stderr for desktop **`show()`** outcomes (see [`docs/SPEC.md`](SPEC.md) §18.13 — manual sign-off 2026-05-12). **Issues #96, #97, #98** cover alerts **`try_save`** failure UX, one coalesced desktop toast per crossing batch, and sanitized notification text (see [`docs/SPEC.md`](SPEC.md) §18.14 — [PR #105](https://github.com/FelipeMorandini/stockterm/pull/105); run the **Issues #96, #97, #98** section for manual sign-off). **Issues #100, #101, #104** cover `centered_rect` percent contract, README debug env documentation, and total notify **`body`** byte cap (see [`docs/SPEC.md`](SPEC.md) §18.15).

## Issues #7, #8, #9 — M4: Charts (candlesticks, viewport, time ranges)

**Scope:**

- [Issue #9](https://github.com/FelipeMorandini/stockterm/issues/9) — `TimeRange` **D1/W1/M1/Y1**, keys **`1`–`4`**, provider windows + intraday/daily bars, title reflects range, viewport resets on range change.
- [Issue #8](https://github.com/FelipeMorandini/stockterm/issues/8) — Zoom **`+`/`-`**, pan **`h`/`l`** (and/or arrows), reset **`0`**, y-axis from visible window, visible dates in UI, clamped at data edges.
- [Issue #7](https://github.com/FelipeMorandini/stockterm/issues/7) — Real candlestick **`Widget`**, green/red bodies + wicks, **`c`** toggles line vs candles, graceful empty/single-point handling.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §11.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Time ranges (#9)

1. **`cargo run --release`**, select a liquid symbol (**`AAPL`**), open **Charts**.

2. Press **`3`** (or the key bound to **M1** per SPEC).  
   **Pass:** Chart loads ~1 month of **daily** (or documented) bars; title/status shows **M1** (or equivalent label).

3. Press **`1`** (**D1**).  
   **Pass:** Chart switches to **intraday** bars (multiple bars for one session window); no crash; label shows **D1**.

4. Press **`2`**, **`4`** in turn.  
   **Pass:** **W1** and **Y1** views load or show a clear **empty/error** message; symbol unchanged (no re-type).

5. **Yahoo (default):** Repeat smoke on **`provider`: `yahoo`**.  
   **Pass:** Same behaviors; no “only daily supported” error.

6. **Polygon:** Set **`provider`: `polygon`** with valid key; repeat **`1`–`4`**.  
   **Pass:** Data or readable error; no panic.

### Manual — Viewport / zoom / pan (#8)

1. On **M1** or **Y1** with enough bars, press **`+`** several times.  
   **Pass:** Visible window **narrows** around the center; prices rescale (y-axis fits visible highs/lows).

2. Press **`-`**.  
   **Pass:** Window **widens** toward full range.

3. Press **`h`** / **`l`** (or arrows if implemented).  
   **Pass:** Chart **pans**; at first/last bar, no crash and no garbage off-screen.

4. Press **`0`**.  
   **Pass:** Full series visible again.

5. Change range with **`1`** then **`3`**.  
   **Pass:** Viewport **resets** to full new series (per SPEC §11.4).

### Manual — Candlesticks (#7)

1. Press **`c`** to switch to **candlestick** mode.  
   **Pass:** Bodies and wicks visible; **up** vs **down** color distinction clear.

2. Press **`c`** again.  
   **Pass:** Returns to **line** chart without restart.

3. Zoom/pan in candlestick mode.  
   **Pass:** Same viewport keys affect candles; y-bounds still track visible window.

4. **Edge cases:** Symbol with **no** history, or **one** bar — **Pass:** Explanatory message; **no panic**.

### Manual — Regression

1. **Stock View / watchlist** after Charts session — **Pass:** Unchanged.

2. **Chord safety:** On Charts, **`Ctrl+h`** does not pan (if SPEC requires plain keys only).  
   **Pass:** Matches handler rules.

### Sign-off — M4 (#7 / #8 / #9)

_Manual validation passed 2026-05-10 (pre-merge). Viewport preserved across background chart refresh; `3` on default M1 forces refresh._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-10 | Pass |
| Time range keys + labels (#9) | maintainer | 2026-05-10 | Pass |
| D1 intraday bars (#9) | maintainer | 2026-05-10 | Pass |
| Yahoo + Polygon smoke (#9) | maintainer | 2026-05-10 | Pass |
| Zoom / pan / reset (#8) | maintainer | 2026-05-10 | Pass |
| Viewport + range change (#8/#9) | maintainer | 2026-05-10 | Pass |
| Candlestick toggle (#7) | maintainer | 2026-05-10 | Pass |
| Empty / single-point (#7/#9) | maintainer | 2026-05-10 | Pass |

---

## Issues #62, #63, #64 — M4 Charts polish

**Scope:**

- [Issue #62](https://github.com/FelipeMorandini/stockterm/issues/62) — No mismatch between chart title / active symbol and the OHLC series after changing symbol **without** visiting Charts (clear or gate stale series per [`docs/SPEC.md`](SPEC.md) §11.11.1).
- [Issue #63](https://github.com/FelipeMorandini/stockterm/issues/63) — Yahoo **W1**: if intraday window is empty, automatic **daily** retry for the same window; illiquid symbols still get a chart when possible.
- [Issue #64](https://github.com/FelipeMorandini/stockterm/issues/64) — Transient historical errors: last-good series + error message; viewport logic uses requested symbol when response **`ticker`** is empty; document UX for background fetch behavior.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §11.11.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Issue #62 (symbol vs series)

1. Open **Charts**, load data for a liquid symbol (**`AAPL`**), press **`3`** (M1) and wait until bars appear. Note the chart is populated.

2. Switch to **Search** (or **Stock View**), pick a **different** symbol (e.g. search **`MSFT`**, **Enter** to go to Stock View). **Do not** open Charts yet.

3. Switch to **Charts**.  
   **Pass:** Chart title / active symbol matches **MSFT** (or the new symbol) and the **plotted series** is for that symbol — **not** a frozen **AAPL** image. Acceptable: empty / loading state until fetch completes, but **not** the previous ticker’s bars.

4. Repeat using **watchlist** navigation: load **AAPL** on Charts, go to Stock View, **`j`/`k`** to another watchlist row, return to **Charts**.  
   **Pass:** Same — no old ticker’s bars under the new title.

5. From **Portfolio**, select a row whose symbol differs from the last charted symbol, press **Enter** to jump to Stock View, then open **Charts**.  
   **Pass:** Series matches the portfolio row’s symbol.

### Manual — Issue #63 (Yahoo W1 empty fallback)

1. Set **`provider`: `yahoo`** in **`~/.stockterm.json`**. Use a symbol that often has **sparse** activity (e.g. a low-volume OTC or thin ETF — pick one you know can return empty intraday; if hard to find, use a mock/stub only in dev — then skip and note **N/A** in sign-off).

2. Open **Charts**, press **`2`** (**W1**).  
   **Pass:** Either intraday **or** (after fallback) **daily** bars appear for the week window, **or** a clear empty message — **no panic**, no infinite spinner.

3. Switch to a liquid symbol (**`AAPL`**), **W1** again.  
   **Pass:** Chart still behaves; primary path unchanged.

### Manual — Issue #64 (resilience + UX)

1. **Transient error / last-good:** With a **loaded** chart (any range with visible bars), simulate a network failure (e.g. disable Wi-Fi / unplug Ethernet) and trigger a **refresh** of historical data (e.g. wait for periodic refresh on Charts, change range and back, or use whatever UX forces refetch per implementation).  
   **Pass:** Per [`docs/SPEC.md`](SPEC.md) §11.11.3 — an **error** appears in the status / error line, and the **previous** bars remain visible (not wiped) until a **successful** fetch replaces them. Restore network; confirm a successful fetch **clears** the error and updates data.

2. **First-load failure:** With network off, open the app, go to **Charts** for a symbol with no cached history.  
   **Pass:** No crash; empty or error state is consistent with SPEC (no fake bars).

3. **Regression — symbol change still clears stale data:** After a failed refresh with last-good series, change symbol via Search (**#62** scenario).  
   **Pass:** Old series is **not** combined with the new symbol (same as §11.11: clearing on symbol change).

### Sign-off — Issues #62 / #63 / #64

_Manual validation passed 2026-05-11._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-11 | Pass |
| #62 Search → Charts mismatch | maintainer | 2026-05-11 | Pass |
| #62 watchlist / Portfolio Enter | maintainer | 2026-05-11 | Pass |
| #63 W1 fallback (Yahoo) | maintainer | 2026-05-11 | Pass |
| #64 transient error + last-good | maintainer | 2026-05-11 | Pass |
| #64 first-load failure | maintainer | 2026-05-11 | Pass |

---

## Issues #71, #72, #73, #74 — M4 Charts / async follow-ups

**Scope:**

- [Issue #71](https://github.com/FelipeMorandini/stockterm/issues/71) — If a background fetch cannot **`send`** its `FetchDone` (or stock batch) result, the UI must not leave **`hist_refresh_inflight`** / **`stock_refresh_inflight`** / **`news_refresh_inflight`** / **`search_refresh_inflight`** stuck; logging is consistent (see [`docs/SPEC.md`](SPEC.md) §11.12.1).
- [Issue #72](https://github.com/FelipeMorandini/stockterm/issues/72) — No unused duplicate historical loader in production; charts use only the async `FetchDone` path (§11.12.2).
- [Issue #73](https://github.com/FelipeMorandini/stockterm/issues/73) — `cargo test` covers Yahoo W1 empty intraday → daily fallback decision (§11.12.3).
- [Issue #74](https://github.com/FelipeMorandini/stockterm/issues/74) — Adding the current symbol to the watchlist when normalization only fixes **case** does not clear the Charts series (§11.12.4).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §11.12.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; tests include **#73** scenarios (Yahoo W1 fallback table).

### Manual — Issue #71 (inflight / channel behavior)

_Normal failure of the `FetchDone` channel is abnormal during a normal run (receiver should stay alive). Validation is mostly regression + code review; optional stress._

1. **Regression — Charts refresh:** Open **Charts**, load **AAPL**, switch ranges, wait for periodic refresh. **Pass:** Chart continues to update across multiple poll cycles; no permanent “stuck loading” where historical never refetches.
2. **Regression — Search / News / Stock batch:** Typeahead on **Search**, **News** list load, **Stock View** quote refresh after tab switches. **Pass:** No tab remains permanently blocked by a spinner / inflight state after errors or slow network (same as pre–#71, but confirm no new stalls).
3. **Optional (maintainer):** If a debug hook exists to drop the fetch receiver, confirm **#71** recovery clears inflight and the app remains usable — **N/A** if no hook.

### Manual — Issue #72 (single historical pipeline)

1. **Smoke:** **Charts** time ranges, zoom/pan, and **#64** last-good behavior still work after removing **`fetch_historical_data`**. **Pass:** No behavior regression vs §11.11 QA.

### Manual — Issue #74 (watchlist add / case normalization)

1. Open **Stock View**. Type **`aapl`** (lowercase) so the buffer shows **`AAPL`** or mixed case per UX; ensure the symbol is **not** yet on the watchlist.
2. Press **`w`** to add to the watchlist. **Pass:** Row is added and symbol normalizes to **`AAPL`**; **Charts** (if you had a loaded **AAPL** chart) does **not** flash empty / full reload solely because of case normalization — series should remain unless the implementation intentionally refetches in place.
3. Add a **different** symbol via **`w`** (e.g. after switching to **MSFT**). **Pass:** **#62** still applies — chart clears stale series when the **effective** ticker changes.

### Sign-off — Issues #71 / #72 / #73 / #74

_Manual validation passed 2026-05-11._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-11 | Pass |
| #71 Charts / Search / News / Stock inflight regression | maintainer | 2026-05-11 | Pass |
| #72 Charts smoke (historical path only) | maintainer | 2026-05-11 | Pass |
| #73 unit tests present in `cargo test` | maintainer | 2026-05-11 | Pass |
| #74 watchlist add case-only (`aapl` → `AAPL`) | maintainer | 2026-05-11 | Pass |
| #74 real symbol change still clears chart | maintainer | 2026-05-11 | Pass |

---

## Issues #17, #46, #77 — Async main loop polish

**Scope:**

- [Issue #17](https://github.com/FelipeMorandini/stockterm/issues/17) — Confirm non-blocking architecture and **5 s+ artificial delay** smoke (see [`docs/SPEC.md`](SPEC.md) §16.1).
- [Issue #46](https://github.com/FelipeMorandini/stockterm/issues/46) — Quote batch remains usable after panics / join errors; **`stock_refresh_inflight`** never stuck; stale-generation path documented (§16.2).
- [Issue #77](https://github.com/FelipeMorandini/stockterm/issues/77) — **`InflightRecovery::Stock`** reconciles **`stock_refresh_pending`** per SPEC §16.3 (recommended: drain pending into **`request_immediate_stock_poll`**).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §16.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

2. If §16.2 adds unit tests for panic-safe completion, **Pass:** those tests are present and green.

### Manual — Issue #17 (responsive UI under slow quotes)

1. Build/run with **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** set to **≥ 5000** (milliseconds), e.g. `STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release` (see [`docs/SPEC.md`](SPEC.md) §16.1). Unset or **0** for normal runs.
2. On **Stock View**, trigger a quote refresh (**Enter** or wait for throttle). While the table shows loading / stale data, rapidly press **Tab** (other tabs), **`j`/`k`** on the watchlist, type letters into the symbol buffer, and **Backspace**.
3. **Pass:** Keystrokes keep changing tabs / selection / buffer; UI keeps redrawing (tick-driven updates); the app does **not** freeze for the full delay on the main thread.
4. **Symbol supersede:** With delay on, start a refresh for symbol **A**, then switch symbol / watchlist row to **B** before the first batch completes. **Pass:** When results land, **B**’s row / detail reflects **B** (or a clear error for **B**); **A**’s stale batch does **not** overwrite **B**’s cache (**generation** / SPEC §16.1).
5. **Code review (maintainer):** Confirm no provider **`await`** sits between **`draw`** and the next **`tokio::select!`** input arm in **`App::run`**.

### Manual — Issue #46 (inflight + panic regression)

1. **Normal run:** Watchlist with ≥2 symbols, Yahoo default, **Stock View** for ~2 minutes. **Pass:** Quotes keep updating on throttle; **`stock_refresh_inflight`** never stays stuck after errors (status bar / table recover).
2. **Maintainer-only (optional):** If a test hook forces a panic inside the quote batch, **Pass:** after the hook, the next **`Enter`** or tick-driven poll still runs (inflight cleared via synthetic result or recovery — per §16.2). **N/A** if no hook.

### Manual — Issue #77 (`stock_refresh_pending` + recovery)

1. **Documented product choice:** Read §16.3 in SPEC (option **A** vs **B**) and note which shipped.
2. **Regression proxy (no channel drop in normal use):** Trigger **`request_immediate_stock_poll`** twice quickly while a batch is in flight (e.g. double **Enter** or **Enter** after an action that calls **`request_immediate_stock_poll`**). **Pass:** When the first batch completes, a **second** batch runs if coalescing promised one; quotes eventually match latest symbol set; **`stock_refresh_pending`** is **false** after the sequence settles (no permanent “pending” with **`stock_refresh_inflight` false** and no further polls).
3. **If a maintainer debug path drops only the fetch receiver** (same class of failure as #71): **Pass:** after recovery, pending coalesced refresh is handled per §16.3 — **N/A** if no hook.

### Sign-off — Issues #17 / #46 / #77

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | | | |
| #17 slow-network smoke (≥5 s delay) | | | |
| #17 supersede / stale generation | | | |
| #46 inflight never stuck (normal + optional panic hook) | | | |
| #77 pending vs `InflightRecovery::Stock` (per §16.3) | | | |

_Shipment PR: [#88](https://github.com/FelipeMorandini/stockterm/pull/88)._

---

## Issue #2 — Latest-session quotes (Yahoo + Polygon)

**Scope:**

- [Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2) — Stock View and watchlist show **current trading-session** (or last close) prices, not a stale fixed-year snapshot; **`TickerResult`** unchanged at UI; no fixed calendar-year literals in **`src/api/`** for live quotes.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §17.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

2. **Regression grep (maintainer):** No fixed multi-year **live-quote** date literals in `src/api/` (per SPEC §17.2). Example check: search the tree for patterns like `2023-01-01` through `2023-12-31` used as quote window endpoints — **Pass:** none for quote paths (historical `period1`/`period2` builders may still parse user-facing dates).

3. **Unit tests:** §17.6 fixtures for Yahoo v7 mapping + empty v7 → v8 fallback — **Pass:** present and green after implementation.

### Manual — Yahoo (default `provider`)

1. **`cargo run --release`**, **Stock View**, symbols **`AAPL`**, **`MSFT`**, **`SPY`** in turn (**Enter** or watchlist row so each is active).  
   **Pass:** Detail pane **Price** / **Change** / **Open** / **High** / **Low** / **Volume** look plausible vs a public finance page for the **same calendar day** (intraday vs last close is acceptable per SPEC).

2. During **US regular session** (if available): pick **`SPY`**, wait one **`refresh_rate`** cycle, note **Price**; wait another cycle.  
   **Pass:** Values may move or stay flat, but do **not** look like an ancient static fixture (e.g. unchanged for days while the market moved sharply).

3. **After hours / weekend:** same three symbols.  
   **Pass:** Prices reflect **last regular session** (or documented extended-hours behavior in adapter comments), not empty/garbage.

### Manual — Polygon (`"provider": "polygon"` + API key)

1. Set **`provider`** to **`polygon`** and a valid key in **`~/.stockterm.json`**. Restart, repeat **AAPL** / **MSFT** / **SPY** on **Stock View**.  
   **Pass:** Same plausibility checks as Yahoo; errors are readable if the key is invalid.

2. **Pass:** **`latest_result()`** bar used for the table/detail is the **most recent** bar in the adapter response (no obvious off-by-years date in volume or price magnitude).

### Manual — Symbol switch

1. With watchlist rows for **two** symbols, use **`j`/`k`** to switch the highlighted row.  
   **Pass:** Detail pane **Open/High/Low/Volume** update to match the **newly selected** symbol without restarting the app.

### Sign-off — Issue #2

_Manual validation passed 2026-05-11._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-11 | Pass |
| Regression grep (no stale fixed-year quote windows in `src/api/`) | maintainer | 2026-05-11 | Pass |
| Yahoo AAPL / MSFT / SPY plausibility | maintainer | 2026-05-11 | Pass |
| Yahoo session vs after-hours | maintainer | 2026-05-11 | Pass |
| Polygon optional smoke | maintainer | 2026-05-11 | Pass |
| Symbol switch updates OHLCV | maintainer | 2026-05-11 | Pass |

---

## Issues #10, #42 — Alerts: dialog, notifications, latched Status

**Scope:**

- [Issue #10](https://github.com/FelipeMorandini/stockterm/issues/10) — Add dialog (symbol, condition, threshold); terminal bell on first threshold cross; optional desktop toast via **`notify-rust`**, gated by **`notifications_enabled`** (default **true**); Settings row to toggle toasts; regressions on existing **`save_alerts`** / **`check_alerts`** / handler wiring.
- [Issue #42](https://github.com/FelipeMorandini/stockterm/issues/42) — Alerts table **Status** column reflects **`Alert.triggered`** (latched), not live price vs threshold; **Armed** when not triggered and a quote exists; **No quote** when **`get_current_price`** is missing.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §18.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0. If **`notify-rust`** is behind a Cargo feature, document the exact **`cargo test`** invocation used in CI (e.g. **`--no-default-features`** vs default).

### Manual — Issue #42 (Status vs JSON)

1. Add a liquid symbol to the watchlist (**`AAPL`**). Open **Alerts**, use **`a`** and create an **Above** alert with threshold **well below** the current quote (e.g. **Above $1.00**). Wait at least one quote refresh cycle.  
   **Pass:** **`check_alerts`** sets **`triggered: true`** in **`~/.stockterm.json`** (inspect file); **Status** shows **TRIGGERED** (red).

2. Without removing the alert, hand-edit **`~/.stockterm.json`**: set **`"triggered": true`** and set **`"price"`** to a value **above** the real market (so live price is *below* threshold). Restart the app, open **Alerts**.  
   **Pass:** **Status** remains **TRIGGERED** (not “Waiting” / armed based on live math). **Current** column may show the real last price.

3. Reset the JSON to a sane **Above** threshold again with **`triggered": false`**, restart.  
   **Pass:** **Status** shows **Armed** while quotes exist, until the first real crossing fires again.

### Manual — Issue #10 (add dialog + persistence)

1. On **Alerts**, press **`a`**.  
   **Pass:** A modal dialog appears (not an immediate silent add at $100).

2. Set **symbol** to **`MSFT`**, **Below**, threshold **1000** (or any value you can later cross with a fake JSON test if needed), **`Enter`** to commit.  
   **Pass:** Row appears; **`~/.stockterm.json`** lists the alert with correct **symbol / condition / price**; restart app — row still present.

3. **`Esc`** while the dialog is open (before commit).  
   **Pass:** Dialog closes; no new row.

4. Invalid threshold (**`0`**, **`-1`**, empty) on commit.  
   **Pass:** Inline error; no row added.

### Manual — Bell and desktop toast

1. Create an **Above** alert with threshold **just under** the current live price (so the next refresh is likely to cross). **`notifications_enabled`** **true** (default).  
   **Pass:** On first transition to **TRIGGERED**, terminal emits a **bell** (audible or visible flash, depending on terminal). If OS permissions allow, a **desktop notification** appears with symbol + condition text.

2. Toggle **Desktop alert toasts** (or equivalent Settings row per §18.7) **off**, **`try_save`** succeeds, repeat a **new** alert fire (use a fresh symbol or reset **`triggered`** in JSON).  
   **Pass:** **Bell** still fires per §18.5; **no** desktop toast (or documented platform limitation).

### Manual — Regression (#15 / §8)

1. **Alerts** tab: **`d`** removes selected row; config updates. **`a`**/**`A`** with Shift still opens add per §8.  
   **Pass:** No panic; watchlist quote batch still updates **Current** column.

### Sign-off — Issues #10, #42

_Manual validation passed 2026-05-11._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-11 | Pass |
| #42 Status latched vs JSON | maintainer | 2026-05-11 | Pass |
| #42 TRIGGERED when live would disagree | maintainer | 2026-05-11 | Pass |
| #10 dialog open / commit / Esc | maintainer | 2026-05-11 | Pass |
| #10 persistence across restart | maintainer | 2026-05-11 | Pass |
| Bell + toast toggle | maintainer | 2026-05-11 | Pass |
| Regression Alerts keys | maintainer | 2026-05-11 | Pass |

---

## Issues #93, #94, #95 — Alerts follow-ups (shared layout, Condition arrows, notify debug)

**Scope:**

- [Issue #93](https://github.com/FelipeMorandini/stockterm/issues/93) — single **`centered_rect`** helper for portfolio + alert add overlays; behavior and per-dialog **percent_y** unchanged.
- [Issue #94](https://github.com/FelipeMorandini/stockterm/issues/94) — on **Condition** focus, **Left** / **Right** (no modifiers) set **Below** / **Above** per [`docs/SPEC.md`](SPEC.md) §18.13.2; overlay copy documents **←/→**; **`;`** / **`a`**/**`b`** unchanged.
- [Issue #95](https://github.com/FelipeMorandini/stockterm/issues/95) — when **`STOCKTERM_DEBUG_ALERT_NOTIFY=1`**, **`eprintln!`** the **`Result`** from **`Notification::show()`** inside the notify thread (feature **`desktop-notify`**).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §18.13.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

2. If CI or local matrix covers **`--no-default-features`**, also:

   ```bash
   cargo clippy --no-default-features -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All invoked commands exit **0**.

### Manual — Issue #93 (modal layout parity)

1. Use a large terminal (e.g. **≥** 100×30). Open **Portfolio** → **`a`** (add holding) and note modal placement and width. Close, open **Alerts** → **`a`** and note the alert modal (slightly different height vs portfolio is expected).
2. After the refactor, repeat on the same geometry.  
   **Pass:** Centering and proportions match the pre-change behavior; no clipped title or missing borders.

### Manual — Issue #94 (Condition **←** / **→**)

1. **Alerts** → **`a`**. **Tab** (or **`;`**) until **Condition** is the focused (highlighted) field.
2. Press **`;`** a few times.  
   **Pass:** Still toggles **Above** ↔ **Below**; **`a`**/**`b`** still set **Above** / **Below** when Condition focused.
3. Set condition to **Above**, then press **Left** (arrow, **no** Shift/Ctrl/Alt).  
   **Pass:** Condition becomes **Below** (per §18.13.2).
4. Press **Right**.  
   **Pass:** Condition becomes **Above**.
5. Read the overlay helper line and Condition hint.  
   **Pass:** Text mentions **←**/**→** (or “Left/Right”) alongside **`;`** / **`a`**/**`b`**.

### Manual — Issue #95 (`STOCKTERM_DEBUG_ALERT_NOTIFY`)

1. Run with default features so **`desktop-notify`** is on. From a terminal:

   ```bash
   export STOCKTERM_DEBUG_ALERT_NOTIFY=1
   # then launch stockterm from the same shell (e.g. cargo run --release, or your installed binary)
   ```

2. Ensure **Settings** → **Desktop alert toasts** is **on**. Create and fire a **new** alert cross (same style as the “Bell and desktop toast” steps in the Issues **#10 / #42** section above).
3. **Pass:** The shell that launched the app prints at least one line to **stderr** reflecting the **`show()`** **`Result`** (e.g. **`Ok(())`** or an **`Err`** message if the OS denied notifications).
4. Unset the variable (or set it to anything other than **`1`**), repeat a fire.  
   **Pass:** No extra stderr noise from this debug path (unless the platform or another layer logs separately).

### Regression — Issues #10 / #42 (spot)

1. Re-run **Manual — Issue #10** steps **1** (dialog opens) and **3** (**Esc** cancels) from the section above.  
   **Pass:** Unchanged behavior.

### Sign-off — Issues #93, #94, #95

_Manual validation passed 2026-05-12 (post-audit)._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-12 | Pass |
| #93 modal parity | maintainer | 2026-05-12 | Pass |
| #94 Left/Right + copy | maintainer | 2026-05-12 | Pass |
| #95 debug stderr | maintainer | 2026-05-12 | Pass |
| #10 dialog spot regression | maintainer | 2026-05-12 | Pass |

---

## Issues #96, #97, #98 — Alerts persistence UX, batched desktop notify, sanitized notification body

**Scope:**

- [Issue #96](https://github.com/FelipeMorandini/stockterm/issues/96) — when **`save_alerts`** → **`try_save`** fails after **`triggered`** latched in memory, show an **Alerts-tab** banner (and optional one-retry-per-quote-batch per [`docs/SPEC.md`](SPEC.md) §18.14.2); keep stable **`Failed to save alerts:`** prefix or equivalent detection contract.
- [Issue #97](https://github.com/FelipeMorandini/stockterm/issues/97) — **BEL** still once per newly triggered alert (§18.5); **desktop:** at most **one** OS notification + **one** notify thread per **`check_alerts`** batch; multi-fire **`body`** lists up to **5** lines + **“… and M more”** per §18.14.3.
- [Issue #98](https://github.com/FelipeMorandini/stockterm/issues/98) — **`sanitize_alert_notify_display_text`** strips control chars / odd whitespace from **`symbol`** before **`Notification::body`**; unit tests on the pure helper.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §18.14.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

2. Matrix (if CI or local policy requires lean builds):

   ```bash
   cargo clippy --no-default-features -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All invoked commands exit **0**.

### Manual — Issue #96 (save failure after cross)

**Setup (destructive to config path — use a throwaway home or backup `~/.stockterm.json`):** make the config file **unwritable** after the app has started (e.g. **`chmod a-w ~/.stockterm.json`** on Unix) **or** point **`HOME`** at a full disk / read-only volume if you have a sandbox.

1. With a normal writable config, add watchlist symbols and **≥2** alerts whose thresholds will **all** newly cross on the **same** next quote refresh (or use one cross if only testing save failure).
2. Make config **unwritable**, return to the app, wait for a refresh that fires **`check_alerts`** (Status **TRIGGERED** in memory).
3. Open **Alerts**.  
   **Pass:** A **visible banner** (per §18.14.2) explains disk may be stale; **status bar** still shows **`Failed to save alerts:`** (or the chosen stable prefix).
4. Restore write permissions; wait for another quote cycle (or trigger any path that completes **`apply_stock_fetch_done`** per SPEC).  
   **Pass:** If §18.14.2 retry is implemented, **`~/.stockterm.json`** eventually reflects **`triggered: true`** without requiring add/remove; banner clears when save succeeds. If minimal ship is **banner-only** (no retry), document in sign-off — user must perform an action that calls **`save_alerts`** (e.g. add dummy alert then remove).

### Manual — Issue #97 (one toast per batch)

1. **`notifications_enabled`** on; **`desktop-notify`** on. Configure **≥3** alerts that will newly cross on the **same** quote batch (tight thresholds just under/over last price).
2. Observe the OS notification layer during the single batch fire.  
   **Pass:** **One** desktop notification (summary may mention multiple alerts); **not** three separate StockTerm toasts. Audible/visual **BEL** count may still match per-alert §18.5 (multiple bells acceptable).

### Manual — Issue #98 (sanitized body)

1. Add an alert whose **symbol** field contains embedded control characters **via a test build** or temporary local patch that bypasses normal validation **only** for QA — **or** use **`cargo test`** output to confirm unit cases if UI cannot enter **`"\n"`** in symbol. Preferred: run **`cargo test`** and read the **`sanitize_alert_notify_display_text`** tests (developer QA).
2. If a manual UI path exists (e.g. future relaxed input), fire a notify with a dirty symbol.  
   **Pass:** OS toast **`body`** shows a **single-line** sensible label (no vertical runaway layout).

### Regression — Issues #10 / #42 / #93–#95 (spot)

1. Re-run **Manual — Bell and desktop toast** (single-alert fire) from the **Issues #10, #42** section — **Pass:** bell + single-line toast still work when only **one** alert crosses.
2. Re-run **Manual — Issue #95** (`STOCKTERM_DEBUG_ALERT_NOTIFY`) after a **multi-alert** batch — **Pass:** **one** stderr line for **`show()`** **`Result`** (coalesced path).

### Sign-off — Issues #96, #97, #98

_Automated checks pass locally / CI on **[PR #105](https://github.com/FelipeMorandini/stockterm/pull/105)**; **manual** steps in this section should be run before closing [#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98)._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-12 | Pass |
| #96 banner + save recovery | | | |
| #97 one toast / multi-fire | | | |
| #98 sanitizer tests or manual | | | |
| Spot regression #10 / #95 | | | |

---

## Issues #100, #101, #104 — Alerts ship triage (layout contract, README debug env, notify body cap)

**Scope:**

- [Issue #100](https://github.com/FelipeMorandini/stockterm/issues/100) — **`debug_assert!(percent_x <= 100 && percent_y <= 100)`** at the start of **`app::layout::centered_rect`**; document **`0..=100`** contract in the function doc comment (see [`docs/SPEC.md`](SPEC.md) §18.15.1).
- [Issue #101](https://github.com/FelipeMorandini/stockterm/issues/101) — **`README.md`** **Developer / debug** subsection: **`STOCKTERM_DEBUG_ALERT_NOTIFY`** (**`1`** exact, stderr logs **`show()`** **`Result`** when **`desktop-notify`** is on); **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** (quote-batch delay, §16); note that other **`STOCKTERM_DEBUG_*`** vars are unsupported unless documented (see [`docs/SPEC.md`](SPEC.md) §18.15.2).
- [Issue #104](https://github.com/FelipeMorandini/stockterm/issues/104) — UTF-8-safe **total** byte cap (default **1024** per SPEC) on the joined coalesced **`body`** before **`Notification::body`**; debug stderr uses the same capped string (see [`docs/SPEC.md`](SPEC.md) §18.15.3).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §18.15.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

2. Lean build matrix (same as §18.14):

   ```bash
   cargo clippy --no-default-features -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All invoked commands exit **0**.

### Manual — Issue #100 (`centered_rect` contract)

1. Run the app on a **debug** build (`cargo run` without `--release` is typical). Open **Portfolio** add overlay and **Alerts** add overlay (paths that call **`centered_rect`**).  
   **Pass:** No panic; modals center as before.
2. (Optional developer check) Confirm **`src/app/layout.rs`** contains the **`debug_assert!`** and doc comment per §18.15.1.

### Manual — Issue #101 (README)

1. Open **`README.md`** at the repo root.  
   **Pass:** A **Developer / debug** (or clearly named) subsection lists **`STOCKTERM_DEBUG_ALERT_NOTIFY`** (exact **`1`**, mentions possible **`Ok(())`** / **`Err`** on stderr) and **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** (milliseconds, quote batch); states other **`STOCKTERM_DEBUG_*`** names are not supported unless listed.

### Manual — Issue #104 (capped notify body)

1. Requires **`desktop-notify`** and **`notifications_enabled`**. After implementation, configure **≥2** alerts that co-fire with thresholds/symbols chosen so the **joined** detail text would exceed the SPEC byte cap without truncation (e.g. many wide lines — may require a temporary local test build that forces long **`format!`** lines **only** for QA, or rely on **`cargo test`** for the truncation helper).
2. Trigger a coalesced batch toast.  
   **Pass:** OS notification **`body`** is readable (not absurdly long); **`STOCKTERM_DEBUG_ALERT_NOTIFY=1`** stderr shows a **`body`** no longer than the capped length (including **`…`** when truncated).
3. Prefer: run **`cargo test`** and confirm a unit test covers UTF-8-safe truncation for the batch **`body`** builder (developer QA acceptable if manual OS check is impractical).

### Regression — Issues #93 / #97 / #98 (spot)

1. **#93:** Portfolio and Alerts modals still use **`centered_rect`** with **55×40** and **55×42** — visual spot-check unchanged proportions.  
2. **#97 / #98:** Multi-alert batch still produces **one** desktop toast; sanitized symbols unchanged aside from optional tail truncation from #104.

### Sign-off — Issues #100, #101, #104

_Ship review 2026-05-12 (automated + doc/code review + audit). Tracked in [PR #107](https://github.com/FelipeMorandini/stockterm/pull/107)._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-12 | Pass |
| #100 debug build modals | maintainer | 2026-05-12 | Pass |
| #101 README subsection | maintainer | 2026-05-12 | Pass |
| #104 cap (manual or test) | maintainer | 2026-05-12 | Pass |
| Spot regression #93 / #97 | maintainer | 2026-05-12 | Pass |

---

## Issues #29, #5, #11, #12 — M3: Search, News, Settings

**Scope:**

- [Issue #29](https://github.com/FelipeMorandini/stockterm/issues/29) — umbrella: non-empty tab UIs + handlers for Search, News, Settings.
- [Issue #5](https://github.com/FelipeMorandini/stockterm/issues/5) — Search typeahead, debounce, navigation, Enter → Stock View + quote fetch.
- [Issue #11](https://github.com/FelipeMorandini/stockterm/issues/11) — News list, scroll, loading/empty, Enter → open URL (and/or copy).
- [Issue #12](https://github.com/FelipeMorandini/stockterm/issues/12) — Settings: edit refresh rate & default symbol, placeholders, `try_save`.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §10.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Search (#5 / #29)

1. **Yahoo (default):** `cargo run --release`, switch to **Search** (Tab).

2. Type **`appl`** slowly then pause ≥300 ms.  
   **Pass:** Results include **`AAPL`** (or equivalent Apple row) within ~500 ms of last keystroke; pane is **not** blank.

3. **Debounce:** Type several letters quickly; use network monitor or logs if available.  
   **Pass:** No unbounded parallel searches; at most one in-flight request for the latest query (stale responses do not overwrite newer typing — per SPEC §10.2).

4. **Navigate:** **`j`/`k`** or arrows move highlight; **Enter** on **`AAPL`**.  
   **Pass:** Switches to **Stock View** with **`AAPL`** active; quote fetch runs (table/detail updates or clear error).

5. **Backspace / Esc:** Shrink query with Backspace; **Esc** clears query and list.  
   **Pass:** Matches acceptance.

6. **Polygon regression:** Set **`provider`: `polygon`** with valid key; repeat a short query.  
   **Pass:** Search works or shows structured error; no panic.

7. **Chord safety:** **`Ctrl+a`** on Search does not append (parity with Issue #44).

### Manual — News (#11 / #29)

1. Set active symbol (**`AAPL`** on Stock View), open **News** tab.

2. **Pass:** Headlines list appears (publisher/title/date); **Loading…** may flash briefly; not an empty pane when data exists.

3. **Scroll:** **`j`/`k`** or arrows.  
   **Pass:** Selection moves; long titles do not break layout catastrophically.

4. **Enter:** On a row with a URL, press **Enter**.  
   **Pass:** Browser opens article **or** URL copied per platform (document which happened); failure shows a short error, no panic.

5. **Empty:** Symbol with no news (or mocked empty) — **Pass:** **No news available** (or equivalent), not a blank screen.

6. **Symbol change:** From Stock View change symbol (or use Search → Enter), return to **News**.  
   **Pass:** List eventually matches new symbol (no permanent stale headlines).

### Manual — Settings (#12 / #29)

1. Open **Settings** tab.  
   **Pass:** Rows show **`refresh_rate`**, **`default_symbol`**, theme summary / placeholder, provider (read-only), keymap placeholder.

2. Edit **refresh rate** to a valid integer (e.g. **10**), commit with **Enter**.  
   **Pass:** `~/.stockterm.json` updates; optional “Saved” flash; quote/news throttle behavior respects new value after change (may still enforce app minimum 5 s — per SPEC).

3. Edit **default symbol** to **`MSFT`**, save. **Quit** and relaunch.  
   **Pass:** Startup symbol is **`MSFT`** when watchlist empty (or as documented in SPEC §10.4); JSON persisted.

4. **Validation:** Try empty default symbol or invalid refresh text.  
   **Pass:** Inline or status error; config file not corrupted.

5. **Save failure (optional):** If safe to simulate read-only config, **Pass:** `error_message` surfaces (Issue #19 pattern).

### Sign-off — M3 (#29 / #5 / #11 / #12)

_Manual validation passed 2026-05-10 (pre-merge). Clipboard copy deferred to [#58](https://github.com/FelipeMorandini/stockterm/issues/58); News `Enter` opens URL._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-10 | Pass |
| Search: typeahead + debounce + Enter | maintainer | 2026-05-10 | Pass |
| Search: Esc / Backspace / chord safety | maintainer | 2026-05-10 | Pass |
| News: list + scroll + Enter open/copy | maintainer | 2026-05-10 | Pass |
| News: empty + symbol change | maintainer | 2026-05-10 | Pass |
| Settings: edit + persist + relaunch default | maintainer | 2026-05-10 | Pass |
| Settings: validation + placeholders | maintainer | 2026-05-10 | Pass |

---

## Issue #31 — Yahoo default provider, Polygon fallback & structured errors

**Scope:** [GitHub Issue #31](https://github.com/FelipeMorandini/stockterm/issues/31) — **`provider`** defaults to **`yahoo`**; shared HTTP client with timeouts; **`ProviderError`** surfaced via **`App.error_message`**; Polygon remains opt-in with **`api_key`**.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §9 (migration playbook).

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. **Unit tests:** Yahoo fixture JSON → **`TickerResponse`** / **`HistoricalResponse`** / **`SymbolSearchResponse`** (per SPEC §9.18); **`ProviderError`** / **`Display`** where implemented.

### Manual — Config migration & default provider

1. Backup **`~/.stockterm.json`**.

2. **Missing `provider` field:** Remove the **`provider`** key from JSON (if present), save. Launch app.  
   **Pass:** App behaves as **`yahoo`** (no Polygon key required); quotes attempted against Yahoo. If implementation writes config back, **`provider`** may reappear as **`yahoo`** — acceptable.

3. **Explicit Yahoo:** Set **`"provider": "yahoo"`**, empty **`api_key`**, unset **`STOCKTERM_API_KEY`**.  
   **Pass:** Same as above — no “missing Polygon API key” on Stock View.

### Manual — Yahoo — Stock View & watchlist

1. **`cargo run --release`**, **Stock View**. Type **`AAPL`**, confirm fetch (**Enter** as per current UX).

2. **Pass:** Table/detail shows plausible **Last** / OHLCV **or** a **single-line** error that is **not** about Polygon keys. Typing nonsense symbol **`ZZZZQQ`** → clear failure (**unknown symbol** / API message), **no panic**.

3. Add **`MSFT`** to watchlist (**`w`**), **`j`/`k`** between rows.  
   **Pass:** Rows refresh; **`symbol`** tracks selection; portfolio price back-fill still works if holdings overlap (regression vs Issue #3).

### Manual — Yahoo — Charts

1. Select a liquid symbol (**`AAPL`**). Switch to **Charts**. Wait for fetch (or trigger refresh per UX).

2. **Pass:** Chart or historical UI shows data **or** a clear error string; **no** stall of input loop; **no** Polygon-key message.

### Manual — Yahoo — Search

1. **Search** tab, enter a query (**`Apple`**, **`micro`**). Trigger search (keybinding per app).

2. **Pass:** Results list populates **or** empty/error message is understandable; **no** Polygon-key gate.

### Manual — Yahoo — News

1. **News** tab with symbol **`AAPL`** (or selected watchlist row).

2. **Pass:** Headlines render **or** empty state without crash; on HTTP failure, **`error_message`** explains failure (not a silent blank). If SPEC allowed empty success on partial outages, document observed behavior in sign-off notes.

### Manual — Polygon regression

1. Set **`"provider": "polygon"`**, restore valid **`api_key`** in file **or** **`STOCKTERM_API_KEY`**.

2. **Stock View:** **`GOOGL`** — quotes load.

3. **Charts / Search / News:** smoke-test same tabs.

4. **Pass:** Functionally equivalent to pre–#31 Polygon behavior; errors use **`ProviderError`** strings where implemented (may differ slightly from raw **`reqwest`** text).

### Manual — Polygon without key (negative)

1. **`provider`: `polygon`**, **empty** key, unset env.

2. **Pass:** User sees message requiring Polygon credentials (**SPEC §9.14**); **no** silent fallback to Yahoo unless explicitly implemented (not in SPEC).

### Manual — Errors & responsiveness

1. **Bad symbol / airplane mode:** Induce failure (invalid ticker or disconnect Wi‑Fi briefly).  
   **Pass:** **`error_message`** updates; UI keeps accepting input during background fetch (Issue #17 behavior preserved).

2. **Code review spot-check:** [`docs/SPEC.md`](SPEC.md) §9.7 — **`shared_client()`** uses non-zero **`timeout`** / **`connect_timeout`**.

### Sign-off — Issue #31

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | | | |
| Default / missing `provider` → Yahoo | | | |
| Yahoo Stock View + watchlist | | | |
| Yahoo Charts / Search / News smoke | | | |
| Polygon happy path | | | |
| Polygon missing key negative | | | |
| Errors readable; UI responsive | | | |

---

## Issue #44 — Stock View & Alerts modifier keys

**Scope:** [GitHub Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) — Shift/lowercase acceptance for symbol typing and for `a`/`d` on Alerts; no accidental triggers with Ctrl/Alt/Meta chords.

**Shipment:** Manual validation passed. **PR:** https://github.com/FelipeMorandini/stockterm/pull/52 — see also `docs/SPEC.md` §8.8.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. **Unit tests:** `letter_key_plain` (or equivalent) allows `NONE` and `SHIFT`; rejects `CONTROL` / `ALT` / `SUPER` (and other disallowed flags per SPEC §8).

### Manual — Stock View

1. Open **Stock View**, clear or set a short symbol buffer.

2. **Lowercase typing:** Type `aapl` without Shift (if the terminal delivers lowercase letters).  
   **Pass:** Status / buffer shows **`AAPL`** (uppercase); no keys ignored solely because of case.

3. **Shift + letter:** Hold Shift and type `MSFT` (or type letters that the terminal reports with `SHIFT` set).  
   **Pass:** Symbol buffer fills as **`MSFT`**; watchlist keys **`w`**, **`x`**, **`j`**, **`k`** still work when pressed with Shift-only (or normal Caps behavior) per SPEC.

4. **Chord safety:** Press **`Ctrl+a`** (or **`Cmd+a`** on macOS if the terminal maps it to `SUPER`/`CONTROL`).  
   **Pass:** Does **not** append to the symbol buffer and does **not** trigger watchlist actions tied to letter keys.

5. **Hotkey vs symbol (regression):** Lowercase **`w`** must still add the **current** symbol to the watchlist, not append `W` to the buffer. Enter ticker **`WMT`** using an uppercase **`W`** first (or full uppercase).  
   **Pass:** Matches SPEC §8 — lowercase `w`/`x`/`j`/`k` remain shortcuts; `W`/`X`/`J`/`K` go to the symbol buffer.

### Manual — Alerts tab

1. Switch to **Alerts** with a valid **`symbol`** on Stock View (e.g. `AAPL`).

2. **`a` / `A`:** Press lowercase **`a`** and, in a separate trial, **`Shift+a`**.  
   **Pass:** Both add an alert row (same stub price/condition as today — behavior unchanged aside from input).

3. **`d` / `D`:** Select a row; press lowercase **`d`** and, in a separate trial, **`Shift+d`** if SPEC maps delete to both.  
   **Pass:** Selected alert is removed without requiring a bare `NONE` modifier only.

4. **`Ctrl+d` or `Alt+d`:**  
   **Pass:** Does **not** remove an alert (no accidental match).

### Sign-off — Issue #44

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Build / clippy | | | |
| Modifier helper unit tests | | | |
| Stock View lowercase → uppercase buffer | | | |
| Stock View Shift + letters | | | |
| Ctrl/Cmd chord does not type/act | | | |
| Alerts `a`/`A` add | | | |
| Alerts `d`/`D` remove | | | |
| Alt/Ctrl chord on Alerts | | | |

---

## Issue #48 — Portfolio tab keyboard parity (Issue #44 follow-up)

**Scope:** [GitHub Issue #48](https://github.com/FelipeMorandini/stockterm/issues/48) — Portfolio **`a`** / **`d`** use `letter_key_plain` and case-insensitive letter matching, consistent with **Issue #44** / [`docs/SPEC.md`](SPEC.md) §8 and §12.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §12.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Portfolio add/remove keys

1. Open **Portfolio** with at least one holding (or empty — **`a`** may open §13 dialog later; for #48 alone, verify key recognition).

2. **Lowercase `a`:** Press **`a`** without Shift.  
   **Pass:** Same behavior as pre–#48 uppercase-only **`A`** (opens add flow or performs add per current implementation).

3. **`Shift+a`:** Press **`Shift+a`** (terminal may send `A` with `SHIFT` set).  
   **Pass:** Still triggers add (not ignored).

4. **Lowercase / Shift `d`:** With a row selected, press **`d`** and **`Shift+d`**.  
   **Pass:** Remove or confirm-remove flow runs per §13; keys are not ignored solely due to modifiers/case.

5. **Chord safety:** **`Ctrl+a`**, **`Alt+d`**.  
   **Pass:** Does **not** add/remove or arm remove.

6. **Regression — Stock View / Alerts:** Re-run a subset of the **Issue #44** QA rows.  
   **Pass:** No behavior change on those tabs.

### Sign-off — Issue #48

_Manual validation passed 2026-05-10._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Build / clippy / tests | maintainer | 2026-05-10 | Pass |
| Portfolio `a` / `A` / Shift | maintainer | 2026-05-10 | Pass |
| Portfolio `d` / `D` / Shift | maintainer | 2026-05-10 | Pass |
| Ctrl/Alt chords blocked | maintainer | 2026-05-10 | Pass |
| Issue #44 regression (spot) | maintainer | 2026-05-10 | Pass |

---

## Issue #6 — Portfolio add dialog, confirm remove, quote coverage

**Scope:** [GitHub Issue #6](https://github.com/FelipeMorandini/stockterm/issues/6) — numeric **add** dialog (shares + purchase price), **two-step** remove confirmation, **all portfolio symbols** included in the quote fan-out, **`try_save`** error surfacing.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §13. **Issue #48** (§12) should be satisfied so Portfolio letter keys behave like Alerts during manual runs.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Add dialog

1. On **Stock View**, set active symbol to **`MSFT`** (type + **Enter** as required).

2. Switch to **Portfolio**, press **`a`**.  
   **Pass:** A dialog (or overlay) appears; **no** silent add with **1 @ 100** defaults.

3. Enter **shares** **`10`** and **purchase price** **`412.55`** per SPEC (**`;`** cycles Shares/Price if needed; **Enter** advances Shares → Price → commit).  
   **Pass:** Row shows **MSFT**, **10**, avg **412.55**; **Current** / **Value** / **P/L** update after quote batch completes (may take one refresh cycle); totals change.

4. **Esc** during dialog.  
   **Pass:** Dialog closes; portfolio unchanged.

5. **Invalid input:** Non-numeric or empty buffers on commit.  
   **Pass:** Inline or status error; no panic; config not corrupted.

### Manual — Confirm remove

1. Select a row, press **`d`** once.  
   **Pass:** UI shows armed / confirm hint; row **not** removed yet.

2. Press **`d`** again **or** **`y`**.  
   **Pass:** Row removed; JSON updated.

3. Arm remove, then **`Esc`** or **`n`**.  
   **Pass:** Armed state clears; row remains.

### Manual — Quote coverage for portfolio-only symbols

1. Edit **`~/.stockterm.json`**: ensure a holding exists for ticker **`IBM`** while **`IBM`** is **not** in **`watchlist`** and active **`symbol`** is **`AAPL`** (adjust paths carefully).

2. Launch app, open **Portfolio**.  
   **Pass:** After a quote cycle, **IBM** row shows a non-zero **Current** when the market data provider returns a quote (or a clear error), not stuck at **0** forever solely because the symbol was omitted from the batch.

### Manual — Persistence / errors

1. After add/remove, verify **`~/.stockterm.json`** **`portfolio`** array. Restart app.  
   **Pass:** Holdings survive.

2. **Optional (#19):** If **`try_save`** fails, **Pass:** **`error_message`** surfaces; no panic.

### Sign-off — Issue #6

_Manual validation passed 2026-05-10._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-10 | Pass |
| Add dialog; MSFT 10 @ 412.55 | maintainer | 2026-05-10 | Pass |
| Esc cancel; invalid input | maintainer | 2026-05-10 | Pass |
| Remove two-step + cancel | maintainer | 2026-05-10 | Pass |
| Portfolio-only symbol quoted | maintainer | 2026-05-10 | Pass |
| JSON persistence + try_save | maintainer | 2026-05-10 | Pass |

---

## Issues #43, #49, #50, #67, #69 — Alerts polish, Stock View hint, Portfolio dialog Tab & validation

**Scope:**

- [Issue #43](https://github.com/FelipeMorandini/stockterm/issues/43) — consistent **Alerts** pane title in empty vs table states.
- [Issue #49](https://github.com/FelipeMorandini/stockterm/issues/49) — **Stock View** status bar: **A–Z** symbol entry + **w/x/j/k** watchlist keys + §8.4 edge case (leading `w`/`x`/`j`/`k` tickers — **Shift** first letter).
- [Issue #50](https://github.com/FelipeMorandini/stockterm/issues/50) — **Alerts** empty-state copy mentions **`a` / `A`** (Shift-friendly add).
- [Issue #67](https://github.com/FelipeMorandini/stockterm/issues/67) — **Portfolio** add dialog: **Tab** / **Shift+Tab** cycle **Shares** / **Price**; with dialog **closed**, **Tab** still switches app tabs.
- [Issue #69](https://github.com/FelipeMorandini/stockterm/issues/69) — failed **commit** (non–`try_save`) sets **`inline_error`**; optional **max shares / max price** reject with **`inline_error`**.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §15.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Issue #43 (Alerts titles)

1. Open **Alerts** with **no** alerts configured.  
   **Pass:** Outer block title matches the titled table state (e.g. both **"Price Alerts"** or one documented hierarchy — no **"Price Alerts"** vs **"Alerts"** mismatch per §15.1).

2. Add at least one alert (`a`), confirm the **table** view title matches the empty-state convention.

### Manual — Issue #50 (Alerts copy)

1. Remove all alerts so the empty state shows.  
   **Pass:** Helper text reflects **`a`** and **`A`** / Shift-friendly wording (Issue #50).

### Manual — Issue #49 (Stock View status)

1. Switch to **Stock View** with an empty or non-empty watchlist. Read the **status bar**.  
   **Pass:** Mentions **A–Z** (or equivalent) for ticker entry alongside **w** / **x** / **D** / **j**/**k**; includes the **w/x/j/k** leading-letter **Shift** tip (§8.4). Text fits a typical 80-column terminal or degrades gracefully per SPEC.

### Manual — Issue #67 (Tab in Portfolio dialog)

1. **Stock View:** set symbol **MSFT**, open **Portfolio**, press **`a`**.

2. With dialog open, press **Tab** repeatedly, then **Shift+Tab**.  
   **Pass:** Focus alternates **Shares** ↔ **Price**; **`;`** still cycles if implemented; **app tab** does **not** change.

3. **Esc** to close dialog. Press **Tab**.  
   **Pass:** App tab advances (same as pre–#67).

### Manual — Issue #69 (inline error + caps)

1. Open add dialog with valid symbol (**MSFT**). Enter valid shares/price, commit.  
   **Pass:** Still works.

2. **Commit failure without `try_save`:** The rare **`add_to_portfolio` → `false`** path where **`error_message`** is still **`None`** (e.g. **`normalize_symbol(&app.symbol)`** `None` at commit) must set **`portfolio_dialog.inline_error`** — **Pass:** covered by **`cargo test`** added for §15.5 **or** maintainer code review of that branch (opening **`a`** with an empty symbol is blocked earlier by **`error_message`**, so this is not easily reproducible from the TUI alone).

3. **`try_save` failure:** Optional: induce a save error (e.g. read-only config path in a throwaway env).  
   **Pass:** **`error_message`** surfaces; dialog may stay open per §13; no panic.

4. **Caps (if implemented):** Enter shares or price **above** the documented maximum.  
   **Pass:** **`inline_error`** explains the bound; portfolio unchanged.

### Sign-off — Issues #43, #49, #50, #67, #69

_Manual validation passed 2026-05-11._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-11 | Pass |
| #43 titles empty + table | maintainer | 2026-05-11 | Pass |
| #50 empty copy | maintainer | 2026-05-11 | Pass |
| #49 Stock View status | maintainer | 2026-05-11 | Pass |
| #67 Tab / Shift+Tab / global Tab | maintainer | 2026-05-11 | Pass |
| #69 commit + caps | maintainer | 2026-05-11 | Pass |

---

## Issue #3 — Watchlist & multi-row quotes

**Scope:** [GitHub Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3) — persisted `watchlist`, multi-row Stock View table, selection drives `symbol`, bounded fan-out fetch, refresh cadence, non-blocking input (with [Issue #17](https://github.com/FelipeMorandini/stockterm/issues/17) as applicable).

Run these when validating the #3 implementation (and after #44, re-run rows that interact with Stock View keys). Automated checks are listed first; the rest are manual.

## Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; no warnings treated as errors by clippy.

2. **Regression:** Config still loads if `watchlist` is omitted from `~/.stockterm.json`:

   - Temporarily rename config, run app once to create defaults, or hand-edit JSON to remove `watchlist` after a run that added it — app must start without panic and treat missing field as empty list.

---

## Manual — Prerequisites

- Valid Polygon credentials: non-empty `api_key` in `~/.stockterm.json` or `STOCKTERM_API_KEY`.
- **Rate limits:** Polygon free tier is 5 requests/minute; use a small watchlist (3 symbols) and a `refresh_rate` ≥ 5 s for testing, or expect throttling if you hammer refresh.

---

## Manual — Watchlist table & symbols

1. **Launch:** `cargo run --release`, **Stock View** tab.

2. **Add three symbols:** Set symbol string to `AAPL` (type uppercase letters, **Enter** to fetch if required by UX), then **`w`**. Repeat for `MSFT` and `NVDA` (or type each, Enter, `w`).  
   **Pass:** Table shows **three rows** with Symbol / Last / Change / % Change / Volume populated (or clear error if API fails — no panic).

3. **Highlight drives detail:** Use **`j`**/**`k`** or arrow keys to change the selected row.  
   **Pass:** Bottom detail pane (or equivalent) reflects the **selected** ticker’s OHLC/volume; `symbol` used on **Charts** / **News** after switching tabs matches the highlighted row (per SPEC).

4. **Remove row:** Select one symbol, press **`x`** (or **`D`** if that is the bound delete key).  
   **Pass:** Row disappears immediately; `~/.stockterm.json` no longer lists that symbol in `watchlist` after the action (verify file on disk).

---

## Manual — Persistence

1. With a non-empty watchlist, **quit** (`q`) and relaunch.  
   **Pass:** Same symbols reappear in the table; order matches last session (or documented sort order).

2. **Save failure (optional, #19 alignment):** If implementation surfaces `try_save` errors, simulate e.g. read-only home or invalid path only if you have a safe test setup.  
   **Pass:** Error appears in status / `error_message`; **no panic**.

---

## Manual — Refresh cadence (#4)

1. Set `refresh_rate` in `~/.stockterm.json` to **5** (seconds). Restart app.

2. Observe quote **Last** / **Change** (or network activity) over ~15–20 s on Stock View.  
   **Pass:** Refreshes occur roughly every ≥ 5 s (respecting app minimum if any), not on every 200 ms UI tick.

3. **In-flight:** During a slow network, confirm behavior matches SPEC: no overlapping pile-up of fan-out jobs, or a clear “refreshing” state (per #4 / §3.3).

---

## Manual — Non-blocking input (#17)

**Primary:** Use **[Issues #17, #46, #77](#issues-17-46-77--async-main-loop-polish)** after §16 ships — it replaces the informal checklist below.

1. With an artificial delay or very slow network (per §16.1 harness), hold a key that navigates tabs or watchlist rows.  
   **Pass:** Input continues to be processed; screen keeps redrawing; a multi-second HTTP wait does not freeze the TUI.

2. **Pass:** No `await` of HTTP on the path between redraw and **`tokio::select!`** input handling (code review / §16.1).

---

## Manual — Alerts integration (regression)

1. Add an alert (**Alerts** tab, **`a`**) for a symbol that is **only** on the watchlist (not the previously single fetched ticker), after watchlist quotes have loaded.  
   **Pass:** **Current** column can show a non-zero price when that symbol’s quote exists in the watchlist cache (per updated `get_current_price`). Document if limitation remains.

---

## Sign-off

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy | | | |
| Three-row watchlist + columns | | | |
| Selection drives `symbol` / detail / other tabs | | | |
| Remove row updates UI + JSON | | | |
| Watchlist survives restart | | | |
| `refresh_rate` honored (≥ min) | | | |
| Bounded fan-out / no runaway concurrency | | | |
| Non-blocking input (#17) or N/A | | | |
| Alerts + watchlist price (if applicable) | | | |

---

**After implementation:** Run the relevant QA sections (#44 and/or #3) and record results in the sign-off tables before merge.
