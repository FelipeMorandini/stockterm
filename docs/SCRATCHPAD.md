# Scratchpad

Use this file for **short-lived** notes during development. Before shipping, turn
bullets into GitHub issues and clear the list.

_Last cleared: 2026-05-12 — `/ship`: audit scratchpad triaged — URL scheme allowlist → existing [#59](https://github.com/FelipeMorandini/stockterm/issues/59); `ProviderError` / `thiserror` polish → existing [#55](https://github.com/FelipeMorandini/stockterm/issues/55); `reqwest::Client::build` `expect` called out in [#18](https://github.com/FelipeMorandini/stockterm/issues/18) / SPEC §19; new [#108](https://github.com/FelipeMorandini/stockterm/issues/108) (event thread clean shutdown)._

- **#18 / wiremock + `tokio::time::advance`:** In **`start_paused`** tests, advancing virtual time while a **`GET`** is in flight against a client whose **per-request timeout** matches production (**10 s**) can still yield **`reqwest::Timeout`** before wiremock responds; **`retry_after_one_second_before_success`** therefore uses **wall-clock** **`Retry-After: 1`**. **`stall_triggers_timeout`** uses a **dedicated** short-timeout client + paused clock per §19.8.

**`/audit` (Issue #18 HTTP layer, 2026-05-12) — advisory**

- **`reqwest::Response::bytes().await`** on non-2xx / **429** paths buffers the **entire** response body in memory before the code keeps only the first **4096** bytes for snippets — a hostile server could send a very large error body and spike RAM (success path still uses full **`text()`** as before). Mitigation would be a bounded reader or **`Content-Length`** guard; tradeoff vs SPEC §19.4 “read body for errors.”
- **`Retry-After` integer seconds:** **`parse::<u64>`** accepts arbitrarily large values → **`tokio::time::sleep`** can stall the **spawned** fetch task for an unbounded wall time (self-DoS / hung tab until timeout). Consider capping (e.g. **24 h**) per product policy.
- **`ProviderError::RateLimited` `Display`:** uses **`Duration::as_secs()`** only — sub-second **`Retry-After`** surfaces as **“retry after 0s”**; minor UX/logging inaccuracy.
- **HTTP-date normalization:** only **` … GMT`** is rewritten for chrono; other suffixes (**`UTC`**, lowercase, IMF variants) may still fail to parse → **`None`** → backoff path (acceptable but document or extend).
- **`execute_get_text_with_retry_inner`:** final **`Transport("HTTP retry loop exhausted…")`** after the **`for`** loop should be unreachable with **`MAX_ATTEMPTS > 0`**; consider **`unreachable!()`** + comment or **`debug_assert!`** for clarity (no runtime issue).
- **Checklist:** **`ProviderError`** remains manual **`Display`** / **`Debug`** (project **`thiserror`** unused here — overlaps existing [#55](https://github.com/FelipeMorandini/stockterm/issues/55)). No new blocking **`std::thread`** or UI-thread HTTP; fetch stays on existing **`tokio::spawn`** paths.
