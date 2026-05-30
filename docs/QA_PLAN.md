# QA Plan — Manual verification


## Issues #19, #103 — Config persistence & alerts-save / quote error coordination

**Scope:**

- [Issue #19](https://github.com/FelipeMorandini/stockterm/issues/19) — `try_save` failures surfaced (no panic); `default_symbol` honored at launch; **`last_tab`** / **`last_symbol`** persistence when implemented; older JSON without new keys loads; README + struct docs list every field.
- [Issue #103](https://github.com/FelipeMorandini/stockterm/issues/103) — With an active **`Failed to save alerts:`** runtime error, a subsequent **failed** or **partial-failure** quote batch must not **erase** that signal from the status line / Alerts banner predicate until alerts save succeeds or the user dismisses per §20.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §22.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Issue #19 (persistence & startup)

1. **Default symbol:** Edit `~/.stockterm.json` — set **`watchlist`** to **`[]`**, set **`default_symbol`** to **`MSFT`**, save. Launch **`cargo run --release`**.  
   **Pass:** Stock View active symbol is **MSFT** (or normalized equivalent), not **AAPL**; no panic.

2. **Last tab / last symbol (when §22 fields ship):** Open **Charts**, select a non-default symbol, quit, relaunch.  
   **Pass:** App restores **Charts** tab and the same **symbol** per SPEC precedence rules.

3. **Schema forward-compat:** Remove **`last_tab`** / **`last_symbol`** keys if present (simulate older file), keep other fields valid, launch.  
   **Pass:** Defaults apply; no panic.

4. **Save failure (optional):** With a **safe** setup only (e.g. copy config to a temp **`HOME`**, chmod config or parent read-only), trigger any action that calls **`Config::try_save`** (watchlist add, Settings save).  
   **Pass:** **`error_message()`** / status line shows a **`[cfg]`** (or equivalent) persistence error; **no panic**.

### Manual — Issue #103 (alerts-save vs quote errors)

**Setup (choose one safe approach):**

- **A:** Point **`HOME`** at a writable temp dir, run the app, get **`Failed to save alerts:`** on screen (e.g. make `~/.stockterm.json` effectively non-writable **after** latch + crossing per §18.14 QA), **then** force a quote batch that returns errors (invalid symbol batch, debug HTTP mock, or disconnect network so batch surfaces **`[net]`** / provider errors).

1. While **`Failed to save alerts:`** is visible on the status line, trigger a stock quote batch that completes with **one or more symbol errors** (non-empty batch errors).  
   **Pass:** Status line (or merged line per §22.2) still contains the substring **`Failed to save alerts:`**; **Alerts** tab top banner strip remains active if SPEC requires it for the same predicate.

2. With alerts-save still failing, add a symbol to the watchlist (**`w`**) such that **`try_save`** **succeeds**.  
   **Pass:** Alerts-save message is **not** cleared solely because watchlist save succeeded (Portfolio-domain clear must not wipe Alerts-domain errors).

3. After **`retry_alerts_save_if_pending`** succeeds (restore write permissions or fix path), wait for a clean quote batch.  
   **Pass:** Alerts-save error clears per §18.14 / §20 sticky rules; no stuck duplicate banners.

### Sign-off (#19 / #103)

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| #19 default_symbol startup | maintainer | 2026-05-18 | Pass |
| #19 last_tab / last_symbol (if shipped) | maintainer | 2026-05-18 | Pass |
| #19 older JSON / defaults | maintainer | 2026-05-18 | Pass |
| #19 optional save failure (no panic) | maintainer | 2026-05-18 | Pass |
| #103 alerts-save survives quote batch errors | maintainer | 2026-05-18 | Pass |
| #103 watchlist success does not hide alerts-save | maintainer | 2026-05-18 | Pass |
| #103 recovery clears alerts-save when disk fixed | maintainer | 2026-05-18 | Pass |

---

## Issues #34, #35, #40, #129 — Config operator docs, load UX, optional I/O, session write coalescing

**Scope:**

- [Issue #34](https://github.com/FelipeMorandini/stockterm/issues/34) — Document plaintext **`api_key`** in **`~/.stockterm.json`** and **`STOCKTERM_API_KEY`** override ([`docs/SPEC.md`](SPEC.md) §22.7.1).
- [Issue #35](https://github.com/FelipeMorandini/stockterm/issues/35) — Config load failures must not appear as a silent fresh install on the **`App`** path ([`docs/SPEC.md`](SPEC.md) §22.7.2).
- [Issue #40](https://github.com/FelipeMorandini/stockterm/issues/40) — Optional: non-blocking **`Config::try_save`** if profiling shows UI stalls ([`docs/SPEC.md`](SPEC.md) §22.7.3).
- [Issue #129](https://github.com/FelipeMorandini/stockterm/issues/129) — Optional: debounce or coalesce frequent session JSON writes ([`docs/SPEC.md`](SPEC.md) §22.7.4).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §22.7 for any shipped slice.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

### Manual — Issue #34 (README / operator docs)

1. Open **[`README.md`](../../README.md)** after the change ships. Locate the **Security — API keys** (or equivalent) subsection.  
   **Pass:** Text states **`api_key`** is stored **in plaintext** in **`~/.stockterm.json`**; describes **`STOCKTERM_API_KEY`** when the file field is empty; includes at least one practical hygiene tip (e.g. file permissions, do not commit secrets).

### Manual — Issue #35 (config load failures)

Use a **disposable** config path via **`HOME`** pointing at a writable temp directory.

1. Write intentionally **invalid JSON** to **`$HOME/.stockterm.json`**. Launch **`cargo run --release`**.  
   **Pass:** App starts with **defaults** (empty watchlist, etc.) **and** the **startup error banner** (top of UI) shows a **config load** failure (wording may vary); **no panic**.

2. Restore **valid** minimal JSON (see [`docs/SPEC.md`](SPEC.md) §22 serde examples), relaunch.  
   **Pass:** Startup error banner **absent**; prior **`portfolio`** / **`watchlist`** values load when present.

### Manual — Issue #40 (non-blocking saves, only if implemented)

If **§22.7.3** code ships: exercise rapid Settings / watchlist saves on a **slow** disk or large config (or maintainer-defined stress path).  
**Pass:** No sustained UI freeze beyond acceptable bounds **and** failed saves still surface via **`error_message()`** / §20 patterns.

If **not** implemented: mark **N/A** in the sign-off table with a pointer to the profiling gate in [`docs/SPEC.md`](SPEC.md) §22.7.3.

### Manual — Issue #129 (session write debounce, only if implemented)

If **§22.7.4** code ships:

1. Rapidly switch tabs (**Tab** / **Shift+Tab** or bound keys) **10+** times within **1 s**, then **quit normally** (`q`). Inspect **`~/.stockterm.json`** **`last_tab`** (or relaunch).  
   **Pass:** Persisted tab matches the **last** tab before quit (or per SPEC-chosen strategy **B**); disk write count is **not** O(N) per keystroke if debounce is the goal (optional: maintainer counts writes with a debug hook if one exists).

2. If debounce delays disk writes while running, **kill -9** is **not** required for acceptance; normal quit must **flush** pending session fields per [`docs/SPEC.md`](SPEC.md) §22.7.4.

If **not** implemented: mark **N/A** in the sign-off table.

### Regression — Issue #3 (watchlist)

Whenever §22.7 touches **`try_save`**, **`persist_session_to_disk`**, or watchlist persistence, re-run **[Issue #3](#issue-3--watchlist--multi-row-quotes)** automated + manual smoke (multi-row table, **`w`/`x`/`j`/`k`**, **`refresh_rate`**, responsive input).

### Sign-off (#34, #35, #40, #129, #3 regression)

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| #34 README security copy | maintainer | 2026-05-18 | Pass |
| #35 corrupt JSON startup banner | maintainer | 2026-05-18 | Pass |
| #35 valid JSON clean startup | maintainer | 2026-05-18 | Pass |
| #40 non-blocking save (or N/A + gate) | maintainer | 2026-05-18 | Pass |
| #129 debounce / quit flush (or N/A) | maintainer | 2026-05-18 | Pass |
| #3 regression smoke (if persistence touched) | maintainer | 2026-05-18 | Pass |

---

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
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| #17 slow-network smoke (≥5 s delay) | maintainer | 2026-05-18 | Pass |
| #17 supersede / stale generation | maintainer | 2026-05-18 | Pass |
| #46 inflight never stuck (normal + optional panic hook) | maintainer | 2026-05-18 | Pass |
| #77 pending vs `InflightRecovery::Stock` (per §16.3) | maintainer | 2026-05-18 | Pass |

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

## Issue #53 — Yahoo batched quotes (single primary `v7` request)

**Scope:**

- [Issue #53](https://github.com/FelipeMorandini/stockterm/issues/53) — With **`provider: "yahoo"`**, quote refresh for the deduplicated symbol set (watchlist + active symbol + portfolio tickers per **`collect_symbols_for_quote_fetch`**) uses **one primary** **`v7/finance/quote`** HTTP round-trip per batch (or chunked sequential GETs per [`docs/SPEC.md`](SPEC.md) §9.15.5), instead of **N** parallel per-symbol **`get_quote`** calls. **Polygon** path unchanged (**bounded parallel `get_quote`**).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §9.15.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; §9.15.7 unit tests for multi-symbol **`v7`** JSON present and green.

### Manual — Yahoo multi-symbol watchlist + portfolio

**Prep:** In **`~/.stockterm.json`**, set **`"provider": "yahoo"`**. Add **at least four** distinct liquid symbols to **`watchlist`** (e.g. **AAPL**, **MSFT**, **GOOGL**, **SPY**). Add **one** portfolio holding for a fifth symbol not in the watchlist (e.g. **NVDA**). Restart **`cargo run --release`**.

1. Open **Stock View**; wait at least **two** full **`refresh_rate`** cycles (or trigger refresh with **Enter** if implemented).  
   **Pass:** Watchlist **Price** / **Change** columns populate for **all** rows; active symbol detail matches the selected row; no sustained **“No quote”** for symbols that previously worked on the pre–#53 build.

2. Switch **`j`/`k`** across watchlist rows.  
   **Pass:** Detail pane updates per symbol; values remain plausible (same-day sanity as Issue #2).

3. Open **Portfolio** tab (or split view per your layout).  
   **Pass:** Portfolio row for the extra symbol shows a current price when quotes succeed; alerts / status bar show **no** new unexplained error class.

### Manual — Fallback smoke (optional but recommended)

1. Add an **unusual** or thin symbol that historically returns empty **`v7`** rows but recovers via **`v8`** (per §17 — if you have a known example from prior testing, use it; otherwise skip).  
   **Pass:** After refresh, either a plausible quote or a **clear** per-symbol error — **no panic**, **no** blank app.

### Manual — Polygon regression

1. Set **`"provider": "polygon"`** with a valid **`api_key`**; use a **two-row** watchlist + one portfolio symbol. Restart, wait for quotes.  
   **Pass:** Same functional behavior as before #53; bounded concurrency unchanged from an operator perspective (no obvious stall vs Yahoo-only change).

### Sign-off — Issue #53

_Manual validation passed 2026-05-13._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-13 | Pass |
| Yahoo ≥4 watchlist + 1 portfolio symbol | maintainer | 2026-05-13 | Pass |
| Row switch / detail pane | maintainer | 2026-05-13 | Pass |
| Polygon smoke (2+ symbols) | maintainer | 2026-05-13 | Pass |

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

### Manual — Regression (Alerts keys / §8 — not layout Issue #15)

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
| #96 banner + save recovery | maintainer | 2026-05-18 | Pass |
| #97 one toast / multi-fire | maintainer | 2026-05-18 | Pass |
| #98 sanitizer tests or manual | maintainer | 2026-05-18 | Pass |
| Spot regression #10 / #95 | maintainer | 2026-05-18 | Pass |

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

## Issue #18 — API robustness (timeouts, 429 / `Retry-After`, backoff, structured errors)

**Scope:**

- [GitHub Issue #18](https://github.com/FelipeMorandini/stockterm/issues/18) — shared **`reqwest::Client`** tuning; **`ProviderError`** including **`RateLimited`**; non-2xx body surfaced before JSON parse failures; exponential backoff + jitter for transient failures; watchlist quote concurrency cap; readable **`error_message`** / per-symbol batch errors.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §19.

### GitHub Issue #18 acceptance ↔ this section

| [Issue #18](https://github.com/FelipeMorandini/stockterm/issues/18) acceptance criterion | Verified by |
|------------------------------------------------------------------------------------------|-------------|
| Simulated **429** + **`Retry-After: 10`** → retry after ~**10 s** with backoff; no crash | **Automated:** §19.8 — **`retry::wiremock_tests::retry_after_one_second_before_success`** uses **`Retry-After: 1`** and asserts **≥ ~900 ms** wall time (same semantics, faster CI); **`tokio::test-util`** + paused time used in **`stall_triggers_timeout`**. |
| Simulated **500** → retries up to cap with exponential backoff | **Automated:** bullet 2 (**≤ 5** attempts). |
| **10 s** server stall → **`Timeout`**, not hang | **Automated:** bullet 3 (mock delay vs client request timeout per SPEC). **Manual:** healthy Yahoo smoke still non-blocking (debug delay step below). |
| Non-JSON **4xx** → status/body-style error, **not** primary **`serde_json`** message | **Automated:** bullet 4. |
| Watchlist concurrent fetches ≤ configured cap | **Manual:** “Concurrency spot-check” below; **Automated** optional (harder — not required for sign-off if manual done). |

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit **0**.

2. **Issue #18 acceptance (required after §19 implementation):** integration tests added per SPEC §19.8 (**`wiremock`**) must cover at minimum:

   - **429** with **`Retry-After`** then success — **`retry_after_one_second_before_success`** asserts **≥ ~900 ms** wall time before the successful attempt (scaled from **`Retry-After: 10`** for CI speed; semantics unchanged).
   - **500** responses then success — bounded retries (**≤ 5** attempts).
   - **401**/**403** with **non-JSON** body — error is **not** primarily a **`serde_json`** parse error string.
   - **`Retry-After`** parsing unit tests — integer seconds, HTTP-date, malformed.
   - **`Timeout`** — **`stall_triggers_timeout`** (short client timeout + **`tokio::test(start_paused = true)`** + **`time::advance`**).

   **Pass:** `cargo test` runs those tests green without real network.

### Manual — Regression (live providers)

1. **Yahoo (default):** `cargo run --release`, **Stock View**, ensure quotes still load for **AAPL** / a small watchlist.  
   **Pass:** No regression vs pre–#18 behavior when the network is healthy.

2. **Polygon (optional):** Valid key, **≥3** symbols on the watchlist, **`refresh_rate`** at **5** or higher.  
   **Pass:** App remains responsive; on throttling, status / **`error_message`** explains rate limiting or HTTP failure without panic; no **`apiKey=`** substring in any visible error string.

3. **Slow path:** With **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** (§16) set high on a **debug** build, confirm the UI still accepts input during an in-flight quote batch (non-blocking loop unchanged).

### Manual — Concurrency spot-check

1. Add **5+** symbols to the watchlist (or use portfolio + watchlist so **`collect_symbols_for_quote_fetch`** returns many symbols). Use a network monitor or temporary logging if available.  
   **Pass:** At most **`MAX_CONCURRENT_QUOTES`** (documented in SPEC / source, default **2**) concurrent **`get_quote`** operations per batch — no unbounded fan-out.

### Sign-off — Issue #18

_Automated §19.8 coverage in `cargo test`; **manual** steps above still required before closing [GitHub Issue #18](https://github.com/FelipeMorandini/stockterm/issues/18) for **PR #115** behavior._

Completing **all** rows below documents acceptance of Issue #18 as shipped in **#115** (timeouts, retries, **`ProviderError`**, watchlist concurrency). **Issues #110–#114** and **#116** are **optional follow-ups** per [`docs/SPEC.md`](SPEC.md) §19.13 — they do **not** gate this table unless the maintainer intentionally bundles them into the same release.

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| §19.8 wiremock / paused-time tests | maintainer | 2026-05-18 | Pass |
| Yahoo smoke | maintainer | 2026-05-18 | Pass |
| Polygon / throttling (optional) | maintainer | 2026-05-18 | Pass |
| Concurrency spot-check | maintainer | 2026-05-18 | Pass |

---

## Issues #110, #111, #112, #113, #114, #116 — §19 HTTP post-audit hardening

**Scope:**

- [#110](https://github.com/FelipeMorandini/stockterm/issues/110) — Bounded read for large **4xx** / **429** error response bodies (no full-body buffer before snippet trim).
- [#111](https://github.com/FelipeMorandini/stockterm/issues/111) — Cap parsed integer **`Retry-After`**; **`ProviderError::RateLimited`** **`Display`** shows sub-second delays meaningfully (not **`0s`**).
- [#112](https://github.com/FelipeMorandini/stockterm/issues/112) — **`Retry-After`** HTTP-date parsing accepts common **`UTC` / `GMT`** / case variants (per SPEC §19.13.3).
- [#113](https://github.com/FelipeMorandini/stockterm/issues/113) — Docs capture **`tokio::test(start_paused = true)`** + **`reqwest`** timeout pitfalls and mitigations (SPEC §19.8 / §19.13.3; README Developer if applicable).
- [#114](https://github.com/FelipeMorandini/stockterm/issues/114) — **`execute_get_text_with_retry_inner`** post-loop path is **`unreachable!`** (or equivalent) with a short comment — no “exhausted unexpectedly” **`Transport`** fallback.
- [#116](https://github.com/FelipeMorandini/stockterm/issues/116) — **`ProviderError::Debug`** (and any stored URL fields used in diagnostics) do **not** leak **`apiKey=`** or other query secrets; **`Display`** unchanged vs §19.7.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §19.13; **`cargo test`** / **`cargo clippy -- -D warnings`** green.

### Automated (local)

1. From the repo root (after the §19.13 PR lands):

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit **0**.

2. **Regression:** Confirm existing §19.8 tests still pass (**429** wall-clock wait, **500** retries, **401** plain text, **`stall_triggers_timeout`**, **`Retry-After`** unit tests including any new variants from #112).

### Manual / spot

1. **Operator-visible strings:** `cargo run --release`, trigger an HTTP error path (invalid Polygon key, or forced offline) such that **`ProviderError::Http`** or **`RateLimited`** reaches the status line.  
   **Pass:** Still **no** **`apiKey=`** substring in **`Display`** output (unchanged from Issue #18).

2. **Rate-limit copy (#111):** If you can force a **429** with **`Retry-After: 1`** (or use code review of **`Display`** for a **`Duration`** of **400 ms** in a unit test), confirm users do **not** see misleading **`retry after 0s`**.

3. **Docs (#113):** Open [`docs/SPEC.md`](SPEC.md) §19.8 / §19.13.3 and **`README.md`** Developer subsection — **Pass** text matches what engineers need to avoid spurious **`Timeout`** in paused-**`tokio`** tests.

### Sign-off — Issues #110–#114, #116

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| §19.8 + new §19.13 unit/integration tests | maintainer | 2026-05-18 | Pass |
| Display / no `apiKey=` spot-check | maintainer | 2026-05-18 | Pass |
| Rate-limit **`Display`** spot-check or test review | maintainer | 2026-05-18 | Pass |
| SPEC / README test-harness note (#113) | maintainer | 2026-05-18 | Pass |

---

## Issue #20 — Error UX (categories, error log, retry, auto-clear)

**Scope:**

- [GitHub Issue #20](https://github.com/FelipeMorandini/stockterm/issues/20) — **`AppError`** + **`UiErrorCategory`** prefixes on the status line; **`retry in Ns`** hint for **`ProviderError::RateLimited`**; ring buffer (**20**) with overlay; **retry** last failed fetch for the active tab domain; **transient** errors auto-clear (**10 s** TTL or success); **sticky** errors until resolved; **startup** vs **runtime** visual distinction.

**Follow-up issues (post-ship `/audit` scratchpad):** [#120](https://github.com/FelipeMorandini/stockterm/issues/120) (unify error-log overlay visible rows for scroll vs draw), [#121](https://github.com/FelipeMorandini/stockterm/issues/121) (avoid mutating `error_log_scroll` inside draw), [#122](https://github.com/FelipeMorandini/stockterm/issues/122) (document `ProviderError::Clone` / `Json` mapping), [#123](https://github.com/FelipeMorandini/stockterm/issues/123) (`q` quit vs overlay — product/QA).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §20. **HTTP behavior** matches [`docs/SPEC.md`](SPEC.md) §19 / Issue #18 (`ProviderError`, retries).

**Binding note (SPEC §20.1):** [Issue #20](https://github.com/FelipeMorandini/stockterm/issues/20) suggests plain **`e`** / **`r`**; Stock View and Search use plain letters for symbol/query input. Manual steps below use **`Ctrl+E`** (error log) and **`Ctrl+R`** (retry) as the **canonical** chords. **Pass** = behavior matches §20.1, not bare **`e`**/**`r`** on Stock View.

### GitHub Issue #20 acceptance ↔ this section

| [Issue #20](https://github.com/FelipeMorandini/stockterm/issues/20) acceptance criterion | Verified by |
|------------------------------------------------------------------------------------------|-------------|
| **429** surfaces as **`[rate] … retry in 10s`** (not raw **`reqwest`**) | **Manual** with throttled provider or mock (if available); else **automated** §19 tests + **code review** of §20.3 mapping from **`RateLimited { retry_after: Some(10s) }`** to status line. |
| Network outage → **`[net] …`** and clears after a successful fetch | **Manual** — toggle network / bad proxy; restore; **Ctrl+R** or natural poll. |
| **Retry** last failed fetch | **Manual** — **`Ctrl+R`** on each tab domain (Stock, Charts, News, Search) after a forced failure. |
| **Error log** lists last **N** with timestamps | **Manual** — **`Ctrl+E`** overlay; generate ≥3 distinct errors; **Esc** closes. |
| Errors never block rest of UI | **Manual** — during **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** (§16) delay, confirm tabs/typing still work; overlay does not freeze terminal. |

### Automated (local)

1. From the repo root (after implementation lands):

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; §20.10 unit tests for category mapping + ring buffer present per SPEC.

### Manual — Status prefixes and rate limit hint

1. **`cargo run --release`**, force a **rate-limited** or **429**-class outcome (Polygon with aggressive refresh, or local mock if wired).  
   **Pass:** Status line shows **`[rate]`** and a **`retry in …s`** (or documented equivalent when **`retry_after`** is **`None`**) — **no** substring **`reqwest`**.

2. Force a **connection refused** or **DNS** failure (invalid proxy host, unplug network).  
   **Pass:** **`[net]`** prefix; body text is short and readable.

3. Provoke a **JSON** / parse failure path if testable without code changes (else skip).  
   **Pass:** **`[parse]`** when implementation maps **`ProviderError::Json`**.

### Manual — Transient auto-clear vs sticky

1. Trigger a **transient** error (**timeout** / **transport**). Wait **≥ 10 s** without fixing the network.  
   **Pass:** Active status error **clears** from the status line per §20.6 (or documented TTL), while **error log** still retains the row.

2. Trigger a **sticky** error (e.g. **401**/**403** or invalid API key message).  
   **Pass:** Message remains past **10 s** until provider succeeds or user fixes config / retries successfully.

### Manual — Error log overlay (**`Ctrl+E`**)

1. Generate several errors (wrong symbol, network off, throttling). Press **`Ctrl+E`**.  
   **Pass:** Overlay lists up to **20** entries with **timestamps** and readable text; **`j`/`k`** scroll if list exceeds viewport.

2. Press **`Esc`**.  
   **Pass:** Overlay closes; underlying tab UI intact.

3. On **Stock View**, type **`aapl`** — confirm plain **`e`** still types **`E`** into the symbol buffer (no regression). Press **`Ctrl+E`**.  
   **Pass:** Overlay toggles; symbol buffer unchanged by **`Ctrl+E`**.

### Manual — Retry (**`Ctrl+R`**)

1. **Stock View:** Cause quote batch failure; press **`Ctrl+R`**.  
   **Pass:** A new quote batch is scheduled; inflight / generation behavior matches SPEC (no panic; no duplicate stuck **`stock_refresh_inflight`**).

2. **Charts:** Force historical fetch error; **`Ctrl+R`**.  
   **Pass:** Historical refetch attempted.

3. **News / Search:** Repeat for tab-appropriate failures.  
   **Pass:** Same domain retry only (no cross-tab accidental fetch).

### Manual — Startup vs runtime

1. Temporarily rename or corrupt **`~/.stockterm.json`** backup, replace with invalid JSON, launch app.  
   **Pass:** **Startup** banner or distinct styling per §20.7; not identical to a mid-session fetch error line.

2. Restore valid config; restart.  
   **Pass:** No startup error; normal status bar.

### Sign-off — Issue #20

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| §20 unit tests (category + ring buffer) | maintainer | 2026-05-18 | Pass |
| Status prefixes + **`[rate]`** hint | maintainer | 2026-05-18 | Pass |
| Transient TTL vs sticky | maintainer | 2026-05-18 | Pass |
| **`Ctrl+E`** overlay + **`Esc`** | maintainer | 2026-05-18 | Pass |
| Stock View plain-letter regression | maintainer | 2026-05-18 | Pass |
| **`Ctrl+R`** per tab domain | maintainer | 2026-05-18 | Pass |
| Startup vs runtime presentation | maintainer | 2026-05-18 | Pass |

---

## Issues #120, #121, #122, #123 — Error log overlay & `ProviderError::Clone` post-ship polish

**Scope:**

- [GitHub Issue #120](https://github.com/FelipeMorandini/stockterm/issues/120) — error log overlay: unify visible-row count for keyboard scroll bound vs the live layout used by `draw_error_log_overlay`; **`j`/`k`** must never scroll past the last *painted* row at any terminal height after resize.
- [GitHub Issue #121](https://github.com/FelipeMorandini/stockterm/issues/121) — `draw_error_log_overlay` must not mutate `error_log_scroll`; clamp lives with input + `App::clamp_error_log_scroll()` helper. Operator behavior unchanged.
- [GitHub Issue #122](https://github.com/FelipeMorandini/stockterm/issues/122) — document `ProviderError::Clone` mapping of `Json(serde_json::Error)` → `ApiMessage` so future code does not silently miss the `Json` variant after a clone.
- [GitHub Issue #123](https://github.com/FelipeMorandini/stockterm/issues/123) — UX: plain **`q`** quits the app even while the error log overlay is open (Option 1 from the issue body).

**Prerequisite:** [PR #124](https://github.com/FelipeMorandini/stockterm/pull/124) (Issue #20) merged or in tree; [`docs/SPEC.md`](SPEC.md) §20 + §20.15 implementation matches.

### Automated (local)

1. From the repo root, with default features:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   And, for the lean build (no desktop notifications):

   ```bash
   cargo clippy --no-default-features -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All exit **0**. New unit tests per [`docs/SPEC.md`](SPEC.md) §20.15.6 are present:
   - `clamp_error_log_scroll` clamps against `error_log_visible_rows` (4 cases listed in §20.15.6).
   - Optional: `clone_of_json_becomes_api_message` regression test in [`src/api/error.rs`](../src/api/error.rs).

2. Verify [`src/app/handlers.rs`](../src/app/handlers.rs) no longer defines `ERROR_LOG_OVERLAY_VISIBLE_ROWS` (or, if retained as a *default page step* derivation, the constant is `ERROR_LOG_OVERLAY_PAGE_ROWS` only). Verify [`src/app/ui.rs`](../src/app/ui.rs) `draw_error_log_overlay` does **not** assign to `app.error_log_scroll` (read-only with respect to scroll; only `app.error_log_visible_rows` is published).

### Manual — Issue #120 (visible-row parity across resize)

> Use `tput cols && tput lines` in another shell to see the current terminal size before resizing. Most terminal emulators expose drag-resize.

1. Launch `cargo run --release`. Generate ≥ **15** distinct error log entries (e.g., set bogus symbol on Stock View and press Enter repeatedly with network off; or hammer Search with a host that 429s). Press **`Ctrl+E`** to open the overlay.
   - **Pass:** Overlay shows the most recent rows; **`j`/`k`** scroll one row at a time within the painted window; the bottom row visible in the viewport is the last reachable row via **`j`**.
2. Drag the terminal **shorter** so only ~**4** list rows fit (overlay still open).
   - **Pass:** **`j`** stops at the last *painted* row; no off-by-one scroll past the bottom; no blank rows above the painted window after pressing **`k`** repeatedly back to the top.
3. Drag the terminal **taller** (overlay still open).
   - **Pass:** The visible window grows on the next frame; **`j`** can now reach further; no entries are skipped.
4. Press **`PageDown`** then **`PageUp`**.
   - **Pass:** Page step ≤ visible rows minus one (no overshoot at small heights); **`PageUp`** returns to the top without underflow.
5. Close overlay (**Esc**), trigger more errors so the ring evicts the oldest, re-open with **`Ctrl+E`**.
   - **Pass:** `error_log_scroll` is clamped on open; no rendering past the new `max_scroll`.

### Manual — Issue #121 (draw is read-only for scroll)

> This is a code-contract item; QA mostly confirms no regression for the operator.

1. Repeat the **Issue #120** flow at default terminal size.
   - **Pass:** **`j`/`k`/`PageUp`/`PageDown`/`Esc`** behave identically to the pre-#121 baseline (no extra repaints, no flicker, no "first key after open is ignored").
2. Spot-check that overlay open → resize → key → resize → key sequence does **not** wedge `error_log_scroll` at a stale value (i.e., scrolling resumes correctly after each resize).
   - **Pass:** Each resize → next key combination scrolls within the freshly painted window.

### Manual — Issue #122 (`ProviderError::Clone` documented contract)

> Doc-only item; verification is by code review and an optional regression test.

1. **Code review** of [`src/api/error.rs`](../src/api/error.rs):
   - **Pass:** The `Json(serde_json::Error)` variant has a `///` doc explicitly stating the lossy `Clone` mapping to `ApiMessage` and the **`[parse]`** vs **`[api]`** prefix consequence.
   - **Pass:** The `impl Clone for ProviderError` block has a `///` doc above it pointing at the `Json` variant doc and noting the `Arc<serde_json::Error>` alternative as an opt-in future change.
2. **Code review** of [`src/app/app_error.rs`](../src/app/app_error.rs) `category_from_provider`:
   - **Pass:** A one-line `///` notes that `ApiMessage` arms include cloned `Json` errors.
3. (Optional) Run the recommended unit test:

   ```bash
   cargo test -p stockterm clone_of_json_becomes_api_message
   ```

   **Pass:** Test passes; cloned `ProviderError::Json` is observed as `ProviderError::ApiMessage(_)` with body containing `"Invalid JSON response:"`.

4. **Cross-check status line behavior** (sanity):
   - Provoke a JSON parse failure path (if reachable without code changes — e.g., HTML response from Yahoo where JSON was expected).
   - **Pass:** First surface renders **`[parse]`** on the status line. Subsequent re-display from `error_log` still classifies the same row as `[parse]` (because the original category was captured in `ErrorLogEntry.category` at push time — see §20.4 / [`src/app/app_error.rs`](../src/app/app_error.rs) `push_error_log`). If the same error is re-cloned and re-classified post-clone (rare path), it would render as **`[api]`** — that is the documented post-clone surface.

### Manual — Issue #123 (`q` quits while overlay open)

1. Launch `cargo run --release`. Press **`Ctrl+E`** to open the error log overlay (it's fine if the log is empty — overlay still draws).
2. Press plain **`q`** (no modifiers).
   - **Pass:** App quits immediately. Terminal is restored (raw mode disabled, alt screen left). No need to press **Esc** first.
3. Restart the app. Open the overlay (**`Ctrl+E`**). Press **`Esc`**.
   - **Pass:** Overlay closes; app **does not** quit (Esc retains its overlay-close meaning).
4. With overlay open, press **`Ctrl+R`**.
   - **Pass:** Retry triggers for the active tab's last failed fetch (regression check; the `Ctrl+R` global path still fires before the overlay early-return).
5. With overlay open, press **`Ctrl+E`** again.
   - **Pass:** Overlay closes (toggle); plain text-input keys are **not** routed to any tab handler while the overlay was open (no symbol typing, etc.).

### Regression — Stock View / Search letter typing

1. **Stock View:** Switch to Stock View tab. Type **`a`** **`a`** **`p`** **`l`**.
   - **Pass:** Symbol buffer reads `AAPL`. Pressing plain **`q`** quits (this matches pre-#123 behavior — `q` was always quit on Stock View; no new regression).
2. **Search:** Switch to Search tab. Type **`a`** **`p`** **`p`** **`l`**.
   - **Pass:** Search query reads `APPL` (uppercased). Pressing plain **`q`** quits — `q` is **not** appended to the query (matches pre-#123 behavior; documented in [`docs/SPEC.md`](SPEC.md) §20.15.4 step 4).
3. **Settings → Refresh rate edit mode:** Open Settings, Enter on the Refresh rate row. Type digits.
   - **Pass:** Editing accepts digits; plain **`q`** is *not* a digit, so per `handle_settings_events` it is ignored inside the edit branch — confirm app does **not** quit while inside Settings text edit mode (the global `q`-quit fires *before* tab dispatch, so… **actually expect: app quits**). 
   - **Pass criterion (as designed):** App quits on plain `q` even inside Settings text-edit mode (Esc cancels edit only when *not* quitting). If product later wants edit-mode-protected `q`, file a follow-up issue — out of scope for #123.

### Regression — Issue #20 (spot)

1. Confirm `Ctrl+E` overlay still lists timestamps, tab labels, category prefixes, and last lines as before.
2. Confirm `Ctrl+R` retry per-tab still works on Stock View, Charts, News, Search after a forced failure (per Issue #20 sign-off).
3. Confirm transient errors (timeout / transport) still auto-clear after the **10 s** TTL (§20.6) and sticky errors persist.

### Sign-off — Issues #120, #121, #122, #123

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests (default features) | maintainer | 2026-05-18 | Pass |
| Automated build / clippy / tests (`--no-default-features`) | maintainer | 2026-05-18 | Pass |
| New `clamp_error_log_scroll` unit tests present (§20.15.6) | maintainer | 2026-05-18 | Pass |
| Issue #120 — visible-row parity across terminal resize | maintainer | 2026-05-18 | Pass |
| Issue #120 — `PageUp`/`PageDown` adaptive page step at small heights | maintainer | 2026-05-18 | Pass |
| Issue #121 — `draw_error_log_overlay` does not write `error_log_scroll` (code review) | maintainer | 2026-05-18 | Pass |
| Issue #121 — operator behavior unchanged (resize + scroll spot-check) | maintainer | 2026-05-18 | Pass |
| Issue #122 — Rustdoc on `ProviderError::Json` + `Clone` impl present | maintainer | 2026-05-18 | Pass |
| Issue #122 — optional clone-mapping regression test passes | maintainer | 2026-05-18 | Pass |
| Issue #123 — plain `q` quits with overlay open | maintainer | 2026-05-18 | Pass |
| Issue #123 — `Esc` still closes overlay (does not quit) | maintainer | 2026-05-18 | Pass |
| Issue #123 — `Ctrl+R` still retries while overlay is open | maintainer | 2026-05-18 | Pass |
| Regression — Issue #20 overlay + retry + TTL spot-check | maintainer | 2026-05-18 | Pass |

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

## Issue #14 — Theme system (palette, Settings picker, no raw `Color::` in draw paths)

**Scope:** [GitHub Issue #14](https://github.com/FelipeMorandini/stockterm/issues/14) — full `Theme` / `ResolvedTheme`, built-in presets, Settings Theme row editing, persistence, and theme-aware rendering per [`docs/SPEC.md`](SPEC.md) §21.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §21 (approval + merge record).

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. **Unit tests:** hex parser, `Theme::resolve` / preset + overrides, serde migration from legacy `accent_hex` / `background_hex` JSON (per §21.7).

### Manual — Settings picker & live preview

1. **`cargo run --release`**, open **Settings**, move to row **3. Theme** (`j`/`k`).

2. Use the keys defined in §21.5 (e.g. **←/→** or **`h`/`l`**) to change the highlighted preset.  
   **Pass:** **Before** save, other tabs already show the **preview** palette **or** the row label updates consistently with §21 spec; no panic.

3. Press **Enter** to save (if §21 specifies Enter-to-commit).  
   **Pass:** **`~/.stockterm.json`** contains updated **`theme`**; optional **Saved** flash; switching to **Stock**, **Charts**, **Portfolio**, **Alerts**, **Search**, **News** shows recolored UI **without** restarting the binary.

4. Cycle through **Default**, **Dark**, **Light**, and **High contrast** (exact labels per implementation).  
   **Pass:** Each preset is visually distinct; borders, positive/negative numbers, and chart candle up/down colors all track the theme (no stray default-green/red islands).

### Manual — Cross-tab regression

1. With a **non-default** theme active, exercise: Stock watchlist + detail, Charts line + candle modes, Portfolio table + add dialog, Alerts table + add dialog, Search results, News list, Settings (all rows), **error** status line and **^E** error log overlay (if present).

   **Pass:** Readable contrast on a normal terminal; no garbled layout; **Esc** / **q** behaviors unchanged from pre-theme build.

### Manual — JSON custom theme

1. Quit the app. Edit **`~/.stockterm.json`** to set **`theme`** to a valid custom object per §21.8 (partial overrides allowed).

2. Relaunch **`cargo run --release`**.  
   **Pass:** Custom colors apply on startup; invalid hex strings do **not** crash — offending slots fall back to preset defaults (confirm by setting one field to **`"not-a-color"`**).

### Manual — Save failure (Issue #19 alignment)

1. If you can safely simulate a **write-protected** config path (or temp `HOME`), change theme from Settings and attempt save.  
   **Pass:** User sees **`AppError::ConfigSave`** / status error path; in-memory theme reverts or stays consistent with §21.5; no silent loss of prior **`theme`** on disk.

### Sign-off — Issue #14

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-13 | Pass |
| Settings: cycle + commit + JSON on disk | maintainer | 2026-05-13 | Pass |
| Live recolor all tabs without restart | maintainer | 2026-05-13 | Pass |
| Presets distinct + charts P/L colors | maintainer | 2026-05-13 | Pass |
| Custom JSON load + invalid hex fallback | maintainer | 2026-05-13 | Pass |
| Save failure surfaced | maintainer | 2026-05-13 | Pass |

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
| Automated build / clippy / tests | maintainer | 2026-05-18 | Pass |
| Default / missing `provider` → Yahoo | maintainer | 2026-05-18 | Pass |
| Yahoo Stock View + watchlist | maintainer | 2026-05-18 | Pass |
| Yahoo Charts / Search / News smoke | maintainer | 2026-05-18 | Pass |
| Polygon happy path | maintainer | 2026-05-18 | Pass |
| Polygon missing key negative | maintainer | 2026-05-18 | Pass |
| Errors readable; UI responsive | maintainer | 2026-05-18 | Pass |

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
| Build / clippy | maintainer | 2026-05-18 | Pass |
| Modifier helper unit tests | maintainer | 2026-05-18 | Pass |
| Stock View lowercase → uppercase buffer | maintainer | 2026-05-18 | Pass |
| Stock View Shift + letters | maintainer | 2026-05-18 | Pass |
| Ctrl/Cmd chord does not type/act | maintainer | 2026-05-18 | Pass |
| Alerts `a`/`A` add | maintainer | 2026-05-18 | Pass |
| Alerts `d`/`D` remove | maintainer | 2026-05-18 | Pass |
| Alt/Ctrl chord on Alerts | maintainer | 2026-05-18 | Pass |

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

**Canonical sign-off:** use the dedicated [**Issue #4**](#issue-4--configurable-data-refresh-interval-refresh_rate) section below. Steps here remain a quick regression subset for Issue #3 runs.

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
| Automated build / clippy | maintainer | 2026-05-18 | Pass |
| Three-row watchlist + columns | maintainer | 2026-05-18 | Pass |
| Selection drives `symbol` / detail / other tabs | maintainer | 2026-05-18 | Pass |
| Remove row updates UI + JSON | maintainer | 2026-05-18 | Pass |
| Watchlist survives restart | maintainer | 2026-05-18 | Pass |
| `refresh_rate` honored (≥ min) | maintainer | 2026-05-18 | Pass |
| Bounded fan-out / no runaway concurrency | maintainer | 2026-05-18 | Pass |
| Non-blocking input (#17) or N/A | maintainer | 2026-05-18 | Pass |
| Alerts + watchlist price (if applicable) | maintainer | 2026-05-18 | Pass |

---

## Issue #16 — Filter stocks (Portfolio + Stock View watchlist)

**Scope:** [GitHub Issue #16](https://github.com/FelipeMorandini/stockterm/issues/16) — ephemeral case-insensitive **substring** filter on the **symbol** column for **Portfolio** holdings and **Stock View** watchlist; **`/`** enters filter input mode; live table narrowing; **Esc** clears filter and restores full list; **Enter** exits input mode while keeping the current filter string; **Tab** / **Shift+Tab** still change tabs; selection stays valid on the **filtered** row set.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §23.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; unit tests for **`filter_symbol_indices`** (or equivalent) per §23.8 are present and green.

### Manual — Portfolio

**Prep:** In **`~/.stockterm.json`**, ensure **`portfolio`** has **at least three** rows whose symbols are easy to distinguish (e.g. **AAPL**, **MSFT**, **GOOGL**). Restart **`cargo run --release`**.

1. Open **Portfolio**. Press **`/`**. Type **`aa`** (lowercase).  
   **Pass:** Holdings table shows only symbols whose ticker **contains** **`AA`** case-insensitively (e.g. **AAPL**); block title reflects the active filter (e.g. contains **`filter`** and the query per §23.6).

2. Press **Enter** (commit, exit input mode). Press **`j`** / **`k`** or arrows.  
   **Pass:** Highlight moves only among **visible** (filtered) rows; no panic; selection never points past the last filtered row.

3. Press **`/`** again, **Backspace** until the query is empty (or use **Esc** per shipped UX), then **Enter** if needed to exit input mode.  
   **Pass:** Full portfolio list returns when filter is cleared.

4. Press **`/`**, type a query that matches **no** holdings (e.g. **`ZZZ`**).  
   **Pass:** Empty filtered state with a readable hint (per §23.6); **no panic**.

5. With a **non-empty** filter showing **≥1** row, press **Tab** then **Shift+Tab** to leave and return to **Portfolio**.  
   **Pass:** Filter was cleared by tab switch (full list); **`/`** starts fresh.

6. Open **add holding** dialog (**`a`**). Press **`/`**.  
   **Pass:** **`/`** does not hijack the modal (ignored or no filter mode — per §23.2.9).

### Manual — Stock View watchlist

**Prep:** **Stock View** with **≥3** watchlist symbols (e.g. **AAPL**, **MSFT**, **SPY**) per [Issue #3](#issue-3--watchlist--multi-row-quotes).

1. Press **`/`**, type **`ms`**.  
   **Pass:** Only **MSFT** (and any other symbol containing **`ms`**) remains visible; watchlist title shows active filter.

2. Press **Esc**.  
   **Pass:** Full watchlist returns; top symbol buffer / detail behavior unchanged after filter clear.

3. Press **`/`**, type **`AAPL`**, **Enter**. Press letter keys (e.g. **`X`**) **without** pressing **`/`** again.  
   **Pass:** After **Enter**, symbol-buffer typing works as before #16 (letters append to the ticker string); while **`/`** mode was active, letters went to the **filter**, not the symbol buffer.

4. With filter active (**`aa`**), press **`j`**/**`k`**.  
   **Pass:** Row highlight and **`symbol`** / detail pane stay coherent for **filtered** rows only.

5. Press **Tab** to another tab and back to **Stock View**.  
   **Pass:** Filter cleared.

### Manual — Regression (#3 / §23.7)

1. With a multi-symbol watchlist and a non-trivial **Portfolio**, confirm quote refresh still updates **all** underlying symbols (not only filtered rows) after **≥1** **`refresh_rate`** cycle or **Enter** refresh — **Pass:** rows off-filter still get prices when their rows are shown again (filter is view-only).

### Sign-off — Issue #16

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests + filter unit tests | maintainer | 2026-05-18 | Pass |
| Portfolio: `/` + substring + title | maintainer | 2026-05-18 | Pass |
| Portfolio: Enter commit + j/k on filtered rows | maintainer | 2026-05-18 | Pass |
| Portfolio: Esc clears + empty-filter UX | maintainer | 2026-05-18 | Pass |
| Portfolio: Tab clears filter | maintainer | 2026-05-18 | Pass |
| Portfolio: `/` blocked in add dialog | maintainer | 2026-05-18 | Pass |
| Stock View: filter + Esc + symbol buffer after Enter | maintainer | 2026-05-18 | Pass |
| Stock View: Tab clears filter | maintainer | 2026-05-18 | Pass |
| Quote batch still covers full symbol set | maintainer | 2026-05-18 | Pass |

---

## Issue #13 — Configurable keymap (`~/.stockterm.json`)

**Scope:** [GitHub Issue #13](https://github.com/FelipeMorandini/stockterm/issues/13) — user-editable **chord → action** map with baked-in defaults matching the pre-ship tree; invalid JSON keymap entries fall back to defaults; global and per-tab handlers resolve **`Action`** instead of ad-hoc `KeyCode` literals for covered bindings.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §24.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; unit tests for **chord parse**, **serde `Action`**, **`ResolvedKeymap` merge / duplicate handling**, and **default keymap regression** samples per §24.7 are present and green.

### Manual — Default keymap regression (no `keymap` field)

**Prep:** Backup **`~/.stockterm.json`**. Use a copy **without** a **`keymap`** key (or with **`"keymap": null`**) per §24.4.

1. Launch **`cargo run --release`**. Press **`q`**.  
   **Pass:** App exits (unchanged global quit).

2. Relaunch. Press **`Tab`** / **`Shift+Tab`** several times across **Stock View**, **Portfolio**, **Charts**, **Alerts**, **Search**, **News**, **Settings**.  
   **Pass:** Tab order matches pre–#13 behavior; no stuck focus.

3. **Stock View:** **`w`** add symbol to watchlist (if not present), **`x`** or **`Shift+d`** remove, **`j`**/**`k`** or arrows move selection, **`Enter`** refetch, **`Backspace`** on symbol buffer, type **`MSFT`** with **`letter_key_plain`** behavior per §8.  
   **Pass:** Same UX as before keymap work.

4. **Charts:** **`1`–`4`**, **`+`**/**`-`**, **`0`**, **`h`**/**`l`**/**arrows**, **`c`**.  
   **Pass:** Range, zoom, pan, candle toggle unchanged.

5. **Portfolio:** **`a`** add dialog, **`Tab`** / **`Shift+Tab`** fields, **`Esc`**, **`j`**/**`k`**, **`/`** filter (§23), **`d`** remove flow.  
   **Pass:** Unchanged.

6. **Alerts:** **`a`**/**`d`**, arrows, dialog keys.  
   **Pass:** Unchanged.

7. Trigger **error log overlay** (e.g. **`Ctrl+e`** if still the default binding after §24). Exercise **Esc**, **list scroll**, **jump keys** per shipped overlay.  
   **Pass:** If overlay keys are keymap-driven in the implementation, defaults match prior behavior; if overlay handling remains hard-coded per **§24.5** (implementation choice), document in QA notes — still **Pass** if behavior unchanged.

### Manual — Remap `Quit` (Issue #13 acceptance)

**Prep:** Edit **`~/.stockterm.json`** per README §Keymap / §24.3 — bind **`Quit`** to **`:`** (exact chord string per shipped grammar, e.g. `char::` or `semicolon` — follow README at ship time).

1. Save JSON; launch **`cargo run --release`**. Press **`:`** (with **`NONE`** modifiers unless README says otherwise).  
   **Pass:** App quits; **`q`** no longer quits **unless** the default map still assigns **`q`** (document merge rule: if user override **replaces** default quit, **`q`** may type into symbol buffer on Stock View — expected per §24.5).

2. Relaunch with **invalid** **`keymap`** (unknown action name **or** malformed chord).  
   **Pass:** App starts; status or startup banner shows a clear **`keymap:`**-style hint; **all** actions behave as **default** map (§24.2).

### Manual — One remapped tab key (spot check)

1. Remap **`NextTab`** from **`Tab`** to another chord documented as supported (e.g. **`ctrl+n`**) — only if product tests modifier chords.  
   **Pass:** New chord advances tab; old **`Tab`** either does nothing for tab switch **or** is documented as still bound — behavior matches README.

### Sign-off — Issue #13

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests + keymap unit tests | maintainer | 2026-05-18 | Pass |
| Default map: `q`, Tab, Stock View, Charts, Portfolio, Alerts | maintainer | 2026-05-18 | Pass |
| Remap `Quit` to `:` (or ship-time equivalent) | maintainer | 2026-05-18 | Pass |
| Invalid keymap → fallback + message | maintainer | 2026-05-18 | Pass |
| README keymap section matches parser | maintainer | 2026-05-18 | Pass |

---

**After implementation:** Run the relevant QA sections (#44, #3, **#16**, and/or **#13**) and record results in the sign-off tables before merge.

---

## Issue #134 — Keymap per-context overlay propagation (portfolio remove-armed)

**Scope:** [GitHub Issue #134](https://github.com/FelipeMorandini/stockterm/issues/134) — user `keymap` remaps for actions registered in more than one [`BindingLayer`](../src/config/keymap.rs) in built-in defaults (today: **`PortfolioRowUp`** / **`PortfolioRowDown`** on list **and** remove-armed) apply to **all** those layers after overlay merge.

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §25. **Depends on:** §24 / Issue **#13** keymap shipped ([`src/config/keymap.rs`](../src/config/keymap.rs)).

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; new tests per §25.6 (`action_overlay_layers`, propagated remap to `PortfolioRemoveArmed`, single-layer confirm action, conflict rejection) are green.

### Manual — Portfolio row nav while remove-armed

**Prep:** Backup **`~/.stockterm.json`**. Ensure at least **two** portfolio holdings (add via Stock View + Portfolio **`a`** if needed).

1. Edit **`keymap`** — remap row down/up, for example:

   ```json
   "keymap": {
     "char:u": "PortfolioRowDown",
     "char:p": "PortfolioRowUp"
   }
   ```

   (Use exact chord grammar from README / §24. Avoid **`char:n`** for row-down — it conflicts with armed **decline** on **`n`**.)

2. Save; launch **`cargo run --release`**. Open **Portfolio**. Press **`u`** / **`p`** (or your chosen chords).  
   **Pass:** Selection moves on the holdings table; default **`j`**/**`k`** (if unbound) do **not** move rows.

3. With a row selected, press **`d`** to arm remove. Status shows remove-armed hint. Press **`u`** / **`p`** again.  
   **Pass:** Selection still moves with the **same** remapped chords; **`j`**/**`k`** do **not** move rows while armed.

4. Press **`y`** or armed **`d`** to confirm remove (defaults) or **`Esc`** / **`n`** to cancel.  
   **Pass:** Confirm/cancel behavior unchanged from §24 / Issue **#13** QA.

### Manual — Armed-only action not leaked to list layer

1. Remap only **`PortfolioRemoveConfirm`** to a test chord (e.g. **`char:z`**) per README. Relaunch.  
   **Pass:** Confirm works on armed flow with **`z`**; main list **`d`** arm behavior unchanged unless also remapped.

### Manual — Conflict fallback (optional)

1. If §25 documents a predictable conflict (remap row-down to a chord still bound to another action on the armed layer), apply that JSON.  
   **Pass:** Startup **`keymap:`** message; defaults used; list **and** armed row nav use default **`j`**/**`k`** again.

### Sign-off — Issue #134

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests + §25.6 unit tests | maintainer | 2026-05-18 | Pass |
| Remapped row nav on portfolio list | maintainer | 2026-05-18 | Pass |
| Same remapped row nav while remove-armed | maintainer | 2026-05-18 | Pass |
| Armed-only confirm remap does not alter list layer | maintainer | 2026-05-18 | Pass |
| README notes multi-layer propagation | maintainer | 2026-05-18 | Pass |

---

## Issue #136 — Keymap phase 2 (symbol buffers + modal digit/symbol entry)

**Scope:** [GitHub Issue #136](https://github.com/FelipeMorandini/stockterm/issues/136) — after **§24** / **§25**, finish wiring **Stock View**, **Search**, **Settings** edit buffers and **portfolio** / **alert** add dialogs so typed characters and digits go through **`ResolvedKeymap`** where [`docs/SPEC.md`](SPEC.md) **§26** specifies (no shadow `KeyCode::Char` after an `Action` match); preserve **§23** filter mode, **§8** modifier rules, and **§25** overlay propagation.

**Spec:** [`docs/SPEC.md`](SPEC.md) §26.

### Preconditions

- Build matches **§26.5** (`cargo clippy -- -D warnings`, `cargo test`).
- Baseline: default `keymap` absent or `null` in `~/.stockterm.json`.

### Manual — Default keymap parity (regression)

Run the same smoke paths as Issue **#13** / **#44** for these surfaces; **Pass** = behavior matches the pre–#136 tree on each row.

| Surface | Steps |
|--------|--------|
| **Stock View** | Type a multi-letter symbol with Shift/Caps where needed (**§8**); **Enter** fetch; **Backspace**; watchlist **`w`** / **`x`** / **`j`**/**`k`**; **`/`** enters filter mode (see §23 block below). |
| **Search** | Type query with space / `-` / `.`; **Esc** clears; **Backspace**; **Enter** picks row; **j**/**k** or arrows move selection. |
| **Settings** | Browse rows; **Enter** on refresh rate + default symbol; type digits / symbol; **Esc** cancel; **Enter** commit; row **3** theme **h**/**l**/**j**/**k** unchanged. |
| **Portfolio add dialog** | **`a`** open; **Tab** (global) / **`;`** field cycle per defaults; digits and **`.`** in shares + price; **Enter** commit path; **Esc** close. |
| **Alert add dialog** | **`a`** open; **Tab** / **Shift+Tab**; **←**/**→** on Condition; symbol + threshold typing; **`;`** condition cycle when not on Condition focus; **Enter** / **Esc**. |

### Manual — §23 filter (no regression)

On **Stock View** and **Portfolio** holdings (default keymap):

1. Press **`/`** → status shows filter mode; type a substring; table narrows live.  
2. **Esc** clears query and exits filter mode; selection clamps.  
3. **Enter** exits filter mode with query applied; **Backspace** edits query while in mode.  
4. **`/`** with empty query exits filter mode (per §23).

**Pass:** Identical to Issue **#16** QA expectations.

### Manual — Remap spot checks (§26 acceptance)

Use a **temporary** `keymap` object in `~/.stockterm.json` (restore after testing). Relaunch between edits.

1. **Portfolio dialog — dialog-layer remap:** Remap **`PortfolioDialogEnter`** (or **`PortfolioDialogEsc`**) from its default chord to an otherwise-unused **`char:p`**. Open add dialog; **`p`** performs the remapped action; old chord no longer does.  
   **Pass:** No second `KeyCode` path fires the same effect; restore JSON.

2. **Portfolio dialog — digits (post–§26.4 default rows):** With **defaults**, type **`12.34`** in shares and **`56.7`** in price; commit or tab through fields.  
   **Pass:** Same parsing / validation as pre–#136; digits and **`.`** arrive only via **`PortfolioDialogDigitOrDot`** (no `KeyModifiers::NONE`-only shadow block).

3. **Settings edit (if §26 adds `SettingsEdit*` buffer actions):** Remap one digit chord for refresh-rate row; confirm digit entry follows remap.  
   **Pass:** Matches README row for the new **`Action`** name. If §26 ships without separate digit **`Action`**s, skip with note “N/A — wildcard only”.

4. **Alert threshold (if §26 adds threshold `Action`):** Remap one threshold digit chord; verify threshold field only.  
   **Pass:** Remap works; symbol field unchanged unless explicitly remapped.

5. **Collision:** Bind **`StockBackspace`** to a chord that previously was unused; verify **keymap wins** and symbol buffer does not also consume that key (**§24.5**).

### Manual — Invalid `keymap` fallback

Introduce a deliberate duplicate-chord or unknown **`Action`** if §26 adds validation paths; relaunch.  
**Pass:** **`keymap:`** prefix message; app runs on **full defaults**; Issue **#13** global **`q`** quit still works.

### Sign-off — Issue #136

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| Default parity table (Stock / Search / Settings / dialogs) | maintainer | 2026-05-18 | Pass |
| §23 filter matrix | maintainer | 2026-05-18 | Pass |
| Shift/Caps symbol typing (**§8**) on Stock View + dialogs | maintainer | 2026-05-18 | Pass |
| At least one remap spot-check + restore defaults | maintainer | 2026-05-18 | Pass |
| README Keymap table lists any **new** `Action` names from §26 | maintainer | 2026-05-18 | Pass |

---

## Issue #137 — Keymap: remappable filter-input mode (`BindingLayer::FilterInput`)

**Scope:** [GitHub Issue #137](https://github.com/FelipeMorandini/stockterm/issues/137) — while **`filter_input_mode`** is active on **Stock View** or **Portfolio** holdings, route **Esc**, **Enter**, **Backspace**, **`/`**, and filter query characters through **`ResolvedKeymap`** on **`BindingLayer::FilterInput`** instead of literal `KeyCode` matching in **`consume_filter_input_key`**. Preserve §23 filter UX with the **default** keymap; allow user remaps for filter-mode keys via **`~/.stockterm.json`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §28.

**Prerequisite:** Implementation matches §28.6–§28.7.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; **`keymap.rs`** unit tests for **`FilterInput`** defaults and at least one user-remap spot-check per §28.6.1.

### Manual — Default keymap parity (§23 regression)

Run the **§23 filter** steps from **Issue #16** and **Issue #136** (default keymap, no custom `keymap`):

| Step | Pass |
|------|------|
| **`/`** enters filter mode on Stock View + Portfolio | Same as #16 |
| Live substring narrows table | Same as #16 |
| **Esc** clears query + exits mode | Same as #16 |
| **Enter** exits mode, keeps query | Same as #16 |
| **Backspace** edits query in mode | Same as #16 |
| **`/`** with empty query exits mode | Same as #16 |
| **Tab** clears filter on tab switch | Same as #16 |
| **`/`** blocked in portfolio add dialog | Same as #16 |
| Letters in filter mode do **not** append to Stock symbol buffer | Same as #16 |

### Manual — Remap spot checks

Use a **temporary** `keymap` in `~/.stockterm.json` (restore after testing). Relaunch between edits.

1. **Filter-mode clear:** Remap **`FilterClear`** from **`esc`** to **`char:;`** (punctuation is free on **`FilterInput`** — do **not** use **`char:x`** etc., which are already **`FilterQueryChar`** defaults and make the whole overlay fall back). Enter filter mode on **Stock View**, type a query, press **`;`**.  
   **Pass:** Query cleared and filter mode exited; **Esc** no longer clears unless also remapped.

2. **Filter-mode commit:** Remap **`FilterCommit`** from **`enter`** to **`char:,`**. Enter filter mode, type **`ms`**, press **`,`**.  
   **Pass:** Filter mode exits, query **`ms`** still applied; **Enter** no longer commits unless remapped.

3. **Filter toggle (regression):** With defaults otherwise intact, remap **`StockFilterToggle`** to **`char:f`**. On Stock View (not in filter mode), **`f`** enters filter mode; **`/`** does not (unless remapped).  
   **Pass:** Toggle layer (**`StockView`**) and edit layer (**`FilterInput`**) are independent.

4. **Swallow:** In filter mode, press a key bound on **Stock View** but **not** on **FilterInput** (e.g. **`w`** for watchlist add with defaults).  
   **Pass:** Watchlist add does **not** run; filter state unchanged.

5. **Invalid keymap fallback:** Introduce a duplicate chord involving a new **`Filter*`** action; relaunch.  
   **Pass:** **`keymap:`** message; app uses full defaults; filter still works per row 1–9 above.

### Sign-off — Issue #137

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| Default §23 parity (Stock + Portfolio) | maintainer | 2026-05-18 | Pass |
| Remap `FilterClear` + `FilterCommit` | maintainer | 2026-05-18 | Pass |
| `StockFilterToggle` remap regression | maintainer | 2026-05-18 | Pass |
| Unmapped key swallowed in filter mode | maintainer | 2026-05-18 | Pass |
| Invalid keymap → defaults + filter works | maintainer | 2026-05-18 | Pass |
| README documents `FilterInput` + new `Action` names | maintainer | 2026-05-18 | Pass |

---

## Issue #139 — Keymap phase 3 (alert dialog symbol + condition actions)

**Scope:** [GitHub Issue #139](https://github.com/FelipeMorandini/stockterm/issues/139) — after **§26** / **#136**, make alert add-dialog **symbol** letters and **Condition** **`a`/`b`** fully keymap-driven via **`AlertDialogSymbolChar`**, **`AlertDialogConditionAbove`**, and **`AlertDialogConditionBelow`**; keep **`AlertDialogDigitOrDot`** for **0–9** and **`.`** on symbol/threshold fields; preserve **§8** **`letter_key_plain`** rules and default UX parity with the pre–#139 wildcard.

**Spec:** [`docs/SPEC.md`](SPEC.md) §29.

**Prerequisite:** Implementation matches §29.4–§29.5.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; **`keymap.rs`** unit tests for **`AlertDialogSymbolChar`**, **`AlertDialogConditionAbove`**, **`AlertDialogConditionBelow`**, and **`AlertDialogDigitOrDot`** counts per §29.4.1.

### Manual — Default keymap parity (regression)

Open **Alerts**, press **`a`** to open the add dialog. **Pass** = behavior matches pre–#139 on each row.

| Step | Pass |
|------|------|
| **Symbol** field: type **`aapl`** (lowercase) → buffer shows **`AAPL`** | Uppercase parity |
| **Symbol** field: type **`brk-b`** or ticker with **`.`** / **`-`** if supported | Charset unchanged |
| **Tab** / **`;`** to **Condition** | Focus moves |
| **Condition**: **`a`** → **Above**, **`b`** → **Below** | Same labels as before |
| **←** / **→** on **Condition** | **Below** / **Above** (existing structured actions) |
| **Threshold**: **`150.25`** via digits and **`.`** | Parsing unchanged |
| **Enter** through fields → commit alert | Same validation / save |
| **Esc** cancels dialog | Unchanged |

### Manual — Shift/Caps on Symbol (**§8**)

1. On **Symbol** field, use Shift or Caps so physical **`a`** types **`A`** into the buffer (ticker prefix).  
   **Pass:** **`A`** appended even though default **`char:a`** is **`AlertDialogConditionAbove`** (focus dispatch on **Symbol**).

### Manual — Remap spot checks

Use a **temporary** `keymap` in `~/.stockterm.json` (restore after testing). Relaunch between edits.

1. **Condition Above:** `"char:u": "AlertDialogConditionAbove"`. Open dialog → **Condition** focus → press **`u`**.  
   **Pass:** **Above** selected; **`a`** does not set **Above** unless remapped.

2. **Symbol letter:** `"char:p": "AlertDialogSymbolChar"` is already default; remap to `"char:9": "AlertDialogSymbolChar"` only if **`char:9`** is not used on **`AlertDialog`** (digits use **`AlertDialogDigitOrDot`** — prefer remapping a **letter** chord, e.g. `"char:x": "AlertDialogSymbolChar"` and verify **`x`** appends on **Symbol**).  
   **Pass:** Remapped chord appends on **Symbol**; old chord does not (unless wildcard fallback).

3. **Threshold digit (regression):** With defaults otherwise intact, type **`99.5`** on **Threshold**.  
   **Pass:** Still via **`AlertDialogDigitOrDot`**; symbol field not affected.

4. **Invalid keymap fallback:** Bind two different actions to the same **`AlertDialog`** chord; relaunch.  
   **Pass:** **`keymap:`** message; defaults restored; dialog typing still works per regression table.

### Sign-off — Issue #139

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| Default parity table (symbol / condition / threshold / commit) | maintainer | 2026-05-18 | Pass |
| Shift/Caps **`a`** on **Symbol** (**§8**) | maintainer | 2026-05-18 | Pass |
| Remap `AlertDialogConditionAbove` | maintainer | 2026-05-18 | Pass |
| Remap `AlertDialogSymbolChar` (one letter chord) | maintainer | 2026-05-18 | Pass |
| Threshold digits regression | maintainer | 2026-05-18 | Pass |
| Invalid keymap → defaults + dialog works | maintainer | 2026-05-18 | Pass |
| README lists new `Action` names; wildcards updated | maintainer | 2026-05-18 | Pass |

---

## Issue #15 — Layout / widget visibility customization

**Scope:** [GitHub Issue #15](https://github.com/FelipeMorandini/stockterm/issues/15) — `Config.layout` with shell visibility toggles, Stock View watchlist pane percent, Charts inner chart-height percent, optional Settings layout presets; defaults match pre-ship behavior.

**Spec:** [`docs/SPEC.md`](SPEC.md) §31.

**Prerequisite:** Implementation matches §31.4–§31.10.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; `config/layout.rs` unit tests for defaults, clamping, preset merge, and serde round-trip per §31.10.

### Manual — Default layout regression

**Prep:** Remove or omit the `layout` key from `~/.stockterm.json` (or use a fresh config). `cargo run --release`.

1. **Shell:** Tab bar (3 rows), status bar (1 row), all seven tabs reachable via **Tab** / **Shift+Tab**.  
   **Pass:** Matches pre-#15 appearance.

2. **Stock View:** Watchlist band vs detail pane proportions look unchanged (~**42%** top band).  
   **Pass:** Detail pane still usable at default terminal size (≥ 24×80).

3. **Charts:** Open **Charts** for a symbol with data.  
   **Pass:** Chart uses full tab body; key hints visible (title or chrome per implementation).

### Manual — `show_status_bar: false` (#15 acceptance)

1. Edit `~/.stockterm.json`:

   ```json
   "layout": { "show_status_bar": false }
   ```

   Restart the app.

2. Visit **Stock View**, **Portfolio**, **Alerts**, **Search**, **News**, **Charts**, **Settings**.  
   **Pass:** No bottom status row on any tab; content extends to the former status area; no stray blank bordered line.

3. Trigger a runtime status message (e.g. invalid alert threshold).  
   **Pass:** Message still surfaces via an existing channel (inline / overlay / error log) — layout change must not swallow errors silently.

### Manual — `show_tab_bar: false`

1. Set `"layout": { "show_tab_bar": false }`, restart.

2. Use **Tab** / **Shift+Tab** (or keymap tab actions) to cycle tabs.  
   **Pass:** Tabs switch; no top tab strip rendered; no panic.

### Manual — Stock View pane percent

1. Set `"layout": { "stock_view_watchlist_pct": 60 }`, restart, open **Stock View** with ≥3 watchlist rows.

2. Compare visually to default **42** (temporary A/B by editing JSON is fine).  
   **Pass:** Watchlist band is visibly taller; detail pane remains at least ~6 rows.

3. Set an out-of-range value (e.g. **5** or **95**), restart.  
   **Pass:** Clamped to **20** or **80** per §31.5 without crash.

### Manual — Charts `charts_chart_pct`

1. Set `"layout": { "charts_chart_pct": 40 }`, restart, open **Charts** with historical data loaded.

2. Set `"charts_chart_pct": 70`, restart, same symbol/range.  
   **Pass:** Chart drawing area is visibly larger at **70** than at **40**; bottom chrome strip present when **&lt; 100**.

3. Set `"charts_chart_pct": 100` (or remove key), restart.  
   **Pass:** Matches default full-body chart layout (§31.3 item 5).

### Manual — Layout presets (if Settings row shipped)

1. Open **Settings**, focus the **Layout** row, cycle presets with **←/→** or **h**/**l** without saving.  
   **Pass:** Live preview (e.g. status bar hides on **compact**) before **Enter**.

2. Select **chart_focused**, press **Enter**.  
   **Pass:** `~/.stockterm.json` contains `"preset": "chart_focused"` (and/or expected pct fields); restart preserves choice.

3. **Esc** while previewing a non-saved preset.  
   **Pass:** Reverts to last saved layout on disk.

### Manual — Persistence & docs

1. After a successful layout save, restart the app.  
   **Pass:** Layout choices persist.

2. **README** lists all `layout` fields with defaults and ranges.  
   **Pass:** Matches §31.8 examples.

### Manual — Cross-feature regression

Spot-check that layout changes do not break:

| Area | Spot-check | Pass |
|------|------------|------|
| Theme | Settings theme preview still works | |
| Filter | **/** filter on Stock View + Portfolio | |
| Keymap | **Tab** navigation with hidden tab bar | |
| Watchlist | **w** add / **j**/**k** with tall watchlist pane | |
| Charts | **1–4**, **+/-**, **h**/**l** pan/zoom at `charts_chart_pct: 70` | |

### Sign-off — Issue #15

_Manual validation passed 2026-05-17._

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-17 | Pass |
| Default layout regression (no `layout` key) | maintainer | 2026-05-17 | Pass |
| `show_status_bar: false` on all tabs | maintainer | 2026-05-17 | Pass |
| `show_tab_bar: false` + Tab navigation | maintainer | 2026-05-17 | Pass |
| `stock_view_watchlist_pct` 60 vs 42 | maintainer | 2026-05-17 | Pass |
| `charts_chart_pct` 40 vs 70 vs 100 | maintainer | 2026-05-17 | Pass |
| Settings layout presets (if shipped) | maintainer | 2026-05-17 | Pass |
| README `layout` documentation | maintainer | 2026-05-17 | Pass |
| Cross-feature regression table | maintainer | 2026-05-17 | Pass |

---

## Issue #138 — Keymap: compile-time default chord table (no `Box::leak`)

**Scope:** [GitHub Issue #138](https://github.com/FelipeMorandini/stockterm/issues/138) — replace runtime **`build_default_bindings_extended`** (`Vec` + per-chord **`Box::leak`**) with a single **`const`** default binding slice in [`src/config/keymap.rs`](../src/config/keymap.rs). **No** new keys, **no** chord or `Action` changes; regression-only verification that §24 / §26 / §28 / §29 behavior is unchanged.

**Spec:** [`docs/SPEC.md`](SPEC.md) §30.

**Prerequisite:** Implementation matches §30.5–§30.6.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; new tests **`default_bindings_total_row_count`** (`len == 220`) and **`default_bindings_slice_is_static`** (same `as_ptr()` on repeated calls) per §30.6.

2. Confirm no leak in default path:

   ```bash
   rg 'Box::leak' src/config/keymap.rs
   ```

   **Pass:** No matches (or only comments stating removed — prefer zero matches).

### Manual — Keymap regression matrix (default `keymap`)

Use **no** custom `keymap` in `~/.stockterm.json` (or remove the field). Run abbreviated spot-checks from prior keymap milestones — full sections remain in this file if any row fails.

| Area | Spot-check (from) | Pass |
|------|-------------------|------|
| Global | **Issue #13** — `q` quit, `Tab` / `Shift+Tab` tabs, `Ctrl+E` / `Ctrl+R` | Same as #13 |
| Stock View | **Issue #13** — `j`/`k` rows, symbol buffer letters | Same |
| Portfolio dialog | **Issue #136** — add holding, digits + **`.`** in share/price fields | Same |
| Filter mode | **Issue #137** — `/` toggle, type query, **Esc** clear, **Enter** commit | Same as #137 default table |
| Alert add dialog | **Issue #139** — symbol **`AAPL`**, condition **`a`/`b`**, threshold **`99.5`**, commit | Same as #139 regression table |
| News | **Issues #58, #59** — `c` copy, **Enter** open (non-blocking) | Same |
| Overlay propagation | **Issue #134** — remap `PortfolioRowDown` to `z`; list + remove-armed both use `z` | Same |

### Manual — User keymap overlay (sanity)

1. Add a **temporary** remap: bind **`Quit`** to **`colon`** via `"colon": "Quit"` in `keymap` (drop or override the default **`q`** binding if needed); relaunch.  
   **Pass:** **`:`** quits; **`q`** does not (unless bound elsewhere).

2. Restore defaults; relaunch.  
   **Pass:** **`q`** quits again.

### Sign-off — Issue #138

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-17 | Pass |
| `default_bindings().len() == 220` test | maintainer | 2026-05-17 | Pass |
| No `Box::leak` in `keymap.rs` | maintainer | 2026-05-17 | Pass |
| Regression matrix (table above) | maintainer | 2026-05-17 | Pass |
| User remap sanity (`Quit` → `colon`) | maintainer | 2026-05-17 | Pass |

---

## Issues #58, #59 — News: clipboard copy + non-blocking URL open

**Scope:**

- [Issue #58](https://github.com/FelipeMorandini/stockterm/issues/58) — copy selected article URL to the system clipboard (`pbcopy` / `wl-copy` / `xclip` / Windows `clip`); dedicated **`c`** key (default); optional fallback when browser open fails.
- [Issue #59](https://github.com/FelipeMorandini/stockterm/issues/59) — open URL via OS helper **without blocking** the TUI main loop (`spawn_blocking` + async result channel); reject non-**`http`/`https`** URLs before spawn.

**Spec:** [`docs/SPEC.md`](SPEC.md) §27.

**Prerequisite:** Implementation matches §27.4–§27.5.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; unit tests for **`normalize_article_url`** and default **`NewsCopyUrl`** binding per §27.5.

### Manual — Non-blocking open (#59)

**Prep:** `cargo run --release`, symbol with news (**`AAPL`**), open **News** tab, wait for headlines.

1. Highlight a row with a normal **`https://`** article link. Press **Enter**.  
   **Pass:** Browser opens (or OS handler runs); **immediately** press **j**/**k** or **Tab** — list selection / tab switch works with **no** multi-second freeze.

2. Rapid **Enter** twice on the same row.  
   **Pass:** At most one open attempt in flight; no runaway lag or panic.

3. **Invalid URL (if reproducible):** Use a feed row with empty URL or inject test data with **`javascript:…`** if available. Press **Enter**.  
   **Pass:** Status shows a clear rejection; OS handler **not** invoked; no panic.

### Manual — Clipboard copy (#58)

**Platform note:** Record which helper worked (**`pbcopy`** on macOS, **`wl-copy`** / **`xclip`** on Linux, **`clip`** on Windows).

1. With a row selected, press **`c`** (default **`NewsCopyUrl`**).  
   **Pass:** Short status flash **“URL copied”** (or equivalent); paste in another app shows the full **`article_url`**.

2. Remap **`NewsCopyUrl`** in `~/.stockterm.json` to another chord (e.g. **`char:y`**), relaunch, repeat copy.  
   **Pass:** New chord copies; old **`c`** does not (unless still bound elsewhere).

### Manual — Open failure → copy fallback (#58 + #59)

1. On a platform where open can fail safely (e.g. Linux without **`DISPLAY`** / misconfigured **`xdg-open`**), press **Enter** on a valid **https** row.  
   **Pass:** Either browser opens, **or** URL lands on clipboard with a message that open failed but copy succeeded, **or** a single combined error if both fail — no hang, no panic.

### Manual — Regression (M3 News)

Re-run the **News** rows from **Manual — News (#11 / #29)** above: list load, **j/k** scroll, symbol change refresh.  
**Pass:** Unchanged apart from new hints (**`c copy`**) and non-blocking **Enter**.

### Sign-off — Issues #58, #59

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests + §27.5 unit tests | maintainer | 2026-05-18 | Pass |
| Enter open: TUI stays responsive (#59) | maintainer | 2026-05-18 | Pass |
| **c** copy URL (#58) | maintainer | 2026-05-18 | Pass |
| http(s) rejection (#59) | maintainer | 2026-05-18 | Pass |
| Open-fail → copy fallback (platform-dependent) | maintainer | 2026-05-18 | Pass |
| README / UI hints mention copy key | maintainer | 2026-05-18 | Pass |
| M3 News regression (list / scroll / symbol change) | maintainer | 2026-05-18 | Pass |

---

## Issue #89 — Yahoo `yahoo_latest_quote` v7→v8 integration test

**Scope:** [GitHub Issue #89](https://github.com/FelipeMorandini/stockterm/issues/89) — prove the **two-request** quote orchestration in [`src/api/yahoo.rs`](../src/api/yahoo.rs): when **`v7/finance/quote`** fails or yields no usable bar, the provider issues **`v8/finance/chart/{symbol}?range=1d&interval=1d`** and returns the chart-mapped **`TickerResponse`**. Closes the gap noted in [`docs/SPEC.md`](SPEC.md) §17.6 after [Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2) ship.

**Spec:** [`docs/SPEC.md`](SPEC.md) §32.

**Prerequisite:** Implementation matches §32.3–§32.4 (`yahoo_latest_quote_at` test seam + **`wiremock`** tests in **`yahoo.rs`**).

### Automated (local) — required

1. From the repo root (no network required for the new tests):

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Run the Issue #89–scoped tests explicitly:

   ```bash
   cargo test wiremock_quote_fallback
   cargo test v7_malformed_json_falls_back_to_v8
   cargo test v7_empty_result_falls_back_to_v8
   cargo test v7_api_error_envelope_falls_back_to_v8
   ```

   **Pass:** Each command exits 0; failures indicate missing v7→v8 fallback or incorrect mock expectations.

3. Confirm tests do not call live Yahoo hosts:

   - Disconnect from the network (or block `query1.finance.yahoo.com`) and re-run step 2.  
   **Pass:** Same as step 2 (wiremock uses `127.0.0.1` only).

4. **Regression:** existing parser-only tests still pass:

   ```bash
   cargo test v7_envelope chart_to_ticker_fixture v7_batch_maps_rows
   ```

   **Pass:** Exit 0.

### Manual — optional live Yahoo smoke (regression only)

_Not required to close #89 if §32 automated tests pass._ Run when touching quote adapters or before a release:

1. **`provider: yahoo`** in `~/.stockterm.json` (or default). Launch **`cargo run --release`** with a liquid symbol (**AAPL**) on the watchlist.  
   **Pass:** Stock View / watchlist show a plausible latest price (no perpetual **No quote** for a valid symbol).

2. Temporarily break v7 only (not in production builds) — e.g. invalid API host in a local experiment — is **out of scope**; rely on **`wiremock`** for fallback proof.

### Sign-off — Issue #89

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| `wiremock_quote_fallback` / v7→v8 named tests (offline) | maintainer | 2026-05-18 | Pass |
| Offline re-run (no live Yahoo) | maintainer | 2026-05-18 | Pass |
| §17 parser regression tests (`v7_envelope`, `chart_to_ticker_fixture`, batch) | maintainer | 2026-05-18 | Pass |
| Optional live Yahoo smoke | maintainer | 2026-05-18 | Pass |

---

## Issue #60 — Search Esc must not clear cross-tab runtime errors

**Scope:** [GitHub Issue #60](https://github.com/FelipeMorandini/stockterm/issues/60) — pressing **Esc** on the **Search** tab clears the query and invalidates in-flight typeahead, but must **not** dismiss a status-line error that originated on another tab (e.g. a failed watchlist quote on **Stock View**). Search-originated errors **may** be cleared by **Esc** when the active error’s domain is **Search**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §33.

**Prerequisite:** Implementation matches §33.5 (`search_esc_reset` domain gate + unit tests).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Run Issue #60–scoped unit tests:

   ```bash
   cargo test search_esc_reset
   ```

   **Pass:** Exit 0; failures mean Search Esc still clears non-Search **`active_runtime_error`** (or tests missing).

### Manual — cross-tab error persistence (required)

**Setup:** Use **`provider: yahoo`** (default) or Polygon with a valid key. Have at least one symbol on the watchlist.

1. **Stock error, then Search Esc:** On **Stock View**, enter an invalid ticker (e.g. **`ZZZZINVALID`**) and press **Enter** to request a quote. Wait until the status line shows a runtime error (e.g. **`[api]`** / **`[net]`** prefix per §20). Switch to the **Search** tab, type a few characters (e.g. **`AA`**), then press **Esc** (plain, no modifiers).  
   **Pass:** Search query clears; status line **still shows** the Stock/quote error from step 1.

2. **Search error, then Search Esc:** On **Search**, with Polygon selected and **no** API key (or disconnect network if using Yahoo), type a query and wait for a search failure on the status line. Press **Esc**.  
   **Pass:** Query clears **and** the Search-domain error clears from the status line.

3. **M3 regression — Esc still clears query:** On **Search**, type **`AAPL`**, press **Esc**.  
   **Pass:** Query empty; results table cleared; no panic.

4. **Optional — error log:** Repeat step 1, open the error log overlay (**`Ctrl+E`**). Press **Esc** on **Search** tab (overlay closed).  
   **Pass:** Log entries from step 1 remain; only the **active** status slot behavior is tested in steps 1–2.

### Sign-off — Issue #60

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| `cargo test search_esc_reset` | maintainer | 2026-05-18 | Pass |
| Stock error survives Search Esc | maintainer | 2026-05-18 | Pass |
| Search error cleared by Search Esc | maintainer | 2026-05-18 | Pass |
| Search query still clears on Esc (M3) | maintainer | 2026-05-18 | Pass |

---

## Issues #90 and #91 — Yahoo quote adapter: v7→v8 debug log + v7 row symbol match

**Scope:**

- [GitHub Issue #90](https://github.com/FelipeMorandini/stockterm/issues/90) — when **`yahoo_latest_quote_at`** falls back from **`v7/finance/quote`** to **`v8/finance/chart`**, emit a **single stderr line** only if **`STOCKTERM_DEBUG_YAHOO_QUOTE=1`** (exact string).
- [GitHub Issue #91](https://github.com/FelipeMorandini/stockterm/issues/91) — **`v7_envelope_to_ticker`** must pick the row whose **`symbol`** matches the requested ticker (case-insensitive), else **`first()`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §34.

**Prerequisite:** Implementation matches §34.4–§34.5 (`yahoo.rs` helpers + unit tests; README debug table row for #90).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Run Issue #91–scoped unit tests (symbol selection):

   ```bash
   cargo test v7_envelope_picks_matching_symbol
   cargo test v7_envelope_case_insensitive
   cargo test v7_envelope_no_symbol_match_falls_back
   cargo test v7_envelope_maps_regular_market_fields
   ```

   **Pass:** Each exits 0; failures mean wrong row chosen or regression on single-row mapping.

3. Run Issue #90–scoped unit tests (debug gate only — no stderr assertion required):

   ```bash
   cargo test yahoo_quote_fallback_debug
   ```

   **Pass:** Exit 0.

4. **Regression — §32 orchestration** (unchanged v7→v8 behavior):

   ```bash
   cargo test wiremock_quote_fallback
   ```

   **Pass:** Exit 0.

5. **Regression — §9.15 batch mapping:**

   ```bash
   cargo test v7_batch_maps_rows_by_symbol_out_of_order
   ```

   **Pass:** Exit 0.

### Manual — Issue #90 (optional stderr smoke)

_Not required to close #90 if §34.4 unit tests pass._ Run once when touching quote orchestration:

1. Ensure **`provider: yahoo`** (default) and a symbol that normally quotes (**AAPL**).
2. From the repo root, run with debug enabled (stderr is separate from the TUI):

   ```bash
   STOCKTERM_DEBUG_YAHOO_QUOTE=1 cargo run --release 2> /tmp/stockterm-yahoo-quote.log
   ```

3. Use the app briefly (watchlist refresh / symbol change) so quote fetches run. Quit with **`q`**.
4. Inspect **`/tmp/stockterm-yahoo-quote.log`**.  
   **Pass:** If any v7→v8 fallbacks occurred, lines contain `yahoo quote`, the symbol, and `v8 chart`; with debug **off**, the same session produces **no** such lines in stderr.

5. **Default quiet check:** Run **`cargo run --release`** without the env var.  
   **Pass:** No `stockterm: yahoo quote` lines on stderr during normal use.

### Manual — Issue #91 (not required)

No live-Yahoo manual step is required — behavior is parser-level. Optional: if you maintain a saved **`v7`** JSON fixture with multiple **`result`** rows, confirm **`v7_envelope_to_ticker`** unit tests cover your shape.

### Sign-off — Issues #90 and #91

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy -- -D warnings` + `cargo test` | maintainer | 2026-05-18 | Pass |
| #91 symbol-match unit tests | maintainer | 2026-05-18 | Pass |
| #90 debug-gate unit tests | maintainer | 2026-05-18 | Pass |
| §32 `wiremock_quote_fallback` regression | maintainer | 2026-05-18 | Pass |
| §53 batch row-index regression | maintainer | 2026-05-18 | Pass |
| README lists `STOCKTERM_DEBUG_YAHOO_QUOTE` | maintainer | 2026-05-18 | Pass |
| Optional: stderr smoke with `=1` | maintainer | 2026-05-18 | Pass |

---

## Issue #4 — Configurable data refresh interval (`refresh_rate`)

**Scope:** [GitHub Issue #4](https://github.com/FelipeMorandini/stockterm/issues/4) — **`Config.refresh_rate`** (seconds) controls how often background network polls run for quotes (Stock View / Alerts), historical (Charts), and news (News). The UI redraw tick stays **~200 ms** via [`spawn_event_thread`](../src/app/event.rs). Overlapping quote batches must not pile up; in-flight work surfaces on the status line.

**Spec:** [`docs/SPEC.md`](SPEC.md) §35.

**Prerequisite:** Core throttle is already in-tree (shipped with Issues #3, #12, #16, #17). This section is the **canonical** sign-off for Issue #4 (supersedes the informal “Manual — Refresh cadence (#4)” bullets under **Issue #3**).

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test data_poll_interval
   ```

   **Pass:** All exit 0. After §35.6.1 lands, **`data_poll_interval_*`** unit tests must be green.

2. Optional — full regression:

   ```bash
   cargo test
   ```

   **Pass:** Exit 0.

### Manual — JSON `refresh_rate` (5 s)

**Prep:** Edit **`~/.stockterm.json`**: set **`refresh_rate`** to **`5`**, ensure **`watchlist`** has at least one symbol (e.g. **AAPL**). Restart **`cargo run --release`**.

1. Open **Stock View**. Note the **Last** price (or **Change**) for a watchlist row.
2. Wait **~15–20 s** without pressing keys that force an immediate poll (**Enter** on symbol, portfolio jump, etc.).
   **Pass:** Values update on a cadence of roughly **every ≥ 5 s** (not on every screen flicker / 200 ms tick). Updates may lag by up to **~200 ms** plus network time (tick-coalesced throttle per §35.3).
3. While quotes are fetching, glance at the bottom status line.
   **Pass:** **“Refreshing quotes…”** appears during **`stock_refresh_inflight`**; returns to normal hints when done.

### Manual — JSON `refresh_rate` (60 s)

1. Set **`refresh_rate`** to **`60`** in JSON. Restart the app. Stay on **Stock View** with a populated watchlist.
2. Observe over **~90 s**.
   **Pass:** No more than **~2** full watchlist refresh cycles in that window (allow network + jitter); UI remains responsive (**j**/**k**, **Tab**, typing still work).

### Manual — UI tick vs data refresh

1. With **`refresh_rate`** at **60**, press **Tab** rapidly and move the watchlist selection.
   **Pass:** Highlight and layout redraw smoothly (~5 Hz feel); quote **Last** does **not** change every 200 ms.

### Manual — In-flight / no pile-up

**Prep:** Use a slow network (VPN throttle, **`wiremock`** dev build, or very large watchlist on a slow link) so a quote batch takes **> 5 s**.

1. With **`refresh_rate`** at **5**, stay on **Stock View** through at least two throttle windows while the first batch is still running.
   **Pass:** Status shows **“Refreshing quotes…”**; you do **not** see multiple overlapping “refresh storms” (no unbounded parallel fan-out). After the batch completes, a later tick may start the next poll.
2. During a slow refresh, press **Enter** on the active symbol (immediate poll request).
   **Pass:** App remains responsive; when the in-flight batch finishes, **one** follow-up poll may run (**`stock_refresh_pending`** — §35.5).

### Manual — Network failure does not stop polling

1. Provoke a quote failure (invalid symbol, offline, or Polygon without API key when provider is **polygon**).
   **Pass:** Error appears on the status line (§20); app does **not** exit. After restoring network / fixing config, wait through **`refresh_rate`** interval.
   **Pass:** Subsequent polls still run (prices may update again).

### Manual — Settings tab (runtime edit)

1. Open **Settings**. Row **0** — **Refresh (seconds)**. Press **Enter**, type **`10`**, commit.
   **Pass:** **`~/.stockterm.json`** contains **`"refresh_rate":10`** after save; no panic.
2. Return to **Stock View** and observe over **~25 s**.
   **Pass:** Refresh cadence reflects **~10 s** (not the previous JSON value); after commit, poll clocks reset so the new interval can apply on the next tick (§35.6.2).

3. Try committing **`3`** in Settings.
   **Pass:** Save succeeds (integer **≥ 1**); effective poll interval is still **≥ 5 s** (floor per §35.4).

### Manual — Default `refresh_rate: 0`

1. Set **`refresh_rate`** to **`0`** in JSON (or omit field on a fresh config). Restart.
   **Pass:** Quotes still refresh on a **~30 s** cadence on Stock View (not disabled, not 200 ms).

### Regression — Issue #3 (spot)

1. Re-run **Issue #3** sign-off row **`refresh_rate` honored (≥ min)`** if throttle code changed in the same PR.

### Sign-off — Issue #4

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test data_poll_interval` | maintainer | 2026-05-18 | Pass |
| JSON `refresh_rate` 5 s cadence | maintainer | 2026-05-18 | Pass |
| JSON `refresh_rate` 60 s cadence | maintainer | 2026-05-18 | Pass |
| UI tick independent of refresh | maintainer | 2026-05-18 | Pass |
| In-flight status + no pile-up | maintainer | 2026-05-18 | Pass |
| Failure → later polls resume | maintainer | 2026-05-18 | Pass |
| Settings edit + persist | maintainer | 2026-05-18 | Pass |
| Default `0` → ~30 s effective | maintainer | 2026-05-18 | Pass |
| Issue #3 regression (if touched) | maintainer | 2026-05-18 | Pass |

---

## Issue #54 — Yahoo news: resilient `query2` parsing & attempt observability

**Scope:** [GitHub Issue #54](https://github.com/FelipeMorandini/stockterm/issues/54) — stop treating Yahoo **`query2`** JSON shape drift as a silent **“No news available”** empty feed; add optional stderr logging of search / RSS / `query2` attempt outcomes when **`STOCKTERM_DEBUG_YAHOO_NEWS=1`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §36.

**Prerequisite:** Yahoo provider with news orchestration search → RSS → `query2` (shipped with Issue #31). News tab UX from Issue #11 / §10.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test yahoo_news yahoo_search_news yahoo_rss
   ```

   **Pass:** All exit 0. After §36.4.6 lands, fixture tests for **`yahoo_news_query2_*`** must be green.

2. Optional — full regression:

   ```bash
   cargo test
   ```

   **Pass:** Exit 0.

### Manual — happy path (default env)

**Prep:** `provider` **`yahoo`** in **`~/.stockterm.json`** (default). Network available.

1. On **Stock View**, select **AAPL** (or another liquid US symbol).
2. Switch to **News**. Wait for load (status should not stay on **Loading…** indefinitely).
   **Pass:** At least one headline row with publisher + title (or a genuine **“No news available”** if the symbol truly has no headlines — rare for **AAPL**).
3. Press **j** / **k**, then **Enter** on a row with an **`https://`** URL.
   **Pass:** Browser or clipboard path still works per §27 (no regression).

### Manual — true empty vs provider error

**Goal:** After implementation, **shape drift on the last-resort `query2` path** must not masquerade as empty when search and RSS already failed.

1. **Regression check (no debug):** With normal Yahoo connectivity, repeat happy path for **MSFT** and **GOOGL**.
   **Pass:** Headlines or documented true empty; **no** spurious **“No news available”** for major symbols when network is healthy.

2. **Optional dev simulation:** If the engineer ships a test-only hook or documents a fixture-backed build, force **`yahoo_news_query2`** to receive drift JSON (per §36.4.6 unit tests) and confirm the **app** surfaces a **News** domain error on the status line (**`Ctrl+R`** retry per §20), **not** silent empty.
   **Pass:** Status shows **`ProviderError`**-style message; **`news_data`** cleared on **`Err`**; retry attempts another fetch.

### Manual — `STOCKTERM_DEBUG_YAHOO_NEWS`

1. Run with stderr captured:

   ```bash
   STOCKTERM_DEBUG_YAHOO_NEWS=1 cargo run --release 2> /tmp/stockterm-yahoo-news.log
   ```

2. Open **News** for **AAPL**; wait for load.
   **Pass:** Log contains **one line per attempt** (`search`, `rss`, `query2` as applicable) with tokens like **`ok_items`**, **`ok_empty`**, or **`err(...)`** / **`parse_mismatch`** per §36.4.4.
   **Pass:** Log does **not** dump full JSON bodies or large URL lists.

3. Unset the env var; run again.
   **Pass:** No news-related stderr during normal use.

### Manual — README

1. Open [`README.md`](../README.md) **Developer / debug**.
   **Pass:** Row for **`STOCKTERM_DEBUG_YAHOO_NEWS`** (`1` exact, describes per-attempt stderr); cross-reference §36.

### Sign-off — Issue #54

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test yahoo_news*` | maintainer | 2026-05-18 | Pass |
| Happy path headlines (AAPL) | maintainer | 2026-05-18 | Pass |
| §27 open/copy regression | maintainer | 2026-05-18 | Pass |
| Drift → error not silent empty (unit or dev hook) | maintainer | 2026-05-18 | Pass |
| Debug env stderr trail | maintainer | 2026-05-18 | Pass |
| Default env quiet stderr | maintainer | 2026-05-18 | Pass |
| README debug row | maintainer | 2026-05-18 | Pass |

---

## Issues #81, #82, #83 — Stock View status, plain Tab dialog focus, portfolio contract docs

**Scope:**

- [Issue #81](https://github.com/FelipeMorandini/stockterm/issues/81) — Stock View status hints readable on **~80-column** terminals (two-line layout when narrow — [`docs/SPEC.md`](SPEC.md) §37.1).
- [Issue #82](https://github.com/FelipeMorandini/stockterm/issues/82) — Portfolio add dialog: only **plain Tab / BackTab** cycle Shares ↔ Price; meta chords ignored while dialog open (§37.2).
- [Issue #83](https://github.com/FelipeMorandini/stockterm/issues/83) — Rustdoc + shared constant documenting **`add_to_portfolio` → `false`** vs **`inline_error`** contract (§37.3).

**Spec:** [`docs/SPEC.md`](SPEC.md) §37.

**Prerequisite:** §15 shipped (Issues #49, #67, #69). Regression: [**Issues #43, #49, #50, #67, #69**](#issues-43-49-50-67-69--alerts-polish-stock-view-hint-portfolio-dialog-tab--validation) section.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test stock_view_status
   cargo test tab_key_plain
   cargo test portfolio_try_commit
   ```

   **Pass:** All exit 0.

2. Optional — full regression:

   ```bash
   cargo test
   ```

   **Pass:** Exit 0.

### Manual — Issue #81 (narrow Stock View status)

**Prep:** Terminal **80 columns** wide (e.g. `stty cols 80` before `cargo run --release`, or resize window). **`layout.show_status_bar`** must be **true** (default / Standard preset). Open **Stock View** with a normal watchlist (no active error on status line).

1. Read the bottom status area.
   **Pass:** **Two** readable hint lines (or one line that does **not** wrap/clobber glyphs). Line 1 includes core keys (**A–Z**, **w** / **x** / **D**, **j**/**k**, **^E** / **^R**). Line 2 (or equivalent) includes the **Shift** tip for tickers starting with **w/x/j/k**.
2. Widen terminal to **≥ 120** columns (or per SPEC constant). Stay on **Stock View**.
   **Pass:** Status returns to a **single** line (same information as pre-change wide layout).
3. Provoke **“Refreshing quotes…”** (slow network or large watchlist).
   **Pass:** Single short status line; no broken two-line hint layout during inflight.
4. Switch to **Portfolio** tab.
   **Pass:** Status is still **one** row (Portfolio / global hints unchanged).

### Manual — Issue #82 (plain Tab in Portfolio dialog)

**Prep:** Valid symbol on Stock View (e.g. **AAPL**). **Portfolio** tab → **`a`** to open add dialog.

1. Press **Tab** (no modifiers) repeatedly.
   **Pass:** Focus alternates **Shares** ↔ **Price**; overlay title still mentions Tab / Shift+Tab / **`;`**.
2. Press **Shift+Tab** (or terminal **BackTab**).
   **Pass:** Focus cycles backward (same two fields).
3. Press **`;`**.
   **Pass:** Still cycles focus (unchanged §15 behavior).
4. With dialog still open, press **Ctrl+Tab** (or **Alt+Tab** if your terminal sends a distinct chord).
   **Pass:** Focus does **not** cycle; app tab does **not** change.
5. Press **Esc** to close dialog. Press **Tab**.
   **Pass:** App tab advances (global tab bar — regression #67).

### Manual — Issue #83 (documentation / contract)

**Maintainer / code review** (no dedicated TUI step unless testing failure paths):

1. Open [`src/app/app.rs`](../src/app/app.rs) **`add_to_portfolio`** and [`src/app/portfolio.rs`](../src/app/portfolio.rs) **`try_commit_portfolio_dialog`**.
   **Pass:** Rustdoc describes all **`false`** paths and the **`inline_error`** vs **`error_message`** split per §37.3.3. User string lives in **`PORTFOLIO_ADD_INVALID_SYMBOL_INLINE`** (or equivalent).
2. Confirm unit test **`portfolio_try_commit_sets_inline_error_when_add_fails_without_try_save`** references Issue **#83** in a comment.
   **Pass:** Test green; comment present.

**Optional manual — invalid symbol at commit** (hard to reach from TUI; covered by test):

- If you can force empty **`App.symbol`** at commit, **Pass:** **`inline_error`** shows the constant message; no status-bar runtime error.

### Regression — Issues #49, #67, #69 (spot)

| Check | Pass criteria |
|-------|----------------|
| #49 wide-terminal hint | On **≥ 120** cols, single-line Stock View status still shows **A–Z** + **w/x/j/k** Shift tip |
| #67 Tab closed | **Tab** switches tabs when dialog closed |
| #69 caps / parse errors | Absurd shares/price still set **`inline_error`** only |

### Sign-off — Issues #81, #82, #83

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + targeted `cargo test` | maintainer | 2026-05-18 | Pass |
| #81 narrow (80 col) two-line status | maintainer | 2026-05-18 | Pass |
| #81 wide single-line status | maintainer | 2026-05-18 | Pass |
| #81 inflight / other tabs unchanged | maintainer | 2026-05-18 | Pass |
| #82 plain Tab / Shift+Tab cycle | maintainer | 2026-05-18 | Pass |
| #82 meta Tab ignored (dialog open) | maintainer | 2026-05-18 | Pass |
| #82 global Tab when dialog closed | maintainer | 2026-05-18 | Pass |
| #83 rustdoc + constant + test comment | maintainer | 2026-05-18 | Pass |
| #49 / #67 / #69 regression | maintainer | 2026-05-18 | Pass |

---

## Issues #76, #85, #86, #117, #118 — Async / HTTP reliability tail

**Scope:**

- [Issue #76](https://github.com/FelipeMorandini/stockterm/issues/76) — **`tracing::warn!`** for dropped **`FetchDone`** / **`InflightRecovery`** sends (file log per [`docs/SPEC.md`](SPEC.md) §38.1).
- [Issue #85](https://github.com/FelipeMorandini/stockterm/issues/85) — Cap **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** at **120000** ms (§38.2).
- [Issue #86](https://github.com/FelipeMorandini/stockterm/issues/86) — **`debug_assertions`** only: log quote-batch panic payload via tracing (§38.3).
- [Issue #117](https://github.com/FelipeMorandini/stockterm/issues/117) — Retry transient HTTP **408** (§38.4).
- [Issue #118](https://github.com/FelipeMorandini/stockterm/issues/118) — **`init_shared_client()`** before TUI; exit **1** with readable stderr on failure (§38.5).

**Spec:** [`docs/SPEC.md`](SPEC.md) §38.

**Prerequisite:** §16 shipped ([#17](https://github.com/FelipeMorandini/stockterm/issues/17) / [#46](https://github.com/FelipeMorandini/stockterm/issues/46) / [#77](https://github.com/FelipeMorandini/stockterm/issues/77)); §19 / §19.13 shipped ([#18](https://github.com/FelipeMorandini/stockterm/issues/18), [#110](https://github.com/FelipeMorandini/stockterm/issues/110)–[#116](https://github.com/FelipeMorandini/stockterm/issues/116)). Regression: [**Issues #17, #46, #77**](#issues-17-46-77--async-main-loop-polish), [**Issue #18**](#issue-18--api-robustness-http-timeouts-429-backoff), [**Issues #110–#116**](#issues-110-111-112-113-114-116--19-http-post-audit-hardening).

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo build
   cargo clippy -- -D warnings
   cargo test
   cargo test four_zero_eight
   cargo test debug_http_delay
   ```

   **Pass:** All exit 0.

### Manual — Issue #76 (tracing / fetch drops)

**Prep:** Know log path — default **`{cache_dir}/stockterm/logs/stockterm.log`** (see README). Optional: **`export STOCKTERM_LOG_DIR=/tmp/stockterm-qa-logs`**.

1. Start app normally: `cargo run --release`.
   **Pass:** TUI starts; **no** `stockterm: dropped` lines on the terminal (stderr stays clean for UI).
2. Quit with **`q`**. Open the log file.
   **Pass:** File exists (or README-documented fallback if cache dir unavailable). No panic stack from logging init.
3. **Optional stress:** Rapid **`q`** during an in-flight quote refresh (large watchlist or **`STOCKTERM_DEBUG_HTTP_DELAY_MS=3000`**).
   **Pass:** If a send fails on shutdown, log contains **`WARN`** with **`dropped fetch result`** / **`channel closed`** — **not** raw URLs with **`apiKey=`**.

### Manual — Issue #85 (debug delay cap)

1. Run with absurd delay (should cap, not hang minutes):

   ```bash
   STOCKTERM_DEBUG_HTTP_DELAY_MS=999999999 cargo run --release
   ```

   On Stock View, trigger a quote refresh; time wall clock.
   **Pass:** Batch stall **≤ ~2 minutes** (120 s cap), then quotes resume or error normally; TUI still accepts input during delay (§16 regression).

2. Run capped smoke (unchanged §16 behavior):

   ```bash
   STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release
   ```

   **Pass:** ~5 s stall once per batch; keys/tabs responsive.

### Manual — Issue #86 (dev panic logging)

**Debug build only** (`cargo build` without `--release`):

1. **Code review / optional:** Inject a test-only panic in **`run_stock_quote_batch`** (local branch only) or rely on unit test noted in SPEC.
   **Pass:** Log file contains **`quote batch task panicked`** with a string payload under **`debug_assertions`**; **release** build shows **no** extra panic detail vs before.

2. **Release check:** `cargo run --release` — normal quote refresh.
   **Pass:** No new stderr noise; status line still shows generic panic message only if a real panic occurs.

### Manual — Issue #117 (408 retry)

**Maintainer / automated primary:** **`cargo test four_zero_eight`** (wiremock) — **Pass** in automated section.

**Optional live spot:** Only if you control a mock returning **408** then **200** — otherwise skip.

### Manual — Issue #118 (client init failure)

**Normal path:**

1. `cargo run --release` on a supported dev machine.
   **Pass:** App starts; Yahoo/Polygon quotes work as before.

**Failure path (optional — only if you can simulate broken TLS):**

1. Document platform-specific repro (e.g. broken **`SSL_CERT_FILE`**) in sign-off notes.
   **Pass:** Process exits **before** alternate screen with one-line **`stockterm: …`** on stderr; exit code **1**; terminal not left in raw mode.

### Regression — §16 / §19 (spot)

| Check | Pass criteria |
|-------|----------------|
| #17 delay smoke | **`STOCKTERM_DEBUG_HTTP_DELAY_MS=5000`** — UI responsive during batch |
| #18 retry | Transient **5xx** still retries (existing wiremock / live spot) |
| #110–#116 | **`ProviderError` Display** still omits **`apiKey=`** |

### Sign-off — Issues #76, #85, #86, #117, #118

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-18 | Pass |
| #76 log file + clean stderr | maintainer | 2026-05-18 | Pass |
| #85 cap 999999999 → ≤120s | maintainer | 2026-05-18 | Pass |
| #85 5000 ms smoke | maintainer | 2026-05-18 | Pass |
| #86 debug vs release panic logging | maintainer | 2026-05-18 | Pass |
| #117 wiremock 408 test | maintainer | 2026-05-18 | Pass |
| #118 normal startup | maintainer | 2026-05-18 | Pass |
| #17 / #18 regression spot | maintainer | 2026-05-18 | Pass |

---

## Issues #108, #78, #87 — Event-loop lifecycle & channel hardening

**Scope:**

- [Issue #108](https://github.com/FelipeMorandini/stockterm/issues/108) — Crossterm event thread stops within a bounded time when **`App::run`** exits; no busy loop after **`event_tx`** is dropped (see [`docs/SPEC.md`](SPEC.md) §39.1).
- [Issue #78](https://github.com/FelipeMorandini/stockterm/issues/78) — Stale-inflight watchdog clears **`*_refresh_inflight`** when both **`FetchDone`** and **`InflightRecovery`** delivery fail; centralized **`deliver_fetch_done`** (§39.2).
- [Issue #87](https://github.com/FelipeMorandini/stockterm/issues/87) — README documents **`mpsc`** policy: unbounded channels acceptable for TUI MVP; bounded **`FetchDone`** deferred (§39.3).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §39.

### Automated (local)

1. Build + lint + tests:

   ```bash
   cargo clippy -- -D warnings
   cargo test
   cargo test fetch_delivery
   ```

   **Pass:** Exit 0; **`fetch_delivery`** (or equivalent) tests cover **`deliver_fetch_done`** and stale-inflight recovery.

2. Optional — release build:

   ```bash
   cargo build --release
   ```

   **Pass:** Exit 0.

### Manual — Issue #108 (event thread shutdown)

**Prep:** Default logging (`STOCKTERM_LOG_DIR` optional). Terminal with normal TTY.

1. Run **`cargo run --release`**. Navigate tabs briefly. Press **`q`** to quit.
   **Pass:** Process exits promptly; terminal restored (cursor visible, no alternate screen); no hung CPU from a background poll loop (spot-check with Activity Monitor / **`top`** if unsure — process should be gone).
2. Run again. Press **`q`** within ~1 s of launch.
   **Pass:** Same clean exit (join path works on fast quit).
3. **Maintainer / code review:** Open [`src/app/event.rs`](../src/app/event.rs) and [`src/main.rs`](../src/main.rs).
   **Pass:** Event thread does **not** call **`disable_raw_mode`** / **`LeaveAlternateScreen`**; README **Terminal lifecycle** subsection documents main-thread ownership (§39.1).

### Manual — Issue #78 (stale inflight recovery)

**Prep:** This is hard to trigger in production without a test hook. Prefer automated tests; optional dev verification:

1. With implementation landed, confirm **`cargo test fetch_delivery`** (or project-named equivalent) passes.
2. **Regression — Charts inflight (#71):** **Charts** tab → switch symbol → wait for chart load → switch symbol again.
   **Pass:** Chart eventually loads; no permanent **“stuck”** state where changing range/symbol never refetches.
3. **Regression — Stock quotes (#17 / #76):** Set **`STOCKTERM_DEBUG_HTTP_DELAY_MS=5000`**. On **Stock View**, press keys during refresh.
   **Pass:** UI stays responsive; quotes eventually update; status does not show perpetual **Refreshing** after delay ends.
4. **Regression — News URL op:** **News** tab → **`Enter`** open / **`c`** copy on a row with valid URL.
   **Pass:** No permanent URL-op inflight lock (buttons work again on next row).

### Manual — Issue #87 (channel policy documentation)

**Maintainer / code review** (no dedicated TUI step):

1. Open [`README.md`](../README.md) Developer section.
   **Pass:** Documents unbounded **`Event`** / **`FetchDone`** policy and rationale per §39.3; states bounded queues are deferred until profiling/embedder need.

### Regression — Issues #71, #17, #76

| Check | Pass criteria |
|-------|----------------|
| #71 recovery channel | Dropped **`FetchDone`** still clears inflight via **`InflightRecovery`** under normal shutdown |
| #17 responsive UI | **`STOCKTERM_DEBUG_HTTP_DELAY_MS`** smoke — keys work during batch |
| #76 tracing | No **`eprintln!`** for fetch drops in [`src/app/app.rs`](../src/app/app.rs) delivery path |

### Sign-off — Issues #108, #78, #87

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-19 | Pass |
| #108 clean quit (`q`) ×2 | maintainer | 2026-05-19 | Pass |
| #108 README terminal lifecycle | maintainer | 2026-05-19 | Pass |
| #78 unit tests + Charts/Stock regression | maintainer | 2026-05-19 | Pass |
| #87 README channel policy | maintainer | 2026-05-19 | Pass |
| #71 / #17 / #76 regression | maintainer | 2026-05-19 | Pass |

---

## Issues #36, #56, #106 — Charts timestamps, quote semaphore, §18.15 post-audit

**Scope:**

- [Issue #36](https://github.com/FelipeMorandini/stockterm/issues/36) — Invalid / out-of-range chart timestamps must not panic; axis labels show **`"?"`** when conversion fails (see [`docs/SPEC.md`](SPEC.md) §40.1).
- [Issue #56](https://github.com/FelipeMorandini/stockterm/issues/56) — Quote fan-out tasks surface **`ProviderError::Transport`** when **`Semaphore::acquire`** fails instead of silently proceeding (§40.2).
- [Issue #106](https://github.com/FelipeMorandini/stockterm/issues/106) — Release-safe **`centered_rect`** percent clamp; coalesced notify **`body`** assembled under byte cap without full pre-truncate **`join`** (§40.3).

**Prerequisite:** Implementation matches [`docs/SPEC.md`](SPEC.md) §40.

### Automated (local)

1. Build + lint + tests:

   ```bash
   cargo clippy -- -D warnings
   cargo test
   cargo test format_time_axis
   cargo test centered_rect
   cargo test notify_body
   cargo test --no-default-features
   cargo build --release
   ```

   **Pass:** Exit 0; new tests cover **`format_time_axis`**, **`centered_rect`** overflow clamp, closed-semaphore acquire, and notify body cap assembly.

### Manual — Issue #36 (Charts timestamps)

**Prep:** Network available; default provider (Yahoo).

1. **Charts** tab → symbol with history (e.g. **AAPL**) → wait for chart load.
   **Pass:** Line or candlestick chart renders; x-axis / title show plausible dates (not blank crash).
2. Change time range **`1`–`4`**, toggle **`c`** (line ↔ candles), **`+`/`-`**, **`h`/`l`**, **`0`** reset.
   **Pass:** No process exit; no terminal corruption; viewport and labels update.
3. Zoom in until only a few bars visible.
   **Pass:** **`UTC … → …`** title still readable; worst case **`?`** labels acceptable, not panic.

### Manual — Issue #56 (quote semaphore — regression)

**Note:** Closed-semaphore failure is not practical to trigger in normal use; rely on automated test. Manual steps confirm no regression.

1. **Stock View** with a watchlist of **3+** symbols → wait for quote refresh.
   **Pass:** All symbols eventually show prices or per-symbol errors in status (no hang).
2. **Portfolio** tab with holdings → switch back to **Stock View**.
   **Pass:** Portfolio tickers included in refresh; quotes update.
3. With **`provider: yahoo`** in **`~/.stockterm.json`**, repeat (exercises **`yahoo_latest_quotes_for_symbols`** path).
   **Pass:** Same as above.

### Manual — Issue #106 (`centered_rect` + notify body)

**Layout (#106.1):**

1. **Portfolio** → **`a`** add-holding dialog.
   **Pass:** Centered modal; fields usable; **Esc** closes.
2. **Alerts** → **`a`** add-alert dialog.
   **Pass:** Centered modal; **Tab** / **←/→** on condition work; **Esc** closes.
3. **Settings** → open any edit overlay that uses a centered popup (if applicable).
   **Pass:** Popup fits terminal; no zero-width modal.

**Notify (#106.2)** — requires **`desktop-notify`** (default features):

1. Enable **Desktop alert toasts** in **Settings**; create **6+** alerts on symbols with live quotes; set thresholds so **multiple** fire on one refresh (or use very tight thresholds).
   **Pass:** One coalesced toast (or bell-only if OS denies notify); body readable, not empty garbage; no app freeze.
2. Optional: **`STOCKTERM_DEBUG_ALERT_NOTIFY=1`** → stderr shows **`Notification::show()`** result; body in log matches on-screen toast length (truncated if many long symbols).

### Regression — Charts / quotes / alerts / modals

| Check | Pass criteria |
|-------|----------------|
| §11 Charts | Symbol change clears stale series; W1 fallback still works |
| #3 watchlist | Multi-symbol table refresh |
| #93 modals | Portfolio **`55×40`**, Alerts **`55×42`** overlays unchanged for valid percents |
| #104 notify cap | Very long symbol lines still truncate in toast body |

### Sign-off — Issues #36, #56, #106

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-19 | Pass |
| #36 Charts manual smoke | maintainer | 2026-05-19 | Pass |
| #56 quote refresh regression | maintainer | 2026-05-19 | Pass |
| #106 modal + notify smoke | maintainer | 2026-05-19 | Pass |
| §11 / #3 / #93 / #104 regression | maintainer | 2026-05-19 | Pass |

---

## Issues #32, #33, #55 — Quote price lookup + `ProviderError` thiserror

**Scope:**

- [GitHub Issue #32](https://github.com/FelipeMorandini/stockterm/issues/32) — **`get_current_price`** finds cached quotes when Polygon omits **`ticker`** or when alert/portfolio symbol casing differs from watchlist keys.
- [GitHub Issue #33](https://github.com/FelipeMorandini/stockterm/issues/33) — API failures remain **`ProviderError`** through **`AppError::Provider`** with stable **`[net]` / `[api]` / `[rate]` / `[parse]`** categories (no raw **`reqwest`** strings on the status line).
- [GitHub Issue #55](https://github.com/FelipeMorandini/stockterm/issues/55) — **`ProviderError`** uses **`thiserror::Error`**; HTTP URLs stay query-stripped in **`Display`** and **`Debug`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §41.

**Prerequisite:** Implementation matches §41.1–§41.4 (`ticker_response_matches_symbol`, **`get_current_price`** tests, **`api/error.rs`** refactor).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Issue #32 — price lookup unit tests:

   ```bash
   cargo test get_current_price
   cargo test ticker_response_matches
   ```

   **Pass:** Exit 0; failures mean alert price lookup regressed on casing or empty **`ticker`**.

3. Issues #33 / #55 — provider error + classification regression:

   ```bash
   cargo test http_display_strips_query
   cargo test http_debug_redacts_query_secrets
   cargo test clone_of_json_becomes_api_message
   cargo test rate_limited_status_includes_rate_prefix_and_retry_hint
   cargo test transport_uses_net_without_double_network
   ```

   **Pass:** Exit 0; **`Display`/`Debug`** must not leak **`apiKey=`**; **`Clone`** JSON contract unchanged.

4. **Grep gate (#33)** — no raw **`reqwest::Error`** formatted into app status paths:

   ```bash
   rg 'reqwest::Error' src/app/
   ```

   **Pass:** No matches (or only comments); API layer uses **`ProviderError`** / **`map_reqwest`**.

### Manual — Issue #32 (Alerts price lookup)

**Prep:** Network available; watchlist includes **AAPL** and **MSFT**; at least one alert per symbol with a reachable threshold.

1. **Alerts** tab → add alert on **AAPL** with symbol typed as **`aapl`** (lowercase) if the add dialog allows; otherwise edit **`~/.stockterm.json`** alert entry to lowercase **`symbol`** once, then restart.
   **Pass:** After quote refresh, **Status** shows **Armed** (not **No quote**) when quotes are live; **Current** column shows a price.
2. **Stock View** → select **MSFT** → wait for detail pane quote → **Alerts** → alert on **MSFT** with threshold near spot.
   **Pass:** **Armed** / **Current** updates for MSFT even if JSON **`ticker`** field were empty (Polygon path: set **`provider: polygon`** only if you have a key; otherwise Yahoo regression in step 3).
3. **Portfolio** → holding with symbol **`aapl`** (mixed case in JSON) → **Alerts** tab alert on same symbol.
   **Pass:** Price resolves from watchlist cache or portfolio **`current_price`** back-fill.

### Manual — Issues #33 and #55 (error UX regression)

**Prep:** Optional — invalid Polygon key or airplane-mode for controlled failures.

1. **Stock View** → trigger a quote failure (bad symbol **`ZZZZINVALID`** or offline).
   **Pass:** Status line shows **`[api]`** or **`[net]`** prefix (not a raw **`reqwest`** stack string); **`apiKey=`** never appears.
2. **`Ctrl+E`** → error log overlay → recent entry matches status category.
   **Pass:** Prefix consistent; **Esc** closes overlay.
3. **`Ctrl+R`** retry after restoring network / fixing symbol.
   **Pass:** Error clears on success; transient errors auto-clear within ~10 s when appropriate (§20).

### Regression — Alerts / watchlist / §20

| Check | Pass criteria |
|-------|----------------|
| §18 Alerts | **TRIGGERED** / **Armed** / **No quote** still driven by **`triggered`** + **`get_current_price`** |
| #3 watchlist | Multi-symbol table still refreshes |
| §20 | **`Ctrl+E`** / **`Ctrl+R`**; rate-limit **`[rate] retry in …`** hint |
| #122 | Cloned **`ProviderError::Json`** still surfaces as **`[api]`** after clone (if manually tested via error log after batch failure) |

### Sign-off — Issues #32, #33, #55

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-19 | Pass |
| #32 Alerts / portfolio casing manual | maintainer | 2026-05-19 | Pass |
| #33 / #55 error prefix + redaction manual | maintainer | 2026-05-19 | Pass |
| §18 / #3 / §20 regression | maintainer | 2026-05-19 | Pass |

---

## Issues #51, #28 — Global quit modifiers + API key resolution contract

**Scope:**

- [GitHub Issue #51](https://github.com/FelipeMorandini/stockterm/issues/51) — **`q`** / **`Q`** quit with **`letter_key_plain`**; Tab / BackTab keep **`tab_key_plain`**; Ctrl/Alt/Meta chords do not quit or switch tabs accidentally.
- [GitHub Issue #28](https://github.com/FelipeMorandini/stockterm/issues/28) — Document and test that **`STOCKTERM_API_KEY`** is a **runtime overlay** only; **`try_load`** does **not** copy env into **`api_key`**; **`try_save`** does not persist env unless the user set **`api_key`** in memory.

**Spec:** [`docs/SPEC.md`](SPEC.md) §42.

**Prerequisite:** Implementation matches §42.1–§42.4 (`global_quit_key`, handler quit arm, config rustdoc + env test, README notes).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Issue #51 — keyboard helper tests:

   ```bash
   cargo test global_quit
   cargo test tab_key_plain
   ```

   **Pass:** Exit 0; **`global_quit_key`** accepts **`q`/`Q`**, rejects Ctrl/Alt chords.

3. Issue #28 — env overlay without mutation:

   ```bash
   cargo test effective_api_key_reads_env
   ```

   **Pass:** Exit 0; in-memory **`api_key`** stays empty while **`effective_api_key()`** reads env.

4. **Grep gate (#28)** — env var read sites:

   ```bash
   rg 'STOCKTERM_API_KEY' src/config/
   ```

   **Pass:** Matches only **`effective_api_key`** (and test helpers), **not** **`try_load`** / **`load_config_from_path`**.

### Manual — Issue #51 (global quit + Tab)

**Prep:** Default keymap (no custom **`keymap`** overrides in **`~/.stockterm.json`**).

1. **Stock View** → press **`q`** (lowercase, no modifiers).
   **Pass:** App exits cleanly; terminal restored (no corrupted TUI).
2. Restart → **Stock View** → **Shift+Q** (uppercase **`Q`**).
   **Pass:** App exits (§42.1 wildcard).
3. Restart → any tab → **Ctrl+Q** (if terminal delivers it).
   **Pass:** Does **not** quit (may be terminal no-op or unrelated binding).
4. **Portfolio** → **`a`** open add dialog → **Tab** (no Ctrl/Alt).
   **Pass:** Cycles Shares ↔ Price; does **not** switch to Alerts tab.
5. Close dialog → **Tab** on **Portfolio**.
   **Pass:** Switches to next app tab.
6. **Alerts** → **`a`** add dialog → **Shift+Tab** / **BackTab**.
   **Pass:** Cycles dialog fields backward; does not change app tab while dialog open.

### Manual — Issue #28 (API key resolution)

**Prep:** Backup **`~/.stockterm.json`**. Use a throwaway Polygon test key or skip live fetch if unavailable.

1. Set **`provider": "polygon"`** and **`"api_key": ""`** in config. Export **`STOCKTERM_API_KEY=<test-key>`** in the shell. Start **`cargo run`**.
   **Pass:** App does not show “missing API key” on startup; quote fetch attempts run (or fail with API error, not “missing key”).
2. While app is running, **`cat ~/.stockterm.json`** — **`api_key`** field still **`""`** (env not copied into file).
3. Quit app. Unset **`STOCKTERM_API_KEY`**. Restart without editing JSON.
   **Pass:** Polygon mode reports missing key / does not fetch until key is configured (file still empty).
4. Optional: set **`api_key`** in JSON to a value, keep env set to a **different** value. Restart.
   **Pass:** File value wins (**`effective_api_key`** prefers config per §42.2).

### Regression — §8 / §24 / §22.7.1

| Check | Pass criteria |
|-------|----------------|
| §8 Stock View | **`w`/`x`/`j`/`k`** and symbol typing unchanged |
| §24 keymap | Custom **`"colon": "Quit"`** (or other remap) still quits |
| §13 Portfolio dialog | **Tab** field cycle in add dialog still works |
| #34 Security | README still warns plaintext **`api_key`**; no new auto-write of env secrets |

### Sign-off — Issues #51, #28

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-19 | Pass |
| #51 quit + Tab manual | maintainer | 2026-05-19 | Pass |
| #28 env overlay + file unchanged manual | maintainer | 2026-05-19 | Pass |
| §8 / §24 / §13 regression | maintainer | 2026-05-19 | Pass |

---

## Issue #23 — Cryptocurrency quotes (Yahoo `BTC-USD`, formatting, Kind column)

**Scope:**

- [GitHub Issue #23](https://github.com/FelipeMorandini/stockterm/issues/23) — Track crypto in the same watchlist/charts as equities: Yahoo spot crypto **`BTC-USD`** / **`ETH-USD`** (Search), adaptive USD formatting, `SymbolKind` UI label, Stock View `-` entry, filter parity, README symbol table.

**Spec:** [`docs/SPEC.md`](SPEC.md) §43.

**Prerequisite:** Implementation matches §43.1–§43.8 (`models/symbol.rs`, `app/format.rs`, Stock View symbol charset, filter `-`/`.`, Kind column, Yahoo fixture test, README).

**Provider:** Manual steps assume **`"provider": "yahoo"`** in `~/.stockterm.json` (Polygon crypto is out of scope per §43.11).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Symbol classification:

   ```bash
   cargo test classify_symbol
   ```

   **Pass:** `BTC-USD` and `BTC` → Crypto, `AAPL` → Equity, `EURUSD=X` → Fx (or per §43.1 table).

3. Price formatting:

   ```bash
   cargo test format_usd_price
   ```

   **Pass:** Micro-price (`0.0000123`) not rendered as `$0.00`; large price (`100_000.45`) uses coarser decimals per §43.2.

4. Stock symbol charset (if named tests exist):

   ```bash
   cargo test stock_symbol_char
   ```

   **Pass:** `-` allowed; digits rejected in Stock View buffer.

5. Yahoo fixture parse (if added):

   ```bash
   cargo test yahoo_quote_btc
   ```

   **Pass:** Fixture deserializes; quote path returns finite price for `BTC-USD`.

### Manual — Issue #23 (crypto watchlist + quote)

**Prep:** `provider: "yahoo"`. Network required for live quotes. Optional: backup `~/.stockterm.json`.

1. **Search** (preferred) — query `bitcoin` → pick **`BTC-USD`** row → **Enter**.
   **Pass:** Active symbol **`BTC-USD`**; detail fetches with spot BTC price (tens of thousands USD, not ~$30 ETF).
2. **Stock View** — type `btc-usd` (no spaces), **Enter**.
   **Pass:** Symbol **`BTC-USD`**; quote succeeds on Yahoo (not HTTP **404**). Typing `btc - usd` with spaces should still work after normalize compacts to **`BTC-USD`**.
3. Press **`w`** → watchlist row **`BTC-USD`**, **Kind** **CRYPTO**, live **Last** price.
4. **Negative check:** type plain **`BTC`** only → **Enter**.
   **Pass:** Either fails or shows ETF-scale price (~$30–$60), **not** spot BTC — documents Yahoo symbol pitfall per README.
5. Add **`ETH-USD`** via Search (`ethereum`) → **`w`**.
   **Pass:** Second crypto row; independent quotes.

### Manual — Issue #23 (charts 24/7)

1. With **`BTC`** or **`BTC-USD`** active, open **Charts** tab.
2. Press **`2`** (**W1**) then **`1`** (**D1**).
   **Pass:** Candlestick/line renders; series is not empty; **W1** does not show a flat empty chart (if empty, status shows API message — note for bug).
3. Visually scan **W1** for weekend days (Sat/Sun).
   **Pass:** Bars present across weekend (crypto trades 24/7); no false “market closed” empty gap unless Yahoo returned no data (then note provider).

### Manual — Issue #23 (adaptive formatting)

1. Watchlist row for **`BTC`** (or **`BTC-USD`**) — high price.
   **Pass:** Last price readable; no column overflow off terminal edge on 80×24.
2. If available, add a very low-priced crypto ticker (e.g. a micro-cap `-USD` pair from Search) or use unit-test values only for sub-cent display.
   **Pass:** Sub-cent prices show **more than two** decimal places (not `$0.00` when price &gt; 0).

### Manual — Issue #23 (filter + equity regression)

1. Press **`/`**, type `btc`.
   **Pass:** Watchlist filters to the crypto row (`btc` matches `BTC`); **Esc** clears filter mode.
2. Switch symbol to **`AAPL`**, confirm **Kind** shows equity (blank or **EQ** per implementation).
   **Pass:** Equities unchanged; **`w`** / **`j`** / **`k`** still work (§8).

### Manual — Issue #23 (portfolio / alerts — optional)

1. **Portfolio** → add holding for active **`BTC-USD`** (if add dialog accepts symbol).
   **Pass:** Row saves; market value uses adaptive formatting.
2. **Alerts** → **`a`** → symbol buffer accepts `BTC-USD` style tickers.
   **Pass:** Alert saves; threshold display uses `format_usd_price`.

### README check

1. Open [`README.md`](../README.md) **Symbols** subsection (§43.7).
   **Pass:** Documents spot crypto **`BTC-USD`** / **`ETH-USD`**, warns plain **`BTC`** is ETF; Polygon crypto unsupported in v1.

### Regression — §3 / §8 / §11 / §23 / §34

| Check | Pass criteria |
|-------|----------------|
| §3 watchlist | Multi-symbol refresh still runs |
| §8 Stock View | `w`/`x`/`j`/`k` hotkeys; Shift-first-letter for `wmt` still documented behavior |
| §11 Charts | `AAPL` charts still load after crypto session |
| §23 filter | Equity filter `aapl` still works |
| §34 Yahoo quote | v7→v8 fallback still works for equities (stderr only when `STOCKTERM_DEBUG_YAHOO_QUOTE=1`) |

### Sign-off — Issue #23

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-19 | Pass |
| #23 watchlist + quote manual | maintainer | 2026-05-19 | Pass |
| #23 charts W1/D1 manual | maintainer | 2026-05-19 | Pass |
| #23 formatting manual | maintainer | 2026-05-19 | Pass |
| §3 / §8 / §11 regression | maintainer | 2026-05-19 | Pass |

---

## Issues #157, #158 — Provider symbol resolver + metadata-driven Kind

**Scope:**

- [GitHub Issue #157](https://github.com/FelipeMorandini/stockterm/issues/157) — Single **`resolve_provider_symbol`** entry point for Yahoo and Polygon HTTP paths; documented mapping table in SPEC/README.
- [GitHub Issue #158](https://github.com/FelipeMorandini/stockterm/issues/158) — **`SymbolKind`** from Yahoo **`quoteType`** (Search + v7 quote) with heuristic fallback; plain **`BTC`** must not show **CRYPTO** when metadata says **ETF**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §44.

**Prerequisite:** Implementation matches §44.1–§44.4 (`api/symbol.rs`, `models/symbol.rs` metadata classifiers, `App.symbol_kind_cache`, Yahoo v7 `quote_type`, UI uses `symbol_kind_for_display`).

**Provider:** Manual steps assume **`"provider": "yahoo"`** unless a step explicitly tests Polygon.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Provider symbol resolver:

   ```bash
   cargo test resolve_provider_symbol
   ```

   **Pass:** Yahoo `btc - usd` → `BTC-USD`; Polygon `aapl` → `AAPL`.

3. Metadata classification:

   ```bash
   cargo test classify_from_instrument_type
   cargo test classify_symbol_with_hint
   cargo test classify_symbol
   ```

   **Pass:** `CRYPTOCURRENCY` → Crypto; `ETF` → Equity; `BTC` + hint `ETF` → Equity; `BTC-USD` + hint `CRYPTOCURRENCY` → Crypto; `AAPL` → Equity; `EURUSD=X` → Fx.

4. Static gate — no duplicate Yahoo helper:

   ```bash
   rg 'yahoo_api_symbol' src/
   ```

   **Pass:** No matches (replaced by `resolve_provider_symbol`).

5. Yahoo v7 BTC fixture (if extended):

   ```bash
   cargo test v7_envelope_maps_btc_usd
   ```

   **Pass:** Parses `quoteType` when present in fixture.

### Manual — Issue #158 (Kind from Search / quoteType)

**Prep:** `provider: "yahoo"`. Network required. Optional: backup `~/.stockterm.json`.

1. **Search** — query `bitcoin` → select **`BTC-USD`** row (verify Search **Type** column shows cryptocurrency if visible) → **Enter**.
   **Pass:** Active symbol **`BTC-USD`**; watchlist **Kind** **CRYPTO** after **`w`** or on detail row.
2. **Search** — query `bitcoin` or `btc` → if a plain **`BTC`** row exists, select it → **Enter**.
   **Pass:** **Kind** shows **EQ** (or equity label), **not** **CRYPTO**; spot price is ETF-scale (~$30–$60), not ~$70k.
3. **Stock View** — type **`BTC-USD`**, **Enter**, add to watchlist.
   **Pass:** **Kind** **CRYPTO** persists after quote refresh (cache updated from v7 if implemented).
4. **Regression — equity / FX** — **`AAPL`** → **EQ**; **`EURUSD=X`** → **FX**.
   **Pass:** Unchanged from §43.

### Manual — Issue #157 (resolver / Yahoo 404 guard)

1. **Stock View** — type `btc - usd` (spaces), **Enter**.
   **Pass:** Symbol **`BTC-USD`**; quote succeeds (not HTTP **404** from spaced chart path).
2. **Charts** — with **`BTC-USD`** active, **W1** / **D1** load bars.
   **Pass:** Same symbol string used as watchlist (no key mismatch after refresh).
3. **Polygon smoke (optional)** — set `"provider": "polygon"` with valid API key; symbol **`AAPL`**.
   **Pass:** Quote/historical still work (resolver identity for equities); README still states Polygon crypto unsupported.

### README check

1. Open [`README.md`](../README.md) **Symbols** subsection.
   **Pass:** Documents user vs wire symbols; Yahoo vs Polygon table includes #157 mapping note; **`BTC`** ETF vs **`BTC-USD`** spot + Kind metadata behavior (#158).

### Regression — §43 / §3 / §10 / §34

| Check | Pass criteria |
|-------|----------------|
| §43 #23 crypto | `ETH-USD` via Search still works; adaptive formatting unchanged |
| §3 watchlist | Multi-symbol refresh; normalized keys in `watchlist_quotes` |
| §10 Search | Pick → Stock View + immediate poll |
| §34 Yahoo quote | v7→v8 fallback for equities |

### Sign-off — Issues #157, #158

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-20 | Pass |
| #158 Search BTC-USD / BTC Kind manual | maintainer | 2026-05-20 | Pass |
| #157 spaced symbol / chart manual | maintainer | 2026-05-20 | Pass |
| §43 / §3 regression | maintainer | 2026-05-20 | Pass |

---

## Issues #160, #161 — Provider switch Kind cache + Polygon crypto wire

**Scope:**

- [GitHub Issue #160](https://github.com/FelipeMorandini/stockterm/issues/160) — Clear **`symbol_kind_cache`** when Settings **provider** is toggled and saved; no cache persistence to disk.
- [GitHub Issue #161](https://github.com/FelipeMorandini/stockterm/issues/161) — Polygon **`resolve_provider_symbol`** maps hyphenated crypto (**`BTC-USD`**) to **`X:BTCUSD`**; Yahoo path unchanged.

**Spec:** [`docs/SPEC.md`](SPEC.md) §45.

**Prerequisite:** §44 shipped; Settings row **4** supports **Enter** provider toggle (not read-only).

**Provider:** Steps below use in-app Settings toggle unless noted.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Polygon crypto wire mapping:

   ```bash
   cargo test resolve_provider_symbol_polygon_maps_btc_usd
   cargo test resolve_provider_symbol_polygon_leaves_equity
   cargo test resolve_provider_symbol_yahoo_unchanged_for_crypto
   ```

   **Pass:** `BTC-USD` + Polygon → `X:BTCUSD`; `aapl` + Polygon → `AAPL`; `BTC-USD` + Yahoo → `BTC-USD`.

### Manual — Issue #160 (Kind cache on provider switch)

**Prep:** `provider: "yahoo"`. Watchlist includes **`BTC-USD`** (and optionally **`AAPL`**). Network required. Valid Polygon API key available for toggle test (env or `api_key`).

1. **Yahoo session** — Search `bitcoin` → pick **`BTC-USD`** → **Enter** → **`w`** to watchlist.
   **Pass:** **Kind** **CRYPTO** (Yahoo metadata).
2. **Settings** — row **4. Provider** → **Enter** to switch to **polygon** (with API key configured).
   **Pass:** Provider shows **polygon**; saved flash or no error.
3. **Stock View** — inspect **`BTC-USD`** row **before** quote refresh completes (immediately after toggle if possible).
   **Pass:** **Kind** is **not** stuck on Yahoo-only metadata — shows heuristic **CRYPTO** (suffix) or blank until Polygon refresh; must **not** incorrectly show **EQ** from stale ETF cache for a different symbol.
4. Wait for quote refresh (status clears / prices update).
   **Pass:** Quotes attempt Polygon wire (no silent Yahoo path); errors surface in status if key/plan invalid.
5. **Toggle back** — Settings row **4** → **Enter** → **yahoo**.
   **Pass:** **`symbol_kind_cache`** cleared again; after Search or quote refresh, **`BTC-USD`** **Kind** **CRYPTO** from Yahoo metadata.
6. **Polygon without key** — clear `api_key` and `STOCKTERM_API_KEY` → Settings → **Enter** on provider while on **yahoo**.
   **Pass:** Inline error; provider stays **yahoo**; cache unchanged.

### Manual — Issue #161 (Polygon crypto wire)

**Prep:** `"provider": "polygon"` with valid API key. Watchlist **`BTC-USD`** only (remove plain **`BTC`** ETF row).

1. **Stock View** — active **`BTC-USD`**, wait for quote refresh.
   **Pass:** Last price loads (non-zero spot-scale BTC price) or clear Polygon error — not permanent empty from wrong ticker namespace.
2. **Charts** — **D1** / **W1** with **`BTC-USD`** active.
   **Pass:** Bars load or provider error shown; URL path uses **`X:BTCUSD`** (optional: `STOCKTERM_DEBUG` / logs if available).
3. **Equity regression** — add **`AAPL`**, refresh.
   **Pass:** Quote works (wire symbol **`AAPL`**, not **`X:AAPL`**).

### README check

1. Open [`README.md`](../README.md) Symbols / provider wire subsection.
   **Pass:** Documents Settings provider **Enter** toggle (#160); Polygon **`BTC-USD` → `X:BTCUSD`** (#161); **`BTCUSD`** without hyphen not translated; Yahoo table unchanged.

### Regression — §44 / §43

| Check | Pass criteria |
|-------|----------------|
| §44 #157 resolver | Spaced `btc - usd` on Yahoo still → `BTC-USD` |
| §44 #158 Kind | Search **`BTC`** ETF → **EQ** on Yahoo |
| §43 #23 crypto | Yahoo **`ETH-USD`** Search + formatting unchanged |

### Sign-off — Issues #160, #161

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-20 | Pass |
| #160 provider toggle + Kind cache manual | maintainer | 2026-05-20 | Pass |
| #161 Polygon BTC-USD quote/chart manual | maintainer | 2026-05-20 | Pass |
| §44 / §43 regression | maintainer | 2026-05-20 | Pass |

---

## Issue #21 — Technical indicators (SMA / EMA / RSI / MACD)

**Scope:**

- [GitHub Issue #21](https://github.com/FelipeMorandini/stockterm/issues/21) — Pure indicator functions (`sma`, `ema`, `rsi`, `macd`) with reference fixtures; Charts tab toggles; SMA/EMA overlay on **line** chart; RSI/MACD sub-pane; disabling all indicators restores pre-#21 chart behavior.

**Spec:** [`docs/SPEC.md`](SPEC.md) §46.

**Prerequisite:** Implementation matches §46.1–§46.4 (`src/indicators/*`, `ChartIndicatorToggles`, `ChartIndicatorCache`, Charts keys **`s` / `e` / `r` / `m`**).

**Provider:** Manual steps assume **`"provider": "yahoo"`** and network for historical data. Symbol **`AAPL`** (liquid, stable daily bars).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Indicator unit tests:

   ```bash
   cargo test sma
   cargo test ema
   cargo test rsi
   cargo test macd
   ```

   **Pass:** Reference fixture comparisons within SPEC tolerance.

3. Static gate — indicator module exists:

   ```bash
   test -d src/indicators && ls src/indicators/*.rs
   ```

   **Pass:** `mod.rs`, `sma.rs`, `ema.rs`, `rsi.rs`, `macd.rs` (or equivalent per §46.1).

4. No draw-path logging:

   ```bash
   rg 'println!|eprintln!|dbg!' src/app/charts.rs
   ```

   **Pass:** No matches.

### Manual — indicator toggles & overlays (line chart)

**Prep:** Launch `cargo run --release`. Go to **Charts** tab with **`AAPL`** active. Wait for historical bars. Press **`c`** until block title shows **line** mode (not candles).

1. **Baseline** — with all indicators off, chart shows single close line only (today’s behavior).
   **Pass:** One price series; no RSI/MACD sub-pane; key hints include **`S SMA │ E EMA │ R RSI │ M MACD`** (or README equivalent).

2. **SMA(20)** — press **`s`** once.
   **Pass:** Second line appears on the price chart (smoother than close); toggling **`s`** again removes it.

3. **EMA(20)** — press **`e`** once (SMA may stay on).
   **Pass:** EMA line visible; distinct from SMA when both enabled.

4. **RSI(14)** — press **`r`** once.
   **Pass:** Chart area splits: price pane on top, RSI pane below with 0–100 scale; pan/zoom (**`h`/`l`**, **`+`/`-`**) keeps RSI aligned with visible price bars.

5. **MACD** — press **`m`** once.
   **Pass:** MACD sub-pane shows MACD line, signal line, and histogram (or documented equivalent); still readable with RSI off.

6. **RSI + MACD** — enable both **`r`** and **`m`**.
   **Pass:** Bottom band stacks RSI and MACD without overlapping labels; price pane remains usable.

7. **Disable all** — toggle **`s`**, **`e`**, **`r`**, **`m`** off until none active.
   **Pass:** Chart returns to single-pane close line only (same as step 1).

### Manual — candlestick interaction

1. Press **`c`** to **candlestick** mode with SMA or EMA still enabled (if toggles persist across mode — enable **`s`** first).
   **Pass:** Candles render; no SMA/EMA drawn through candle bodies; muted hint to switch to line chart for overlays (per §46.3).

2. Press **`c`** back to line — overlays reappear if toggles still on.
   **Pass:** SMA/EMA visible again without refetch.

### Manual — range & symbol regression

1. **`1`–`4`** — cycle **D1 / W1 / M1 / Y1** with **SMA** enabled.
   **Pass:** Indicator recomputes after new data loads; no panic; empty/error states show existing loading messages.

2. **Symbol change** — **Stock View** → **`MSFT`** → **Charts**.
   **Pass:** Prior symbol’s indicator lines do not bleed; cache rebuilds after MSFT historical loads.

### README check

1. Open [`README.md`](../README.md) Charts / keymap section.
   **Pass:** Documents **`s` / `e` / `r` / `m`** toggles and default periods (20 / 20 / 14 / 12-26-9).

### Regression — §11 / §31 / §40

| Check | Pass criteria |
|-------|----------------|
| §11 viewport | `0` full range; `+`/`-` zoom; `h`/`l` pan |
| §11 candle | `c` toggles line ↔ candles when indicators off |
| §31 layout | `charts_chart_pct: 100` → full chart; &lt;100 → chrome strip still works with sub-pane |
| §40 timestamps | No panic switching ranges with indicators on |

### Sign-off — Issue #21

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-21 | Pass |
| Indicator fixture tests | maintainer | 2026-05-21 | Pass |
| #21 SMA/EMA overlay manual | maintainer | 2026-05-21 | Pass |
| #21 RSI/MACD sub-pane manual | maintainer | 2026-05-21 | Pass |
| #21 disable-all / candle hint manual | maintainer | 2026-05-21 | Pass |
| §11 regression | maintainer | 2026-05-21 | Pass |

---

## Issue #25 — Backtesting (strategy engine + Backtest tab)

**Scope:**

- [GitHub Issue #25](https://github.com/FelipeMorandini/stockterm/issues/25) — `Strategy` trait + in-process simulator over `HistoricalData`; reference **SMA crossover** and **RSI mean-reversion** strategies; **Backtest** tab with summary stats, equity curve, trade list; configurable capital / commission / slippage; export **JSON** + **CSV**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §47.

**Prerequisite:** Implementation matches §47.1–§47.6 (`src/backtest/*`, `Tab::Backtest`, `FetchDone::Backtest`, `Config.backtest`, non-blocking `spawn_blocking` run). **Depends on** §46 indicators and §11 historical fetch.

**Provider:** Manual steps assume **`"provider": "yahoo"`** and network. Symbol **`AAPL`**, **`TimeRange::Y1`** on Charts before backtest (daily/weekly bars).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Backtest unit tests:

   ```bash
   cargo test backtest
   cargo test sma_crossover
   cargo test rsi_reversion
   ```

   **Pass:** Fixture comparisons within SPEC tolerance (§47.2.4).

3. Module layout:

   ```bash
   test -d src/backtest && ls src/backtest/*.rs
   test -f src/models/backtest.rs
   ```

   **Pass:** `mod.rs`, `strategy.rs`, `engine.rs`, `metrics.rs`, `sma_crossover.rs`, `rsi_reversion.rs` (or equivalent per §47.2).

4. No draw-path logging:

   ```bash
   rg 'println!|eprintln!|dbg!' src/app/backtest.rs
   ```

   **Pass:** No matches.

5. Config serde round-trip (if tests added per §47.7):

   ```bash
   cargo test backtest_config
   ```

   **Pass:** `BacktestConfig` defaults deserialize from `{}`.

### Manual — historical prep

**Prep:** Launch `cargo run --release`. Set symbol **`AAPL`** (Stock View). Open **Charts**, press **`4`** for **Y1**, wait until bars render (not empty / not perpetual loading).

1. **Data present** — switch to **Backtest** tab.
   **Pass:** Status or left pane indicates chart data is available (or shows bar count); no panic.

2. **No data path** — fresh session: clear symbol or use symbol with no historical load; open **Backtest** without visiting Charts.
   **Pass:** **Run** shows helpful error (no hang); UI remains responsive.

### Manual — SMA crossover (acceptance scenario)

1. On **Backtest**, cycle strategy to **SMA crossover** (**`n`** if multiple strategies).
2. Confirm defaults **50 / 200** (or README values) in left pane.
3. Press **`Enter`** or **`r`** to run.
   **Pass:** Within a few seconds, summary shows non-empty metrics; trade list has ≥1 row for 1Y **AAPL** (market-dependent — at minimum engine completes without error).

4. **Equity curve** — right pane line chart tracks equity over time.
   **Pass:** Curve visible; Y scale includes min/max equity.

5. **Scroll trades** — **`j`** / **`k`** when trade list focused.
   **Pass:** Selection moves; no crash on empty list after failed run.

### Manual — RSI mean-reversion

1. **`n`** to **RSI mean-reversion**.
2. **Run** again.
   **Pass:** Metrics update; trade list may differ from SMA; no stale SMA-only trades labeled as RSI.

### Manual — capital / commission sensitivity

1. Edit `~/.stockterm.json` (or Settings rows if shipped):

   ```json
   "backtest": { "initial_capital": 50000.0, "commission_per_trade": 5.0, "slippage_bps": 10.0 }
   ```

   Restart app or reload config per implementation.

2. **Run** same strategy again.
   **Pass:** **Total PnL** and/or **return %** change vs prior run (commission/slippage non-zero reduces net vs zero-fee baseline when trades occur).

3. Lower **`initial_capital`** to **`1000.0`**, rerun.
   **Pass:** Position sizing respects cash (no negative cash; trades still deterministic).

### Manual — export

1. After a successful run, press **`x`**.
   **Pass:** Status names `~/.stockterm/backtest_*.json` and `.csv` paths.

2. Verify files exist:

   ```bash
   ls -la ~/.stockterm/backtest_*
   ```

   **Pass:** JSON contains `summary` + `trades`; CSV has header row and trade lines.

3. **Export without run** — clear report or fresh tab, press **`x`**.
   **Pass:** Inline error or status message; no empty file overwrite.

### Manual — non-blocking UI

1. During **Run**, press **`Tab`** / arrow keys / switch to **Stock View**.
   **Pass:** UI accepts input; no multi-second freeze (work happens off UI thread per §47.3).

### README check

1. Open [`README.md`](../README.md).
   **Pass:** Documents **Backtest** tab, run/export keys, `backtest` config fields, execution assumptions (long-only, close fills, force-flat on last bar).

### Regression

| Check | Pass criteria |
|-------|----------------|
| §11 Charts | Ranges, zoom, indicators still work after Backtest tab added |
| §46 indicators | Charts **`s`/`e`/`r`/`m`** unchanged |
| §24 keymap | Tab switch to **Backtest** respects remapped `NextTab` if configured |
| §22 `last_tab` | Quit/restart restores **Backtest** when it was active (`"backtest"`) |

### Sign-off — Issue #25

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | | | |
| Backtest fixture tests | | | |
| #25 SMA run + equity curve | | | |
| #25 RSI strategy run | | | |
| #25 capital/commission sensitivity | | | |
| #25 CSV/JSON export | | | |
| #25 non-blocking run | | | |
| §11 / §46 regression | | | |

---

## Issue #22 — Options chains (Options tab)

**Scope:**

- [GitHub Issue #22](https://github.com/FelipeMorandini/stockterm/issues/22) — Yahoo options chain fetch; **Options** tab with expiration selector, synchronized calls/puts strike tables, optional Greeks columns; empty state for non-optionable symbols.

**Spec:** [`docs/SPEC.md`](SPEC.md) §48.

**Prerequisite:** Implementation matches §48.1–§48.6 (`src/models/options.rs`, `src/api/yahoo_options.rs`, `Tab::Options`, `FetchDone::Options`, `BindingLayer::Options`). **Provider:** manual steps use **`"provider": "yahoo"`** and network.

**Symbol:** **`AAPL`** (liquid options). **Negative:** **`BRK.A`** or an index without listed options (e.g. **`^GSPC`** if unsupported) for empty-state checks.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Options parser tests:

   ```bash
   cargo test options
   cargo test yahoo_options
   ```

   **Pass:** Fixture `tests/fixtures/yahoo_options_aapl.json` parses; strikes sorted; ≥1 expiration.

3. Module layout:

   ```bash
   test -f src/models/options.rs
   test -f src/api/yahoo_options.rs
   test -f src/app/options.rs || test -f src/app/options_ui.rs
   ```

   **Pass:** Files exist per §48.7.

4. No draw-path logging:

   ```bash
   rg 'println!|eprintln!|dbg!' src/app/options.rs src/app/options_ui.rs 2>/dev/null || true
   ```

   **Pass:** No matches in options draw modules.

5. Tab config round-trip:

   ```bash
   cargo test options_tab
   ```

   **Pass:** `Tab::from_config_str("options")` works (test name may vary per §48.6).

### Manual — load chain (acceptance: liquid symbol)

**Prep:** `cargo run --release`. Confirm `~/.stockterm.json` has **`"provider": "yahoo"`**. Set symbol **`AAPL`** on Stock View (wait for spot quote).

1. Switch to **Options** tab.
   **Pass:** Within a few seconds, calls and puts tables populate (or explicit loading status); UI stays responsive.

2. **Expiration header** — note displayed expiry (e.g. nearest Friday).
   **Pass:** At least one expiration label visible.

3. **`]` / `[`** (or **`l` / `h`**) — cycle expiration.
   **Pass:** Header updates; table rows change; brief loading acceptable; no panic.

4. **`j` / `k`** — move strike highlight.
   **Pass:** Highlight moves in sync on **both** calls and puts sides (same strike row).

5. **`g`** — toggle Greeks.
   **Pass:** Extra columns appear/disappear when Yahoo provides greeks; no crash when greeks absent (cells show `—`).

6. **`r`** — manual refresh.
   **Pass:** Chain reloads; status returns to ready.

### Manual — empty / non-optionable

1. Set symbol with no listed options (e.g. **`^GSPC`** or per README negative example).
2. Open **Options**, wait for fetch.
   **Pass:** Centered **“No options available”** (or equivalent §48.3 copy); no hang; terminal usable.

3. With Polygon provider selected in Settings (if implemented as §48.2 error):
   **Pass:** Clear error or hint to switch to Yahoo — not a silent empty table.

### Manual — symbol change invalidation

1. On **Options** with **AAPL** chain loaded, switch to **Stock View**, change symbol to **`MSFT`**, return to **Options**.
   **Pass:** Stale AAPL strikes not shown; fetch runs for **MSFT** (or prompt **Press r** then loads MSFT).

### Manual — non-blocking UI

1. During options load, press **`Tab`**, switch to **Charts**, return.
   **Pass:** No multi-second UI freeze; inflight clears or completes without stuck spinner.

### README check

1. Open [`README.md`](../README.md).
   **Pass:** Documents **Options** tab, expiration/strike/Greeks keys, Yahoo-only note for v1, optional **`STOCKTERM_DEBUG_YAHOO_OPTIONS`**.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §47 Backtest | **Backtest** tab still runs and exports |
| §11 Charts | Historical fetch unchanged |
| §24 keymap | Tab order includes **Options**; remapped `NextTab` still works |
| §22 `last_tab` | Quit/restart restores **Options** when active (`"options"`) |
| §9 quotes | Stock View / watchlist quotes still refresh |

### Sign-off — Issue #22

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | | | |
| Yahoo options fixture tests | | | |
| #22 AAPL chain load | | | |
| #22 expiration cycle | | | |
| #22 strike j/k sync | | | |
| #22 Greeks toggle | | | |
| #22 empty / no options | | | |
| #22 symbol invalidation | | | |
| #22 non-blocking fetch | | | |
| §47 / §11 / §24 regression | | | |

---

## Issue #165 — Backtest golden reference vectors (§47 follow-up)

**Scope:**

- [GitHub Issue #165](https://github.com/FelipeMorandini/stockterm/issues/165) — JSON golden `expected` metrics for `run_backtest` fixture(s); tolerance **§46.1** (`1e-4` abs / `1e-6` rel). No UI or strategy changes.

**Spec:** [`docs/SPEC.md`](SPEC.md) §49.1.

**Prerequisite:** Implementation matches §49.1 (`src/backtest/test_util.rs`, updated `tests/fixtures/backtest_sma_crossover_50_200.json` with required `expected` block).

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Backtest golden tests:

   ```bash
   cargo test run_backtest_fixture
   cargo test backtest
   ```

   **Pass:** `run_backtest_fixture_sma_golden_metrics` (or equivalent per §49.1) asserts `trade_count`, `total_pnl`, `total_return_pct`, `max_drawdown_pct`, `win_rate_pct`, `sharpe`, and `final_equity` against fixture `expected` — not only loose equity bounds.

3. Fixture shape:

   ```bash
   python3 -c "import json; d=json.load(open('tests/fixtures/backtest_sma_crossover_50_200.json')); assert 'expected' in d and 'trade_count' in d['expected']"
   ```

   **Pass:** Required `expected` object present (script may be adjusted to `jq` if preferred).

4. No production `println!` in backtest hot path:

   ```bash
   rg 'println!|eprintln!|dbg!' src/backtest/ 2>/dev/null || true
   ```

   **Pass:** No matches outside `#[cfg(test)]` blocks.

### Manual — none required

Issue #165 is **CI-only**; no TUI steps. Spot-check optional: run **Backtest** tab on **AAPL** once to confirm no behavioral change vs Issue #25 sign-off.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §47 Backtest tab | **Enter** / **`r`** run, export **`x`**, strategy **`n`** unchanged |
| §46 Charts indicators | Unaffected |
| §25 prior sign-off | Same hand-run outcomes qualitatively (golden tests lock metrics) |

### Sign-off — Issue #165

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-21 | Pass |
| Golden fixture `expected` block | maintainer | 2026-05-21 | Pass |
| `cargo test run_backtest_fixture` | maintainer | 2026-05-21 | Pass |
| §47 regression (optional manual) | maintainer | 2026-05-21 | Pass |

---

## Issue #168 — Yahoo options expiration slice cache (§48 follow-up)

**Scope:**

- [GitHub Issue #168](https://github.com/FelipeMorandini/stockterm/issues/168) — Reuse inline multi-expiration blocks from the first Yahoo `optionChain` response; cycle **`[` / `]`** without extra HTTP when the slice is cached; still refetch on cache miss; preserve §48.3 error behavior.

**Spec:** [`docs/SPEC.md`](SPEC.md) §49.2.

**Prerequisite:** Implementation matches §49.2 (`options_slices_by_ts`, `options_select_expiration`, multi-block parser, extended `tests/fixtures/yahoo_options_aapl.json`).

**Symbol:** **`AAPL`**. **Provider:** **`"provider": "yahoo"`**. Network required for manual cache-hit verification.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0.

2. Options parser / cache unit tests:

   ```bash
   cargo test yahoo_options
   cargo test options_select_expiration
   ```

   **Pass:** Multi-block fixture parses ≥2 slices; cache-hit test does not arm `options_inflight` when slice is seeded.

3. Extended fixture has two `options[]` blocks:

   ```bash
   python3 -c "import json; o=json.load(open('tests/fixtures/yahoo_options_aapl.json'))['optionChain']['result'][0]['options']; assert len(o)>=2"
   ```

   **Pass:** At least two expiration blocks in fixture.

### Manual — cache hit (no extra HTTP when cycling)

**Prep:** `cargo run --release`. `~/.stockterm.json` → **`"provider": "yahoo"`**. Set **`AAPL`** on Stock View.

1. Optional: enable debug logging:

   ```bash
   export STOCKTERM_DEBUG_YAHOO_OPTIONS=1
   ```

   Inspect log file (not terminal) per project logging rules.

2. Open **Options** tab; wait for initial chain load.
   **Pass:** Calls/puts populate; expiration header shows at least two dates (live Yahoo) or fixture-equivalent behavior in dev.

3. Note the **second** expiration label in the header (or cycle **`]`** once). Press **`]`** / **`[`** to move between **two expirations that were both listed immediately after the first load** (do not pick a far-dated expiry only reachable via many steps).
   **Pass:** Tables update quickly; **no** prolonged **“Loading options…”** spinner between those two expirations.

4. With **`STOCKTERM_DEBUG_YAHOO_OPTIONS=1`**, repeat step 3.
   **Pass:** Log shows **`cache hit`** (or equivalent §49.2.3 message) when switching between cached expirations; **`cache miss`** only when moving to an expiration not yet fetched.

5. Cycle to an expiration **not** present in the initial payload (many **`]`** steps until header shows a distant date, if Yahoo lists more dates than inline blocks).
   **Pass:** Brief load acceptable; chain updates; no panic.

6. **`r`** refresh on Options tab.
   **Pass:** Full reload; expirations still navigable.

### Manual — error preservation (regression §48.3)

1. With a loaded chain, simulate or wait for a failed expiration fetch (e.g. airplane mode mid-cycle to a **cache-miss** expiry, then restore network).
   **Pass:** Prior chain rows remain visible when §48.3 `preserve_chain` applies; status shows error without blanking tables.

### Manual — symbol invalidation

1. Load **AAPL** chain on **Options**; switch symbol to **MSFT** on Stock View; return to **Options**.
   **Pass:** No AAPL strikes; cache repopulates for **MSFT** (may require **`r`** per §48.3).

### Regression

| Check | Pass criteria |
|-------|----------------|
| §48 Options UX | **`g`**, **`j`/`k`**, **`r`**, Greeks toggle unchanged |
| §47 Backtest | Unaffected |
| §22 `last_tab` | **`"options"`** still restores |
| §9 quotes | Stock View refresh unchanged |

### Sign-off — Issue #168

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-21 | Pass |
| Multi-block fixture + unit tests | maintainer | 2026-05-21 | Pass |
| #168 cache hit between inline expirations | maintainer | 2026-05-21 | Pass |
| #168 cache miss distant expiration | maintainer | 2026-05-21 | Pass |
| #168 error preservation | maintainer | 2026-05-21 | Pass |
| #168 symbol invalidation | maintainer | 2026-05-21 | Pass |
| §48 / §47 regression | maintainer | 2026-05-21 | Pass |

---

## Issue #167 — Polygon.io options chain provider (§48 follow-up)

**Scope:**

- [GitHub Issue #167](https://github.com/FelipeMorandini/stockterm/issues/167) — `PolygonProvider::get_options_chain` parity with Yahoo for the **Options** tab: `v3/reference/options/contracts` (expirations) + `v3/snapshot/options/{underlying}` (calls/puts per expiration). Yahoo path unchanged.

**Spec:** [`docs/SPEC.md`](SPEC.md) §50.

**Prerequisite:** Implementation matches §50 (`src/api/polygon_options.rs`, Polygon branch in `request_options_fetch`, fixture `tests/fixtures/polygon_options_aapl_snapshot.json`, README provider note).

**Requires:** Valid Polygon API key with **Options Starter** (or higher) plan in `~/.stockterm.json` **`api_key`** or **`STOCKTERM_API_KEY`**. Free/options-basic plans may return plan errors — treat as pass if error message is clear.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test polygon_options
   cargo test options
   ```

   **Pass:** All exit 0.

2. Fixture parser:

   ```bash
   cargo test parse_snapshot_fixture -- --nocapture
   ```

   **Pass:** `tests/fixtures/polygon_options_aapl_snapshot.json` yields sorted calls/puts, non-empty expirations when wired through `build_chain` helper tests per §50.5.

3. Regression — Yahoo options unchanged:

   ```bash
   cargo test yahoo_options
   ```

   **Pass:** Existing §48 / §49.2 Yahoo tests still green.

4. Draw-path hygiene:

   ```bash
   rg 'println!|eprintln!|dbg!' src/app/options.rs 2>/dev/null || true
   ```

   **Pass:** No matches.

### Manual — Polygon chain load (acceptance)

**Prep:** `cargo run --release`. Set `~/.stockterm.json`:

```json
"provider": "polygon",
"api_key": "<your-polygon-key>"
```

(or export **`STOCKTERM_API_KEY`**). Confirm Stock View loads a spot quote for **`AAPL`** (validates key + equity wire).

1. Open **Options** tab with symbol **`AAPL`**.
   **Pass:** Within a few seconds, calls and puts tables populate (or explicit **No options available** / status error with Polygon message). UI stays responsive — **not** stuck on **“Options require Yahoo provider”**.

2. **Expiration header** — note at least one expiration label.
   **Pass:** Header shows `YYYY-MM-DD` style date(s).

3. **`]` / `[`** (or **`l` / `h`**) — cycle to a second expiration.
   **Pass:** Tables update; brief loading acceptable on first visit to a new expiration; no panic.

4. Return to the **first** expiration via **`[`**.
   **Pass:** Tables restore quickly (§49.2 session cache hit — no prolonged spinner).

5. **`j` / `k`** — strike highlight moves on **both** calls and puts.
   **Pass:** Same strike row highlighted on both sides.

6. **`g`** — Greeks toggle.
   **Pass:** Extra columns appear/disappear when Polygon returns greeks; `—` when absent.

7. **`r`** — manual refresh.
   **Pass:** Chain reloads for current symbol + expiration.

### Manual — Polygon errors

1. Set **`provider": "polygon"`** with **empty** `api_key` and no env var; open **Options**.
   **Pass:** Clear error (existing §9 `polygon_key` copy) — not a silent empty table.

2. Optional: symbol unlikely to have options (e.g. illiquid penny) or plan without options access.
   **Pass:** **No options available** or explicit API/plan message; terminal usable.

### Manual — Yahoo regression (§48 unchanged)

**Prep:** `"provider": "yahoo"`. Symbol **`AAPL`**.

1. Load **Options** tab; cycle expirations; toggle Greeks; **`r`** refresh.
   **Pass:** Same behavior as Issue **#22** sign-off (no regression from Polygon adapter).

### Manual — provider switch invalidation (§45 / §48.3)

1. Load **AAPL** chain on **Options** with **Polygon**.
2. **Settings** → switch provider to **Yahoo** → return to **Options**.
   **Pass:** No stale Polygon strikes; Yahoo fetch runs (or **Press r** then loads).
3. Switch back to **Polygon** with valid key.
   **Pass:** Polygon chain loads; no Yahoo-only error string.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §48 Yahoo Options | Issue **#22** flows still pass on Yahoo provider |
| §49.2 cache | Expiration revisit avoids refetch (Polygon) |
| §47 Backtest | Unaffected |
| §17 quotes | Polygon Stock View quotes still work |
| §22 `last_tab` | **`"options"`** restores after quit |

### Sign-off — Issue #167

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-22 | Pass |
| Polygon fixture unit tests | maintainer | 2026-05-22 | Pass |
| #167 Polygon AAPL chain load | maintainer | 2026-05-22 | Pass |
| #167 expiration cycle + cache revisit | maintainer | 2026-05-22 | Pass |
| #167 strike j/k + Greeks | maintainer | 2026-05-22 | Pass |
| #167 missing API key error | maintainer | 2026-05-22 | Pass |
| #167 Yahoo regression | maintainer | 2026-05-22 | Pass |
| #167 provider switch invalidation | maintainer | 2026-05-22 | Pass |
| §48 / §47 / §17 regression | maintainer | 2026-05-22 | Pass |

---

## Issue #171 — Polygon options expiration list cache (§50 follow-up)

**Scope:**

- [GitHub Issue #171](https://github.com/FelipeMorandini/stockterm/issues/171) — Session-cache Polygon `v3/reference/options/contracts` expiration list per wire symbol; on expiration **`[` / `]`** cycle, skip contracts when list is warm and only fetch snapshot when slice is not in `options_slices_by_ts` (**§49.2** / **#168**). Yahoo path unchanged.

**Spec:** [`docs/SPEC.md`](SPEC.md) §51.

**Prerequisite:** **§50** shipped (Issue **#167**); **§49.2** slice cache shipped (Issue **#168**).

**Requires:** Valid Polygon API key with **Options Starter** (or higher) in `~/.stockterm.json` **`api_key`** or **`STOCKTERM_API_KEY`**.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test polygon_options
   cargo test options
   ```

   **Pass:** All exit 0.

2. Unit tests for cached-expirations code path (§51.2 table) — no live contracts HTTP in the test that asserts skip behavior.

### Manual — Polygon L2 cache (contracts skip on expiration change)

Use a liquid equity with multiple listed expirations (e.g. **AAPL**). Enable debug logging:

```bash
export STOCKTERM_DEBUG_POLYGON_OPTIONS=1
# Optional: trace HTTP if project documents STOCKTERM_DEBUG_HTTP for Polygon
cargo run
```

1. **Settings** → provider **Polygon** (valid API key). **Stock View** → symbol **AAPL** → **Options** tab → **`r`** load chain.
   **Pass:** Log shows `contracts_fetch=true` (first load). Status shows multiple expirations; calls/puts populated.

2. Press **`]`** repeatedly until you land on an expiration **not** yet visited (watch log / network).
   **Pass:** Log shows `contracts_fetch=false` on slice miss (snapshot-only). No second contracts-reference request for the same wire symbol.

3. Press **`[`** to return to a previously visited expiration.
   **Pass:** Log shows slice cache hit (no new HTTP) — same as Issue **#167** / **#168** behavior.

4. Press **`r`** refresh on **Options**.
   **Pass:** `contracts_fetch=true` again (L1 + L2 cleared); full chain reloads.

### Manual — invalidation (§45 / §48.3)

1. Load **AAPL** Polygon chain on **Options**.
2. Change symbol to **MSFT** on Stock View → **Options** → **`r`**.
   **Pass:** MSFT chain loads; no stale AAPL expirations in header (`Exp n / m` matches MSFT).
3. **Settings** → toggle provider to **Yahoo** → back to **Polygon** → reload **AAPL**.
   **Pass:** Fresh contracts fetch on first Polygon load after toggle; no mixed-provider strikes.

### Manual — error preservation

1. With a loaded Polygon chain, cycle to a distant expiration (network fetch).
2. If fetch fails (simulate offline or invalid key mid-session), verify prior expiration table still visible per §48.3.
   **Pass:** Expiration-list cache not cleared on expiration-only error; user can **`[`** back to prior slice without full reload.

### Regression — Issue #168 (Yahoo)

Re-run on **Yahoo** provider (no Polygon key required):

1. **AAPL** → **Options** → **`r`** → cycle **`]`** between expirations present in initial payload.
   **Pass:** Issue **#168** cache-hit behavior unchanged (see **Issue #168** sign-off table).

| Check | Pass criteria |
|-------|----------------|
| §50 Polygon Options | Issue **#167** flows still pass |
| §49.2 slice cache | Revisit expiration avoids refetch (Yahoo + Polygon) |
| §48 Yahoo Options | Issue **#22** unchanged |
| §47 Backtest | Unaffected |

### Sign-off — Issue #171

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | maintainer | 2026-05-22 | Pass |
| #171 first load contracts+snapshot | maintainer | 2026-05-22 | Pass |
| #171 expiration change snapshot-only | maintainer | 2026-05-22 | Pass |
| #171 slice cache revisit (no HTTP) | maintainer | 2026-05-22 | Pass |
| #171 `r` refresh clears L2 | maintainer | 2026-05-22 | Pass |
| #171 symbol / provider invalidation | maintainer | 2026-05-22 | Pass |
| #171 error preservation | maintainer | 2026-05-22 | Pass |
| #168 Yahoo regression | maintainer | 2026-05-22 | Pass |
| §50 / §48 / §47 regression | maintainer | 2026-05-22 | Pass |

---

## Issue #65 — Polygon historical limit + free-tier messaging (§11 follow-up)

**Scope:**

- [GitHub Issue #65](https://github.com/FelipeMorandini/stockterm/issues/65) — Cap Polygon aggregates `limit=` per `TimeRange`; detect partial pages (`next_url` / `resultsCount`); surface **“Polygon: partial chart (plan/limit)”** on Charts without clearing last-good series; map plan/auth errors to clear `ProviderError` text.

**Spec:** [`docs/SPEC.md`](SPEC.md) §52.1.

**Prerequisite:** `provider: polygon` and a valid API key in `~/.stockterm.json` or **`STOCKTERM_API_KEY`**.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test polygon
   cargo test time_range
   ```

   **Pass:** All exit 0.

2. Unit tests per §52.1.5:

   ```bash
   cargo test polygon_historical_limit
   cargo test polygon_page_truncated
   ```

   **Pass:** Per-range limits ≤ 5000; truncation helper flags `next_url` and `results_count > results.len()`.

3. No `limit=50000` in Polygon historical URL builder:

   ```bash
   rg 'limit=50000' src/api/polygon.rs
   ```

   **Pass:** No match (limit comes from `HistoricalQuery.polygon_limit`).

### Manual — capped request + partial notice (Polygon)

**Prep:** Set **`"provider": "polygon"`** in `~/.stockterm.json`. Symbol **AAPL** (liquid).

1. **Charts** tab → press **`1`** (**D1**), wait for load.
   **Pass:** Chart renders; status/title does **not** show a panic or blank crash.

2. Optional (free-tier / tight plan): if status shows **`Polygon: partial chart (plan/limit)`** or a plan-related error string, note it in sign-off.
   **Pass:** Message is human-readable (not raw JSON); prior chart bars remain visible if a previous range had loaded (**§64**).

3. Cycle **`2`–`4`** (**W1**, **M1**, **Y1**).
   **Pass:** Each range loads or surfaces a clear provider error; UI stays responsive.

4. Switch provider to **Yahoo** in **Settings** → reload **Charts** on **AAPL**.
   **Pass:** No Polygon partial notice on Yahoo; historical behavior matches prior §11 sign-off.

### Manual — plan / key errors

1. Clear API key (empty `api_key`, unset env) → **Charts** on **AAPL**.
   **Pass:** Status/error mentions Polygon key requirement (existing §9 copy).

2. Restore key. If you have a restricted key, trigger a plan error (e.g. very old **`Y1`** on a symbol your tier blocks).
   **Pass:** `ProviderError`-style status line; no hang; terminal usable.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §11 Charts viewport | Zoom/pan **`+/-/h/l/0`** still work on loaded data |
| §64 last-good | Transient **Err** still keeps series when one exists |
| §48 Options | Unaffected by Charts-only changes |
| §47 Backtest | Unaffected |

### Sign-off — Issue #65

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + `cargo test` | | | |
| Polygon limit unit tests | | | |
| #65 D1–Y1 Charts (Polygon) | | | |
| #65 partial notice (if observed) | | | |
| #65 key/plan error copy | | | |
| Yahoo Charts regression | | | |
| §11 / §48 regression | | | |

---

## Options tab polish (§52.2)

**Scope:** Post-**§48** UX hardening shipped in the same PR as Issue **#65** — no new GitHub issue.

**Spec:** [`docs/SPEC.md`](SPEC.md) §52.2.

**Prerequisite:** Liquid symbol **AAPL**; test on **Yahoo** (primary) and spot-check **Polygon** if that PR touches options draw only indirectly.

### Automated (local) — required

1. Options module tests:

   ```bash
   cargo test options::
   ```

   **Pass:** Vol/OI formatting tests pass; existing strike/cache tests pass.

2. Draw-path hygiene:

   ```bash
   rg '\.clone\(\)' src/app/options.rs
   ```

   **Pass:** No `.clone()` in `draw_options` (clones only in Update/cache rebuild / `#[cfg(test)]`).

3. No draw logging:

   ```bash
   rg 'println!|eprintln!|dbg!' src/app/options.rs
   ```

   **Pass:** No matches.

### Manual — Vol / OI columns (Yahoo)

**Prep:** `provider: yahoo`, symbol **AAPL**, **Options** tab → **`r`** load chain.

1. Inspect **CALLS** / **PUTS** headers.
   **Pass:** Columns include **Vol** and **OI** between **Last** and **IV** (order per §52.2.1).

2. Scan rows near the money.
   **Pass:** Numeric vol/OI or **`—`** when missing; no misaligned columns.

### Manual — strike scroll + j/k sync

1. On a loaded chain with many strikes, press **`k`** repeatedly toward OTM.
   **Pass:** Highlight moves; highlighted row stays visible (table scrolls, not only a off-screen state).

2. Press **`j`** back toward ATM.
   **Pass:** Both **CALLS** and **PUTS** highlight the same strike; scroll follows on both panes.

### Manual — Greeks + refresh

1. **`g`** toggle Greeks.
   **Pass:** Extra Greek columns appear/disappear; Vol/OI/IV columns remain stable.

2. **`r`** refresh.
   **Pass:** Chain reloads; scroll/highlight sane (ATM-ish default per §48).

### Manual — narrow terminal (optional)

1. Resize terminal to ~80 columns wide; reload **Options**.
   **Pass:** Tables remain usable (no panic); columns may compress but headers readable.

### Regression

| Check | Pass criteria |
|-------|----------------|
| Issue #22 / §48 | Expiration **`[`/`]`**, cache hits (#168 Yahoo, #171 Polygon) |
| Issue #167 Polygon options | Chain still loads on Polygon provider |
| §52.1 Charts | Issue **#65** sign-off still valid |

### Sign-off — Options tab polish

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test options` | | | |
| Vol/OI columns visible | | | |
| Strike scroll + j/k sync | | | |
| Greeks toggle + refresh | | | |
| #168 / #171 cache regression | | | |
| Narrow terminal (optional) | | | |

---

## Issue #176 — Polygon historical `next_url` pagination (§52 follow-up)

**Scope:**

- [GitHub Issue #176](https://github.com/FelipeMorandini/stockterm/issues/176) — Follow Polygon `next_url` in `get_historical` until exhausted or `POLYGON_HISTORICAL_MAX_PAGES`; merge pages into one `HistoricalResponse`; keep §52 truncation notice accurate when cap-hit.

**Spec:** [`docs/SPEC.md`](SPEC.md) §53.1.

**Prerequisite:** `provider: polygon`, valid API key, liquid symbol **AAPL** (or **SPY**).

### Automated (local) — required

1. From repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test polygon
   cargo test historical
   ```

   **Pass:** All exit 0.

2. Pagination unit tests per §53.1.4:

   ```bash
   cargo test merge_historical_pages
   cargo test validate_polygon_next_url
   cargo test polygon_page_truncated
   ```

   **Pass:** Multi-page fixtures merge; foreign `next_url` host rejected.

3. Shared validator (no duplicate allowlist):

   ```bash
   rg 'fn validate_polygon_next_url' src/api/
   ```

   **Pass:** Single definition in `polygon_pagination.rs` (or documented re-export), used by `polygon.rs` and `polygon_options.rs`.

### Manual — multi-page load (Polygon)

**Prep:** `"provider": "polygon"` in `~/.stockterm.json`.

1. **Charts** tab → symbol **AAPL** → **`1`** (**D1**), wait for load.
   **Pass:** Chart renders with expected intraday density; no hang.

2. Optional: set **`STOCKTERM_DEBUG_POLYGON_HISTORICAL=1`**, reload **D1**, inspect log file (not stderr).
   **Pass:** Log shows `pages_fetched >= 1`; when Polygon returns `next_url`, log shows `pages_fetched > 1` without logging full API key.

3. If a range previously showed **`Polygon: partial chart (plan/limit)`** and pagination now completes, reload same range.
   **Pass:** Truncation suffix clears when merged series is complete; last-good bars remain during load (**§64**).

4. Cycle **`2`–`4`** (**W1**, **M1**, **Y1**).
   **Pass:** Each range loads or surfaces clear provider error; UI responsive.

### Manual — page-cap behavior (optional)

If engineer documents a test symbol/range that exceeds `POLYGON_HISTORICAL_MAX_PAGES`:

1. Load that range on Polygon.
   **Pass:** Chart still shows merged bars (partial **`Ok`**); truncation notice may remain; no panic.

### Regression

| Check | Pass criteria |
|-------|----------------|
| Issue #65 | Per-range limits still capped (no `limit=50000`) |
| §11 viewport | Zoom/pan unchanged on loaded data |
| §48 Options | Unaffected |
| Yahoo Charts | No Polygon pagination side effects |

### Sign-off — Issue #176

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo clippy` + pagination tests | maintainer | 2026-05-22 | Pass |
| #176 D1–Y1 Charts (Polygon) | maintainer | 2026-05-22 | Pass |
| #176 debug pagination log (optional) | maintainer | 2026-05-22 | Pass |
| #176 truncation notice behavior | maintainer | 2026-05-22 | Pass |
| §65 / §11 regression | maintainer | 2026-05-22 | Pass |

---

## Issue #177 — Options tab zero-clone draw (§52 follow-up)

**Scope:**

- [GitHub Issue #177](https://github.com/FelipeMorandini/stockterm/issues/177) — Remove per-frame `.clone()` of Options table rows/header in `draw_options`; use `TableState` + `render_stateful_widget` (ratatui bump only if required).

**Spec:** [`docs/SPEC.md`](SPEC.md) §53.2.

**Prerequisite:** Liquid symbol **AAPL**; test **Yahoo** (primary) and spot-check **Polygon**.

### Automated (local) — required

1. Options tests:

   ```bash
   cargo test options::
   cargo clippy -- -D warnings
   ```

   **Pass:** All exit 0.

2. Zero-clone draw guard:

   ```bash
   rg -n 'fn draw_options' -A80 src/app/options.rs | rg '\.clone\(\)'
   ```

   **Pass:** No `.clone()` inside `draw_options` (Update/cache paths may still clone).

3. Visible-row fields removed (if spec followed):

   ```bash
   rg 'call_table_visible|put_table_visible' src/app/options.rs
   ```

   **Pass:** No references (or only in migration comments/tests).

### Manual — strike scroll + highlight (Yahoo)

**Prep:** `provider: yahoo`, **AAPL**, **Options** tab → **`r`** load.

1. Press **`k`** / **`j`** through several strikes.
   **Pass:** Highlight moves; selected strike stays visible in **both** CALLS and PUTS panes (§52.2.2).

2. **`g`** toggle Greeks.
   **Pass:** Columns expand/collapse; scroll/highlight stable.

3. **`[` / `]`** change expiration (if multiple dates).
   **Pass:** Chain swaps; scroll resets sensibly; no panic.

### Manual — Polygon spot-check

1. Switch to **Polygon** provider → **Options** → **`r`** on **AAPL**.
   **Pass:** Chain loads; strike navigation still works.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §52.2 Vol/OI | Columns still present |
| #168 / #171 | Expiration cache hits still work |
| §52.1 Charts | Unaffected |
| Other tabs | Watchlist/portfolio tables still render |

### Sign-off — Issue #177

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test options` + clippy | maintainer | 2026-05-22 | Pass |
| draw_options no clone (rg) | maintainer | 2026-05-22 | Pass |
| Strike scroll + j/k (Yahoo) | maintainer | 2026-05-22 | Pass |
| Greeks + expiration nav | maintainer | 2026-05-22 | Pass |
| Polygon options spot-check | maintainer | 2026-05-22 | Pass |
| §52.2 / #168 / #171 regression | maintainer | 2026-05-22 | Pass |

---

## Issue #180 — Persist Charts `time_range` and `chart_mode` (§11 / §22 follow-up)

**Scope:**

- [GitHub Issue #180](https://github.com/FelipeMorandini/stockterm/issues/180) — Save selected **`TimeRange`** (`1`–`4`) and line vs candlestick mode to **`~/.stockterm.json`**; restore on launch via the same debounced session-save path as **`last_tab`** / **`last_symbol`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §54.

**Prerequisite:** Yahoo provider (default), liquid symbol **AAPL** (or any symbol with historical data).

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Config serde tests (after implementation):

   ```bash
   cargo test config::tests::serde_last_time_range
   cargo test config::tests::serde_last_chart_mode
   cargo test time_range::tests::from_config_str
   cargo test charts::tests::chart_display_mode_from_config_str
   ```

   **Pass:** All named tests exist and pass (exact module paths may vary — run `cargo test last_time_range` / `cargo test from_config_str` if names differ).

3. Sync helper writes chart fields:

   ```bash
   rg 'last_time_range|last_chart_mode' src/app/app.rs src/config/config.rs
   ```

   **Pass:** Fields present in `Config`; `sync_session_fields_into_config` assigns both.

### Manual — persist time range

**Prep:** Back up `~/.stockterm.json` if needed. Note whether `last_time_range` / `last_chart_mode` keys exist (they should not before this feature).

1. Launch StockTerm → **Charts** tab → press **`1`** (1D).
   **Pass:** Chart title/status shows **1D** (or equivalent label).

2. Wait **≥ 1 s** (debounce + tick) or quit with **`q`**.
   **Pass:** `~/.stockterm.json` contains `"last_time_range": "d1"` (or equivalent documented string).

3. Quit and relaunch.
   **Pass:** **Charts** tab (or navigate there) still shows **1D** without pressing **`1`** again; historical fetch runs for the 1D window.

### Manual — persist chart mode

1. On **Charts**, press **`c`** until candlestick mode is active (title shows **candles** / candlestick widget).
2. Wait for debounced save or quit.
   **Pass:** JSON contains `"last_chart_mode": "candles"`.

3. Relaunch → **Charts**.
   **Pass:** Candlestick mode restored (not line).

### Manual — defaults unchanged

1. Copy config to a temp file; remove `last_time_range` and `last_chart_mode` keys (or use a fresh test config).
2. Launch.
   **Pass:** Charts default to **1M** (`M1`) and **line** mode — same as pre-#180 behavior.

### Manual — invalid values ignored

1. Edit `~/.stockterm.json`: set `"last_time_range": "bogus"` and `"last_chart_mode": "invalid"`.
2. Launch.
   **Pass:** App starts without config parse error; Charts use **M1** + **line** defaults.

### Regression

| Check | Pass criteria |
|-------|----------------|
| §22 `last_tab` / `last_symbol` | Tab and symbol still restore |
| §11 range keys | `2`/`3`/`4` still change range and refetch |
| §11 `c` toggle | Mode toggle still works in-session |
| §46 indicators | SMA/RSI toggles still session-only (not in JSON) |
| §22.7 debounce | Rapid tab changes do not corrupt JSON |

### Sign-off — Issue #180

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-23 | Pass |
| Serde / parse unit tests | maintainer | 2026-05-23 | Pass |
| Manual: 1D persist + relaunch | maintainer | 2026-05-23 | Pass |
| Manual: candlestick persist + relaunch | maintainer | 2026-05-23 | Pass |
| Manual: defaults without keys | maintainer | 2026-05-23 | Pass |
| Manual: invalid strings ignored | maintainer | 2026-05-23 | Pass |
| §22 / §11 regression | maintainer | 2026-05-23 | Pass |

---

## Issue #182 — Portfolio row edit UI (§13 / §15 follow-up)

**Scope:**

- [GitHub Issue #182](https://github.com/FelipeMorandini/stockterm/issues/182) — Edit an existing holding's **shares** and **avg cost** (`purchase_price`) in place on the **Portfolio** tab without remove + re-add; two-step save confirm; persist via **`Config::try_save`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §55.

**Prerequisite:** Yahoo provider (default), at least one holding in **`~/.stockterm.json`** (e.g. add **AAPL** 10 shares @ 150 via existing add dialog).

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted unit tests (after implementation):

   ```bash
   cargo test format_holding_input_value
   cargo test update_portfolio_holding
   cargo test portfolio_row_edit
   ```

   **Pass:** Named tests exist and pass (exact paths may vary — run `cargo test portfolio` / `cargo test holding` if names differ).

3. Keymap defaults wired:

   ```bash
   rg 'PortfolioRowEdit|PortfolioDialogSaveConfirm' src/config/keymap.rs src/app/portfolio.rs
   ```

   **Pass:** **`PortfolioRowEdit`** default **`char:e`** on **`BindingLayer::Portfolio`**; confirm actions on **`PortfolioDialog`**.

### Manual — open edit dialog

**Prep:** Launch StockTerm → **Portfolio** tab → ensure at least one row is visible with known shares and avg cost.

1. Highlight a holding (`j`/`k`).
2. Press **`e`**.
   **Pass:** Centered **"Edit holding"** overlay opens; **Symbol** matches the selected row (not necessarily Stock View's active symbol); **Shares** and **Price** fields prefilled with current values.

3. Press **Esc** (not armed).
   **Pass:** Dialog closes; row values unchanged in the table.

### Manual — two-step save

1. Press **`e`** on the same row.
2. Change **Shares** (e.g. `10` → `12`) and/or **Price** (e.g. `150` → `145`).
3. Tab to **Price** field; press **Enter** once.
   **Pass:** Dialog shows **"Save armed — confirm: Enter or y | cancel: Esc or n"**; values **not** yet persisted.

4. Press **`n`** or **Esc**.
   **Pass:** Armed state clears; dialog stays open with edited buffers; table still shows old values.

5. Press **Enter** on **Price** again to re-arm; press **`y`** (or **Enter**).
   **Pass:** Dialog closes; table row shows new shares/price; **Summary** cost basis and P/L update immediately.

6. Quit with **`q`**; inspect **`~/.stockterm.json`**.
   **Pass:** **`portfolio`** entry for that symbol reflects new **shares** and **purchase_price**.

7. Relaunch.
   **Pass:** Edited values still present.

### Manual — direct override (not weighted average)

**Prep:** Holding **TEST** with 10 shares @ $100 (cost basis $1000).

1. Edit to **10** shares @ **$200** (not adding shares — full replace).
2. Two-step save and confirm.
   **Pass:** Row shows **10 @ $200**; cost basis **$2000** — **not** a blended average from the prior $100 cost.

### Manual — validation and caps

1. Open edit; clear **Shares** to empty; arm + confirm.
   **Pass:** **`inline_error`** (e.g. "Value required"); dialog stays open.

2. Enter shares above cap (see **`MAX_HOLDING_SHARES`** in SPEC §15.5); confirm.
   **Pass:** **`inline_error`** about maximum; no save.

3. Enter valid values; confirm.
   **Pass:** Save succeeds.

### Manual — save failure banner

**Prep:** Make config unwritable temporarily (e.g. `chmod 000 ~/.stockterm.json` on a test copy, or point **`HOME`** at a read-only dir if your test harness supports it).

1. Open edit; change values; two-step confirm.
   **Pass:** Runtime error banner on **Portfolio** tab (same pattern as add/remove save failure); dialog remains open; in-memory table unchanged from pre-commit state.

2. Restore write permissions; retry save.
   **Pass:** Save succeeds.

### Manual — mutual exclusion

| Step | Pass criteria |
|------|----------------|
| Remove armed (`d`) then **`e`** | Remove disarmed; edit dialog opens |
| Edit dialog open then **`a`** | Add does not open until edit closed (or **`a`** ignored while dialog open — per §55.1) |
| Edit dialog open then **`d`** | Remove does not arm while dialog open |
| Edit open; **Tab** / **Shift+Tab** | Cycles Shares/Price; does **not** switch app tabs |
| Leave Portfolio tab while edit open | Dialog cleared; return to Portfolio shows table only |

### Regression

| Check | Pass criteria |
|-------|----------------|
| §13 add (`a`) | Still adds for **`App::symbol`** with immediate save on Price **Enter** |
| §13 remove (`d` → `d`/`y`) | Two-step remove unchanged |
| §1 §23 filter (`/`) | Filter + edit on filtered row uses correct underlying index |
| §15 Tab cycle | Shares ↔ Price in add dialog still works |
| §22 persistence | Other config fields untouched after edit save |

### Sign-off — Issue #182

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-23 | Pass |
| Unit: prefill / update / keymap | maintainer | 2026-05-23 | Pass |
| Manual: open edit prefilled | maintainer | 2026-05-23 | Pass |
| Manual: two-step save | maintainer | 2026-05-23 | Pass |
| Manual: direct override | maintainer | 2026-05-23 | Pass |
| Manual: validation / caps | maintainer | 2026-05-23 | Pass |
| Manual: save failure banner | maintainer | 2026-05-23 | Pass |
| Manual: mutual exclusion | maintainer | 2026-05-23 | Pass |
| §13 / §23 regression | maintainer | 2026-05-23 | Pass |

---

## Issue #183 — Options theme cache invalidation (§53.2 / §21 follow-up)

**Scope:**

- [GitHub Issue #183](https://github.com/FelipeMorandini/stockterm/issues/183) — After committing a theme preset on **Settings**, pre-built **Options** CALLS/PUTS tables must repaint with the new palette without requiring refresh, Greeks toggle, or expiration change.

**Spec:** [`docs/SPEC.md`](SPEC.md) §56.

**Prerequisite:** Yahoo provider (default), symbol with listed options (e.g. **AAPL**), chain loaded on **Options** (`r`).

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted unit tests (after implementation):

   ```bash
   cargo test refresh_options_display_for_theme
   ```

   **Pass:** Tests from SPEC §56.5 exist and pass.

3. Draw-path zero-clone guard (§53.2 regression):

   ```bash
   rg '\.clone\(\)' src/app/options.rs
   ```

   **Pass:** No `.clone()` inside the `draw_options` function body (matches elsewhere in file are OK if outside `draw_options`).

### Manual — stale colors repro (before fix) / fixed behavior (after)

**Prep:**

1. Launch StockTerm; set symbol **AAPL** on Stock View.
2. Open **Options** tab; press **`r`**; wait for CALLS/PUTS tables.
3. Note current theme (default **Dark** unless changed).

**Theme commit + return:**

1. Switch to **Settings** tab; focus row **3** (Theme).
2. Cycle preset with **`]`** / **`[`** until **Light** (or any preset clearly different from step 3).
3. Press **Enter** to commit (saved flash on Settings).
4. Switch back to **Options** tab (same symbol; chain still loaded — no **`r`**).

   **Pass:** CALLS/PUTS table foreground, background, borders, and row highlight match **Light** (or chosen preset) — **not** the old **Dark** colors.

5. Repeat **Settings** → commit back to **Dark** → **Options**.

   **Pass:** Tables return to **Dark** styling immediately.

### Manual — theme save failure does not restyle

**Prep:** Writable `~/.stockterm.json` (normal install).

1. Load options on **Options** (**Dark** tables).
2. Make config unwritable (e.g. `chmod 000 ~/.stockterm.json` on a test copy, or read-only `HOME` test harness).
3. **Settings** → Theme row → select **Light** → **Enter**.

   **Pass:** Settings runtime error banner; theme **not** persisted; return to **Options** — tables still **Dark**.

4. Restore write permissions; commit **Light** successfully.

   **Pass:** Tables update to **Light** on **Options** without **`r`**.

### Manual — draft preview does not require early rebuild

1. **Options** with loaded chain (**Dark** committed).
2. **Settings** row **3**: cycle draft to **Light** with **`]`** but **do not** press **Enter**.
3. Switch to **Options**.

   **Pass:** Tables remain **Dark** (committed theme) — expected; not a failure.

4. **Settings** → **Enter** commit **Light** → **Options**.

   **Pass:** Tables **Light**.

### Regression — §53.2 / §52.2 Options smoke

| Step | Pass criteria |
|------|----------------|
| **`r`** refresh | Chain reloads; tables populate |
| **`g`** Greeks toggle | Columns add/remove; styles use current theme |
| **`h`** / **`l`** expiration | Tables rebuild; colors match current theme |
| **`j`** / **`k`** strike scroll | Highlight moves; no panic |
| §21 other tabs | Stock View / Charts / Portfolio colors still follow theme after commit |

### Sign-off — Issue #183

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-23 | Pass |
| Unit: theme refresh | maintainer | 2026-05-23 | Pass |
| Manual: commit theme → Options colors | maintainer | 2026-05-23 | Pass |
| Manual: save failure no restyle | maintainer | 2026-05-23 | Pass |
| Manual: draft vs commit | maintainer | 2026-05-23 | Pass |
| §53.2 / §52.2 regression | maintainer | 2026-05-23 | Pass |

---

## Issue #181 — GitHub Actions CI (§57 / M7)

**Scope:**

- [GitHub Issue #181](https://github.com/FelipeMorandini/stockterm/issues/181) — Add `.github/workflows/ci.yml` so `cargo clippy -- -D warnings` and `cargo test` run on every PR and on pushes to `main`.

**Spec:** [`docs/SPEC.md`](SPEC.md) §57.

**Prerequisite:** None (first M7 deliverable).

### Automated (local) — required before opening PR

1. Reproduce the CI matrix locally:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --all-features
   cargo clippy --no-default-features --all-targets -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All commands exit **0** except `rustfmt` may fail locally without blocking merge (CI marks fmt advisory per §57).

2. After adding the workflow file, push a branch and confirm GitHub runs both jobs (`default-features`, `no-default-features`).

### Manual — GitHub UI

| Step | Pass criteria |
|------|----------------|
| Open PR from feature branch | **CI** workflow appears under Checks |
| Introduce intentional `clippy` warning on branch (then revert) | Workflow fails; PR shows red check |
| Merge to `main` | `main` branch workflow run is green |
| README **Continuous integration** subsection | Documents commands + advisory `rustfmt` |

### Regression

- No change to TUI runtime behavior — smoke: `cargo run` still launches (optional quick check).

### Sign-off — Issue #181

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Local matrix (§57) | maintainer | 2026-05-23 | Pass |
| PR CI checks green | maintainer | 2026-05-23 | Pass |
| `main` CI green post-merge | maintainer | | Pending merge of [#188](https://github.com/FelipeMorandini/stockterm/pull/188) |
| README CI docs | maintainer | 2026-05-23 | Pass |

---

## Issue #184 — TestBackend error overlay snapshots (§58 / M7)

**Scope:**

- [GitHub Issue #184](https://github.com/FelipeMorandini/stockterm/issues/184) — `insta` snapshot tests for `draw_error_log_overlay` (SPEC §20.15); runs in CI after **#181**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §58.

**Prerequisite:** Issue **#181** merged (CI workflow on `main`).

### Automated (local) — required

1. Full test + lint (same as §57 matrix):

   ```bash
   cargo test --all-features
   cargo clippy --all-targets -- -D warnings
   ```

   **Pass:** Both exit **0**; includes new snapshot tests.

2. Targeted snapshot tests:

   ```bash
   cargo test error_log_overlay
   ```

   **Pass:** Tests from SPEC §58.3 exist and pass.

3. Snapshot hygiene (after intentional UI change):

   ```bash
   cargo insta test
   cargo insta review   # accept updated .snap files when expected
   ```

   **Pass:** Review UI shows diff; only committed snapshots match intended visual change.

4. Scroll invariant micro-check (engineer): each snapshot test asserts `error_log_scroll` unchanged after draw (§58.4).

### Manual — snapshot diff UX (recommended)

1. Temporarily change one overlay footer string (e.g. add a trailing space) on a throwaway branch.
2. Run `cargo insta test`.

   **Pass:** Test fails with readable diff pointing at `src/app/snapshots/*.snap`.
3. Revert string; `cargo insta test` passes.

### Manual — overlay behavior unchanged (§20.15 smoke)

**Prep:** Writable config; network optional.

1. `cargo run` → trigger a quote error (bad symbol or offline) → **`Ctrl+E`** open error log.
2. **`j`** / **`k`** scroll; resize terminal taller/shorter; scroll again.

   **Pass:** Scroll matches visible rows; no panic; overlay closes with **Esc**.

### Regression

| Step | Pass criteria |
|------|----------------|
| Issues **#120–#123** behaviors | Visible-row bound + scroll read-only draw unchanged in manual use |
| `cargo test` (full lib) | No new failures outside snapshot module |

### Sign-off — Issue #184

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-23 | Pass |
| Snapshot tests §58.3 | maintainer | 2026-05-23 | Pass |
| `insta` diff workflow | maintainer | 2026-05-23 | Pass |
| Manual §20.15 overlay smoke | maintainer | 2026-05-23 | Pass |
| CI green on PR (post-#181) | maintainer | 2026-05-23 | Pass |

---

## Issue #189 — Expand TestBackend snapshots (§59 / M7 Phase 2)

**Scope:**

- [GitHub Issue #189](https://github.com/FelipeMorandini/stockterm/issues/189) — Extend **§58** `insta` snapshots to Stock View narrow status bar and portfolio add dialog (optional: Options tab).

**Spec:** [`docs/SPEC.md`](SPEC.md) §59.

**Prerequisite:** Issue **#184** merged (**§58** error overlay snapshots + CI `cargo test` gate).

### Automated (local) — required

1. Full test + lint matrix (same as §57):

   ```bash
   cargo test --all-features
   cargo clippy --all-targets -- -D warnings
   ```

   **Pass:** Both exit **0**; includes new snapshot tests and refactored §58 overlay tests.

2. Targeted snapshot tests:

   ```bash
   cargo test status_bar_
   cargo test portfolio_add_dialog_
   cargo test error_log_overlay   # §58 regression
   ```

   **Pass:** All §59.2–§59.3 scenarios exist and pass; §58 scenarios still pass after harness extraction.

3. Snapshot hygiene:

   ```bash
   cargo insta test
   cargo insta review   # when intentional UI change
   ```

   **Pass:** New `.snap` files under `src/app/snapshots/` committed; no unexpected drift.

4. Determinism check (engineer): snapshot tests use fixed symbol (`AAPL`), fixed theme, no `push_error_log` / no `App::run`.

### Manual — status bar regression (§37.1 smoke)

**Prep:** Terminal **80 columns** wide; Stock View tab.

1. `cargo run` → Stock View with no active error and no quote refresh in progress.

   **Pass:** Status area shows **two** hint lines; second line mentions **Shift** / `w/x/j/k` edge case (§37.1.3).
2. Widen terminal to **≥100 columns** (or use wide layout).

   **Pass:** Status collapses to **one** line with `Shift+1st letter` wide copy.
3. Trigger quote refresh (**`r`** or wait for poll).

   **Pass:** Status shows single **"Refreshing quotes…"** line (two-line hint suppressed).

### Manual — portfolio dialog regression (§18.13 / §15 smoke)

**Prep:** At least one symbol selected; Portfolio tab.

1. Press **`a`** → add dialog opens.

   **Pass:** Modal title **"Add to portfolio"**; **Symbol** shows active ticker; **Shares** field focused (accent).
2. Type invalid shares (e.g. `abc`) → tab to **Price** → **`Enter`**.

   **Pass:** Inline error on dialog; dialog stays open (not silent failure).
3. **`Esc`** closes dialog without saving.

   **Pass:** Holdings table unchanged.

### Manual — snapshot diff UX (recommended)

1. On throwaway branch, change one status hint string or dialog title by one character.
2. Run `cargo insta test`.

   **Pass:** Relevant snapshot test fails with readable diff in `src/app/snapshots/*.snap`.
3. Revert change; `cargo insta test` passes.

### Regression

| Step | Pass criteria |
|------|----------------|
| §58 error overlay snapshots | `cargo test error_log_overlay` still green |
| §37.1 unit tests | `cargo test status_bar` / `stock_view_status_lines` unchanged behavior |
| Portfolio commit paths | `try_commit_portfolio_dialog` tests still pass |

### Sign-off — Issue #189

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-24 | Pass |
| §59.2 status snapshots | maintainer | 2026-05-24 | Pass |
| §59.3 portfolio snapshots | maintainer | 2026-05-24 | Pass |
| §58 regression (overlay) | maintainer | 2026-05-24 | Pass |
| Manual §37.1 status smoke | maintainer | 2026-05-24 | Pass |
| Manual portfolio dialog smoke | maintainer | 2026-05-24 | Pass |
| CI green on PR | maintainer | 2026-05-24 | Pass |

---

## Issue #193 — Blocking rustfmt CI gate (§60)

**Scope:**

- [GitHub Issue #193](https://github.com/FelipeMorandini/stockterm/issues/193) — Remove advisory `continue-on-error` on `cargo fmt --check`; add blocking fmt to both CI jobs; format entire tree.

**Spec:** [`docs/SPEC.md`](SPEC.md) §60 (updates **§57.1**).

**Prerequisite:** Issue **#181** merged (`.github/workflows/ci.yml` exists).

### Automated (local) — required before opening PR

1. Format check (must pass on PR branch):

   ```bash
   cargo fmt --all -- --check
   ```

   **Pass:** Exit **0** on the implementing branch (after one-time `cargo fmt --all`).

2. Full CI matrix:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --all-features
   cargo clippy --no-default-features --all-targets -- -D warnings
   cargo test --no-default-features
   ```

   **Pass:** All exit **0**.

### Manual — GitHub CI verification

| Step | Pass criteria |
|------|----------------|
| Push PR branch | Both jobs (`default-features`, `no-default-features`) run **`rustfmt`** step |
| PR with formatted code | Both jobs green |
| Introduce intentional fmt violation (throwaway commit) | **`rustfmt`** step fails; workflow red |
| Revert violation | Workflow green again |
| README **Continuous integration** | Documents **blocking** `cargo fmt --all -- --check` before clippy |

### Regression

- No TUI behavior change — optional smoke: `cargo run` launches.
- Snapshot tests (§58 / §59) still pass after format-only diff.

### Sign-off — Issue #193

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Local `cargo fmt --check` | maintainer | 2026-05-24 | Pass |
| CI fmt blocking (both jobs) | maintainer | 2026-05-24 | Pass |
| CI red on fmt violation | maintainer | 2026-05-24 | Pass |
| README CI docs updated | maintainer | 2026-05-24 | Pass |
| Full test + clippy matrix | maintainer | 2026-05-24 | Pass |

---

## Issue #195 — Options lazy `ThemeStamp` compare (§61 / §56.7 follow-up)

**Scope:**

- [GitHub Issue #195](https://github.com/FelipeMorandini/stockterm/issues/195) — Skip redundant `OptionsDisplayCache` style rebuilds when the resolved palette fingerprint is unchanged; lazy sync at `draw_options` entry as a safety net.

**Spec:** [`docs/SPEC.md`](SPEC.md) §61.

**Prerequisite:** **§56** shipped ([#183](https://github.com/FelipeMorandini/stockterm/issues/183)); Yahoo provider; symbol with listed options (e.g. **AAPL**).

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted unit tests:

   ```bash
   cargo test theme_stamp
   cargo test sync_options_display_theme
   cargo test refresh_options_display_for_theme
   ```

   **Pass:** §61.5 tests exist and pass, including §56.5 regressions.

3. Draw-path zero-clone guard (§53.2 regression):

   ```bash
   rg '\.clone\(\)' src/app/options.rs
   ```

   **Pass:** No `.clone()` inside the `draw_options` function body.

### Manual — stamp optimization does not regress #183

**Prep:** Load options chain on **Options** (**AAPL**, **`r`**, **Dark** committed).

1. **Settings** → Theme row **3** → commit **Light** → **Options**.

   **Pass:** CALLS/PUTS tables immediately **Light** (same as Issue **#183**).

2. Stay on **Options** (no tab change); press **`g`** (Greeks toggle).

   **Pass:** Columns toggle; colors remain **Light**.

3. **Settings** → commit back to **Dark** → **Options**.

   **Pass:** Tables **Dark** without **`r`**.

### Manual — repeated commit no visible glitch

1. **Options** with loaded chain (**Dark**).
2. **Settings** → Theme row **3** → **Enter** commit **Dark** again (same preset).

   **Pass:** No flicker/panic; tables unchanged; app responsive.

### Regression — §56 / §53.2 / §52.2

| Step | Pass criteria |
|------|----------------|
| Issue **#183** checklist | Theme save failure does not restyle; draft preview stays on committed palette |
| **`h`** / **`l`** expiration | Tables rebuild; colors match current theme |
| **`j`** / **`k`** strike scroll | Highlight moves; no panic |
| §21 other tabs | Stock View / Portfolio / Alerts still follow theme after commit |

### Sign-off — Issue #195

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-24 | Pass |
| Unit: `ThemeStamp` + sync skip/rebuild | maintainer | 2026-05-24 | Pass |
| Manual: #183 regression (commit theme → Options) | maintainer | 2026-05-24 | Pass |
| Manual: same-preset re-commit | maintainer | 2026-05-24 | Pass |
| §53.2 zero-clone guard | maintainer | 2026-05-24 | Pass |

---

## Issue #196 — Theme audit: watchlist / portfolio / alerts (§62 / §56.7 follow-up)

**Scope:**

- [GitHub Issue #196](https://github.com/FelipeMorandini/stockterm/issues/196) — Document live-draw vs baked-draw theme policy; centralize `on_theme_preset_committed`; prove watchlist/portfolio/alerts track committed palette without restart.

**Spec:** [`docs/SPEC.md`](SPEC.md) §62.

**Prerequisite:** Non-empty watchlist + portfolio holding + at least one alert (armed or triggered); quotes loaded.

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted unit tests (after implementation):

   ```bash
   cargo test watchlist_table_bg_tracks_committed_theme
   cargo test portfolio_pl_color_tracks_committed_theme
   cargo test alerts_status_color_tracks_committed_theme
   ```

   **Pass:** §62.4 tests exist and pass.

### Manual — watchlist restyles on commit

**Prep:** **Stock View** with watchlist entries (**AAPL**, **MSFT**); quotes visible; theme **Dark**.

1. **Settings** → row **3** → commit **Light** → **Stock View**.

   **Pass:** Watchlist table borders, row text, selection highlight, and P/L column colors match **Light** — no restart.

2. Commit back to **Dark**.

   **Pass:** Watchlist returns to **Dark** immediately.

### Manual — portfolio restyles on commit

**Prep:** **Portfolio** tab with at least one holding and live **Current** price; theme **Dark**.

1. Note summary **P/L** color (green/red) and holdings row styling.
2. **Settings** → commit **Light** → **Portfolio**.

   **Pass:** Summary line, holdings table, borders, and P/L colors match **Light**.

3. Open add/edit dialog (**`a`** or **`e`**) — cancel with **Esc**.

   **Pass:** Overlay uses **Light** palette.

### Manual — alerts restyles on commit

**Prep:** **Alerts** tab with at least one alert; **Armed** or **TRIGGERED** status visible; theme **Dark**.

1. **Settings** → commit **Light** → **Alerts**.

   **Pass:** Table borders, status column colors, and row text match **Light**.

2. Open add dialog (**`a`**) — **Esc** to close.

   **Pass:** Dialog overlay uses **Light** palette.

### Manual — Options still correct (§61 / #183 cross-check)

1. With chain loaded on **Options**, commit theme on **Settings**.

   **Pass:** Options tables still update (covered by Issue **#195** / **#183**); watchlist/portfolio/alerts also updated in same session.

### Regression

| Step | Pass criteria |
|------|----------------|
| §21 Options theme commit | CALLS/PUTS restyle (Issue **#183** / **#195**) |
| §59 snapshots | `cargo test snapshot` / insta suite green if touched |
| Theme save failure | Settings error; tabs keep prior committed colors |

### Sign-off — Issue #196

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | maintainer | 2026-05-24 | Pass |
| Unit: watchlist / portfolio / alerts theme tests | maintainer | 2026-05-24 | Pass |
| Manual: watchlist theme commit | maintainer | 2026-05-24 | Pass |
| Manual: portfolio theme commit | maintainer | 2026-05-24 | Pass |
| Manual: alerts theme commit | maintainer | 2026-05-24 | Pass |
| Cross-check Options (#183/#195) | maintainer | 2026-05-24 | Pass |

---

## Issue #190 — Charts: candlestick visual density polish (§63)

**Scope:**

- [GitHub Issue #190](https://github.com/FelipeMorandini/stockterm/issues/190) — Integer slot layout for `CandlestickChart`: wider bodies at common terminal widths, wick/body separation, no provider or viewport contract changes.

**Spec:** [`docs/SPEC.md`](SPEC.md) §63.

**Prerequisite:** Yahoo provider (or any provider with historical OHLC); symbol **AAPL** (or liquid equity with 20+ daily bars). Charts tab functional (**§11** shipped).

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted unit tests (after implementation):

   ```bash
   cargo test layout_candles
   ```

   **Pass:** §63.5 layout tests exist and pass; existing `visible_slice` / viewport tests unchanged.

3. If §63.5 `insta` snapshots were added:

   ```bash
   cargo insta test
   ```

   **Pass:** `candlestick_density_80x24` and `candlestick_density_120x40` (or names from §63.5) green without unexpected diff.

### Manual — terminal size fixtures

**Prep:** Build release binary. Resize terminal **exactly** to each size below (or use tmux window dimensions). Load **AAPL** historical data on **Charts**; press **`c`** for candlestick mode; **`0`** full viewport before each size block.

#### 80 columns × 24 rows

1. **Y1** (`4`), wait for chart.
   **Pass:** Candle bodies visibly occupy most of each bar column; minimal empty gaps between adjacent bars; wicks centered in column; no heavy column “bleed” (one bar’s color dominating neighbor cells).
2. **`+`** zoom in twice, then **`h`** / **`l`** pan.
   **Pass:** Zoom/pan updates candles; no panic; status/chrome intact.
3. **D1** (`1`), wait for chart.
   **Pass:** Same density improvement vs pre-#190 baseline (maintainer notes Pass if bodies are ≥1 cell wide per slot and alignment is column-stable).

#### 120 columns × 40 rows

1. **Y1** (`4`), full viewport.
   **Pass:** With ~40+ visible bars, many slots show **2–3** cell-wide bodies where slot width allows (per §63.1); chart reads denser than line mode at same zoom (subjective but obvious side-by-side with **`c`** toggle).
2. Toggle **`c`** → line → **`c`** → candles.
   **Pass:** Mode toggle works; line uses Braille; returning to candles preserves viewport range.

### Manual — theme and indicators (regression)

1. **Dark** theme, candles, enable **`s`** (SMA).
   **Pass:** Muted hint *“Indicators: press `c` for line chart”* — no SMA drawn through candles (**§46.3**).
2. **Settings** → commit **Light** → **Charts** (candles).
   **Pass:** Up/down colors track **Light** preset.

### Manual — persistence and time ranges (regression)

| Step | Pass criteria |
|------|----------------|
| Change range **`2`** (W1) / **`3`** (M1) | Chart refetches; candles render; viewport resets per §11 |
| **`c`** toggle + relaunch app | **`last_chart_mode`** restores candlestick (**§54** / #180) |
| Partial-page notice (Polygon tier) | Unchanged if applicable — no layout crash |

### Manual — side-by-side density check (maintainer)

Record **before** screenshot (pre-#190 build) and **after** at **80×24** Y1 and **120×40** Y1.

**Pass:** Post-change chart shows measurably tighter column usage (wider bodies, cleaner wick alignment) without unreadable overlap.

### Sign-off — Issue #190

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | Maintainer | 2026-05-25 | Pass |
| Unit: `layout_candles` | Maintainer | 2026-05-25 | Pass |
| `insta` snapshots (if added) | Maintainer | 2026-05-25 | Pass |
| Manual: 80×24 Y1 + D1 + M1 | Maintainer | 2026-05-25 | Pass |
| Manual: 120×40 Y1 | Maintainer | 2026-05-25 | Pass |
| Manual: zoom/pan/`0`/`1`–`4`/`c` | Maintainer | 2026-05-25 | Pass |
| Manual: theme + indicator hint | Maintainer | 2026-05-25 | Pass |
| Manual: §54 chart_mode persist | Maintainer | 2026-05-25 | Pass |
| Density vs baseline (screenshot) | Maintainer | 2026-05-25 | Pass |

## Issue #199 — Charts: precompute candle layout off render path (§64)

**Scope:**

- [GitHub Issue #199](https://github.com/FelipeMorandini/stockterm/issues/199) — Move `layout_candles(...)` work out of the 60fps `Widget::render` loop. Cache the precomputed `Vec<CandleBarLayout>` on `App`; invalidate on viewport / bar-count / `time_range` / area changes. **Visual output must remain byte-identical to §63.**

**Spec:** [`docs/SPEC.md`](SPEC.md) §64.

**Prerequisite:** §63 shipped. Yahoo provider (or any provider with historical OHLC); symbol **AAPL** (or liquid equity with 20+ daily bars). Charts tab functional.

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Layout-cache unit tests (§64.6):

   ```bash
   cargo test candle_layout_cache
   cargo test charts_price_area_pure
   cargo test historical_data_stamp_bumps_on_apply
   ```

   **Pass:** All §64.6 tests green; the build-counter test proves `ensure_candle_layout` allocates **once** across N identical-key calls.

3. **`insta` snapshot regression — mandatory, no expected diff:**

   ```bash
   cargo insta test
   ```

   **Pass:** `candlestick_density_80x24` and `candlestick_density_120x40` accept clean (no diff vs §63 baseline). If a diff appears, **fail** — the §64 refactor must preserve pixels.

4. Static check — render loop has no layout allocation:

   ```bash
   rg -n "layout_candles\(|Vec::new|vec!\[" src/app/charts.rs | rg -i "Widget::render|impl Widget"
   ```

   **Pass:** No matches inside the `impl Widget for CandlestickChart` body (manually verify the function body via `Read` on `src/app/charts.rs`).

### Manual — rebuild trigger matrix (80×24)

**Prep:** Build release. Resize terminal to **exactly 80×24**. Load **AAPL** on **Charts**, press **`c`** (candlestick).

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Wait for full Y1 series to load | Candles render; visual matches §63 baseline (no shift / no gap change). |
| 2 | Press **`+`** (zoom in) twice | Candles redraw within 1 frame; no flicker; no panic; layout adjusts to new viewport. |
| 3 | Press **`h`** then **`l`** (pan) | Smooth pan; bodies/wicks track viewport; no `debug_assert` panic in dev build. |
| 4 | Press **`0`** (reset) | Full viewport restores; candles redraw correctly. |
| 5 | Press **`1`** → **`2`** → **`3`** → **`4`** (range) | Each range fetches new data; candles redraw correctly per range; no stale layout artifacts. |
| 6 | Press **`c`** → line → **`c`** → candles | Mode toggle works; returning to candles uses the **current viewport**, not a stale cache. |
| 7 | Type a new ticker (e.g. `M`, `S`, `F`, `T`, **Enter**) | Symbol change triggers fetch; old candles cleared; new bars render with fresh layout. |

### Manual — resize / pane split (120×40 and back)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Resize terminal **80×24 → 120×40** while Charts is open | Candles redraw at new width within 1 frame; body widths reflow per §63.1; **no panic**. |
| 2 | Resize back **120×40 → 80×24** | Layout recomputes; candles return to dense mode if applicable. |
| 3 | Toggle indicators **`r`** (RSI) — sub-pane appears | Candles redraw in smaller price-pane area; no overlap with RSI sub-pane. |
| 4 | Toggle **`m`** (MACD) added | Same — price pane shrinks further; candles still fit. |
| 5 | Disable **`r`** + **`m`** | Price pane returns to full height; candles redraw correctly. |

### Manual — performance smoke (subjective)

1. Open Charts Y1, hold **`l`** (pan right) for 5 seconds.
   **Pass:** UI remains responsive; no perceptible slowdown vs §63 baseline. If timing-suspicious, set `RUST_LOG=stockterm=debug` and confirm no per-frame layout log spam (none expected — `layout_candles` only logs nothing today).

### Manual — visual parity (maintainer)

1. Side-by-side: §63-baseline screenshot at **80×24** Y1 vs §64 build at same size/symbol.
   **Pass:** Pixels match. Any visible difference is a §64 regression.

### Sign-off — Issue #199

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | Maintainer | 2026-05-25 | Pass |
| Unit: `candle_layout_cache_*` | Maintainer | 2026-05-25 | Pass |
| `insta` snapshots — no diff | Maintainer | 2026-05-25 | Pass |
| Static check: no `Vec` in `render` | Maintainer | 2026-05-25 | Pass |
| Manual: rebuild triggers (80×24) | Maintainer | 2026-05-25 | Pass |
| Manual: resize 80×24 ↔ 120×40 | Maintainer | 2026-05-25 | Pass |
| Manual: indicator sub-pane resize | Maintainer | 2026-05-25 | Pass |
| Manual: performance smoke (5s pan) | Maintainer | 2026-05-25 | Pass |
| Visual parity vs §63 baseline | Maintainer | 2026-05-25 | Pass |

## Issue #200 — `HistoricalData::t`: normalize bar timestamp unit at ingest (§65)

**Scope:**

- [GitHub Issue #200](https://github.com/FelipeMorandini/stockterm/issues/200) — Document `HistoricalData::t` as Unix milliseconds; route every provider write through `normalize_bar_timestamp_to_ms`; remove the `TIMESTAMP_MS_EPOCH_THRESHOLD` / `bar_timestamps_are_millis` runtime heuristic from `src/app/charts.rs`.

**Spec:** [`docs/SPEC.md`](SPEC.md) §65.

**Prerequisite:** §63 shipped (layout heuristic in tree to be removed). Both Yahoo and Polygon providers reachable for regression smoke.

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Targeted normalize tests:

   ```bash
   cargo test normalize_bar_timestamp_to_ms
   cargo test yahoo_chart_to_historical_emits_ms
   cargo test polygon_merge_normalizes_t
   ```

   **Pass:** All §65.7 unit tests present and green.

3. Static check — heuristic removed:

   ```bash
   rg -n "TIMESTAMP_MS_EPOCH_THRESHOLD|bar_timestamps_are_millis" src/
   ```

   **Pass:** **Zero matches** anywhere in `src/`. (If any match remains, fail.)

4. `insta` snapshot regression:

   ```bash
   cargo insta test
   ```

   **Pass:** `candlestick_density_80x24` and `candlestick_density_120x40` accept clean (no diff).

### Manual — Yahoo regression (smoke)

**Prep:** `provider: "yahoo"` in `~/.stockterm.json` (or default). Symbol **AAPL**.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Charts tab, range **`1`** (D1) | Intraday candles render; x-axis labels show **HH:MMZ** (intraday format from `format_time_axis`); no `?` placeholder. |
| 2 | Range **`3`** (M1) | Daily bars; x-axis shows **MM/DD**. **Index** layout per §63 (equal stride). |
| 3 | Range **`4`** (Y1) | Weekly+ bars; x-axis shows **MM/DD**. **Time** layout per §63 (weekend / holiday gaps visible). |
| 4 | Switch symbol → **BTC-USD** | Crypto Y1 loads; candles render; no `?` axis labels; price formatting per §43. |

### Manual — Polygon regression (smoke)

**Prep:** `provider: "polygon"` with valid `STOCKTERM_API_KEY` (or `api_key`). Symbol **AAPL**.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Range **`3`** (M1) | Daily aggregates render; x-axis MM/DD; partial-page notice per §52.1 if applicable. |
| 2 | Range **`4`** (Y1) | Same — weekly+ where Polygon supports; no `?` labels. |
| 3 | Trigger pagination (free-tier truncation) | `merge_historical_pages` runs; **no panic**; bars sorted; candle layout correct. |

### Manual — heuristic-trap regression

**Prep:** Synthetic test only — confirm via unit run:

```bash
cargo test polygon_merge_normalizes_t
```

The test injects a synthetic Polygon page with `t = 1_700_000_000` (seconds-shaped); after merge, `results[0].t == 1_700_000_000_000` (ms).

**Pass:** Single assertion green. Documents that **even if a future provider regresses to seconds**, the ingest guard upgrades the value silently and Charts keeps working.

### Manual — §63 invariants preserved

| Step | Pass criteria |
|------|---------------|
| **D1** intraday — bodies do not touch | Per `layout_candles_d1_bodies_do_not_touch` (existing test still green). |
| **Y1** weekly+ — time-mapped centers, irregular gaps spread | Per `layout_candles_time_irregular_gap_spread` (existing test still green). |
| **M1** daily — index stride | Per `m1_daily_bars_use_index_not_time_layout` (existing test still green). |

### Manual — §40 timestamp safety preserved

| Step | Pass criteria |
|------|---------------|
| Force corrupt `historical_data.results[0].t = u64::MAX / 2` in a debug fixture (or via unit harness) | `format_time_axis` returns `?` for invalid; **no panic**; chart renders with whatever else is available. |

### Sign-off — Issue #200

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | Maintainer | 2026-05-25 | Pass |
| Unit: `normalize_bar_timestamp_to_ms_*` | Maintainer | 2026-05-25 | Pass |
| Static: heuristic removed (`rg`) | Maintainer | 2026-05-25 | Pass |
| `insta` snapshots — no diff | Maintainer | 2026-05-25 | Pass |
| Manual: Yahoo D1 / M1 / Y1 | Maintainer | 2026-05-25 | Pass |
| Manual: Polygon M1 / Y1 / pagination | Maintainer | 2026-05-25 | Pass |
| Manual: heuristic-trap unit | Maintainer | 2026-05-25 | Pass |
| Manual: §63 invariants green | Maintainer | 2026-05-25 | Pass |
| Manual: §40 timestamp safety | Maintainer | 2026-05-25 | Pass |

## Issue #192 — Config::save: stop silently dropping I/O errors (§66)

**Scope:**

- [GitHub Issue #192](https://github.com/FelipeMorandini/stockterm/issues/192) — Retire silent `Config::save` error discard; ensure no `let _ = try_save_config_with_session()` on user-facing paths; log final persist failures on quit; surface Backtest strategy-toggle save failures in the status bar.

**Spec:** [`docs/SPEC.md`](SPEC.md) §66.

**Prerequisite:** §20 error UX + §22 persistence baseline shipped. Writable `~/.stockterm.json` for happy-path smoke; optional second terminal or log tail for quit-path verification.

### Automated (local) — required

1. Full test + lint:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0.

2. Config persistence unit tests (§66.7):

   ```bash
   cargo test config_save
   cargo test backtest_cycle_strategy_surfaces
   ```

   **Pass:** New §66 tests green.

3. Static audit — no silent drops (§66.5):

   ```bash
   rg -n 'Config::save\(' src/
   rg -n 'let _ = .*try_save' src/
   ```

   **Pass:** No matches under `src/` (definition of `save` in `src/config/config.rs` is allowed; zero call sites).

### Manual — interactive `[cfg]` surface (Backtest)

**Prep:** `cargo build --release`. Backup `~/.stockterm.json`. Note the active log file path from your `tracing` subscriber (project uses rotating file per workspace rules).

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Launch StockTerm; open **Backtest** tab | Tab loads; no startup `[cfg]` error. |
| 2 | Press the key that cycles strategy (per keymap — default chord documented in README / §47) | Strategy label toggles SMA ↔ RSI; no error. |
| 3 | **Simulated failure:** make `~/.stockterm.json` read-only (`chmod 444 ~/.stockterm.json` on Unix) while app is running | — |
| 4 | Cycle strategy again | Status bar shows **`[cfg]`** / **Failed to save backtest settings:** (or equivalent §66 message); in-memory toggle may revert or stay per implementation — **disk must not silently diverge without a signal**. |
| 5 | Restore permissions (`chmod 644 ~/.stockterm.json`) | — |
| 6 | Cycle strategy; quit with **`q`** | Save succeeds; relaunch — last strategy persisted. |

### Manual — debounced session save regression (§22.7 / #129)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | With writable config, switch tabs 3× quickly | No panic; after ~400 ms idle, `last_tab` persists (relaunch restores tab). |
| 2 | With read-only config, switch tabs once, wait ≥500 ms | Status shows **Failed to save session:** (existing §22.7 behavior); §66 must not regress this path. |

### Manual — quit-path logging

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | `chmod 444 ~/.stockterm.json`; launch app; change tab or symbol; quit with **`q`** | App exits cleanly (terminal restored). |
| 2 | Tail the StockTerm log file | Contains **`final config persist failed on shutdown`** (or §66 equivalent) with I/O error detail — **no** `println!` / stderr spam in the TUI. |
| 3 | Restore config file permissions | — |

### Manual — portfolio / alerts / watchlist regression (§22)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Writable config: add watchlist row **`w`**, remove **`x`** | Persists across relaunch; failures would show `[cfg]` (spot-check one happy path). |
| 2 | Alerts: add a test alert; trigger save path | No regression vs §18.14 banner behavior. |

### Sign-off — Issue #192

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | Maintainer | 2026-05-25 | Pass |
| Unit: config + backtest save tests | Maintainer | 2026-05-25 | Pass |
| Static audit (`rg`) | Maintainer | 2026-05-25 | Pass |
| Manual: Backtest `[cfg]` on RO config | Maintainer | 2026-05-25 | Pass |
| Manual: session debounce + RO session error | Maintainer | 2026-05-25 | Pass |
| Manual: quit-path log line | Maintainer | 2026-05-25 | Pass |
| Manual: watchlist / alerts smoke | Maintainer | 2026-05-25 | Pass |

## Issue #79 — Unicode / full case-folding for ticker normalization (§67)

**Scope:**

- [GitHub Issue #79](https://github.com/FelipeMorandini/stockterm/issues/79) — Unicode-aware **`normalize_symbol`**, **`symbols_equivalent`**, and migration of ticker **`eq_ignore_ascii_case`** call sites per [`docs/SPEC.md`](SPEC.md) §67.

**Spec:** [`docs/SPEC.md`](SPEC.md) §67.

**Status:** **Shipped** (2026-05-25) — manual sign-off complete.

**Prerequisite:** §11.12.4 / Issue **#74** shipped (case-only watchlist add skip). Example Unicode test symbols agreed in the issue before manual QA.

### Automated (local) — required when implementing

1. Full test + lint:

   ```bash
   cargo test symbol
   cargo test normalize_symbol
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; new §67 unit tests green.

2. Regression — ASCII paths unchanged:

   ```bash
   cargo test normalize_symbol_trims
   cargo test classify_symbol
   ```

   **Pass:** §43 crypto / equity classification tests still pass.

### Manual — watchlist / chart case equivalence (§11.12.4 extension)

**Prep:** `cargo build --release`. Use maintainer-provided Unicode ticker examples (provider must return quotes for them). Backup `~/.stockterm.json`.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Enter a Unicode ticker that case-folds to an existing watchlist key (per §67 test matrix) | Normalizes to canonical stored form; no duplicate row for equivalent symbols. |
| 2 | Press **`w`** (add to watchlist) when only casing/script variant differs from active symbol | Chart series **does not** clear (same effective ticker as §11.12.4). |
| 3 | Change to a **different** Unicode ticker | Chart clears and reloads (§11.11.1 regression). |
| 4 | Relaunch app | Watchlist persists canonical normalized symbols. |

### Manual — quotes / alerts / portfolio lookup

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Hold Unicode symbol in watchlist; wait for quote refresh | Stock View + watchlist row show price (or documented provider error — not silent empty due to key mismatch). |
| 2 | Add portfolio row + alert for same Unicode symbol | **`get_current_price`** / alert evaluation find cached quote (§41.1 alignment). |

### Manual — filter (if §67.4.4 ships in same PR)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Stock View **`/`** filter; type Unicode substring from ticker | Filter narrows rows; **Esc** clears. |

### Sign-off — Issue #79

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test` + clippy | Maintainer | 2026-05-25 | Pass |
| Unit: Unicode fold / NFC tests | Maintainer | 2026-05-25 | Pass |
| Manual: watchlist dedup + chart skip | Maintainer | 2026-05-25 | Pass |
| Manual: quote cache key alignment | Maintainer | 2026-05-25 | Pass |
| Manual: filter (if in scope) | Maintainer | 2026-05-25 | Pass |

---

## Issue #191 — Optional CancellationToken for superseded quote batches (§68)

**Scope:**

- [GitHub Issue #191](https://github.com/FelipeMorandini/stockterm/issues/191) — Cooperative cancellation for **`run_stock_quote_batch`** when overlapping quote batches are allowed; stale **`FetchDone::Stock`** must not mutate **`watchlist_quotes`**.

**Spec:** [`docs/SPEC.md`](SPEC.md) §68 (architect plan **2026-05-30**).

**Status:** **Implemented** (2026-05-30) — phases 0–3 shipped in-tree. Run manual sign-off below (single-flight baseline always; overlap steps only with **`allow_overlapping_quote_batches": true`** in `~/.stockterm.json`).

**Prerequisite:** §16 shipped ([#17](https://github.com/FelipeMorandini/stockterm/issues/17), [#46](https://github.com/FelipeMorandini/stockterm/issues/46), [#77](https://github.com/FelipeMorandini/stockterm/issues/77)). For overlap tests (phase 3), feature flag or build that enables **`allow_overlapping_quote_batches`** (per §68.4.5) must be documented in the PR README.

### Doc-only / phase 0 sign-off (no code change)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Read §16.1 item 2 + §68.3 in SPEC | Confirms generation ignore is the supported supersede model under single-flight. |
| 2 | `cargo test` + `cargo clippy -- -D warnings` on `main` | Green (no §68 code required). |
| 3 | Optional: `STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release` | §16.1 smoke: UI responsive during inflight refresh (validates baseline before any overlap work). |

### Automated (local) — required when implementing (phase 1+)

1. Full test + lint:

   ```bash
   cargo test stock_fetch
   cargo test quote_batch
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; §68 supersede/cancel tests green.

2. §16 regression (always):

   ```bash
   cargo test
   ```

   **Pass:** No regressions in inflight recovery / generation ignore tests.

3. **Phase 1 only** — stale generation unit test (no overlap):

   ```bash
   cargo test apply_stock_fetch_done
   ```

   **Pass:** Test proves **`apply_stock_fetch_done`** with **`generation < stock_fetch_generation`** does not insert into **`watchlist_quotes`** (see §68.6).

### Manual — single-flight baseline (always run before merge)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | `STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release` | During delay, tab switch / **`j`/`k`** / typing remain responsive (§16.1 smoke). |
| 2 | Rapid **`Enter`** on symbol change 3× during inflight refresh | Status **Refreshing quotes…**; final quotes match last symbol; no panic. |
| 3 | Coalesced refresh: start refresh, trigger another poll while inflight (e.g. add symbol + immediate poll) | Second batch runs **after** first completes (**`stock_refresh_pending`**); no overlapping HTTP without mode **B**. |

### Manual — overlap + cancel (only when §68 mode B enabled)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Start large watchlist refresh (many symbols or debug delay) | Batch in flight. |
| 2 | Trigger product-defined “priority” refresh (per PR README — e.g. immediate active-symbol-only batch) | Prior HTTP work cancelled or ignored; UI shows quotes for **latest** user intent only. |
| 3 | Tail log file (`tracing`, not stderr) | **`quote batch cancelled`** at **debug** level may appear; **no** `println!` in terminal. |
| 4 | Confirm superseded symbol’s stale price does not flash on screen | **`watchlist_quotes`** reflect newest generation only. |

### Sign-off — Issue #191

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Doc-only / phase 0 (§68.3 read + baseline clippy/test) | Maintainer | 2026-05-30 | Pass |
| `cargo test` + clippy (implementation PR) | Maintainer | 2026-05-30 | Pass |
| Unit/integration: superseded batch (phase 1+) | Maintainer | 2026-05-30 | Pass |
| Manual: §16 delay smoke | Maintainer | 2026-05-30 | Pass |
| Manual: coalesced pending (single-flight) | Maintainer | 2026-05-30 | Pass |
| Manual: overlap cancel (mode B only) | Maintainer | 2026-05-30 | N/A (default single-flight) |

---

## Issue #194 — Saved named filters + optional regex mode (§69)

**Scope:**

- [GitHub Issue #194](https://github.com/FelipeMorandini/stockterm/issues/194) — Extend §23 table filtering on **Portfolio** holdings and **Stock View** watchlist with (1) opt-in **regex** mode (`r` toggle in filter input mode, invalid pattern → inline error, no panic, all rows visible until valid) and (2) **saved named filters** in `~/.stockterm.json` (save / recall / delete from filter input mode; survive restart). Substring mode remains the default.

**Spec:** [`docs/SPEC.md`](SPEC.md) §69.

**Prerequisite:** §23 / Issue #16 behavior unchanged when regex mode is off and no saved filter is applied.

### Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   cargo test
   ```

   **Pass:** All exit 0; unit tests cover regex match, invalid-regex → all rows, and `SavedFilter` / config load per §69.7.

### Manual — Regex mode (Portfolio)

**Prep:** `~/.stockterm.json` **`portfolio`** has **AAPL**, **MSFT**, **GOOGL**. Restart **`cargo run --release`**.

1. **Portfolio** → **`/`** → type **`^MS`** → press **`r`** (regex toggle per README / default keymap).  
   **Pass:** Only **MSFT** remains (regex `^MS` matches); title or status indicates **regex** mode.

2. Press **`r`** again (back to substring) with query **`^MS`**.  
   **Pass:** Substring mode: any symbol **containing** the literal `^MS` (likely none) — behavior differs from step 1; mode indicator shows **substring**.

3. **`/`** → **`r`** → type **`[`** (incomplete character class).  
   **Pass:** Inline **invalid regex** message; **all** holdings still listed (no panic, no empty crash).

4. Fix pattern to **`^A`** → **Enter** to commit input mode. Press **`j`**/**`k`**.  
   **Pass:** Navigation only on filtered rows (**AAPL** only for `^A`).

5. **Esc** → full list returns; regex mode cleared per §69.5.

### Manual — Regex mode (Stock View)

1. **Stock View** with watchlist **AAPL**, **MSFT**, **BTC-USD** → **`/`** → **`r`** → pattern **`USD$`**.  
   **Pass:** **BTC-USD** matches; others hidden.

2. **Tab** away and back to **Stock View**.  
   **Pass:** Active filter cleared (§69.5); full watchlist visible.

### Manual — Saved filters (persistence)

1. **Portfolio** → **`/`** → type **`aa`** (substring) → **`Ctrl+s`** → name **`contains-a`** → confirm save.  
   **Pass:** Status OK; quit app.

2. Inspect `~/.stockterm.json` — **`saved_filters`** contains **`contains-a`** with **`"regex": false`** and pattern **`aa`**.  
   **Pass:** JSON matches §69.3.2 shape.

3. Relaunch → **Portfolio** → **`/`** → **`Ctrl+n`** (or **`Ctrl+p`**) until **`contains-a`** loads.  
   **Pass:** Filter applies without retyping; **AAPL**-style rows visible.

4. **`Ctrl+d`** then **`y`** (delete saved filter per §69.3.2) → save config → restart.  
   **Pass:** Entry removed from JSON; recall no longer finds it.

5. **Regression §23.7:** With an active saved/regex filter, wait one quote refresh cycle.  
   **Pass:** Symbols **not** in the filtered view still receive quotes when filter is cleared (view-only filter).

### Manual — Keymap / README

1. Open **README** filter / keymap section.  
   **Pass:** Documents **`r`** (regex toggle), **`Ctrl+s` / `Ctrl+n` / `Ctrl+p` / `Ctrl+d`** (or remapped equivalents), and **`saved_filters`** config field.

2. (Optional) Remap **`FilterRegexToggle`** in **`keymap`** JSON; retest toggle.  
   **Pass:** Regex toggle follows remapped chord on **`FilterInput`** layer only.

### Sign-off — Issue #194

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build / clippy / tests | maintainer | 2026-05-26 | Pass |
| Portfolio: regex toggle + match | maintainer | 2026-05-26 | Pass |
| Portfolio: invalid regex safe UX | maintainer | 2026-05-26 | Pass |
| Stock View: regex + tab clear | maintainer | 2026-05-26 | Pass |
| Saved filter: save / JSON / recall / delete | maintainer | 2026-05-26 | Pass |
| Quote batch full symbol set (§23.7 regression) | maintainer | 2026-05-26 | Pass |
| README / keymap docs | maintainer | 2026-05-26 | Pass |

---

## Issue #24 — Custom dashboard panes (§70)

**Scope:**

- [GitHub Issue #24](https://github.com/FelipeMorandini/stockterm/issues/24) — Composable dashboard panes on a **Dashboard** tab from `~/.stockterm.json`. **Phases A–C (shipped):** config-driven grid + read-only panes for all `DashboardPaneKind` values in v1 (`watchlist`, `stock_detail`, `news`, `portfolio`, `alerts_list`, `chart`, `indicator_summary`). In-app editor (Phase D) → [#208](https://github.com/FelipeMorandini/stockterm/issues/208) (**§71**).

**Spec:** [`docs/SPEC.md`](SPEC.md) §70.

**Status:** **Phases A–C shipped** (Phase A **PR:** [#207](https://github.com/FelipeMorandini/stockterm/pull/207); Phases B–C **PR:** [#210](https://github.com/FelipeMorandini/stockterm/pull/210); sign-off **2026-05-27**). Issue **#24** may stay open for Phase D per triage policy.

**Prerequisites:** §3 watchlist + quote batch, §31 layout (regression), §23 filters (watchlist panes respect `watchlist_filter_indices_cache`), §69 saved filters optional, §58 snapshot patterns.

### Automated (local) — required when implementing

1. From the repo root:

   ```bash
   cargo test dashboard
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** All exit 0; grid/config unit tests green.

2. If `insta` snapshots added (§70.10):

   ```bash
   cargo test dashboard_snapshot
   ```

   **Pass:** Snapshots committed or reviewed via `insta review`.

### Manual — Phase A: config + dual watchlist (Issue #24 acceptance)

**Setup:**

1. Stop StockTerm.
2. Backup `~/.stockterm.json`.
3. Merge the README `dual_watchlist` block (or copy from [`tests/fixtures/dashboard_dual_watchlist.json`](../tests/fixtures/dashboard_dual_watchlist.json)):

   ```json
   "active_dashboard": "dual_watchlist",
   "dashboards": [ { "name": "dual_watchlist", "rows": 1, "cols": 2, "panes": [ ... ] } ]
   ```

4. Ensure watchlist has ≥2 symbols with quotes (e.g. `AAPL`, `MSFT`).
5. `cargo run --release`.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Launch | No panic; config loads; tab bar shows **Dash**. |
| 2 | Open **Dashboard** | Two bordered panes (titles **Watchlist (left)** / **Watchlist (right)** or custom `title`). |
| 3 | Compare to **Stock View** watchlist | Same symbol rows, prices, % change; read-only (no `>` selection cursor in dashboard panes). |
| 4 | **Stock View:** press `/`, type filter substring | Return to **Dashboard** — both panes show filtered subset (§23 cache). |
| 5 | **Stock View:** clear watchlist (or use empty config copy) + restart | Both panes: **No symbols** placeholder; no crash. |
| 6 | Invalid `active_dashboard` name | Centered message (unknown dashboard); no panic. |
| 7 | Omit `active_dashboard` | **Dashboard not configured** (or equivalent §70 message). |
| 8 | Quote error (bad symbol / offline) | Pane body still renders; one-line error footer if `active_runtime_error` set; second pane unaffected. |
| 9 | JSON: remove `wl_right` pane, restart | Single pane only; no ghost rect. |
| 10 | JSON: add `kind: "news"` pane, restart | News pane shows headlines (Phases B–C shipped); not a stub. |
| 11 | Regression: **Stock View**, **Portfolio**, **Charts**, **Settings** | Tab order, §31 splits, candles, filters, keymap unchanged. |

### Manual — Phase B (§70.9.1 shipped 2026-05-27)

**Setup:** Config with `market_overview` preset (2×2) or custom JSON with `stock_detail`, `news`, `portfolio`, `alerts_list` panes. Restart.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | **Dashboard** with four Phase B kinds | Each pane shows live data (not stub text). |
| 2 | Press **a** on Portfolio tab (add dialog) | Dialog on Portfolio only — **not** on Dashboard portfolio pane. |
| 3 | News pane | Headlines for active symbol; row count ≤ `options.max_rows` (default full list). |
| 4 | Alerts pane | Table visible; **a** add dialog does not open from Dashboard. |
| 5 | Narrow terminal (e.g. 80×24) | Grid degrades; **Terminal too small** or partial panes — no panic. |

### Manual — Phase C (§70.9.2 shipped 2026-05-27)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | `chart` pane with active symbol + historical data | Candlestick or line per session `chart_mode`; uses cached layout (no flicker). |
| 2 | `options.symbol` override in JSON | Placeholder explains override unsupported in v1 (per SPEC). |
| 3 | `indicator_summary` with §46 toggles on Charts | Summary lines match enabled indicators; empty state when none. |
| 4 | Switch symbol on Stock View, return to Dashboard | Chart pane updates after fetch completes. |

### Sign-off — Issue #24

**Phase A (shipped 2026-05-26; PR #207):**

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated: `cargo test dashboard` + full `cargo test` + clippy | maintainer | 2026-05-26 | Pass |
| Manual: dual watchlist panes + Stock View parity | maintainer | 2026-05-26 | Pass |
| Manual: filter respects dashboard panes | maintainer | 2026-05-26 | Pass |
| Manual: empty / unknown dashboard / stub kind | maintainer | 2026-05-26 | Pass |
| Manual: config pane add/remove on restart | maintainer | 2026-05-26 | Pass |
| Manual: error footer degradation | maintainer | 2026-05-26 | Pass |
| Regression: core tabs | maintainer | 2026-05-26 | Pass |

**Phase B/C (shipped 2026-05-27):**

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Phase B: market_overview / four pane kinds | maintainer | 2026-05-27 | Pass |
| Phase C: chart + indicator_summary | maintainer | 2026-05-27 | Pass |

---

## Issue #208 — Dashboard in-app pane editor (§71 / §70 Phase D)

**Scope:**

- [GitHub Issue #208](https://github.com/FelipeMorandini/stockterm/issues/208) — In-app dashboard editor on the **Dashboard** tab: create/edit layouts from presets or scratch, add/remove panes, edit grid `rows`/`cols` and pane placement, inline validation (overlap / out-of-bounds), persist to `~/.stockterm.json`, **live preview without restart**. Wizard modal (**`e`** on Dashboard); drag-resize out of scope.

**Spec:** [`docs/SPEC.md`](SPEC.md) §71.

**Prerequisite:** §70 Phases A–C shipped (read-only panes, `dashboard_layout_cache`, `normalize_dashboards` on load). §24 keymap, §22 `try_save_config_with_session`, §18.13 modal layout.

**Status:** **Implemented** (2026-05-27) — automated tests + audit passed; **manual sign-off pending** before merge.

### Automated (local) — required when implementing

1. From the repo root:

   ```bash
   cargo test dashboard
   cargo test dashboard_editor
   cargo test validate_dashboard
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** All exit 0.

2. If `insta` snapshots added (§71.9):

   ```bash
   cargo test dashboard_editor_snapshot
   ```

   **Pass:** Snapshots committed or reviewed via `insta review`.

### Manual — setup

1. Stop StockTerm.
2. Backup `~/.stockterm.json`.
3. Remove `dashboards` / `active_dashboard` keys (or use a temp `HOME`) to test empty-state editor.
4. `cargo run --release`.

### Manual — open editor and preset flow (§71.1 D2)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Open **Dashboard** with no config | Message prompts configuration; press **`e`**. |
| 2 | Editor: **New from preset** → `dual_watchlist` | Prompt for name (or default); draft preview shows two watchlist panes behind overlay. |
| 3 | **Save** | `~/.stockterm.json` contains new `dashboards[]` entry + `active_dashboard`; Dashboard tab renders two panes **without restart**. |
| 4 | Quit and relaunch | Layout persists. |

### Manual — edit layout: grid and panes (§71.1 D3)

**Setup:** Active `market_overview` or custom 2×2 dashboard.

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | **`e`** on Dashboard | Editor opens on **EditLayout**; dimmed live preview matches draft. |
| 2 | Change `rows` or `cols` (within 1..4) | Preview grid updates; invalid values show inline error, **Save** disabled. |
| 3 | **`a`** add pane (`Watchlist`) | New row in pane list; preview shows new pane (auto id). |
| 4 | **`d`** then **`d`** / **`y`** remove pane | Two-step confirm; pane removed from preview. |
| 5 | Create overlapping panes (same cell) | Footer shows overlap error; **Save** blocked; disk JSON unchanged. |
| 6 | Fix overlap, **Save** | Config updated; preview matches saved layout after overlay closes. |

### Manual — edit pane fields (§71.1 D4)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Select pane, **`e`** (EditPane) | Form shows `kind`, row/col, spans, optional title. |
| 2 | Change `kind` to `news`, set `max_rows` | Preview news pane respects row cap (§70). |
| 3 | Set `row_span` past grid edge | Inline error on pane form; cannot return to layout with invalid pane. |
| 4 | **Esc** back to list, **Save** | Committed JSON reflects edits. |

### Manual — validation and save failure

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Duplicate dashboard name on **New** | Inline error; no partial write. |
| 2 | **Esc** with dirty draft | Confirm discard prompt; **n** stays in editor; **y** closes without save. |
| 3 | Save with read-only config (safe temp `HOME` chmod) | Status/runtime error surfaced; in-memory config reverted to snapshot; no panic. |

### Manual — regression

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | §70 read-only behavior | Without editor open, dashboard panes still read-only (no portfolio add from dashboard pane). |
| 2 | Stock View / Portfolio / Charts / Settings | Tab order, §31 layout, filters, keymap unchanged. |
| 3 | JSON power-user path | Hand-edit `dashboards` still loads after restart; editor does not corrupt unrelated config keys. |

### Sign-off — Issue #208

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated: dashboard + editor + validate tests + clippy | | | |
| Manual: preset create without JSON | | | |
| Manual: add/remove pane + grid edit + live preview | | | |
| Manual: overlap / OOB inline errors block save | | | |
| Manual: save persists without restart | | | |
| Manual: discard / save-failure paths | | | |
| Regression: §70 read-only + core tabs | | | |

---

## Issue #209 — Dashboard: gate historical/news fetch on pane kinds (§72)

**Scope:**

- [GitHub Issue #209](https://github.com/FelipeMorandini/stockterm/issues/209) — On **`Tab::Dashboard`**, spawn historical and news background fetches **only** when the **active** dashboard definition (committed config) includes panes that need that data. **`dual_watchlist`** must not trigger historical/news HTTP; **`market_overview`** must still refresh news; dashboards with **`Chart`** / **`IndicatorSummary`** panes must still refresh historical.

**Spec:** [`docs/SPEC.md`](SPEC.md) §72.

**Prerequisite:** §70 Phases A–C shipped (dashboard tab + pane kinds). §71 editor optional but useful for adding/removing pane kinds without hand-editing JSON.

**Status:** **Shipped** (2026-05-27) — automated tests + audit passed; manual sign-off **2026-05-27**.

### Automated (local) — required

1. From the repo root:

   ```bash
   cargo test dashboard_fetch_needs
   cargo test dashboard
   cargo test on_background_tick_dashboard
   cargo test
   cargo clippy -- -D warnings
   ```

   **Pass:** All exit 0.

2. Predicate matrix (§72.6) — spot-check test names:

   ```bash
   cargo test dashboard_fetch_needs_dual_watchlist -- --nocapture
   cargo test dashboard_fetch_needs_market_overview -- --nocapture
   ```

   **Pass:** `dual_watchlist` → no historical, no news; `market_overview` → news only, no historical.

3. Static audit — single gated Dashboard spawn site:

   ```bash
   rg -n "try_spawn_historical_fetch|try_spawn_news_fetch" src/app/app.rs
   ```

   **Pass:** Dashboard tab calls are only inside the `active_tab == Tab::Dashboard` block guarded by `dashboard_fetch_needs_*` (Charts / News tab arms unchanged).

### Manual — setup

1. Backup `~/.stockterm.json`.
2. Ensure Yahoo provider (or Polygon with key) and symbol **AAPL** on watchlist.
3. Optional: enable fetch logging for observation:

   ```bash
   RUST_LOG=stockterm::fetch=debug cargo run --release
   ```

   (Log file path per README / `tracing` subscriber in `main.rs`.)

### Manual — `dual_watchlist` (no historical / no news)

**Setup:** Set `active_dashboard` to `dual_watchlist` (editor preset or JSON). Open **Dashboard** tab. Wait ≥ one full `refresh_rate` cycle (default 30 s, or lower `refresh_rate` in Settings for faster test).

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Stay on Dashboard 60 s | **No** `historical` / `news` fetch log lines (or network calls to chart/news endpoints) while quote refresh may still occur. |
| 2 | Switch to **Charts** tab | Historical fetch **resumes** (status / chart loads). |
| 3 | Return to **Dashboard** (`dual_watchlist`) | Historical fetch **stops** again on subsequent ticks; Charts data may remain in memory but no new historical polls while idle on Dashboard. |

### Manual — `market_overview` (news only)

**Setup:** Active dashboard `market_overview` (four panes including **News**; no **Chart** / **IndicatorSummary** in default preset).

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Dashboard tab, wait one refresh cycle | News pane populates or shows loading/error placeholder; **news** fetch activity present in logs. |
| 2 | Same session | **No** historical fetch activity attributable to Dashboard ticks (Charts tab not focused). |
| 3 | Add **Chart** pane via editor (**`e`**), save, stay on Dashboard | After save, historical fetch **starts** on throttle; chart pane shows series or loading state (not permanently empty due to missing fetch). |

### Manual — regression

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | **Charts** tab | Historical refresh unchanged vs pre-#209 (`1`–`4`, pan/zoom). |
| 2 | **News** tab | News list refresh unchanged. |
| 3 | **Stock View** on Dashboard switch | Quote batch still updates watchlist/detail panes on Dashboard (**§23.7**). |
| 4 | **`Ctrl+R`** on Dashboard (`dual_watchlist`) after a Charts historical failure | Retry does **not** spawn historical while on watchlist-only dashboard. |

### Sign-off — Issue #209

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated: `dashboard_fetch_needs` + app tick guard tests + clippy | maintainer | 2026-05-27 | Pass |
| Manual: `dual_watchlist` — no hist/news network work | maintainer | 2026-05-27 | Pass |
| Manual: `market_overview` — news yes, hist no (until Chart added) | maintainer | 2026-05-27 | Pass |
| Manual: Chart pane enables historical on Dashboard | maintainer | 2026-05-27 | Pass |
| Regression: Charts / News tabs + quote batch on Dashboard | maintainer | 2026-05-27 | Pass |

---

## Issue #204 — Config: canonicalize persisted symbols on load (§73)

**Scope:**

- [GitHub Issue #204](https://github.com/FelipeMorandini/stockterm/issues/204) — On **`Config::try_load`**, rewrite persisted ticker fields through **`normalize_symbol`** (§67), dedupe equivalent **watchlist** / **portfolio** rows, and align JSON with runtime **`watchlist_quotes`** keys. Persist canonical form on the **next normal save** (no immediate write on load).

**Spec:** [`docs/SPEC.md`](SPEC.md) §73.

**Status:** **Shipped** (2026-05-28) — manual sign-off complete.

**Prerequisite:** §67 / Issue **#79** shipped (`normalize_symbol`, `symbols_equivalent`).

### Automated (local) — required when implementing

1. Symbol migration unit tests:

   ```bash
   cargo test canonicalize_persisted
   cargo test load_config
   cargo clippy -- -D warnings
   ```

   **Pass:** Exit 0; §73.7 table tests green.

2. Regression — §67 symbol tests unchanged:

   ```bash
   cargo test normalize_symbol
   cargo test symbols_equivalent
   ```

   **Pass:** §67 / §43 tests still green.

### Manual — legacy mixed-case config migration

**Prep:** Backup `~/.stockterm.json`. Craft a test file (or hand-edit) with mixed-case symbols, e.g.:

```json
{
  "watchlist": ["aapl", "AAPL", "MSFT"],
  "portfolio": [
    { "symbol": "aapl", "shares": 1.0, "purchase_price": 100.0 },
    { "symbol": "AAPL", "shares": 2.0, "purchase_price": 110.0 }
  ],
  "alerts": [{ "symbol": "msft", "condition": "Above", "price": 1.0, "triggered": false }],
  "default_symbol": "btc - usd",
  "last_symbol": "aapl",
  "refresh_rate": 0,
  "api_key": "",
  "provider": "yahoo"
}
```

(Adjust field names to match your file; ensure valid JSON.)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Start app with test config | App launches; watchlist shows **one** **AAPL** row (not two); **MSFT** present. |
| 2 | Open Portfolio tab | **One** **AAPL** holding (first row kept per §73.2); symbol column uppercase/canonical. |
| 3 | Open Alerts tab | Alert symbol displays **MSFT** (canonical). |
| 4 | Wait for quote refresh on watchlist | Prices appear for **AAPL** / **MSFT** (no silent empty rows from key mismatch). |
| 5 | Trigger a config save (e.g. switch tab, toggle a setting, or add/remove a watchlist symbol) | Quit app; reopen `~/.stockterm.json` — symbols are canonical (`AAPL`, `MSFT`, `BTC-USD`, …); no duplicate case variants in **watchlist**. |

### Manual — invalid symbol drop

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Add a watchlist entry with a control character or disallowed symbol (if reproducible) or use a crafted JSON invalid ticker | Entry **dropped** on load (not shown) or rejected on add per §67 policy; app does not panic. |

### Manual — dashboard symbol override (optional)

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Hand-edit a dashboard pane **`options.symbol`** with spaces/mixed case (e.g. `"aapl"`) | After load, override is canonical or cleared; chart pane still resolves symbol. |

### Regression

| Step | Action | Pass criteria |
|------|--------|---------------|
| 1 | Fresh default config (no file) | Behavior unchanged vs pre-#204. |
| 2 | §67 Unicode watchlist / chart smoke | Case-only add still skips chart clear; Unicode tickers still normalize. |
| 3 | Saved filters / dashboards | Unrelated config keys unchanged after load. |

### Sign-off — Issue #204

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| `cargo test canonicalize_persisted` + `load_config` + clippy | Maintainer | 2026-05-28 | Pass |
| Manual: mixed-case JSON → deduped watchlist + portfolio | Maintainer | 2026-05-28 | Pass |
| Manual: quotes populate after refresh | Maintainer | 2026-05-28 | Pass |
| Manual: disk JSON canonical after save | Maintainer | 2026-05-28 | Pass |
| Regression: default config + §67 smoke | Maintainer | 2026-05-28 | Pass |
