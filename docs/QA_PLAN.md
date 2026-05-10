# QA Plan — Manual verification

Run these after implementation of **[Issue #27](https://github.com/FelipeMorandini/stockterm/issues/27)** (persist alerts to `~/.stockterm.json`). Automated checks are listed first; the rest are manual.

## Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; `cargo build` shows no warnings.

2. **Regression:** Prior milestone checks still compile (no placeholder Polygon key in source):

   ```bash
   rg 'YOUR_POLYGON_API_KEY' src/ || true
   ```

   **Pass:** No matches.

---

## Manual — persistence

**Prerequisites:** Backup `~/.stockterm.json` if it contains data you care about. A valid file is optional; the app creates/overwrites it on save.

1. **Baseline file:** Note whether `~/.stockterm.json` exists and, if so, the current `"alerts"` array (may be `[]` or absent).

2. **Launch:** `cargo run --release`. Switch to the **Alerts** tab.

3. **Add alert:** Ensure **Stock View** has a non-empty symbol (e.g. `AAPL`), go to **Alerts**, press **`a`** (adds a row per current handler).  
   **Pass:** Row appears in the table.

4. **Verify disk:** Open `~/.stockterm.json` (or `cat` it).  
   **Pass:** `alerts` includes at least one object with fields consistent with the model (`symbol`, `condition`, `price`, `triggered`).

5. **Remove alert:** With a row selected, press **`d`**.  
   **Pass:** Row disappears from the UI; `alerts` in `~/.stockterm.json` matches the new count (e.g. empty array if all removed).

6. **Survive restart:** Add at least one alert again, quit with **`q`**, relaunch `cargo run --release`, open **Alerts**.  
   **Pass:** The same alert(s) appear without re-adding.

7. **Save error (optional):** If feasible, make the config path unwritable (e.g. read-only parent) and trigger **`a`** or **`d`**.  
   **Pass:** Status area shows a clear error mentioning save/config failure; app does not panic.

---

## Sign-off

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated build/clippy | | | |
| JSON updated on add | | | |
| JSON updated on remove | | | |
| Restart restores alerts | | | |
| Save error surfaced (optional) | | | |
