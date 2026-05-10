# QA Plan — Manual verification

**Scope:** [Issue #37](https://github.com/FelipeMorandini/stockterm/issues/37) (alerts table constraints) and [Issue #30](https://github.com/FelipeMorandini/stockterm/issues/30) / [#38](https://github.com/FelipeMorandini/stockterm/issues/38) (`check_alerts` + Alerts-tab quote cadence). Same release also fixed Polygon aggregate **volume** (`v`) JSON when it arrives as a float (Stock View / charts).

Run these when validating that release. Automated checks are listed first; the rest are manual.

## Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; `cargo build` shows no warnings.

2. **Regression:** No placeholder Polygon key in source:

   ```bash
   rg 'YOUR_POLYGON_API_KEY' src/ || true
   ```

   **Pass:** No matches.

---

## Manual — Issue #37 (alerts table layout)

**Prerequisites:** Valid Polygon credentials in `~/.stockterm.json` or `STOCKTERM_API_KEY` so quotes load.

1. **Launch:** `cargo run --release`. Add at least one alert (**Alerts** tab, **`a`**) so the table is visible.

2. **Columns:** Confirm the table shows exactly **five** logical columns: Symbol, Condition, Price, Current, Status — headers line up with cells; no overlapping or truncated header text inconsistent with data.

3. **Narrow terminal:** Resize the terminal to ~80 columns; ensure the table remains readable (no garbled layout).

4. **Pass:** Constraint count matches visible columns; layout acceptable on typical width.

---

## Manual — Issues #30 / #38 (`check_alerts` + Alerts tab polling)

**Prerequisites:** Backup `~/.stockterm.json` if needed. API key configured.

1. **Baseline:** Note `alerts` in `~/.stockterm.json` (or empty array).

2. **Symbol and alert:** On **Stock View**, set symbol to a liquid ticker (e.g. `AAPL`). Open **Alerts**, press **`a`** (adds alert per current handler — e.g. Above $100).  
   **Pass:** Row appears; JSON contains the new alert with `"triggered": false` (or equivalent).

3. **Triggered persistence:** With **Stock View** active, wait at least one **refresh interval** (`refresh_rate` in config, minimum 5s per app logic) or trigger a refetch (**Enter** if bound), until the live price is above the alert threshold (or temporarily use a threshold you know is crossed).  
   **Pass:** Status column shows **TRIGGERED** (or equivalent); `~/.stockterm.json` shows `"triggered": true` for that alert after the transition.

4. **Alerts tab polling:** Restart the app, set symbol and an alert that will **not** trigger immediately (e.g. Above well above market). Switch to **Alerts** only (do not stay on Stock View). Wait **two** refresh intervals.  
   **Pass:** **Current** column updates over time (quotes still fetched while on Alerts); no need to switch to Stock View solely to refresh prices.

5. **Limitation (documented):** Add an alert for symbol **B** while **Stock View** shows symbol **A**, and **B** is not in portfolio.  
   **Pass:** Until **B** is fetched or in portfolio with a price, automatic `triggered` transition may not run — acceptable; tester notes behavior matches SPEC.

6. **No panic / errors:** During the above, status area should not show spurious save errors; app must not panic.

---

## Sign-off

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build/clippy | | | |
| Alerts table 5 columns / layout (#37) | | | |
| `triggered` flips and persists (#30/#38) | | | |
| Quotes refresh on Alerts tab (#30/#38) | | | |
| Documented cross-symbol limitation | | | |
