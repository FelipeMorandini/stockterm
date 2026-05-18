# StockTerm

Terminal UI (TUI) for stock quotes, watchlists, charts, and alerts. Rust + [ratatui](https://github.com/ratatui-org/ratatui) + Tokio.

Product behavior and milestones are documented in [`docs/SPEC.md`](docs/SPEC.md). Manual verification steps live in [`docs/QA_PLAN.md`](docs/QA_PLAN.md).

## Config file (`~/.stockterm.json`)

| Field | Type | Default | Notes |
|-------|------|---------|--------|
| `portfolio` | array | `[]` | Holdings (symbol, shares, cost). |
| `watchlist` | array of strings | `[]` | Stock View symbols (uppercase). |
| `refresh_rate` | number | `0` | Quote/charts/news poll interval in seconds (`0` → 30 s effective; minimum 5 s). |
| `api_key` | string | `""` | Polygon key; optional if `STOCKTERM_API_KEY` is set. |
| `alerts` | array | `[]` | Price alerts. |
| `default_symbol` | string | `""` | Startup symbol when `watchlist` is empty (empty → `AAPL`). |
| `theme` | object or null | `null` | Theme preset and hex overrides (see [`docs/SPEC.md`](docs/SPEC.md) §21). |
| `provider` | string | `"yahoo"` | `"yahoo"` or `"polygon"`. |
| `notifications_enabled` | boolean | `true` | Desktop toasts for alert fires (bell always rings). |
| `last_tab` | string or omitted | omitted | Last tab: `stock_view`, `portfolio`, `alerts`, `search`, `news`, `charts`, `settings` (Issue #19 / §22). |
| `last_symbol` | string or omitted | omitted | Last active ticker when `watchlist` was empty at launch (normalized). |
| `keymap` | object or omitted | omitted | Optional chord → action overrides (see **Keymap** below; Issue #13 / [`docs/SPEC.md`](docs/SPEC.md) §24). |
| `layout` | object | omitted → built-in defaults | Shell chrome and pane sizing (Issue #15 / [`docs/SPEC.md`](docs/SPEC.md) §31). |

#### `layout` object

| Field | Type | Default | Valid range / values |
|-------|------|---------|----------------------|
| `show_tab_bar` | boolean or omitted | `true` | Top tab strip (3 rows). |
| `show_status_bar` | boolean or omitted | `true` | Bottom status line (1 row). |
| `stock_view_watchlist_pct` | number or omitted | `42` | Stock View watchlist band height **20–80**. |
| `charts_chart_pct` | number or omitted | `100` | Charts tab chart height **30–100**; **100** = full body (no inner split). |
| `preset` | string or omitted | omitted | `default`, `compact`, `wide`, `chart_focused` — base values; omitted fields inherit from preset. |

**Examples:**

```json
"layout": { "show_status_bar": false }
```

```json
"layout": { "preset": "chart_focused" }
```

```json
"layout": {
  "stock_view_watchlist_pct": 60,
  "charts_chart_pct": 70
}
```

On **Settings** row **6. Layout**, use **←/→** or **h**/**l** to preview presets and **Enter** to save.

### Keymap (`keymap` field)

Optional JSON object: each key is a **chord** string, each value is an **`Action`** name in **PascalCase** (for example `"Quit"`, `"StockRowDown"`). Overrides replace the default binding for that action in every [`BindingLayer`](src/config/keymap.rs) where built-in defaults register it (for example portfolio row **↑/↓** while remove-confirm is armed — Issue #134 / [`docs/SPEC.md`](docs/SPEC.md) §25); see [`src/config/keymap.rs`](src/config/keymap.rs) for the full default table. **Issues #58 / #59 / §27:** On the **News** tab, default **`NewsEnter`** is **Enter** (open selected article URL in the browser) and **`NewsCopyUrl`** is **`c`** (copy URL to the clipboard). **Issue #136 / §26:** These stay **wildcard** (no per-letter `Action` rows): Stock View symbol letters and Search query characters. Explicit defaults cover portfolio / alert dialog **digits** and **`.`**, plus Settings edit buffer input: **`PortfolioDialogDigitOrDot`**, **`AlertDialogDigitOrDot`**, **`SettingsEditDigit`**, and **`SettingsEditSymbolChar`** (default-symbol row only for letters). **Issue #139 / §29 — alert add dialog:** **`AlertDialogSymbolChar`** (`c`–`z`, `-`), **`AlertDialogConditionAbove`** (`a`), **`AlertDialogConditionBelow`** (`b`); on **Symbol** focus, `a`/`b` still append **`A`/`B`** via the condition actions (Shift/Caps per §8). Remapping a condition key frees that chord for symbol typing when unbound (optional wildcard fallback). **Issue #137 / §28 — table filter:** **`StockFilterToggle`** / **`PortfolioFilterToggle`** enter filter mode on **Stock View** / **Portfolio**; while filter input is active, keys resolve on **`FilterInput`** only — **`FilterClear`**, **`FilterCommit`**, **`FilterBackspace`**, **`FilterSlash`**, and per-character **`FilterQueryChar`** (`char:0`–`9`, `char:a`–`z` defaults). Unmapped keys in filter mode are ignored (they do not reach watchlist/portfolio actions). Remapping a **`Filter*`** action onto a chord already used by another action on **`FilterInput`** (for example **`FilterClear`** → **`char:a`**) is rejected and the app falls back to the full built-in keymap (same as §24 duplicate-chord rules).

**Chord grammar** (ASCII, case-insensitive except `char:` payload):

- Combine with **`+`**: `shift`, `ctrl` (or `control`), `alt` — e.g. `ctrl+e`, `shift+d`.
- **Named keys**: `tab`, `backtab`, `esc`, `enter`, `backspace`, `up`, `down`, `left`, `right`, `pageup`, `pagedown`, `colon`, `semicolon`, `slash`, **`plus`** (the `+` key — use this token because a raw `+` would split the parser), **`minus`**.
- **One ASCII character**: either a single visible character (`q`, `1`, `/`, …) or `char:x` for a single character `x` (useful when `x` is `:` / `;` / etc.).
- **Invalid** `keymap` (unknown action name, unknown chord, or conflicting chord assignment): startup shows a line starting with **`keymap:`** and the built-in defaults are used for the whole map.
- **Shift+Tab:** terminals may send `BackTab` with `SHIFT`, plain `BackTab`, or `Tab` with `SHIFT` only. StockTerm tries those variants when resolving `Action::GlobalBackTab` and dialog `BackTab` bindings (see `chord_lookup_candidates` in [`src/config/keymap.rs`](src/config/keymap.rs)).

**Example** — bind quit to `:` (colon key):

```json
"keymap": {
  "colon": "Quit"
}
```

To discover exact **`Action`** names, see the `Action` enum in [`src/config/keymap.rs`](src/config/keymap.rs) (serde renames match PascalCase JSON).

The Polygon **`api_key`** is stored **in plaintext** inside **`~/.stockterm.json`**. If **`api_key`** is empty, StockTerm uses a non-empty **`STOCKTERM_API_KEY`** environment variable instead (resolution: [`Config::effective_api_key`](https://github.com/FelipeMorandini/stockterm/blob/main/src/config/config.rs)). Treat the config file like a secret: use restrictive file permissions where your OS supports them (for example **`chmod 600 ~/.stockterm.json`** on Unix), do not commit real keys to git, and avoid pasting keys into logs or screenshots. Yahoo mode does not require a key.

Provider selection and HTTP behavior are specified in [`docs/SPEC.md`](docs/SPEC.md) (§9 and §31).

## Developer / debug

These environment variables are supported for local diagnosis. Any other `STOCKTERM_DEBUG_*` name is **not** supported unless it appears here or in `docs/SPEC.md`.

| Variable | When it applies | Behavior |
|----------|-----------------|----------|
| _(none)_ | Default HTTP stack | **5 s** connect + **10 s** request timeout on the shared `reqwest::Client` ([`docs/SPEC.md`](docs/SPEC.md) §19 / Issue #18; `src/api/http.rs`). |
| _(tests)_ | Authors writing **`#[tokio::test(start_paused = true)]`** + **`reqwest`** | Paused **`tokio::time::advance`** can fire **`reqwest`**’s request **`timeout`** while a **`GET`** is still in flight → spurious **`Timeout`**. Prefer wall-clock waits for **`Retry-After`** assertions or an isolated **`Client`** with a short timeout for stall tests — [`docs/SPEC.md`](docs/SPEC.md) §19.8 / §19.13.3. |
| `STOCKTERM_DEBUG_ALERT_NOTIFY` | Build with the default **`desktop-notify`** Cargo feature | Set to exactly `1` (no trimming; no other value enables it). After `notify-rust` `Notification::show()`, StockTerm may `eprintln!` the `Result` to stderr on the **coalesced** desktop notify path (including `Ok(())`) so you can confirm the call completed. |
| `STOCKTERM_DEBUG_HTTP_DELAY_MS` | Any build | Non-negative integer: milliseconds to sleep **once per stock quote batch** before HTTP fan-out (`src/api/http.rs`). `0`, unset, or invalid → no delay. See `docs/SPEC.md` §16. |
| `STOCKTERM_DEBUG_YAHOO_QUOTE` | Any build | Set to exactly `1` (no trimming; no other value enables it). When Yahoo **`yahoo_latest_quote`** falls back from **`v7/finance/quote`** to **`v8/finance/chart`**, one line is written to **stderr** with the symbol and reason (`empty_v7` or `v7_error`). See `docs/SPEC.md` §34. |
| `STOCKTERM_DEBUG_YAHOO_NEWS` | Any build | Set to exactly `1` (no trimming; no other value enables it). When Yahoo news is fetched, one **stderr** line per attempt (`search`, `rss`, `query2`) with outcome tokens such as `ok_items(n)`, `ok_empty`, `parse_mismatch`, or `err(…)`. See `docs/SPEC.md` §36. |

Run from the repo root, for example:

```bash
STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release
STOCKTERM_DEBUG_ALERT_NOTIFY=1 cargo run --release
STOCKTERM_DEBUG_YAHOO_QUOTE=1 cargo run --release 2> yahoo-quote.log
STOCKTERM_DEBUG_YAHOO_NEWS=1 cargo run --release 2> yahoo-news.log
```
