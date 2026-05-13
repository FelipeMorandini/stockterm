# StockTerm

Terminal UI (TUI) for stock quotes, watchlists, charts, and alerts. Rust + [ratatui](https://github.com/ratatui-org/ratatui) + Tokio.

Product behavior and milestones are documented in [`docs/SPEC.md`](docs/SPEC.md). Manual verification steps live in [`docs/QA_PLAN.md`](docs/QA_PLAN.md).

## Developer / debug

These environment variables are supported for local diagnosis. Any other `STOCKTERM_DEBUG_*` name is **not** supported unless it appears here or in `docs/SPEC.md`.

| Variable | When it applies | Behavior |
|----------|-----------------|----------|
| _(none)_ | Default HTTP stack | **5 s** connect + **10 s** request timeout on the shared `reqwest::Client` ([`docs/SPEC.md`](docs/SPEC.md) §19 / Issue #18; `src/api/http.rs`). |
| _(tests)_ | Authors writing **`#[tokio::test(start_paused = true)]`** + **`reqwest`** | Paused **`tokio::time::advance`** can fire **`reqwest`**’s request **`timeout`** while a **`GET`** is still in flight → spurious **`Timeout`**. Prefer wall-clock waits for **`Retry-After`** assertions or an isolated **`Client`** with a short timeout for stall tests — [`docs/SPEC.md`](docs/SPEC.md) §19.8 / §19.13.3. |
| `STOCKTERM_DEBUG_ALERT_NOTIFY` | Build with the default **`desktop-notify`** Cargo feature | Set to exactly `1` (no trimming; no other value enables it). After `notify-rust` `Notification::show()`, StockTerm may `eprintln!` the `Result` to stderr on the **coalesced** desktop notify path (including `Ok(())`) so you can confirm the call completed. |
| `STOCKTERM_DEBUG_HTTP_DELAY_MS` | Any build | Non-negative integer: milliseconds to sleep **once per stock quote batch** before HTTP fan-out (`src/api/http.rs`). `0`, unset, or invalid → no delay. See `docs/SPEC.md` §16. |

Run from the repo root, for example:

```bash
STOCKTERM_DEBUG_HTTP_DELAY_MS=5000 cargo run --release
STOCKTERM_DEBUG_ALERT_NOTIFY=1 cargo run --release
```
