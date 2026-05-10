# SPEC — Issue #27: Persist alerts to config (`save_alerts`)

**Source:** [GitHub Issue #27](https://github.com/FelipeMorandini/stockterm/issues/27).  
**Goal:** `add_alert` and `remove_alert` must mirror the in-memory `App.alerts` vector into `Config.alerts` and persist `~/.stockterm.json` so alerts survive process restart. Replace the `save_alerts` no-op with real I/O.

**Prerequisite:** Issue #1 work is assumed shipped (Alerts tab, `App::new` already hydrates `alerts` from `Config::load()` — see `src/app/app.rs`).

---

## 1. Current gaps (verified in tree)

| Location | Problem |
|----------|---------|
| `src/app/alerts.rs` — `save_alerts` | Empty body; commented-out intended logic. |
| `add_alert` / `remove_alert` | Already invoke `save_alerts()` after mutating `self.alerts`, but persistence never runs. |
| `check_alerts` | On `triggered` transition, duplicates persistence with `self.config.alerts = self.alerts.clone(); self.config.save();` — silent `save()`, divergent from a unified alerts persistence path. |

**Non-goals for #27:** Wiring `check_alerts` into the main loop (ROADMAP follow-up), portfolio persistence behavior, or changing alert editor UX (still hard-coded demo price in handler).

---

## 2. Crate & module layout

- **Single package:** `stockterm` (no new crates).
- **Touch points:** `src/app/alerts.rs` (`impl App` block for `save_alerts`, optional dedup in `check_alerts`). No new modules unless a tiny helper is needed (prefer keeping logic in `alerts.rs`).
- **Config:** Reuse existing `Config::try_save` / `ConfigError` in `src/config/config.rs`. `Alert` is already `Serialize`/`Deserialize` (`src/models/alerts.rs`).

---

## 3. Implementation plan (Rust)

### 3.1 Implement `save_alerts`

- Change `fn save_alerts(&self)` to **`fn save_alerts(&mut self)`** — assignment to `self.config.alerts` requires mutable access.
- Body:
  1. `self.config.alerts = self.alerts.clone();` (same shape as portfolio updates in `src/app/app.rs`).
  2. Call **`self.config.try_save()`** (not `save()`), map `Err(e)` to **`self.error_message = Some(...)`** with a short prefix, e.g. `Failed to save config: {e}` or `Failed to save alerts: {e}`, using `ConfigError`’s `Display` via `.to_string()`.
  3. On `Ok(())`, do **not** blindly clear `error_message` (other subsystems may have set it); optionally clear only if the message is known to be save-related — **SPEC preference:** only set on failure; leave success path a no-op for `error_message` to avoid surprising wipes.

### 3.2 Call sites

- `add_alert` / `remove_alert` already call `save_alerts()` — no signature changes beyond `save_alerts` taking `&mut self` (callers already have `&mut App`).
- Refactor **`check_alerts`**: replace the `if updated { self.config.alerts = ...; self.config.save(); }` block with **`self.save_alerts()`** so triggered-state updates and add/remove share one path and consistent error surfacing.

### 3.3 Async / threading

- No change: persistence remains synchronous on the main async task, same as portfolio `config.save()` today.

### 3.4 Serialization compatibility

- `Alert` includes `triggered: bool`; persisting the full vector preserves fired state across restarts (already implied by `check_alerts` writing `config.alerts`). No schema migration.

---

## 4. Verification targets

- `cargo build --release` and `cargo clippy -- -D warnings` — clean.
- After add/remove alert, `~/.stockterm.json` contains an `alerts` array consistent with the UI.
- Restart binary: alerts list matches previous session (count, symbol, condition, price, `triggered` if applicable).
- Induce a save failure (optional dev test): e.g. read-only home or invalid path — `error_message` reflects failure (manual or scripted).

---

## 5. Out of scope (Issue #27)

- Calling `check_alerts` from `App::run` tick path.
- Improving alert creation UX (price/condition input).
- Changing portfolio to `try_save` + surfaced errors.

---

## 6. Approval

After maintainer approval of this SPEC, implementation may proceed per `.cursor/rules/sdd_workflow.mdc` and `docs/QA_PLAN.md`.

## 7. Shipment

- **Status:** Implemented; manual QA passed per `docs/QA_PLAN.md` (Issue #27).
- **Deferred:** Scratchpad items filed as GitHub Issues [#37](https://github.com/FelipeMorandini/stockterm/issues/37), [#38](https://github.com/FelipeMorandini/stockterm/issues/38), [#39](https://github.com/FelipeMorandini/stockterm/issues/39), [#40](https://github.com/FelipeMorandini/stockterm/issues/40).
- **PR:** https://github.com/FelipeMorandini/stockterm/pull/41
