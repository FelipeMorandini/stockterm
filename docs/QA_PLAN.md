# QA Plan — Manual verification

Use the sections below per milestone. **Issue #3** remains the regression baseline for the watchlist; **Issue #44** adds keyboard modifier behavior.

---

## Issue #44 — Stock View & Alerts modifier keys

**Scope:** [GitHub Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) — Shift/lowercase acceptance for symbol typing and for `a`/`d` on Alerts; no accidental triggers with Ctrl/Alt/Meta chords.

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
