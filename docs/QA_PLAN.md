# QA Plan — Manual verification

Use the sections below per milestone. **[Issues #32](https://github.com/FelipeMorandini/stockterm/issues/32), [#33](https://github.com/FelipeMorandini/stockterm/issues/33), [#55](https://github.com/FelipeMorandini/stockterm/issues/55)** cover **`get_current_price`** symbol/ticker alignment, **`ProviderError`** **`thiserror`** migration, and API error taxonomy audit (see [`docs/SPEC.md`](SPEC.md) §41; sign-off **2026-05-19**; **PR:** [#155](https://github.com/FelipeMorandini/stockterm/pull/155)). **[Issues #36](https://github.com/FelipeMorandini/stockterm/issues/36), [#56](https://github.com/FelipeMorandini/stockterm/issues/56), [#106](https://github.com/FelipeMorandini/stockterm/issues/106)** cover Charts timestamp panic hardening + tests, explicit quote-batch **`Semaphore`** acquire failures, and §18.15 post-audit **`centered_rect`** / notify **`body`** hardening (see [`docs/SPEC.md`](SPEC.md) §40; sign-off **2026-05-19**; **PR:** [#154](https://github.com/FelipeMorandini/stockterm/pull/154)). **[Issues #108](https://github.com/FelipeMorandini/stockterm/issues/108), [#78](https://github.com/FelipeMorandini/stockterm/issues/78), [#87](https://github.com/FelipeMorandini/stockterm/issues/87)** cover event-thread clean shutdown, inflight recovery when both channel sends fail, and documented **`mpsc`** back-pressure policy (see [`docs/SPEC.md`](SPEC.md) §39; sign-off **2026-05-19**; **PR:** [#153](https://github.com/FelipeMorandini/stockterm/pull/153)). **[Issues #76](https://github.com/FelipeMorandini/stockterm/issues/76), [#85](https://github.com/FelipeMorandini/stockterm/issues/85), [#86](https://github.com/FelipeMorandini/stockterm/issues/86), [#117](https://github.com/FelipeMorandini/stockterm/issues/117), [#118](https://github.com/FelipeMorandini/stockterm/issues/118)** cover async/HTTP reliability tail: **`tracing`** for dropped fetch results, cap **`STOCKTERM_DEBUG_HTTP_DELAY_MS`**, dev panic payload logging, **408** retry, structured HTTP client init (see [`docs/SPEC.md`](SPEC.md) §38; sign-off **2026-05-18**; **PR:** [#152](https://github.com/FelipeMorandini/stockterm/pull/152)). **[Issues #81](https://github.com/FelipeMorandini/stockterm/issues/81), [#82](https://github.com/FelipeMorandini/stockterm/issues/82), [#83](https://github.com/FelipeMorandini/stockterm/issues/83)** cover Stock View narrow-terminal status hints, plain-**Tab** portfolio dialog focus, and **`add_to_portfolio`** false-path documentation (see [`docs/SPEC.md`](SPEC.md) §37; sign-off **2026-05-18**; **PR:** [#151](https://github.com/FelipeMorandini/stockterm/pull/151)). **[Issue #54](https://github.com/FelipeMorandini/stockterm/issues/54)** covers Yahoo news **`query2`** parse resilience and optional **`STOCKTERM_DEBUG_YAHOO_NEWS`** attempt logging (see [`docs/SPEC.md`](SPEC.md) §36; sign-off **2026-05-18**; **PR:** [#150](https://github.com/FelipeMorandini/stockterm/pull/150)). **[Issue #4](https://github.com/FelipeMorandini/stockterm/issues/4)** covers **`Config.refresh_rate`** vs the ~200 ms UI tick, in-flight quote guards, and Settings persistence (see [`docs/SPEC.md`](SPEC.md) §35; sign-off **2026-05-18**; **PR:** [#149](https://github.com/FelipeMorandini/stockterm/pull/149)). **[Issue #60](https://github.com/FelipeMorandini/stockterm/issues/60)** covers Search **Esc** not clearing runtime errors from other tabs (see [`docs/SPEC.md`](SPEC.md) §33). **[Issue #89](https://github.com/FelipeMorandini/stockterm/issues/89)** covers automated **`wiremock`** proof that Yahoo **`yahoo_latest_quote`** falls back from **`v7/finance/quote`** to **`v8/finance/chart`** (see [`docs/SPEC.md`](SPEC.md) §32). **[Issues #90](https://github.com/FelipeMorandini/stockterm/issues/90) and [#91](https://github.com/FelipeMorandini/stockterm/issues/91)** cover optional v7→v8 stderr diagnostics and v7 multi-row **symbol** matching (see [`docs/SPEC.md`](SPEC.md) §34). **Issue #3** remains the regression baseline for the watchlist; **Issue #44** adds keyboard modifier behavior (Stock View / Alerts). **Issue #15** covers **layout / widget visibility** (`Config.layout`, shell chrome, Stock View + Charts pane splits — see [`docs/SPEC.md`](SPEC.md) §31). **Issues #48 / #6** extend modifier parity and portfolio add/remove UX on the Portfolio tab (see [`docs/SPEC.md`](SPEC.md) §§12–13). **Issue #31** covers the Yahoo/Polygon provider adapter and structured errors. **Issue #53** covers Yahoo **`v7`** multi-symbol quote batching (see [`docs/SPEC.md`](SPEC.md) §9.15). **Issues #29 / #5 / #11 / #12** cover the Search, News, and Settings tabs (M3). **Issues #9, #8, #7** cover Charts time ranges, zoom/pan, and candlesticks (M4 — see [`docs/SPEC.md`](SPEC.md) §11). **Issues #62, #63, #64** cover M4 Charts polish (symbol/series coherence, Yahoo W1 fallback, fetch resilience — see [`docs/SPEC.md`](SPEC.md) §11.11). **Issues #71, #72, #73, #74** cover M4 follow-up hardening (inflight/channel parity, dead historical helper removal, W1 unit tests, watchlist chart flicker — see [`docs/SPEC.md`](SPEC.md) §11.12). **Issues #43, #49, #50, #67, #69** cover Alerts title/copy, Stock View typing hint, Portfolio dialog Tab focus, and commit validation (see [`docs/SPEC.md`](SPEC.md) §15). **Issues #17, #46, #77** cover async loop close-out, quote-batch panic hardening, and pending-flag behavior on stock recovery (see [`docs/SPEC.md`](SPEC.md) §16). **Issue #2** covers latest-session quote adapters (Yahoo v7 primary + v8 fallback, Polygon daily latest bar — see [`docs/SPEC.md`](SPEC.md) §17). **Issues #10, #42** cover Alerts add dialog, bell + desktop notifications, Settings toggle, and latched Status vs `triggered` (see [`docs/SPEC.md`](SPEC.md) §18). **Issues #93, #94, #95** cover shared modal `centered_rect`, alert Condition **←/→** keys, and optional stderr for desktop **`show()`** outcomes (see [`docs/SPEC.md`](SPEC.md) §18.13 — manual sign-off 2026-05-12). **Issues #96, #97, #98** cover alerts **`try_save`** failure UX, one coalesced desktop toast per crossing batch, and sanitized notification text (see [`docs/SPEC.md`](SPEC.md) §18.14 — [PR #105](https://github.com/FelipeMorandini/stockterm/pull/105); run the **Issues #96, #97, #98** section for manual sign-off). **Issues #100, #101, #104** cover `centered_rect` percent contract, README debug env documentation, and total notify **`body`** byte cap (see [`docs/SPEC.md`](SPEC.md) §18.15). **Issue #18** covers API robustness: HTTP timeouts, 429 / **`Retry-After`**, backoff, and extended **`ProviderError`** (see [`docs/SPEC.md`](SPEC.md) §19 — [PR #115](https://github.com/FelipeMorandini/stockterm/pull/115); **manual sign-off** in **Issue #18** below). **Issues #110, #111, #112, #113, #114, #116** cover §19 post-audit hardening (bounded error reads, **`Retry-After`** cap + rate-limit **`Display`**, HTTP-date tolerance, test-harness docs, retry cleanup, **`Debug`** URL redaction — see [`docs/SPEC.md`](SPEC.md) §19.13 and **Issues #110–#114, #116** in this file). **Issue #20** covers error UX: categorized status line, **`Ctrl+E`** error log overlay, **`Ctrl+R`** retry last failed fetch, transient auto-clear, startup banner (see [`docs/SPEC.md`](SPEC.md) §20). **Issue #14** covers the theme system: presets + JSON overrides, Settings row **3** commit + preview, and `ResolvedTheme` / `theme.canvas()` across tabs (see [`docs/SPEC.md`](SPEC.md) §21 — [PR #126](https://github.com/FelipeMorandini/stockterm/pull/126); manual sign-off 2026-05-13). **Issues #19, #103** cover config persistence polish (`~/.stockterm.json`, last tab/symbol, documented schema) and keeping **`Failed to save alerts:`** visible when quote batches surface errors (see [`docs/SPEC.md`](SPEC.md) §22). **Issues #34, #35, #40, #129** cover API-key operator documentation, config load failure visibility, optional non-blocking saves, and session-write debouncing (see [`docs/SPEC.md`](SPEC.md) §22.7). **[Issue #16](https://github.com/FelipeMorandini/stockterm/issues/16)** covers substring filter on **Portfolio** holdings and **Stock View** watchlist (`/`, **Esc** clears, **Enter** commits, **Tab** unchanged — see [`docs/SPEC.md`](SPEC.md) §23). **Issues [#58](https://github.com/FelipeMorandini/stockterm/issues/58), [#59](https://github.com/FelipeMorandini/stockterm/issues/59)** cover News clipboard copy and non-blocking **`http`/`https`** URL open (see [`docs/SPEC.md`](SPEC.md) §27). **[Issue #137](https://github.com/FelipeMorandini/stockterm/issues/137)** covers remappable filter-input mode (`BindingLayer::FilterInput` — see [`docs/SPEC.md`](SPEC.md) §28). **[Issue #139](https://github.com/FelipeMorandini/stockterm/issues/139)** covers explicit alert add-dialog symbol + condition keymap actions (see [`docs/SPEC.md`](SPEC.md) §29). **[Issue #138](https://github.com/FelipeMorandini/stockterm/issues/138)** covers compile-time default keymap chord table refactor — no user-visible behavior change (see [`docs/SPEC.md`](SPEC.md) §30; manual sign-off **2026-05-17**).

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
