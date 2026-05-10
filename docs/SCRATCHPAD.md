# Scratchpad

Use this file for **short-lived** notes during development. Before shipping, turn
bullets into GitHub issues and clear the list.

_Deferred items from the last ship were filed as GitHub issues (see repository Issues)._

- **Issue #44 follow-up:** `handle_portfolio_events` still matches `KeyModifiers::NONE` only on letter keys — consider the same `letter_key_plain` + case rules for parity (SPEC §8.6).
- **UX:** Stock View status/footer one-liner for `w` / `x` / `j` / `k` (and rare `w`/`x`/`j`/`k` ticker edge case) would reduce confusion; not in #44 scope.

_Audit (Issue #44 ship):_

- **Advisory:** `draw_alerts` empty-state copy says “Add alerts with the **`a`** key” only; **`Shift+a`** also works per `letter_key_plain`. Optional string tweak for parity with behavior.
- **Advisory:** Global **`q`** / tab switching still use unconditional or loose modifier matching in places (`handle_event` requires `NONE` for `q` only). Inconsistent with Stock View letter rules but low risk; revisit if users report Caps Lock / layout issues on quit.
