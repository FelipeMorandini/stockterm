# StockTerm — Product Roadmap

_A living gap analysis between the current codebase and the StockTerm product
requirements. Source of truth for the next round of `docs/SPEC.md` work._

Last updated: 2026-05-25 — **§67 ([#79](https://github.com/FelipeMorandini/stockterm/issues/79)):** Unicode ticker normalization — [`docs/SPEC.md`](SPEC.md) §67; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#79** (sign-off **2026-05-25**; **PR:** pending). Follow-up: [#204](https://github.com/FelipeMorandini/stockterm/issues/204) (config canonicalization on load). **§66 ([#192](https://github.com/FelipeMorandini/stockterm/issues/192)):** `Config::save` silent I/O drop elimination — [`docs/SPEC.md`](SPEC.md) §66; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#192** (sign-off **2026-05-25**). **§64 ([#199](https://github.com/FelipeMorandini/stockterm/issues/199)) + §65 ([#200](https://github.com/FelipeMorandini/stockterm/issues/200)):** §63 follow-ups shipped — precompute candle layout off the render path + normalize `HistoricalData::t` to Unix ms at provider ingest — [`docs/SPEC.md`](SPEC.md) §64–§65; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#199, #200** (sign-off **2026-05-25**; **PR:** [#202](https://github.com/FelipeMorandini/stockterm/pull/202)). **§63 ([#190](https://github.com/FelipeMorandini/stockterm/issues/190)):** Charts candlestick visual density polish — [`docs/SPEC.md`](SPEC.md) §63; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#190** (sign-off **2026-05-25**; **PR:** [#201](https://github.com/FelipeMorandini/stockterm/pull/201)). **§61 ([#195](https://github.com/FelipeMorandini/stockterm/issues/195)) + §62 ([#196](https://github.com/FelipeMorandini/stockterm/issues/196)):** Options lazy `ThemeStamp` compare + theme audit / commit hook for watchlist/portfolio/alerts — [`docs/SPEC.md`](SPEC.md) §61–§62; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#195, #196** (sign-off **2026-05-24**; **PR:** [#198](https://github.com/FelipeMorandini/stockterm/pull/198)). **§59 ([#189](https://github.com/FelipeMorandini/stockterm/issues/189)) + §60 ([#193](https://github.com/FelipeMorandini/stockterm/issues/193)):** expand `insta` snapshots (status bar, portfolio dialog, Options) + blocking `cargo fmt` CI — [`docs/SPEC.md`](SPEC.md) §59–§60; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#189, #193** (sign-off **2026-05-24**; **PR:** [#197](https://github.com/FelipeMorandini/stockterm/pull/197)). **§57 ([#181](https://github.com/FelipeMorandini/stockterm/issues/181)) + §58 ([#184](https://github.com/FelipeMorandini/stockterm/issues/184)):** GitHub Actions CI + `insta` error-overlay `TestBackend` snapshots — [`docs/SPEC.md`](SPEC.md) §57–§58; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#181, #184** (sign-off **2026-05-23**; **PR:** [#188](https://github.com/FelipeMorandini/stockterm/pull/188)). **§56 ([#183](https://github.com/FelipeMorandini/stockterm/issues/183)):** Options display cache invalidation on theme commit — [`docs/SPEC.md`](SPEC.md) §56; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#183** (sign-off **2026-05-23**; **PR:** [#187](https://github.com/FelipeMorandini/stockterm/pull/187)). **§55 ([#182](https://github.com/FelipeMorandini/stockterm/issues/182)):** Portfolio row edit UI for existing holdings — [`docs/SPEC.md`](SPEC.md) §55; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#182** (sign-off **2026-05-23**). **§54 ([#180](https://github.com/FelipeMorandini/stockterm/issues/180)):** Persist Charts `time_range` + `chart_mode` in `~/.stockterm.json` — [`docs/SPEC.md`](SPEC.md) §54; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#180** (sign-off **2026-05-23**; **PR:** [#185](https://github.com/FelipeMorandini/stockterm/pull/185)). **Backlog sync (2026-05-23):** Filed follow-on issues [#189](https://github.com/FelipeMorandini/stockterm/issues/189)–[#196](https://github.com/FelipeMorandini/stockterm/issues/196) for §5 gaps — see [§2.1](#21-active-backlog-github). **Earlier (2026-05-22):** §53 shipped (**PR:** [#179](https://github.com/FelipeMorandini/stockterm/pull/179)); duplicate closed [#174](https://github.com/FelipeMorandini/stockterm/issues/174) / [#175](https://github.com/FelipeMorandini/stockterm/issues/175) annotated as duplicates of [#176](https://github.com/FelipeMorandini/stockterm/issues/176) / [#177](https://github.com/FelipeMorandini/stockterm/issues/177). **§53 ([#176](https://github.com/FelipeMorandini/stockterm/issues/176), [#177](https://github.com/FelipeMorandini/stockterm/issues/177)):** §52 follow-ups — Polygon historical `next_url` pagination + Options zero-clone draw — [`docs/SPEC.md`](SPEC.md) §53; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#176, #177** (sign-off **2026-05-22**; **PR:** [#179](https://github.com/FelipeMorandini/stockterm/pull/179)). **§52 ([#65](https://github.com/FelipeMorandini/stockterm/issues/65)):** Polygon historical `limit` cap + partial-chart messaging + **Options** tab polish (Vol/OI, strike scroll, draw hygiene) — [`docs/SPEC.md`](SPEC.md) §52; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#65** + Options polish (sign-off **2026-05-22**; **PR:** [#178](https://github.com/FelipeMorandini/stockterm/pull/178)). **§51 ([#171](https://github.com/FelipeMorandini/stockterm/issues/171)):** Polygon options expiration list session cache (§50 follow-up; builds on shipped [#168](https://github.com/FelipeMorandini/stockterm/issues/168) slice cache) — [`docs/SPEC.md`](SPEC.md) §51; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#171** (sign-off **2026-05-22**; **PR:** [#173](https://github.com/FelipeMorandini/stockterm/pull/173)). **§50 ([#167](https://github.com/FelipeMorandini/stockterm/issues/167)):** Polygon.io options chain provider — `v3/snapshot/options` + `v3/reference/options/contracts`, **Options** tab parity with Yahoo — [`docs/SPEC.md`](SPEC.md) §50; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#167** (sign-off **2026-05-22**; **PR:** [#172](https://github.com/FelipeMorandini/stockterm/pull/172)). **§49 ([#165](https://github.com/FelipeMorandini/stockterm/issues/165), [#168](https://github.com/FelipeMorandini/stockterm/issues/168)):** §47 / §48 follow-ons — backtest golden metric vectors + Yahoo options expiration slice cache — [`docs/SPEC.md`](SPEC.md) §49; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#165, #168** (sign-off **2026-05-21**; **PR:** [#170](https://github.com/FelipeMorandini/stockterm/pull/170)). **§48 ([#22](https://github.com/FelipeMorandini/stockterm/issues/22)):** Options chains — Yahoo `v7/finance/options`, **Options** tab, calls/puts tables, expiration selector, Greeks toggle — [`docs/SPEC.md`](SPEC.md) §48; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#22** (sign-off **2026-05-21**; **PR:** [#169](https://github.com/FelipeMorandini/stockterm/pull/169)). **§47 ([#25](https://github.com/FelipeMorandini/stockterm/issues/25)):** Backtesting — strategy engine, **Backtest** tab, reference strategies, export — [`docs/SPEC.md`](SPEC.md) §47; **PR:** [#166](https://github.com/FelipeMorandini/stockterm/pull/166). **§46 ([#21](https://github.com/FelipeMorandini/stockterm/issues/21)):** Technical indicators (SMA / EMA / RSI / MACD) — [`docs/SPEC.md`](SPEC.md) §46; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#21** (sign-off **2026-05-21**; **PR:** [#164](https://github.com/FelipeMorandini/stockterm/pull/164)). **§45 ([#160](https://github.com/FelipeMorandini/stockterm/issues/160), [#161](https://github.com/FelipeMorandini/stockterm/issues/161)):** §44 follow-ons — clear **`symbol_kind_cache`** on Settings provider toggle + Polygon crypto **`X:`** wire in **`resolve_provider_symbol`** — [`docs/SPEC.md`](SPEC.md) §45; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#160, #161** (sign-off **2026-05-20**; **PR:** [#163](https://github.com/FelipeMorandini/stockterm/pull/163)). **§44 ([#157](https://github.com/FelipeMorandini/stockterm/issues/157), [#158](https://github.com/FelipeMorandini/stockterm/issues/158)):** Provider-aware symbol resolver + **`SymbolKind`** from Yahoo **`quoteType`** — [`docs/SPEC.md`](SPEC.md) §44; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#157, #158** (sign-off **2026-05-20**; **PR:** [#162](https://github.com/FelipeMorandini/stockterm/pull/162)). **§43 ([#23](https://github.com/FelipeMorandini/stockterm/issues/23)):** Cryptocurrency quotes — Yahoo **`BTC-USD`** symbols, **`SymbolKind`** UI, adaptive **`format_usd_price`**, Stock View hyphen entry, §43.13 quote-cache alignment — [`docs/SPEC.md`](SPEC.md) §43; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#23** (sign-off **2026-05-19**; **PR:** [#159](https://github.com/FelipeMorandini/stockterm/pull/159)). **§42 ([#51](https://github.com/FelipeMorandini/stockterm/issues/51), [#28](https://github.com/FelipeMorandini/stockterm/issues/28)):** Global **`q`/`Q`** quit via **`should_global_quit`**, documented Tab meta policy, **`STOCKTERM_API_KEY`** runtime overlay (no merge on load) — [`docs/SPEC.md`](SPEC.md) §42; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#51, #28** (sign-off **2026-05-19**; **PR:** [#156](https://github.com/FelipeMorandini/stockterm/pull/156)). **§41 ([#32](https://github.com/FelipeMorandini/stockterm/issues/32), [#33](https://github.com/FelipeMorandini/stockterm/issues/33), [#55](https://github.com/FelipeMorandini/stockterm/issues/55)):** **`get_current_price`** symbol/ticker alignment, **`ProviderError`** **`thiserror`** migration, API error taxonomy audit — [`docs/SPEC.md`](SPEC.md) §41; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#32, #33, #55** (sign-off **2026-05-19**; **PR:** [#155](https://github.com/FelipeMorandini/stockterm/pull/155)). **§40 ([#36](https://github.com/FelipeMorandini/stockterm/issues/36), [#56](https://github.com/FelipeMorandini/stockterm/issues/56), [#106](https://github.com/FelipeMorandini/stockterm/issues/106)):** Charts timestamp safety + tests, quote **`Semaphore`** acquire failures, §18.15 post-audit **`centered_rect`** / notify **`body`** — [`docs/SPEC.md`](SPEC.md) §40; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#36, #56, #106** (sign-off **2026-05-19**; **PR:** [#154](https://github.com/FelipeMorandini/stockterm/pull/154)). **§39 ([#108](https://github.com/FelipeMorandini/stockterm/issues/108), [#78](https://github.com/FelipeMorandini/stockterm/issues/78), [#87](https://github.com/FelipeMorandini/stockterm/issues/87)):** Event-thread clean shutdown, stale-inflight watchdog when both **`FetchDone`** and **`InflightRecovery`** sends fail, documented **`mpsc`** back-pressure policy — [`docs/SPEC.md`](SPEC.md) §39; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#108, #78, #87** (sign-off **2026-05-19**; **PR:** [#153](https://github.com/FelipeMorandini/stockterm/pull/153)). **§38 ([#76](https://github.com/FelipeMorandini/stockterm/issues/76), [#85](https://github.com/FelipeMorandini/stockterm/issues/85), [#86](https://github.com/FelipeMorandini/stockterm/issues/86), [#117](https://github.com/FelipeMorandini/stockterm/issues/117), [#118](https://github.com/FelipeMorandini/stockterm/issues/118)):** Async/HTTP reliability tail — **`tracing`** fetch drops, cap debug HTTP delay, dev panic logging, **408** retry, structured client init — [`docs/SPEC.md`](SPEC.md) §38; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#76, #85, #86, #117, #118** (sign-off **2026-05-18**; **PR:** [#152](https://github.com/FelipeMorandini/stockterm/pull/152)). **§37 ([#81](https://github.com/FelipeMorandini/stockterm/issues/81), [#82](https://github.com/FelipeMorandini/stockterm/issues/82), [#83](https://github.com/FelipeMorandini/stockterm/issues/83)):** Stock View narrow status, plain-**Tab** portfolio dialog, **`add_to_portfolio`** contract docs — [`docs/SPEC.md`](SPEC.md) §37; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#81–#83** (sign-off **2026-05-18**; **PR:** [#151](https://github.com/FelipeMorandini/stockterm/pull/151)). **§36 ([#54](https://github.com/FelipeMorandini/stockterm/issues/54)):** Yahoo news — resilient **`query2`** parsing + **`STOCKTERM_DEBUG_YAHOO_NEWS`** attempt logging — [`docs/SPEC.md`](SPEC.md) §36; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#54** (sign-off **2026-05-18**; **PR:** [#150](https://github.com/FelipeMorandini/stockterm/pull/150)). **§35 ([#4](https://github.com/FelipeMorandini/stockterm/issues/4)):** **`Config.refresh_rate`** vs UI tick — [`docs/SPEC.md`](SPEC.md) §35; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#4** (sign-off **2026-05-18**; **PR:** [#149](https://github.com/FelipeMorandini/stockterm/pull/149)). **§34 ([#90](https://github.com/FelipeMorandini/stockterm/issues/90), [#91](https://github.com/FelipeMorandini/stockterm/issues/91)):** Yahoo quote adapter — **`STOCKTERM_DEBUG_YAHOO_QUOTE`** v7→v8 stderr + **`v7_envelope_to_ticker`** symbol-aware row pick — [`docs/SPEC.md`](SPEC.md) §34; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#90, #91** (sign-off **2026-05-18**; **PR:** [#148](https://github.com/FelipeMorandini/stockterm/pull/148)). **§33 ([#60](https://github.com/FelipeMorandini/stockterm/issues/60)):** Search **Esc** must not clear cross-tab **`active_runtime_error`** — [`docs/SPEC.md`](SPEC.md) §33; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#60** (sign-off **2026-05-18**; **PR:** [#147](https://github.com/FelipeMorandini/stockterm/pull/147)). **§32 ([#89](https://github.com/FelipeMorandini/stockterm/issues/89)):** Yahoo **`yahoo_latest_quote`** **v7→v8** orchestration **`wiremock`** integration test — [`docs/SPEC.md`](SPEC.md) §32; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#89** (sign-off **2026-05-18**; **PR:** [#146](https://github.com/FelipeMorandini/stockterm/pull/146)). **§31 ([#15](https://github.com/FelipeMorandini/stockterm/issues/15)):** **Layout / widget visibility** (`Config.layout`, shell + pane splits, Settings presets) — [`docs/SPEC.md`](SPEC.md) §31; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#15** (sign-off **2026-05-17**; **PR:** [#145](https://github.com/FelipeMorandini/stockterm/pull/145)). **§30 ([#138](https://github.com/FelipeMorandini/stockterm/issues/138)):** Keymap **compile-time default chord table** (remove runtime `Box::leak`) — [`docs/SPEC.md`](SPEC.md) §30; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#138** (sign-off **2026-05-17**; **PR:** [#144](https://github.com/FelipeMorandini/stockterm/pull/144)). **§29 ([#139](https://github.com/FelipeMorandini/stockterm/issues/139)):** Keymap **phase 3** — explicit alert dialog **symbol** + **condition** actions — [`docs/SPEC.md`](SPEC.md) §29; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#139** (sign-off **2026-05-18**; **PR:** [#143](https://github.com/FelipeMorandini/stockterm/pull/143)). **§28 ([#137](https://github.com/FelipeMorandini/stockterm/issues/137)):** Keymap **remappable filter-input mode** (`BindingLayer::FilterInput`) — [`docs/SPEC.md`](SPEC.md) §28; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#137** (sign-off **2026-05-18**; **PR:** [#142](https://github.com/FelipeMorandini/stockterm/pull/142)). **§27 ([#58](https://github.com/FelipeMorandini/stockterm/issues/58), [#59](https://github.com/FelipeMorandini/stockterm/issues/59)):** News **clipboard copy** + **non-blocking** URL open (`http`/`https` allowlist) — [`docs/SPEC.md`](SPEC.md) §27; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#58, #59** (sign-off **2026-05-18**; **PR:** [#141](https://github.com/FelipeMorandini/stockterm/pull/141)). **§26 ([#136](https://github.com/FelipeMorandini/stockterm/issues/136)):** Keymap **phase 2** (symbol buffers + modal digit/symbol entry) — [`docs/SPEC.md`](SPEC.md) §26; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#136** (sign-off **2026-05-18**; **PR:** [#140](https://github.com/FelipeMorandini/stockterm/pull/140)). **§25 ([#134](https://github.com/FelipeMorandini/stockterm/issues/134)):** Keymap **per-context overlay propagation** — [`docs/SPEC.md`](SPEC.md) §25; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#134** (sign-off **2026-05-18**; **implementation shipped** 2026-05-15). **§24 ([#13](https://github.com/FelipeMorandini/stockterm/issues/13)):** Configurable **keymap** — [`docs/SPEC.md`](SPEC.md) §24; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#13** (**sign-off **2026-05-18**; **PR:** [#133](https://github.com/FelipeMorandini/stockterm/pull/133)). **§23 ([#16](https://github.com/FelipeMorandini/stockterm/issues/16)):** Portfolio + Stock View **substring filter** — [`docs/SPEC.md`](SPEC.md) §23; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#16** (sign-off **2026-05-18**); **PR:** [#132](https://github.com/FelipeMorandini/stockterm/pull/132). **Earlier (2026-05-13):** **§22.7 ([#34](https://github.com/FelipeMorandini/stockterm/issues/34), [#35](https://github.com/FelipeMorandini/stockterm/issues/35), [#129](https://github.com/FelipeMorandini/stockterm/issues/129)):** README **Security — API keys**; `load_config_from_path` + corrupt-json test (no `HOME` mutation); `App::run` event channel `None` best-effort session save; **400 ms** debounced `persist_session_to_disk` + tick flush — see [`docs/SPEC.md`](SPEC.md) §22.7 / §22.9 and [`docs/QA_PLAN.md`](QA_PLAN.md) **Issues #34, #35, #40, #129** (**manual QA sign-off **2026-05-18**; **PR:** [#131](https://github.com/FelipeMorandini/stockterm/pull/131)). **Earlier same day:** **§9.15 ([#53](https://github.com/FelipeMorandini/stockterm/issues/53))** — Yahoo watchlist quote batching: primary **`v7/finance/quote`** per URL chunk + per-symbol **`yahoo_latest_quote`** when batched **`v7`** is rejected (e.g. HTTP **401**) or unusable; Polygon **`run_stock_quote_batch`** unchanged — see [`docs/SPEC.md`](SPEC.md) §9.15.9 and [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#53** (**manual QA sign-off 2026-05-13**; **PR:** [#127](https://github.com/FelipeMorandini/stockterm/pull/127)). **Earlier same day:** **§21 ([#14](https://github.com/FelipeMorandini/stockterm/issues/14))** — [PR #126](https://github.com/FelipeMorandini/stockterm/pull/126): theme presets + JSON overrides + `ResolvedTheme` / `theme.canvas()` draw paths, Settings row **3** commit + live preview, candlestick inner fill — see [`docs/SPEC.md`](SPEC.md) §21.11 and [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#14** (**manual QA sign-off 2026-05-13**). **Earlier (2026-05-12):** **§20.15 ([#120](https://github.com/FelipeMorandini/stockterm/issues/120)–[#123](https://github.com/FelipeMorandini/stockterm/issues/123))** — [PR #125](https://github.com/FelipeMorandini/stockterm/pull/125): error-log overlay polish (single-source visible-row count + `clamp_error_log_scroll`, scroll-read-only `draw_error_log_overlay`, function-entry clamp guarding resize-larger, `q`-quits-from-overlay) and `ProviderError::Clone` `Json → ApiMessage` Rustdoc (see [`docs/SPEC.md`](SPEC.md) §20.15, [`docs/QA_PLAN.md`](QA_PLAN.md) "Issues #120, #121, #122, #123" — **manual QA sign-off **2026-05-18**). **Earlier same day:** **§18.14 ([#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98))** — [PR #105](https://github.com/FelipeMorandini/stockterm/pull/105): alerts save-failure banner + quote-batch **`save_alerts`** retry, one coalesced desktop toast per crossing batch, sanitized notify `body` (see [`docs/SPEC.md`](SPEC.md) §18.14.9, [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#96–#98** — **manual QA sign-off **2026-05-18**). **Shipped same day:** **§18.13 ([#93](https://github.com/FelipeMorandini/stockterm/issues/93)–[#95](https://github.com/FelipeMorandini/stockterm/issues/95))** [PR #102](https://github.com/FelipeMorandini/stockterm/pull/102) — shared `app::layout::centered_rect`, alert add dialog **Condition** **←/→** keys, optional **`STOCKTERM_DEBUG_ALERT_NOTIFY`** stderr for `Notification::show()` (§18.13.8). **§18.15** ([#100](https://github.com/FelipeMorandini/stockterm/issues/100), [#101](https://github.com/FelipeMorandini/stockterm/issues/101), [#104](https://github.com/FelipeMorandini/stockterm/issues/104)): `centered_rect` **`debug_assert!`**, root **`README.md`** debug env table, coalesced notify **`body`** UTF-8 byte cap — see [`docs/SPEC.md`](SPEC.md) §18.15.8; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#100–#104** (signed 2026-05-12); **PR:** [#107](https://github.com/FelipeMorandini/stockterm/pull/107). **§22 ([#19](https://github.com/FelipeMorandini/stockterm/issues/19), [#103](https://github.com/FelipeMorandini/stockterm/issues/103)):** `last_tab` / `last_symbol`, README `~/.stockterm.json` table, `try_save_config_with_session`, merged alerts-save vs quote-batch errors — [`docs/SPEC.md`](SPEC.md) §22; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) **Issues #19, #103** (sign-off **2026-05-18**); **PR:** [#130](https://github.com/FelipeMorandini/stockterm/pull/130). **§40 ([#36](https://github.com/FelipeMorandini/stockterm/issues/36), [#56](https://github.com/FelipeMorandini/stockterm/issues/56), [#106](https://github.com/FelipeMorandini/stockterm/issues/106))** — shipped; see [`docs/SPEC.md`](SPEC.md) §40.10. **§19 ([#18](https://github.com/FelipeMorandini/stockterm/issues/18)):** [PR #115](https://github.com/FelipeMorandini/stockterm/pull/115) — HTTP timeouts, 429/`Retry-After`, backoff, error snippets (**manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue #18). **§19.13** ([#110](https://github.com/FelipeMorandini/stockterm/issues/110)–[#114](https://github.com/FelipeMorandini/stockterm/issues/114), [#116](https://github.com/FelipeMorandini/stockterm/issues/116)) — [PR #128](https://github.com/FelipeMorandini/stockterm/pull/128): bounded error-body drain, `Retry-After` cap + HTTP-date normalization, `ProviderError` `Debug` redaction, rate-limit `Display` / status hint, docs — see [`docs/SPEC.md`](SPEC.md) §19.13.7 and [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#110–#116** (**manual QA sign-off **2026-05-18**). Post-audit tail **[#117](https://github.com/FelipeMorandini/stockterm/issues/117)–[#118](https://github.com/FelipeMorandini/stockterm/issues/118)** not in #128. **§20 ([#20](https://github.com/FelipeMorandini/stockterm/issues/20)):** Error UX shipped in-tree — [`docs/SPEC.md`](SPEC.md) §20; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue #20 (sign-off table). **PR:** [#124](https://github.com/FelipeMorandini/stockterm/pull/124). Follow-ups [#120](https://github.com/FelipeMorandini/stockterm/issues/120)–[#123](https://github.com/FelipeMorandini/stockterm/issues/123). **Earlier:** **Alerts (#10 / #42)** [PR #99](https://github.com/FelipeMorandini/stockterm/pull/99) / §18; Issue #2 [PR #92](https://github.com/FelipeMorandini/stockterm/pull/92) / §17; scratch [#89](https://github.com/FelipeMorandini/stockterm/issues/89)–[#91](https://github.com/FelipeMorandini/stockterm/issues/91); §16 [PR #88](https://github.com/FelipeMorandini/stockterm/pull/88); audit [#85](https://github.com/FelipeMorandini/stockterm/issues/85)–[#87](https://github.com/FelipeMorandini/stockterm/issues/87); §15 (#43, #49, #50, #67, #69); [#81](https://github.com/FelipeMorandini/stockterm/issues/81)–[#83](https://github.com/FelipeMorandini/stockterm/issues/83); charts [#76](https://github.com/FelipeMorandini/stockterm/issues/76)–[#79](https://github.com/FelipeMorandini/stockterm/issues/79).

---

## 1. Project Snapshot

**StockTerm** is a Rust-based, terminal UI (TUI) stock-tracking application.

Stack (from `Cargo.toml`):

| Concern              | Crate / Version                 |
| -------------------- | ------------------------------- |
| Async runtime        | `tokio = "1"` (full features)   |
| HTTP client          | `reqwest = "0.11"` (json, rustls-tls) |
| TUI framework        | `ratatui = "0.26.2"`            |
| Terminal backend     | `crossterm = "0.27.0"`          |
| Serialization        | `serde = "1"` + `serde_json = "1"` |
| CLI parsing          | `clap = "4"` (derive)           |
| Time / dates         | `chrono = "0.4.40"`             |
| Config dirs          | `dirs = "6.0.0"`                |
| Edition              | `2021`                          |

Crate layout (from `src/`):

- `main.rs` — terminal bootstrap (raw mode, alt screen, `App::run`).
- `lib.rs` — re-exports `app`, `api`, `config`, `models`.
- `api/http.rs` — shared **`reqwest::Client`** (timeouts); **`api/http_fetch.rs`** / **`api/retry.rs`** — Issue #18 GET + **`Retry-After`** / backoff (**§19**).
- `api/polygon.rs`, `api/yahoo.rs` — **`MarketDataProvider`** implementations.
- `app/` — `app.rs` (state machine), `ui.rs`, `event.rs`, `handlers.rs`,
  `charts.rs`, `layout.rs` (modal `centered_rect`), `portfolio.rs`, `alerts.rs`.
- `config/config.rs` — JSON-backed config at `~/.stockterm.json`.
- `models/` — `ticker`, `historical`, `search`, `news`, `portfolio`, `alerts`.
- `tests/` — JSON fixtures for provider tests; `wiremock` suites live in `src/` `#[cfg(test)]` blocks (no top-level `tests/*.rs` integration crate yet).

See `docs/SPEC.md`, `docs/QA_PLAN.md`, and this roadmap for product/engineering docs.

---

## 2. GitHub Issues

Queried via the GitHub MCP `list_issues` tool against
`FelipeMorandini/stockterm` (no state filter, both `OPEN` and `CLOSED`).

- Issues are tracked on GitHub (`FelipeMorandini/stockterm`); M0 was Issue **#1**.
  Tech-debt follow-ups from the ship phase are filed as separate issues.

This roadmap remains the de-facto starting backlog. Actionable open work is in [§2.1](#21-active-backlog-github); [§6](#6-recommended-next-milestones) preserves milestone history.

---

## 2.1 Active backlog (GitHub)

All currently **OPEN** `roadmap`-labelled issues on `FelipeMorandini/stockterm`, verified 2026-05-24. Closed issues are intentionally omitted — see commit history / per-section `Shipped` lines for what has landed.

### Actionable

_None at this time — all roadmap-labelled issues are either shipped or deferred (see below)._

**Recently shipped:** [#79](https://github.com/FelipeMorandini/stockterm/issues/79) / **§67** — Unicode `symbols_equivalent` + NFC `normalize_symbol` (sign-off **2026-05-25**; **PR:** pending). [#192](https://github.com/FelipeMorandini/stockterm/issues/192) / **§66** — `Config::save` deprecation + `persist_config_*` helpers (code + manual QA sign-off **2026-05-25**). [#199](https://github.com/FelipeMorandini/stockterm/issues/199) / **§64**, [#200](https://github.com/FelipeMorandini/stockterm/issues/200) / **§65** — sign-off **2026-05-25**; **PR:** [#202](https://github.com/FelipeMorandini/stockterm/pull/202).

### Optional / deferred

| # | Title | Scope (one line) |
|---|-------|------------------|
| [#191](https://github.com/FelipeMorandini/stockterm/issues/191) | Optional CancellationToken for quote batches | Only if overlapping batches become a product requirement — **§68** (SPEC + QA planned 2026-05-25). |
| [#194](https://github.com/FelipeMorandini/stockterm/issues/194) | Filters: saved / regex mode | M6 follow-on beyond §23 substring filter. |
| [#204](https://github.com/FelipeMorandini/stockterm/issues/204) | Config: canonicalize symbols on load | §67 follow-up — migrate legacy mixed-case `~/.stockterm.json` rows. |
| [#68](https://github.com/FelipeMorandini/stockterm/issues/68)  | Portfolio: optional decimal types | Optional precision upgrade; current `f64` matches providers. |
| [#24](https://github.com/FelipeMorandini/stockterm/issues/24)  | Advanced: custom widgets — post-MVP | Deferred until widget need outgrows ratatui built-ins. |

**Triage policy:** [#24](https://github.com/FelipeMorandini/stockterm/issues/24), [#68](https://github.com/FelipeMorandini/stockterm/issues/68), [#191](https://github.com/FelipeMorandini/stockterm/issues/191), [#194](https://github.com/FelipeMorandini/stockterm/issues/194), [#204](https://github.com/FelipeMorandini/stockterm/issues/204) are intentionally deferred/low priority and should not be auto-closed during backlog sweeps.

---

## 3. Process / SDD Gap

Workspace rule `.cursor/rules/sdd_workflow.mdc` requires Spec-Driven Development:
> No feature code changes are permitted unless the `docs/SPEC.md` is updated
> first... `engineer` must verify implementation against the `QA_PLAN.md`.

Current state:

- `docs/SPEC.md` — maintained (SDD baseline + milestones; latest shipped slices §11.12 / [#71](https://github.com/FelipeMorandini/stockterm/issues/71)–[#74](https://github.com/FelipeMorandini/stockterm/issues/74), §15 / [#43](https://github.com/FelipeMorandini/stockterm/issues/43) [#49](https://github.com/FelipeMorandini/stockterm/issues/49) [#50](https://github.com/FelipeMorandini/stockterm/issues/50) [#67](https://github.com/FelipeMorandini/stockterm/issues/67) [#69](https://github.com/FelipeMorandini/stockterm/issues/69), **§18 / [#10](https://github.com/FelipeMorandini/stockterm/issues/10) [#42](https://github.com/FelipeMorandini/stockterm/issues/42)** — alerts dialog, notifications, latched Status, **§18.13 / [#93](https://github.com/FelipeMorandini/stockterm/issues/93)–[#95](https://github.com/FelipeMorandini/stockterm/issues/95)** [PR #102](https://github.com/FelipeMorandini/stockterm/pull/102), and **§18.14 / [#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98)** — [PR #105](https://github.com/FelipeMorandini/stockterm/pull/105): alerts persistence banner + retry, notify batching, symbol sanitize — **§18.14.9**; sign-off **2026-05-18**. **§18.15** / [#100](https://github.com/FelipeMorandini/stockterm/issues/100) [#101](https://github.com/FelipeMorandini/stockterm/issues/101) [#104](https://github.com/FelipeMorandini/stockterm/issues/104) — layout percent assert, README debug env, notify **`body`** byte cap (**§18.15.8**; [PR #107](https://github.com/FelipeMorandini/stockterm/pull/107); QA sign-off 2026-05-12). Post-audit follow-ups: [#106](https://github.com/FelipeMorandini/stockterm/issues/106). **§9.15 / [#53](https://github.com/FelipeMorandini/stockterm/issues/53)** — Yahoo batched **`v7`** quotes (§9.15.9; QA sign-off 2026-05-13). **§21 / [#14](https://github.com/FelipeMorandini/stockterm/issues/14)** — theme system ([PR #126](https://github.com/FelipeMorandini/stockterm/pull/126); QA sign-off 2026-05-13). **§19 / [#18](https://github.com/FelipeMorandini/stockterm/issues/18)** — API HTTP robustness (**§19.12**); **PR:** [#115](https://github.com/FelipeMorandini/stockterm/pull/115) (sign-off **2026-05-18**. **§19.13** ([#110](https://github.com/FelipeMorandini/stockterm/issues/110)–[#114](https://github.com/FelipeMorandini/stockterm/issues/114), [#116](https://github.com/FelipeMorandini/stockterm/issues/116)) — [PR #128](https://github.com/FelipeMorandini/stockterm/pull/128) (sign-off **2026-05-18**. **§24 / [#13](https://github.com/FelipeMorandini/stockterm/issues/13)** — configurable keymap (**shipped** 2026-05-14; **manual QA** Issue **#13** pending). **§27 / [#58](https://github.com/FelipeMorandini/stockterm/issues/58) [#59](https://github.com/FelipeMorandini/stockterm/issues/59)** — News clipboard + non-blocking URL open (shipped in-tree; manual QA sign-off **2026-05-18**).
- `docs/QA_PLAN.md` — maintained (manual steps per milestone).
- `docs/ROADMAP.md` — this file (gap analysis vs product goals).

**Process:** new feature code follows `.cursor/rules/sdd_workflow.mdc` — update SPEC first, then implement, then verify against QA_PLAN.

---

## 4. Requirement Coverage

Legend: **Implemented** = working end-to-end; **Partial** = code exists but
incomplete, broken, or unwired; **Missing** = no code path.

### 4.1 Core — Real-time quotes

- **Implemented — latest-session quotes via REST ([Issue #2](https://github.com/FelipeMorandini/stockterm/issues/2), [`docs/SPEC.md`](SPEC.md) §17)** — not streaming / not Level-2.
  - Evidence: **`MarketDataProvider::get_quote`** — **Yahoo:** **`v7/finance/quote`** primary, **`v8/finance/chart`** `range=1d&interval=1d` fallback (`yahoo_latest_quote` in `src/api/yahoo.rs`); maps into **`TickerResult`**. **Polygon:** `PolygonProvider::get_quote` — daily aggregates, rolling window, **`sort=desc`** + **`limit=5`** + `latest_result()` (`src/api/polygon.rs`). Batched in **`run_stock_quote_batch`** (`src/app/app.rs`). **`draw_stock_detail`** / watchlist (`src/app/ui.rs`) unchanged at **`TickerResult`**.
  - **Shipped:** [#90](https://github.com/FelipeMorandini/stockterm/issues/90) (fallback observability), [#91](https://github.com/FelipeMorandini/stockterm/issues/91) (v7 row symbol match) — **§34** / [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#90, #91** (sign-off **2026-05-18**; **PR:** [#148](https://github.com/FelipeMorandini/stockterm/pull/148)). **Shipped:** [#89](https://github.com/FelipeMorandini/stockterm/issues/89) v7→v8 **`wiremock`** orchestration test — **§32** / [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#89** (sign-off **2026-05-18**; **PR:** [#146](https://github.com/FelipeMorandini/stockterm/pull/146)). **Shipped:** Yahoo **`v7`** multi-symbol batching ([#53](https://github.com/FelipeMorandini/stockterm/issues/53) / [`docs/SPEC.md`](SPEC.md) §9.15.9, [`docs/QA_PLAN.md`](QA_PLAN.md) Issue #53 — 2026-05-13). **§19.13** ([#110](https://github.com/FelipeMorandini/stockterm/issues/110)–[#114](https://github.com/FelipeMorandini/stockterm/issues/114), [#116](https://github.com/FelipeMorandini/stockterm/issues/116)) — [PR #128](https://github.com/FelipeMorandini/stockterm/pull/128) / [`docs/SPEC.md`](SPEC.md) §19.13.7 (sign-off **2026-05-18**).
- **Implemented — watchlist + multi-row table (Issue #3)**
  - Evidence: `Config.watchlist`, `App.watchlist` / `watchlist_quotes`,
    `run_stock_quote_batch` + bounded concurrency (`src/app/app.rs`); Stock View
    table + detail pane; persist via `Config::try_save`.
- **Implemented — configurable refresh ([Issue #4](https://github.com/FelipeMorandini/stockterm/issues/4), [`docs/SPEC.md`](SPEC.md) §35)**
  - Evidence: `data_poll_interval()` maps `Config.refresh_rate` (`0` → 30 s, floor 5 s);
    `on_background_tick` throttles quote / charts / news spawns; UI tick ~200 ms via
    `spawn_event_thread` (`src/app/event.rs`); `stock_refresh_inflight` + status
    **“Refreshing quotes…”**; Settings row **0** persists `refresh_rate`.
  - **Shipped:** §35.6.1 unit tests + [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#4** sign-off **2026-05-18**.

### 4.2 Core — Symbol search with typeahead

- **Implemented (Issues #5 / #29)**
  - Evidence: `draw_search` + `handle_search_events` (`src/app/ui.rs`,
    `handlers.rs`); debounced `FetchDone::Search` + `spawn_search_task`
    (`src/app/app.rs`); provider `search_symbols` via Yahoo/Polygon.

### 4.3 Core — Portfolio (CRUD, totals, P/L, share counts)

- **Implemented (Issues [#6](https://github.com/FelipeMorandini/stockterm/issues/6) / [#48](https://github.com/FelipeMorandini/stockterm/issues/48))**
  - Evidence: `models/portfolio.rs::PortfolioItem`; `App::add_to_portfolio` /
    `remove_from_portfolio` with **`Config::try_save`**; weighted-average cost;
    totals helpers; `draw_portfolio` + add dialog / two-step remove /
    `letter_key_plain` (`src/app/portfolio.rs`).
  - Quote batch includes **watchlist + active symbol + all portfolio tickers**
    (`collect_symbols_for_quote_fetch`); `apply_stock_fetch_done` back-fills
    `current_price` from `watchlist_quotes`.
  - `handle_portfolio_events` from `handlers.rs` on `Tab::Portfolio`; Enter → Stock
    View + `request_immediate_stock_poll`.
- **Implemented (Issues [#43](https://github.com/FelipeMorandini/stockterm/issues/43) / [#49](https://github.com/FelipeMorandini/stockterm/issues/49) / [#50](https://github.com/FelipeMorandini/stockterm/issues/50) / [#67](https://github.com/FelipeMorandini/stockterm/issues/67) / [#69](https://github.com/FelipeMorandini/stockterm/issues/69), `docs/SPEC.md` §15)** — Alerts **Price Alerts** title parity + empty-state **a/A** copy; Stock View status **A–Z** + **w/x/j/k** Shift hint; portfolio add dialog **Tab**/**Shift+Tab** field focus; **`inline_error`** on commit when `add_to_portfolio` fails without **`try_save`**; **`validate_holding_limits`** (shares/price caps).
- **Shipped — §37 / [PR #151](https://github.com/FelipeMorandini/stockterm/pull/151)** — [#81](https://github.com/FelipeMorandini/stockterm/issues/81) narrow-terminal status bar, [#82](https://github.com/FelipeMorandini/stockterm/issues/82) plain-Tab-only dialog cycle, [#83](https://github.com/FelipeMorandini/stockterm/issues/83) **`add_to_portfolio`** error-path docs — all CLOSED 2026-05-19; sign-off **2026-05-18**.
- **Shipped — §55 / [#182](https://github.com/FelipeMorandini/stockterm/issues/182)** — Portfolio row edit UI: **`e`** opens prefilled edit dialog, two-step save confirm, direct **`update_portfolio_holding`** override (not weighted average) — [`docs/SPEC.md`](SPEC.md) §55; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#182** (sign-off **2026-05-23**).
- **Open follow-ups:** optional decimal money ([#68](https://github.com/FelipeMorandini/stockterm/issues/68), deferred).

### 4.4 Core — Historical charts in terminal

- **Implemented (Issues #7 / #8 / #9, M4)** — line + candlestick widget, viewport zoom/pan, `TimeRange` keys; see `docs/SPEC.md` §11.
- **Implemented (Issues [#62](https://github.com/FelipeMorandini/stockterm/issues/62) / [#63](https://github.com/FelipeMorandini/stockterm/issues/63) / [#64](https://github.com/FelipeMorandini/stockterm/issues/64), §11.11)** — symbol change clears stale `historical_data`; Yahoo W1 intraday empty → daily retry; transient historical errors keep last-good series; viewport ticker uses requested symbol when response `ticker` is empty; see `docs/SPEC.md` §11.11.7.
- **Implemented (Issues [#71](https://github.com/FelipeMorandini/stockterm/issues/71)–[#74](https://github.com/FelipeMorandini/stockterm/issues/74), §11.12)** — `InflightRecovery` + second channel when `FetchDone` send fails; removed dead **`fetch_historical_data`**; **`yahoo_w1_daily_fallback_interval`** + tests; watchlist add skips chart clear on case-only normalization — see [`docs/SPEC.md`](SPEC.md) §11.12.8.
- **Shipped — [#65](https://github.com/FelipeMorandini/stockterm/issues/65) / [`docs/SPEC.md`](SPEC.md) §52.1** — Polygon aggregates `limit` cap, partial-page notice on Charts (provider-gated), plan-tier error copy — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#65** (sign-off **2026-05-22**; **PR:** [#178](https://github.com/FelipeMorandini/stockterm/pull/178)).
- **Shipped — [#190](https://github.com/FelipeMorandini/stockterm/issues/190) / [`docs/SPEC.md`](SPEC.md) §63** — fixed-width candle bodies, index layout for **D1**/**W1**/**M1**, time layout for **Y1** weekly+; `insta` density snapshots — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#190** (sign-off **2026-05-25**; **PR:** [#201](https://github.com/FelipeMorandini/stockterm/pull/201)).
- **Shipped — [#199](https://github.com/FelipeMorandini/stockterm/issues/199) / [`docs/SPEC.md`](SPEC.md) §64** — precompute candle column layout off the 60fps render path: cache `Vec<CandleBarLayout>` on `App`, invalidate on viewport/bar-count/`time_range`/area change; visual parity with §63 enforced by existing `insta` snapshots — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#199** (sign-off **2026-05-25**; **PR:** [#202](https://github.com/FelipeMorandini/stockterm/pull/202)).
- **Shipped — [#200](https://github.com/FelipeMorandini/stockterm/issues/200) / [`docs/SPEC.md`](SPEC.md) §65** — normalize `HistoricalData::t` to Unix milliseconds at provider ingest (`normalize_bar_timestamp_to_ms`); remove `TIMESTAMP_MS_EPOCH_THRESHOLD` / `bar_timestamps_are_millis` heuristic from `src/app/charts.rs`; simplify `median_bar_gap_secs` to single ms-only branch — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#200** (sign-off **2026-05-25**; **PR:** [#202](https://github.com/FelipeMorandini/stockterm/pull/202)).
- **Shipped** — [#79](https://github.com/FelipeMorandini/stockterm/issues/79) Unicode ticker normalization — **§67** (sign-off **2026-05-25**; **PR:** pending).

### 4.5 Core — Time ranges (1D/1W/1M/1Y)

- **Implemented (Issue #9 / M4)** — `TimeRange`, provider mapping, Charts keys `1`–`4`; see `docs/SPEC.md` §11.

### 4.6 Core — Price alerts and notifications

- **Shipped — [#32](https://github.com/FelipeMorandini/stockterm/issues/32)** — **`get_current_price`** case-insensitive portfolio/watchlist lookup + session-scoped empty **`ticker`** on **`ticker_data`** ([`docs/SPEC.md`](SPEC.md) §41.1).
- **Implemented ([Issues #10](https://github.com/FelipeMorandini/stockterm/issues/10) / [#42](https://github.com/FelipeMorandini/stockterm/issues/42), [`docs/SPEC.md`](SPEC.md) §18)**
  - Evidence: `models/alerts.rs` (`Alert`, `AlertCondition`, `process_alert_crossings`); `App::{add_alert, remove_alert, check_alerts, get_current_price}`; `draw_alerts` / `AlertAddDialog` / `handle_alerts_events` (`src/app/alerts.rs`); `save_alerts` → `Config::try_save`; `check_alerts` after `apply_stock_fetch_done`; terminal **BEL** + optional **`notify-rust`** (Cargo feature **`desktop-notify`**, default on) when `notifications_enabled`; Settings row **Desktop alert toasts**; **Status** uses latched **`triggered`** (**TRIGGERED** / **Armed** / **No quote**). Shipped: [PR #99](https://github.com/FelipeMorandini/stockterm/pull/99).
  - **Follow-ups:** [#96](https://github.com/FelipeMorandini/stockterm/issues/96)–[#98](https://github.com/FelipeMorandini/stockterm/issues/98) — [PR #105](https://github.com/FelipeMorandini/stockterm/pull/105) (**§18.14.9**); manual QA in [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#96–#98** signed off **2026-05-18**. Post-ship scratch: [#103](https://github.com/FelipeMorandini/stockterm/issues/103). **§18.15** ([#100](https://github.com/FelipeMorandini/stockterm/issues/100), [#101](https://github.com/FelipeMorandini/stockterm/issues/101), [#104](https://github.com/FelipeMorandini/stockterm/issues/104)) — [PR #107](https://github.com/FelipeMorandini/stockterm/pull/107); QA sign-off 2026-05-12 per [`docs/QA_PLAN.md`](QA_PLAN.md). **Shipped — §40:** [#106](https://github.com/FelipeMorandini/stockterm/issues/106) release **`centered_rect`** clamp + incremental notify **`body`** — see [`docs/SPEC.md`](SPEC.md) §40.3. [#19](https://github.com/FelipeMorandini/stockterm/issues/19) (persistence UX overlap for failed saves). **§18.13 (#93–#95)** shipped 2026-05-12 — `src/app/layout.rs`, Condition **←/→**, `STOCKTERM_DEBUG_ALERT_NOTIFY`.

### 4.7 Core — News headlines

- **Implemented (Issues #11 / #29)** — list, scroll, Enter → open URL (sync).
  - Evidence: `draw_news`, `news_list_state`, `handle_news_events`; throttled
    `try_spawn_news_fetch` + `FetchDone::News`. Yahoo path uses `query1`
    search `news` + RSS fallback before legacy `query2` (`src/api/yahoo.rs`);
    [`src/app/open_url.rs`](../src/app/open_url.rs) + [`App::news_try_open_selected`](../src/app/app.rs).
  - **Shipped — `query2` drift (#54):** [`yahoo_news_query2_from_text`](../src/api/yahoo.rs) lenient paths + **`STOCKTERM_DEBUG_YAHOO_NEWS`** — **§36** / [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#54** (sign-off **2026-05-18**; **PR:** [#150](https://github.com/FelipeMorandini/stockterm/pull/150)).
- **Implemented — [#58](https://github.com/FelipeMorandini/stockterm/issues/58) / [#59](https://github.com/FelipeMorandini/stockterm/issues/59)** — clipboard copy (`c`), non-blocking open, `http`/`https` validation — [`docs/SPEC.md`](SPEC.md) **§27.9**; manual QA [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#58, #59** (sign-off **2026-05-18**).
- **Implemented — [#60](https://github.com/FelipeMorandini/stockterm/issues/60)** — Search **Esc** vs global error: domain-gated clear in [`search_esc_reset`](../src/app/app.rs) (**§33**); manual QA [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#60** (sign-off **2026-05-18**).

### 4.8 TUI — Layout, color, formatting

- **Implemented — base layout**
  - Evidence: `ui.rs::draw` builds a top tab bar + content + status bar with
    `ratatui::Layout`, `Tabs`, `Block::borders`, color spans for change/P/L.
- **Implemented — every tab** — Stock View, Charts, Portfolio, Alerts, Search, News, Settings, Options, Backtest are all live (Issues #5 / #11 / #12 / #21 / #22 / #25 / #29 across M3 → M8). Per-tab layout overrides land via Settings row **6** (Issue [#15](https://github.com/FelipeMorandini/stockterm/issues/15) / §31).

### 4.9 TUI — Interactive charts (zoom/pan)

- **Implemented (Issue #8 / M4)** — `ChartViewport` + `viewport_zoom_in` / `viewport_zoom_out` / `viewport_pan_left` / `viewport_pan_right` / `viewport_reset` in [`src/app/charts.rs`](../src/app/charts.rs); default chords `+` / `-` (zoom), `h` / `l` (pan), `0` (reset) via `Action::Chart*` in `src/config/keymap.rs` (configurable per §24). Unit tests cover the zoom / pan / clamp invariants (`zoom_in_shrinks_width`, `pan_left_moves_window`, `pan_right_at_end_noop`). **Shipped — [#180](https://github.com/FelipeMorandini/stockterm/issues/180) / [`docs/SPEC.md`](SPEC.md) §54** — `time_range` / `chart_mode` persist via `last_time_range` / `last_chart_mode` (manual QA [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#180** — sign-off **2026-05-23**).

### 4.10 TUI — Keyboard navigation & customizable shortcuts

- **Implemented — [Issue #13](https://github.com/FelipeMorandini/stockterm/issues/13) / [`docs/SPEC.md`](SPEC.md) §24** — `Config.keymap`, [`ResolvedKeymap`](../src/config/keymap.rs) with per-[`BindingLayer`](../src/config/keymap.rs) lookup; global + tab handlers dispatch **`Action`**; README Keymap; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#13** (sign-off **2026-05-18**).
- **Implemented — [#134](https://github.com/FelipeMorandini/stockterm/issues/134) / [`docs/SPEC.md`](SPEC.md) §25:** User remaps propagate to every [`BindingLayer`](../src/config/keymap.rs) where `default_bindings()` registers the same [`Action`](../src/config/keymap.rs) (today **`PortfolioRowUp`** / **`PortfolioRowDown`** on list and remove-armed). **Manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#134** (sign-off **2026-05-18**).
- **Implemented — [#136](https://github.com/FelipeMorandini/stockterm/issues/136) / [`docs/SPEC.md`](SPEC.md) §26 — [PR #140](https://github.com/FelipeMorandini/stockterm/pull/140):** Keymap phase 2 — explicit default chords for portfolio / alert dialog digits and Settings edit buffer (`PortfolioDialogDigitOrDot`, `AlertDialogDigitOrDot`, `SettingsEditDigit`, `SettingsEditSymbolChar`); Settings edit **Shift+letter** fallback when no chord matches; Stock View symbol + Search query + alert dialog letters / condition **`a`/`b`** remain §24.5 / §26 wildcards; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#136** (sign-off **2026-05-18**).
- **Implemented — [#137](https://github.com/FelipeMorandini/stockterm/issues/137) / [`docs/SPEC.md`](SPEC.md) §28:** Filter **toggle** via **`StockFilterToggle`** / **`PortfolioFilterToggle`**; keys **inside** `filter_input_mode` via **`BindingLayer::FilterInput`** (`FilterClear`, `FilterCommit`, `FilterBackspace`, `FilterSlash`, `FilterQueryChar`) in [`consume_filter_input_key`](../src/app/app.rs) — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#137** (sign-off **2026-05-18**).
- **Implemented — [#139](https://github.com/FelipeMorandini/stockterm/issues/139) / [`docs/SPEC.md`](SPEC.md) §29:** Alert add dialog **`AlertDialogSymbolChar`**, **`AlertDialogConditionAbove`**, **`AlertDialogConditionBelow`** — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#139** (sign-off **2026-05-18**).
- **Implemented — [#138](https://github.com/FelipeMorandini/stockterm/issues/138) / [`docs/SPEC.md`](SPEC.md) §30:** **`DEFAULT_BINDINGS`** compile-time table; removed runtime **`Box::leak`** — **no** user-visible keymap change — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#138** (sign-off **2026-05-17**).
- **Implemented — [#51](https://github.com/FelipeMorandini/stockterm/issues/51) / [`docs/SPEC.md`](SPEC.md) §42.1:** Global **`q`/`Q`** quit via **`should_global_quit`** (respects §24 when **`q`** remapped on Global); Tab / BackTab policy documented — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#51** (sign-off **2026-05-19**; **PR:** [#156](https://github.com/FelipeMorandini/stockterm/pull/156)).

### 4.11 TUI — Configurable display / layout / theme

- **Implemented — Issue [#14](https://github.com/FelipeMorandini/stockterm/issues/14) / [`docs/SPEC.md`](SPEC.md) §21 — [PR #126](https://github.com/FelipeMorandini/stockterm/pull/126).**
  - Evidence: `src/config/theme.rs` (`ThemePreset`, `Theme`, `ThemePalette`, `parse_hex_rgb`, `Theme::resolve_rgb`); `src/app/styles.rs` (`ResolvedTheme`); `App::theme_palette_for_render`, `settings_commit_theme_preset`, Settings row **3** draft + **Enter** save; `ui` / `charts` / `portfolio` / `alerts` draw with `theme.canvas()` and slot helpers so truecolor backgrounds match presets (including Light).
- **Implemented — Issue [#15](https://github.com/FelipeMorandini/stockterm/issues/15) / [`docs/SPEC.md`](SPEC.md) §31** — `Config.layout`, shell visibility toggles, Stock View watchlist %, Charts inner split, Settings layout presets (theme-style preview/commit); **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#15** (sign-off **2026-05-17**; **PR:** [#145](https://github.com/FelipeMorandini/stockterm/pull/145)).
  - Evidence: [`src/config/layout.rs`](../src/config/layout.rs); [`App::layout_for_render`](../src/app/app.rs), [`shell_vertical_constraints`](../src/app/layout.rs); [`draw`](../src/app/ui.rs) / [`draw_charts`](../src/app/charts.rs); Settings row **6**; [`README.md`](../README.md) `layout` table.

### 4.12 TUI — Filter stocks

- **Implemented — Issue [#16](https://github.com/FelipeMorandini/stockterm/issues/16) / [`docs/SPEC.md`](SPEC.md) §23 — [PR #132](https://github.com/FelipeMorandini/stockterm/pull/132).**
  - Evidence: `App::{filter_query, filter_input_mode}`; `clear_table_filter` on tab change; `src/app/table_filter.rs` (`filter_row_indices`, `filter_symbol_indices`, `filter_title_suffix`); Portfolio **Holdings** + Stock View **Watchlist** draw filtered rows; **`/`** filter input mode; **Esc** / **Enter** / alphanumeric per §23; `consume_filter_input_key` in `handlers.rs` / `portfolio.rs`; `watchlist_filter_indices` / `portfolio_filter_indices` + selection mapping in `app.rs`.
  - **Manual QA:** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#16** (sign-off **2026-05-18**).

### 4.13 Technical — Async fetching, non-blocking UI

- **Partial — §16 slice shipped (Issues [#17](https://github.com/FelipeMorandini/stockterm/issues/17) / [#46](https://github.com/FelipeMorandini/stockterm/issues/46) / [#77](https://github.com/FelipeMorandini/stockterm/issues/77); [`docs/SPEC.md`](SPEC.md) §16.8); optional cancel token remains**
  - Evidence: `App::run` uses `tokio::select!` over async event + `FetchDone` + `InflightRecovery`; `event.rs` bridges crossterm from a std thread; stock / historical / news / search HTTP runs in `tokio::spawn` (Issue #3, §11.12).
  - **Shipped (§40 / 2026-05-19):** [#56](https://github.com/FelipeMorandini/stockterm/issues/56) explicit **`Semaphore::acquire`** handling in quote fan-out — see [`docs/SPEC.md`](SPEC.md) §40.2. **Optional follow-up:** [#191](https://github.com/FelipeMorandini/stockterm/issues/191) **`CancellationToken`** — **§68** (planned; only if overlapping batches are introduced).

### 4.14 Technical — Stock API integration with rate limits & errors

- **Implemented (Issue [#18](https://github.com/FelipeMorandini/stockterm/issues/18); [PR #115](https://github.com/FelipeMorandini/stockterm/pull/115); [`docs/SPEC.md`](SPEC.md) §19)** — shared **`reqwest::Client`** (**5 s** / **10 s** timeouts), **`ProviderError`** with **`RateLimited`** / **`Http { body_snippet }`**, **`execute_get_text_with_retry`** on Yahoo/Polygon GETs, **`wiremock`** tests. **Manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#18** sign-off **2026-05-18**.
  - **§19.13 (Issues [#110](https://github.com/FelipeMorandini/stockterm/issues/110)–[#114](https://github.com/FelipeMorandini/stockterm/issues/114), [#116](https://github.com/FelipeMorandini/stockterm/issues/116)):** [PR #128](https://github.com/FelipeMorandini/stockterm/pull/128) — bounded error-body drain, **`Retry-After`** hardening, **`Debug`** redaction, rate-limit copy — [`docs/SPEC.md`](SPEC.md) §19.13.7; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#110–#116** (sign-off **2026-05-18**). **Shipped (§38 / 2026-05-18):** [#117](https://github.com/FelipeMorandini/stockterm/issues/117) (**408** retry), [#118](https://github.com/FelipeMorandini/stockterm/issues/118) (client init) — see [`docs/SPEC.md`](SPEC.md) §38.12. **Shipped:** Yahoo batched **`v7`** quotes ([#53](https://github.com/FelipeMorandini/stockterm/issues/53) / §9.15.9).

### 4.15 Technical — Config file for prefs / portfolio

- **Implemented**
  - Evidence: `Config::{load, save, try_save, get_config_path}` in
    `src/config/config.rs` reads/writes JSON at
    `$HOME/.stockterm.json`. Persists portfolio, watchlist, alerts, theme, layout, keymap, provider, `last_tab`, `last_symbol`, and provider preference.
  - **`STOCKTERM_API_KEY`:** runtime overlay via **`effective_api_key()`** (§22.7.1 / #34). **Shipped — [#28](https://github.com/FelipeMorandini/stockterm/issues/28) / §42.2:** document **no** env→file merge on load; unit test env read without mutating **`api_key`** (sign-off **2026-05-19**; **PR:** [#156](https://github.com/FelipeMorandini/stockterm/pull/156)).
  - **`refresh_rate`** is consumed: `data_poll_interval_secs(Config.refresh_rate)` (default `0` → 30 s, floor 5 s) drives the background poll cadence in `App::on_background_tick` and is editable via Settings row **0** (§35 / Issue #4 sign-off **2026-05-18**; **PR:** [#149](https://github.com/FelipeMorandini/stockterm/pull/149)).
  - **`default_symbol`** is consumed: `App::new` falls back to `normalize_symbol(&config.default_symbol)` when no `last_symbol` is recorded and the watchlist is empty.
  - **Shipped — [#192](https://github.com/FelipeMorandini/stockterm/issues/192) / §66:** deprecated `Config::save` (logs via `tracing`); `persist_config_interactive` / `persist_config_on_shutdown`; Backtest strategy toggle surfaces `[cfg]` — [`docs/SPEC.md`](SPEC.md) §66 (**manual QA sign-off 2026-05-25**).

### 4.16 Technical — Clear errors

- **Implemented (Issue [#20](https://github.com/FelipeMorandini/stockterm/issues/20) / §20)** — **`AppError`**, categorized status prefixes, error log, **`Ctrl+R`** retry, transient TTL.
- **Shipped — [#33](https://github.com/FelipeMorandini/stockterm/issues/33) / [#55](https://github.com/FelipeMorandini/stockterm/issues/55)** — **`ProviderError`** via **`thiserror`**; flows through **`AppError::Provider`** with URL redaction preserved ([`docs/SPEC.md`](SPEC.md) §41.2).

### 4.17 Technical — Cross-platform

- **Implemented (by virtue of stack)** — `ratatui` + `crossterm` cover
  Linux/macOS/Windows; no platform-specific code present.

### 4.18 Technical — Persistence between sessions

- **Implemented — session fields**
  - Portfolio persists via `Config::try_save` after add/remove.
  - Alerts persist on add/remove via `save_alerts` → `Config::try_save` (Issue
    #27); `triggered` transitions run via `check_alerts` after quote refresh
    (Issues #30 / #38 / #3).
  - **Watchlist persists** (`Config.watchlist`, Issue #3).
  - **Last-selected tab and last symbol persist** — Issue [#19](https://github.com/FelipeMorandini/stockterm/issues/19) / [#129](https://github.com/FelipeMorandini/stockterm/issues/129) shipped: `Config.last_tab` + `Config.last_symbol` are written through `persist_session_to_disk` (400 ms debounce + tick flush per [`docs/SPEC.md`](SPEC.md) §22.7 / §22.9). `App::new` restores both on launch.
  - **Charts `time_range` and `chart_mode` persist** — Issue [#180](https://github.com/FelipeMorandini/stockterm/issues/180) / §54 shipped: `Config.last_time_range` + `Config.last_chart_mode` via the same debounced session-save path; `App::new` restores on launch (manual QA [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#180** — sign-off **2026-05-23**).
  - **Theme** preset + overrides persist via `Config.theme` (Issue [#14](https://github.com/FelipeMorandini/stockterm/issues/14) / §21; [PR #126](https://github.com/FelipeMorandini/stockterm/pull/126)). **Layout** preset + overrides persist via `Config.layout` (Issue [#15](https://github.com/FelipeMorandini/stockterm/issues/15) / §31; [PR #145](https://github.com/FelipeMorandini/stockterm/pull/145)).

### 4.19 Advanced / optional

- **Implemented — [#23](https://github.com/FelipeMorandini/stockterm/issues/23) / [`docs/SPEC.md`](SPEC.md) §43** — Cryptocurrency quotes (Yahoo `BTC-USD`, `SymbolKind`, adaptive price formatting, Stock View `-` entry, §43.13 Enter/quote keys) — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#23** (sign-off **2026-05-19**; **PR:** [#159](https://github.com/FelipeMorandini/stockterm/pull/159)). **Shipped — [#157](https://github.com/FelipeMorandini/stockterm/issues/157) / [#158](https://github.com/FelipeMorandini/stockterm/issues/158) / [`docs/SPEC.md`](SPEC.md) §44** — provider symbol resolver + metadata-driven **Kind** — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#157, #158** (sign-off **2026-05-20**; **PR:** [#162](https://github.com/FelipeMorandini/stockterm/pull/162)). **Shipped — [#160](https://github.com/FelipeMorandini/stockterm/issues/160) / [#161](https://github.com/FelipeMorandini/stockterm/issues/161) / [`docs/SPEC.md`](SPEC.md) §45** — provider-switch Kind cache clear + Polygon **`X:BTCUSD`** wire mapping — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#160, #161** (sign-off **2026-05-20**; **PR:** [#163](https://github.com/FelipeMorandini/stockterm/pull/163)).
- **Shipped — [#21](https://github.com/FelipeMorandini/stockterm/issues/21) / [`docs/SPEC.md`](SPEC.md) §46** — Technical indicators (SMA/EMA/RSI/MACD): `src/indicators/` pure functions, Charts tab **`s`/`e`/`r`/`m`** toggles, line-chart overlays + RSI/MACD sub-pane; session-only (not persisted). **Manual QA:** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#21** (sign-off **2026-05-21**).
- **Shipped — [#165](https://github.com/FelipeMorandini/stockterm/issues/165) / [#168](https://github.com/FelipeMorandini/stockterm/issues/168) / [`docs/SPEC.md`](SPEC.md) §49** — Backtest golden metric fixtures (`src/backtest/test_util.rs`, `tests/fixtures/backtest_sma_crossover_50_200.json`) + Yahoo options expiration slice cache (`options_slices_by_ts`, multi-block parser); **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#165, #168** (sign-off **2026-05-21**; **PR:** [#170](https://github.com/FelipeMorandini/stockterm/pull/170)).
- **Shipped — [#25](https://github.com/FelipeMorandini/stockterm/issues/25) / [`docs/SPEC.md`](SPEC.md) §47** — Backtesting: `Strategy` trait, `src/backtest/` simulator, SMA crossover + RSI mean-reversion (reuses §46), **Backtest** tab, metrics + equity curve, CSV/JSON export; **`Config.backtest`** for capital/fees. **PR:** [#166](https://github.com/FelipeMorandini/stockterm/pull/166).
- **Shipped — [#22](https://github.com/FelipeMorandini/stockterm/issues/22) / [`docs/SPEC.md`](SPEC.md) §48** — Options chains: `src/models/options.rs`, Yahoo `v7/finance/options`, **`Tab::Options`**, calls/puts tables, expiration selector, Greeks toggle — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#22** (sign-off **2026-05-21**; **PR:** [#169](https://github.com/FelipeMorandini/stockterm/pull/169)). **Shipped — [#167](https://github.com/FelipeMorandini/stockterm/issues/167) / [`docs/SPEC.md`](SPEC.md) §50** — Polygon options (`src/api/polygon_options.rs`); **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#167** (sign-off **2026-05-22**; **PR:** [#172](https://github.com/FelipeMorandini/stockterm/pull/172)). **Shipped — [#171](https://github.com/FelipeMorandini/stockterm/issues/171) / [`docs/SPEC.md`](SPEC.md) §51** — Polygon expiration list session cache (`options_polygon_expirations_cache`, snapshot-only on slice miss); **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#171** (sign-off **2026-05-22**; **PR:** [#173](https://github.com/FelipeMorandini/stockterm/pull/173)). **Shipped — [#168](https://github.com/FelipeMorandini/stockterm/issues/168) / §49.2** — Yahoo expiration slice cache.
- **Shipped — Options tab polish / [`docs/SPEC.md`](SPEC.md) §52.2** (bundled with [#65](https://github.com/FelipeMorandini/stockterm/issues/65)) — Vol/OI columns, strike-visible scroll, bounded draw-path allocations (§52.2.3 interim).
- **Shipped — [#176](https://github.com/FelipeMorandini/stockterm/issues/176) / [#177](https://github.com/FelipeMorandini/stockterm/issues/177) / [`docs/SPEC.md`](SPEC.md) §53** — Polygon historical `next_url` pagination + Options zero-clone draw — **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#176, #177** (sign-off **2026-05-22**; **PR:** [#179](https://github.com/FelipeMorandini/stockterm/pull/179)).
- **Shipped — [#183](https://github.com/FelipeMorandini/stockterm/issues/183) / [`docs/SPEC.md`](SPEC.md) §56** — Options `OptionsDisplayCache` restyle on theme preset commit (`refresh_options_display_for_theme`); **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issue **#183** (sign-off **2026-05-23**; **PR:** [#187](https://github.com/FelipeMorandini/stockterm/pull/187)).
- **Shipped — [#195](https://github.com/FelipeMorandini/stockterm/issues/195) / [#196](https://github.com/FelipeMorandini/stockterm/issues/196) / [`docs/SPEC.md`](SPEC.md) §61–§62** — Options lazy `ThemeStamp` compare + theme audit / commit hook for watchlist/portfolio/alerts; **manual QA** [`docs/QA_PLAN.md`](QA_PLAN.md) Issues **#195, #196** (sign-off **2026-05-24**; **PR:** [#198](https://github.com/FelipeMorandini/stockterm/pull/198)).
- **Deferred — [#24](https://github.com/FelipeMorandini/stockterm/issues/24)** — Custom widgets (post-MVP).

---

## 5. Code-quality / Stability Gaps

_Many pre–M0 items (Theme, Polygon key plumbing, tab handlers, async portfolio
Enter) were fixed in Issue #1 ([PR #26](https://github.com/FelipeMorandini/stockterm/pull/26)).
Alert persistence landed in Issue #27._

Open gaps worth tracking (each has a GitHub issue — see [§2.1](#21-active-backlog-github)):

1. **CI (M7):** **Shipped** — `.github/workflows/ci.yml` + `insta` error-overlay snapshots (**§57–§58**, Issues [#181](https://github.com/FelipeMorandini/stockterm/issues/181), [#184](https://github.com/FelipeMorandini/stockterm/issues/184); **PR:** [#188](https://github.com/FelipeMorandini/stockterm/pull/188)). **Shipped — §59 / [#189](https://github.com/FelipeMorandini/stockterm/issues/189) + §60 / [#193](https://github.com/FelipeMorandini/stockterm/issues/193):** expanded UI snapshots + blocking `cargo fmt --check` (**PR:** [#197](https://github.com/FelipeMorandini/stockterm/pull/197); QA sign-off **2026-05-24**).
3. **Optional:** [#191](https://github.com/FelipeMorandini/stockterm/issues/191) **`CancellationToken`** — [`docs/SPEC.md`](SPEC.md) **§68** (planned; §16.1 item 2 cross-link).
4. **Shipped:** [#192](https://github.com/FelipeMorandini/stockterm/issues/192) `Config::save` I/O errors — [§4.15](#415-technical--config-file-for-prefs--portfolio) / [`docs/SPEC.md`](SPEC.md) **§66** (code + manual QA sign-off **2026-05-25**).
5. **Optional:** [#194](https://github.com/FelipeMorandini/stockterm/issues/194) saved/regex filters — M6 follow-on beyond §23.

_Recent follow-ups from ship:_ [Issue #39](https://github.com/FelipeMorandini/stockterm/issues/39)
(portfolio `try_save` parity), [Issue #40](https://github.com/FelipeMorandini/stockterm/issues/40)
(non-blocking config I/O). Issues [#30](https://github.com/FelipeMorandini/stockterm/issues/30)/[#37](https://github.com/FelipeMorandini/stockterm/issues/37)/[#38](https://github.com/FelipeMorandini/stockterm/issues/38) (alerts loop + table) shipped in the PR linked from `docs/SPEC.md` §7.
[Issue #44](https://github.com/FelipeMorandini/stockterm/issues/44) (keyboard modifiers for Stock View / Alerts) shipped in [PR #52](https://github.com/FelipeMorandini/stockterm/pull/52); deferred polish → [#48](https://github.com/FelipeMorandini/stockterm/issues/48)–[#51](https://github.com/FelipeMorandini/stockterm/issues/51).

---

## 6. Recommended Next Milestones

> **Status (2026-05-23):** M0 – M8 are largely delivered. Actionable backlog: [§2.1](#21-active-backlog-github) (issues [#189](https://github.com/FelipeMorandini/stockterm/issues/189)–[#196](https://github.com/FelipeMorandini/stockterm/issues/196) + deferred [#24](https://github.com/FelipeMorandini/stockterm/issues/24)/[#68](https://github.com/FelipeMorandini/stockterm/issues/68)/[#79](https://github.com/FelipeMorandini/stockterm/issues/79)). The milestone list below is preserved for historical context.

Suggested ordering (each should land its own `docs/SPEC.md` update + GitHub
issue before code):

1. **M0 — Stabilize build & SDD baseline** ✅ **Delivered** (GitHub Issue #1)
   - Fix `Theme`, `get_ticker_data` signature, hard-coded API key.
   - Wire `next_tab`/`prev_tab` and per-tab handlers into `handle_event`.
   - Author initial `docs/SPEC.md` + `docs/QA_PLAN.md` covering the existing
     tabs.
   - **Merge:** https://github.com/FelipeMorandini/stockterm/pull/26 — manual verification: `docs/QA_PLAN.md`. Follow-up tech debt → GitHub issues filed at ship.
2. **M1 — Swap data source to Yahoo Finance**
   - Replace Polygon client with a Yahoo-Finance-backed module
     (see §7). Keep model layer (`TickerResult`, `HistoricalData`, etc.) as
     an internal contract; add an adapter from the Yahoo response.
   - Add request timeout, non-2xx handling, structured errors.
3. **M2 — Real-time-ish quotes & multi-symbol watchlist**
   - **Partial — delivered:** [Issue #3](https://github.com/FelipeMorandini/stockterm/issues/3) — `Watchlist` in `Config`, multi-row table on Stock View, bounded concurrent Polygon quotes, `refresh_rate` throttle, background fetch via `tokio::select!` (see `docs/SPEC.md`).
   - **Remaining:** intraday / "latest quote" feel (likely **M1** Yahoo `quote` or `chart?range=1d&interval=1m`); optional [#191](https://github.com/FelipeMorandini/stockterm/issues/191) cancel semantics — [`docs/SPEC.md`](SPEC.md) §16.1.
4. **M3 — Search typeahead + News + Settings UI**
   - Implement `draw_search` with debounced typeahead suggestions.
   - Implement `draw_news` listing headlines (publisher, title, date, link).
   - Implement `draw_settings` to edit `refresh_rate`, `default_symbol`,
     theme, and (later) keymap.
5. **M4 — Time ranges & interactive charts**
   - Add `TimeRange::{D1, W1, M1, Y1}` selector (e.g. `1`, `2`, `3`, `4`).
   - Implement zoom/pan via `+`/`-`/`h`/`l`.
   - Replace text-table candlestick with a real candlestick widget
     (custom `ratatui::Widget` impl).
6. **M5 — Alerts polish**
   - Persist alerts — **done** (Issue #27: `save_alerts` → `Config::try_save`).
   - Drive `check_alerts` after quote refresh — **done** (Issues #30 / #38); table constraints — **done** (#37).
   - Add OS notification (e.g. `notify-rust`) and terminal bell.
   - Add input dialog for symbol/condition/price.
7. **M6 — Filters, customizable shortcuts, themes**
   - Substring filter over watchlist/portfolio — **done** (Issue [#16](https://github.com/FelipeMorandini/stockterm/issues/16) / §23, [PR #132](https://github.com/FelipeMorandini/stockterm/pull/132)); broader saved/regex filters → [#194](https://github.com/FelipeMorandini/stockterm/issues/194).
   - Define `Keymap` in `Config`, look up actions via map.
   - Define `Theme` (palette) and apply via a `Style`-builder helper.
8. **M7 — Tests & CI** ✅ **Largely delivered**
   - Unit tests for portfolio math, alerts, providers, backtest — in `src/` `#[cfg(test)]` (~320+ tests).
   - **`wiremock`** integration coverage (§19, §32, etc.).
   - **Shipped — [#181](https://github.com/FelipeMorandini/stockterm/issues/181) / [#184](https://github.com/FelipeMorandini/stockterm/issues/184):** GitHub Actions CI + `insta` error-overlay snapshots (**§57–§58**, **PR:** [#188](https://github.com/FelipeMorandini/stockterm/pull/188)).
   - **Shipped — §63 / [#190](https://github.com/FelipeMorandini/stockterm/issues/190):** candlestick density polish (**PR:** [#201](https://github.com/FelipeMorandini/stockterm/pull/201)). **Shipped — §59 / [#189](https://github.com/FelipeMorandini/stockterm/issues/189):** expand snapshots. **Shipped — §60 / [#193](https://github.com/FelipeMorandini/stockterm/issues/193):** blocking fmt.
9. **M8 — Optional / advanced**
   - **Shipped — [#23](https://github.com/FelipeMorandini/stockterm/issues/23):** crypto symbols on Yahoo (`§43`; sign-off **2026-05-19**).
   - **Shipped — [#21](https://github.com/FelipeMorandini/stockterm/issues/21):** indicators (SMA/EMA/RSI/MACD) — **§46** (sign-off **2026-05-21**).
   - **Shipped — [#25](https://github.com/FelipeMorandini/stockterm/issues/25):** backtesting — **§47** (sign-off **2026-05-21**).
   - **Shipped — [#22](https://github.com/FelipeMorandini/stockterm/issues/22):** options chains (**§48**; **PR:** [#169](https://github.com/FelipeMorandini/stockterm/pull/169)).
   - Custom widgets remain future.

---

## 7. API Strategy Note (Yahoo vs Polygon vs Alpha Vantage / IEX)

The codebase currently targets **Polygon.io** (`src/api/polygon.rs`). The user
prefers a free / cheap source and has accepted **Yahoo Finance** as the default.

Recommendation: **migrate to Yahoo Finance as the primary source**, but
abstract the call sites behind a trait so we can swap providers later.

- **Yahoo Finance (recommended)**
  - Pros: free, no API key, broad coverage (US + international tickers,
    crypto, FX), supports search (`v1/finance/search`), quote
    (`v7/finance/quote`), and historical OHLC (`v8/finance/chart`).
  - Cons: unofficial / undocumented endpoints, can rate-limit by IP, occasional
    schema drift; news endpoint requires scraping or a feed.
  - Rust options: use `reqwest` directly against the public endpoints, or
    adopt a maintained crate (e.g. `yahoo_finance_api`) — pin and vendor
    types into `models/` to insulate the rest of the app.
- **Polygon.io (current code)**
  - Pros: clean REST + docs, official, supports tickers/news/aggregates.
  - Cons: free tier is **5 requests/minute**, end-of-day data only on free
    tier, requires an API key. Real-time and intraday require a paid plan.
- **Alpha Vantage**
  - Pros: free key, simple REST.
  - Cons: free tier is **5 req/min, 500/day**; tighter than even Polygon.
- **IEX Cloud**
  - Pros: low-cost paid tiers, real-time US equities.
  - Cons: paid; 2024+ migration to "IEX Cloud retired" / new platform — risk.

Concrete next step: introduce `src/api/mod.rs` with a `MarketDataProvider`
trait (`get_quote`, `get_history(range)`, `search`, `get_news`), implement
`YahooProvider`, keep `PolygonProvider` as an opt-in alternative wired through
`Config` (e.g. `provider: "yahoo" | "polygon"`, plus optional `api_key`). This
satisfies "use Yahoo Finance for free" while keeping the door open to a paid
provider without rewriting the app layer.

---

## 8. Deliverables checklist for this pass

- [x] `docs/ROADMAP.md` (this file)
- [x] `docs/SPEC.md` — Issue #3 SPEC + shipment section (SDD)
- [x] `docs/QA_PLAN.md` — manual steps for Issue #3
- [x] GitHub issues — backlog tracked in repo (see Issues)

