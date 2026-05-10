# QA Plan — Manual verification

Run these after implementation of **Issue #1** (stabilize build & API wiring). Automated checks are listed first; the rest are manual.

## Automated (local)

1. From the repo root:

   ```bash
   cargo build --release
   cargo clippy -- -D warnings
   ```

   **Pass:** Both exit 0; `cargo build` shows no warnings.

2. Confirm no placeholder API key in source:

   ```bash
   rg 'YOUR_POLYGON_API_KEY' src/ || true
   ```

   **Pass:** No matches.

3. **Pass (optional if `rg` unavailable):** Open `src/api/polygon.rs` and confirm there is no string literal placeholder key; all paths use config and/or `STOCKTERM_API_KEY` per SPEC.

---

## Manual — runtime

**Prerequisites:** To validate **Stock / Charts / News** (live Polygon data), you need a **non-empty** Polygon key in `~/.stockterm.json` (`api_key` string) **or** `export STOCKTERM_API_KEY=...`. If both are missing or empty, the app shows an explicit “Missing Polygon API key…” message and does not call Polygon with `apiKey=` (no spurious 401 from an empty key). You can still validate tabs, portfolio, and alerts without a key. A Yahoo/other provider migration is **not** in scope for Issue #1; this build remains Polygon-only.

1. **Launch:** Run `cargo run --release` (or the release binary).  
   **Pass:** TUI starts; no immediate panic; status bar or UI visible.

2. **Tab / Shift+Tab:** Press `Tab` repeatedly, then `Shift+Tab`.  
   **Pass:** Tab highlight cycles through all tabs including **Alerts** in a consistent order; content area changes to match (Portfolio shows portfolio UI, Alerts shows alerts UI, etc.).

3. **Portfolio tab — `a` / arrows / `d`:** Switch to **Portfolio**. With a non-empty symbol on Stock View first (e.g. `AAPL`), press `a`. Use **↑** / **↓** to move the `>` highlight; the first **↓** should select a row when none was selected.  
   **Pass:** A row appears (or holdings update) per `add_to_portfolio` defaults; **`d` removes the highlighted row**. Uppercase letters must **not** append to the ticker while on Portfolio (no accidental symbol mutation from `a`).

4. **Alerts tab:** Switch to **Alerts**. Press `a`, then use **↑** / **↓** to highlight a row.  
   **Pass:** An alert row appears; **`d` removes the highlighted alert**.

5. **Portfolio Enter (async refresh):** On Portfolio, select a row (arrow keys), press `Enter`.  
   **Pass:** App switches to Stock View (or defined behavior); ticker data updates without panic; no “future not awaited” / deadlock — if issues occur, capture stderr.

6. **Stock View — existing flow:** On Stock View, type a symbol with uppercase letters, press `Enter`.  
   **Pass:** Fetch triggers (or error message in UI if API key missing); behavior matches pre-fix intent.

7. **Quit:** Press `q`.  
   **Pass:** Terminal restores; process exits 0.

---

## Sign-off

| Check | Tester | Date | Pass/Fail |
|-------|--------|------|-----------|
| Automated | | | |
| Tab cycling | | | |
| Portfolio a/d | | | |
| Alerts tab | | | |
| Portfolio Enter | | | |
| Stock Enter | | | |
