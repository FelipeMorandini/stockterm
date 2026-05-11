# QA Plan — Manual verification

Use the sections below per milestone. **Issue #3** remains the regression baseline for the watchlist; **Issue #44** adds keyboard modifier behavior (Stock View / Alerts). **Issues #48 / #6** extend modifier parity and portfolio add/remove UX on the Portfolio tab (see [`docs/SPEC.md`](SPEC.md) §§12–13). **Issue #31** covers the Yahoo/Polygon provider adapter and structured errors. **Issues #29 / #5 / #11 / #12** cover the Search, News, and Settings tabs (M3). **Issues #9, #8, #7** cover Charts time ranges, zoom/pan, and candlesticks (M4 — see [`docs/SPEC.md`](SPEC.md) §11).

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

**Applies only if #17 is included in the same delivery; otherwise mark N/A and file follow-up.**

1. With an artificial delay or very slow network (per developer test harness or #17 smoke test), hold a key that navigates tabs or watchlist rows.  
   **Pass:** Input continues to be processed; screen keeps redrawing; a multi-second HTTP wait does not freeze the TUI.

2. **Pass:** No `await` of HTTP on the path between redraw and input handling (verified by code review / #17 acceptance).

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
