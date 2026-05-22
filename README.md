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
| `last_tab` | string or omitted | omitted | Last tab: `stock_view`, `portfolio`, `alerts`, `search`, `news`, `charts`, `settings`, `backtest`, `options` (Issue #19 / §22). |
| `backtest` | object | see below | Simulation capital, commission, slippage (Issue #25 / §47). |
| `backtest_strategy` | object | SMA 50/200 | Strategy kind and periods (Issue #25 / §47). |
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

#### `backtest` / `backtest_strategy` (Issue #25)

| Field | Default | Notes |
|-------|---------|--------|
| `backtest.initial_capital` | `10000` | Starting cash (USD). |
| `backtest.commission_per_trade` | `0` | Flat fee per buy/sell fill. |
| `backtest.slippage_bps` | `5` | Slippage in basis points on fill price. |
| `backtest_strategy.kind` | `sma_crossover` | `sma_crossover` or `rsi_mean_reversion`. |
| `backtest_strategy.sma_fast` / `sma_slow` | `50` / `200` | SMA crossover periods. |
| `backtest_strategy.rsi_period` | `14` | RSI length. |
| `backtest_strategy.rsi_oversold` / `rsi_overbought` | `30` / `70` | RSI thresholds. |

Edit capital/fees on **Settings** rows **7–9**. Strategy kind cycles on the **Backtest** tab with **`n`**.

### Symbols — equities, crypto, FX (Issue #23 / [`docs/SPEC.md`](docs/SPEC.md) §43)

Symbols are stored **uppercase** in `watchlist` and `portfolio` after normalization. On **Stock View**, type letters plus **`-`** and **`.`** (for example `BTC-USD`, `BRK.B`), then **Enter** to fetch.

| Provider | Equities | Crypto (examples) | FX (examples) |
|----------|----------|-------------------|---------------|
| **yahoo** (default) | `AAPL`, `BRK.B` | Spot crypto: **`BTC-USD`**, **`ETH-USD`** (Search → bitcoin / ethereum) | `EURUSD=X` |
| **polygon** | `AAPL` | **`BTC-USD`**, **`ETH-USD`** (wire `X:BTCUSD`, `X:ETHUSD`) | Not supported in v1 |

**Yahoo crypto symbols (important):**

- **Spot Bitcoin** on Yahoo is **`BTC-USD`** (cryptocurrency). Search → `bitcoin` → pick the **BTC-USD** row.
- Plain **`BTC`** on Yahoo is a **different instrument** (an ETF, not spot BTC). Quotes/charts for `BTC` are not spot-crypto prices.
- Type **`BTC-USD`** without spaces (`BTC - USD` can return HTTP **404** from Yahoo). The app compacts whitespace on input.
- **`BTCUSD`** (no hyphen) is **not** a valid Yahoo chart symbol (404).

Crypto and FX use the same quote and chart paths as equities when `provider` is **yahoo**. On **polygon**, hyphenated crypto pairs in the watchlist are translated to Polygon’s `X:…` wire symbols (see below).

**Settings provider (Issue #160 / §45):** On the **Settings** tab, row **4. Provider** — press **Enter** to toggle **yahoo** ↔ **polygon** (requires a Polygon API key to switch to **polygon**). Switching provider clears in-session **Kind** metadata so labels are not reused from the previous provider.

**Provider wire symbols (Issues #157 / #161 / §44–§45):** The app stores **normalized user** tickers in `watchlist` / `portfolio` (uppercase, compact, e.g. `BTC-USD`). HTTP requests map through a single resolver: Yahoo uses the same compact form; Polygon uses the same for US equities (`AAPL`). Polygon maps hyphenated crypto pairs to `X:…` wire symbols (e.g. `BTC-USD` → `X:BTCUSD`). **`BTCUSD`** without a hyphen is **not** auto-translated — use **`BTC-USD`** in the watchlist.

**Kind column (Issue #158 / §44):** When Yahoo Search or v7 quotes return **`quoteType`**, the **Kind** label uses that metadata (e.g. `BTC-USD` → **CRYPTO**, plain `BTC` ETF → **EQ**). Without metadata, hyphenated pairs (`BTC-USD`) still show **CRYPTO** via suffix rules; plain `BTC` is **EQ**, not spot crypto.

### Keymap (`keymap` field)

Optional JSON object: each key is a **chord** string, each value is an **`Action`** name in **PascalCase** (for example `"Quit"`, `"StockRowDown"`). Overrides replace the default binding for that action in every [`BindingLayer`](src/config/keymap.rs) where built-in defaults register it (for example portfolio row **↑/↓** while remove-confirm is armed — Issue #134 / [`docs/SPEC.md`](docs/SPEC.md) §25); see [`src/config/keymap.rs`](src/config/keymap.rs) for the full default table. **Issues #58 / #59 / §27:** On the **News** tab, default **`NewsEnter`** is **Enter** (open selected article URL in the browser) and **`NewsCopyUrl`** is **`c`** (copy URL to the clipboard). **Issue #136 / §26:** These stay **wildcard** (no per-letter `Action` rows): Stock View symbol letters and Search query characters. Explicit defaults cover portfolio / alert dialog **digits** and **`.`**, plus Settings edit buffer input: **`PortfolioDialogDigitOrDot`**, **`AlertDialogDigitOrDot`**, **`SettingsEditDigit`**, and **`SettingsEditSymbolChar`** (default-symbol row only for letters). **Issue #139 / §29 — alert add dialog:** **`AlertDialogSymbolChar`** (`c`–`z`, `-`), **`AlertDialogConditionAbove`** (`a`), **`AlertDialogConditionBelow`** (`b`); on **Symbol** focus, `a`/`b` still append **`A`/`B`** via the condition actions (Shift/Caps per §8). Remapping a condition key frees that chord for symbol typing when unbound (optional wildcard fallback). **Issue #137 / §28 — table filter:** **`StockFilterToggle`** / **`PortfolioFilterToggle`** enter filter mode on **Stock View** / **Portfolio**; while filter input is active, keys resolve on **`FilterInput`** only — **`FilterClear`**, **`FilterCommit`**, **`FilterBackspace`**, **`FilterSlash`**, and per-character **`FilterQueryChar`** (`char:0`–`9`, `char:a`–`z` defaults). Unmapped keys in filter mode are ignored (they do not reach watchlist/portfolio actions). Remapping a **`Filter*`** action onto a chord already used by another action on **`FilterInput`** (for example **`FilterClear`** → **`char:a`**) is rejected and the app falls back to the full built-in keymap (same as §24 duplicate-chord rules).

**Charts tab (Issue #21 / §46):** With the Charts tab focused, **`s`** toggles SMA(20), **`e`** EMA(20), **`r`** RSI(14) (sub-pane), **`m`** MACD 12/26/9 (sub-pane). SMA/EMA overlay the **line** chart (`c` to switch from candles). Indicator toggles are session-only (not saved in `~/.stockterm.json`).

**Backtest tab (Issue #25 / §47):** Load historical bars on **Charts** first ( **`4`** = Y1 recommended). **Backtest** tab: **`Enter`** or **`r`** run, **`n`** cycle strategy (SMA crossover ↔ RSI mean-reversion), **`x`** export `~/.stockterm/backtest_<symbol>_<ts>.{csv,json}`, **`j`**/**`k`** scroll trades. Long-only, fills at bar **close**, force-flat on the last bar.

**Options tab (Issue #22 / §48, Polygon #167 / §50):** **Yahoo** (`provider: "yahoo"`) or **Polygon** (`provider: "polygon"` + `api_key` / `STOCKTERM_API_KEY`; requires Polygon **Options Starter** plan or higher). Set a symbol on **Stock View** (e.g. **`AAPL`**), open **Options**: **`r`** refresh chain, **`[`** / **`]`** or **`h`** / **`l`** cycle expirations, **`j`** / **`k`** move strike highlight (calls + puts), **`g`** toggle Greeks columns. Symbols without listed options show **No options available**.

**Chord grammar** (ASCII, case-insensitive except `char:` payload):

- Combine with **`+`**: `shift`, `ctrl` (or `control`), `alt` — e.g. `ctrl+e`, `shift+d`.
- **Named keys**: `tab`, `backtab`, `esc`, `enter`, `backspace`, `up`, `down`, `left`, `right`, `pageup`, `pagedown`, `colon`, `semicolon`, `slash`, **`plus`** (the `+` key — use this token because a raw `+` would split the parser), **`minus`**.
- **One ASCII character**: either a single visible character (`q`, `1`, `/`, …) or `char:x` for a single character `x` (useful when `x` is `:` / `;` / etc.).
- **Invalid** `keymap` (unknown action name, unknown chord, or conflicting chord assignment): startup shows a line starting with **`keymap:`** and the built-in defaults are used for the whole map.
- **Shift+Tab:** terminals may send `BackTab` with `SHIFT`, plain `BackTab`, or `Tab` with `SHIFT` only. StockTerm tries those variants when resolving `Action::GlobalBackTab` and dialog `BackTab` bindings (see `chord_lookup_candidates` in [`src/config/keymap.rs`](src/config/keymap.rs)).
- **Global quit / Tab (Issue #51 / §42):** **`q`** and **`Q`** (Shift) quit from any tab; **Ctrl/Alt/Meta+Q** does not. **Tab** / **Shift+Tab** switch tabs only when modifiers pass the same “plain” rules as letter keys (no Ctrl/Alt/Meta on Tab). Custom `keymap` quit chords (for example `"colon": "Quit"`) still use exact chord match.

**Example** — bind quit to `:` (colon key):

```json
"keymap": {
  "colon": "Quit"
}
```

To discover exact **`Action`** names, see the `Action` enum in [`src/config/keymap.rs`](src/config/keymap.rs) (serde renames match PascalCase JSON).

The Polygon **`api_key`** is stored **in plaintext** inside **`~/.stockterm.json`**. If **`api_key`** is empty, StockTerm uses a non-empty **`STOCKTERM_API_KEY`** environment variable at request time instead (resolution: [`Config::effective_api_key`](https://github.com/FelipeMorandini/stockterm/blob/main/src/config/config.rs)); the env value is **not** copied into the JSON file on load or save (Issue #28 / SPEC §42.2). Treat the config file like a secret: use restrictive file permissions where your OS supports them (for example **`chmod 600 ~/.stockterm.json`** on Unix), do not commit real keys to git, and avoid pasting keys into logs or screenshots. Yahoo mode does not require a key.

Provider selection and HTTP behavior are specified in [`docs/SPEC.md`](docs/SPEC.md) (§9 and §31).

### Terminal lifecycle

StockTerm’s CLI ([`src/main.rs`](src/main.rs)) owns terminal mode on the **main** thread:

```
main thread:  enable_raw_mode → EnterAlternateScreen → App::run → disable_raw_mode → LeaveAlternateScreen
event thread: poll/read keys + ticks only (no terminal mode changes)
```

The crossterm bridge ([`src/app/event.rs`](src/app/event.rs)) stops when `App::run` drops its event sender and joins the thread (bounded wait; override with `STOCKTERM_EVENT_JOIN_MS`, default **2000** ms). Embedders that call `App::run` more than once per process should reset `App` state between sessions.

### Async channels (back-pressure)

`App::run` uses **`tokio::sync::mpsc::unbounded_channel`** for UI events, `FetchDone`, `InflightRecovery`, and `UrlOpDone`. A single consumer drains each queue every `select!` iteration; **bounded** channels are deferred until profiling or embedder needs justify `try_send` with drop/coalesce semantics. **Input events are never dropped** (unbounded `Event` channel). See [`docs/SPEC.md`](docs/SPEC.md) §39.3 (Issue #87).

## Developer / debug

These environment variables are supported for local diagnosis. Any other `STOCKTERM_DEBUG_*` name is **not** supported unless it appears here or in `docs/SPEC.md`.

| Variable | When it applies | Behavior |
|----------|-----------------|----------|
| _(none)_ | Default HTTP stack | **5 s** connect + **10 s** request timeout on the shared `reqwest::Client` ([`docs/SPEC.md`](docs/SPEC.md) §19 / Issue #18; `src/api/http.rs`). Startup calls `init_shared_client()` before the TUI; failure prints to stderr and exits with code **1**. |
| `STOCKTERM_LOG_DIR` | Any build | Directory for `stockterm.log` (default: `{cache_dir}/stockterm/logs`). Supports `~/…` paths. See `docs/SPEC.md` §38. |
| `STOCKTERM_LOG_STDERR` | Any build | Set to exactly `1` to mirror **WARN+** logs to stderr in addition to the log file (default: off — keeps the TUI clean). |
| `STOCKTERM_EVENT_JOIN_MS` | Any build | Milliseconds to wait for the crossterm event thread after `App::run` exits (default **2000**). See §39.1 / [`src/app/event.rs`](src/app/event.rs). |
| `STOCKTERM_INFLIGHT_STALE_SECS` | Tests / diagnostics | Seconds before the main loop clears a stuck `*_refresh_inflight` flag when both fetch and recovery sends failed (default **120**). See §39.2. |
| `RUST_LOG` | Any build | Standard `tracing` filter (e.g. `stockterm=debug`). Default: `warn,stockterm=warn`. |
| _(tests)_ | Authors writing **`#[tokio::test(start_paused = true)]`** + **`reqwest`** | Paused **`tokio::time::advance`** can fire **`reqwest`**’s request **`timeout`** while a **`GET`** is still in flight → spurious **`Timeout`**. Prefer wall-clock waits for **`Retry-After`** assertions or an isolated **`Client`** with a short timeout for stall tests — [`docs/SPEC.md`](docs/SPEC.md) §19.8 / §19.13.3. |
| `STOCKTERM_DEBUG_ALERT_NOTIFY` | Build with the default **`desktop-notify`** Cargo feature | Set to exactly `1` (no trimming; no other value enables it). After `notify-rust` `Notification::show()`, StockTerm may `eprintln!` the `Result` to stderr on the **coalesced** desktop notify path (including `Ok(())`) so you can confirm the call completed. |
| `STOCKTERM_DEBUG_HTTP_DELAY_MS` | Any build | Non-negative integer: milliseconds to sleep **once per stock quote batch** before HTTP fan-out (`src/api/http.rs`). `0`, unset, or invalid → no delay. Capped at **120000** ms. See `docs/SPEC.md` §16 / §38. |
| `STOCKTERM_DEBUG_YAHOO_QUOTE` | Any build | Set to exactly `1` (no trimming; no other value enables it). When Yahoo **`yahoo_latest_quote`** falls back from **`v7/finance/quote`** to **`v8/finance/chart`**, one line is written to **stderr** with the symbol and reason (`empty_v7` or `v7_error`). See `docs/SPEC.md` §34. |
| `STOCKTERM_DEBUG_YAHOO_NEWS` | Any build | Set to exactly `1` (no trimming; no other value enables it). When Yahoo news is fetched, one **stderr** line per attempt (`search`, `rss`, `query2`) with outcome tokens such as `ok_items(n)`, `ok_empty`, `parse_mismatch`, or `err(…)`. See `docs/SPEC.md` §36. |
| `STOCKTERM_DEBUG_YAHOO_OPTIONS` | Any build | Set to exactly `1`. Logs parsed options chain summary (expiration count, strikes) via **`tracing`** at **info** level — not stderr. See `docs/SPEC.md` §48. |
| `STOCKTERM_DEBUG_POLYGON_OPTIONS` | Any build | Set to exactly `1`. Logs Polygon options fetch summary (expiration count, selected date, row counts) via **`tracing`** at **info** level — not stderr. See `docs/SPEC.md` §50. |

Run from the repo root, for example:

```bash
STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release
STOCKTERM_DEBUG_ALERT_NOTIFY=1 cargo run --release
STOCKTERM_DEBUG_YAHOO_QUOTE=1 cargo run --release 2> yahoo-quote.log
STOCKTERM_DEBUG_YAHOO_NEWS=1 cargo run --release 2> yahoo-news.log
```
