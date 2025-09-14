# Switchyard Fix Implementation Plan

This is the granular, step‑by‑step implementation plan to remediate all items in `cargo/switchyard/BUGS.md`. Tasks are grouped by bug. Check items off as you implement. Tests listed are those referenced in BUGS.md.

Legend:
- [ ] task pending
- [x] task done

---

## 1) provenance-completeness

- [ ] Preflight: add provenance for RestoreFromBackup rows
  - [ ] Edit `cargo/switchyard/src/api/preflight/mod.rs`
    - [ ] In match arm `Action::RestoreFromBackup { target }`, compute provenance using `api.owner` when present (same as EnsureSymlink arm).
    - [ ] Inject `{uid,gid,pkg}` into the `provenance` field via `RowEmitter::emit_row` args.
  - [ ] Add unit test to validate `provenance.uid/gid` present for restore rows.

- [ ] Apply: emit uid/gid/pkg in apply events
  - [ ] Edit `cargo/switchyard/src/api/apply/executors/ensure_symlink.rs`
    - [ ] Before `ensure_provenance(&mut extra);`, look up `api.owner` and, if present, call `owner_of(target)`; insert `{uid,gid,pkg}` under `extra["provenance"]`.
  - [ ] Edit `cargo/switchyard/src/api/apply/executors/restore.rs`
    - [ ] Same as above for the `target` path.
  - [ ] Add unit tests for both executors: ensure `apply.result` carries `provenance.uid/gid`.

- [ ] Builder default (optional, safe fallback)
  - [ ] Edit `cargo/switchyard/src/api/builder.rs`
    - [ ] In `build()`, if `owner.is_none()`, optionally set a default `FsOwnershipOracle` behind a feature flag (e.g., `default_oracles`). Document that tests should provide the oracle explicitly when strict.

- [ ] Verify tests
  - [ ] Un‑ignore and run `requirements::provenance_completeness::req_o7_provenance_completeness`.

---

## 2) preflight-rescue-verification (baseline OK when rescue not required)

- [ ] Ensure baseline policy does not STOP on unrelated gates
  - [ ] Edit `cargo/switchyard/src/policy/config.rs`
    - [ ] Consider relaxing default `source_trust` to `WarnOnUntrusted` to make baselines pass, while keeping `Policy::production_preset()` strict.
    - [ ] Confirm `durability.preservation` default is `Off` (already is).
  - [ ] Alternatively, adjust the baseline test to set `policy.risks.source_trust = WarnOnUntrusted` if keeping default strict is desired.

- [ ] Mount gate notes (non‑blocking to pass, but makes notes consistent)
  - [ ] Edit `cargo/switchyard/src/policy/gating.rs`
    - [ ] Change notes like `"target not rw+exec"` to include the word `"mount"` (e.g., `"mount: target not rw+exec"`).

- [ ] Verify tests
  - [ ] Un‑ignore and run `preflight::baseline_ok::e2e_preflight_004_rescue_not_required_ok`.

---

## 3) preflight-backup-tag-handling (empty tag)

- [ ] Backup readiness with empty tag
  - [ ] Edit `cargo/switchyard/src/fs/backup/index.rs`
    - [ ] Update `find_latest_backup_and_sidecar(...)` to treat `tag.is_empty()` as a wildcard: match both untagged and any-tag backups. Approach: when tag is empty, scan entries with pattern `.{name}.*.\d+.bak[.meta.json]` and take latest.
  - [ ] Edit `cargo/switchyard/src/fs/backup/snapshot.rs`
    - [ ] Ensure `has_backup_artifacts(...)` delegates correctly to the updated index logic.

- [ ] Verify tests
  - [ ] Un‑ignore and run `preflight::baseline_ok::e2e_preflight_009_empty_backup_tag_ok`.

---

## 4) preflight-exec-check-handling (exec_check=false)

- [ ] Rescue semantics with exec_check disabled
  - [ ] Edit `cargo/switchyard/src/policy/rescue.rs`
    - [ ] In `verify_rescue_tools_with_exec_min(exec_check, min_count)`, when `exec_check == false && min_count == 0`, return `Ok(...)` (treat as pass for presence check disabled).
    - [ ] Add unit tests covering `(false, 0)` and `(false, >0)`.

- [ ] Verify tests
  - [ ] Un‑ignore and run `preflight::baseline_ok::e2e_preflight_010_exec_check_disabled_ok`.

---

## 5) preflight-coreutils-tag-handling

- [ ] Tag handling parity
  - [ ] Ensure `index.rs` logic from item (3) supports any explicit tag, including `"coreutils"`.

- [ ] Verify tests
  - [ ] Un‑ignore and run `preflight::baseline_ok::e2e_preflight_011_coreutils_tag_ok`.

---

## 6) preflight-mount-check-notes

- [ ] Emit mount notes that match tests
  - [ ] Edit `cargo/switchyard/src/policy/gating.rs`
    - [ ] Ensure all mount‑related notes include the term `"mount"` so the substring check in tests passes.
  - [ ] Optionally add a helper in `cargo/switchyard/src/fs/mount.rs` to format human‑readable notes for reuse.

- [ ] Verify tests
  - [ ] Un‑ignore and run `preflight::extra_mount_checks_five::e2e_preflight_006_extra_mount_checks_five`.

---

## 7) lockmanager-required-production

- [ ] Map lock acquisition failure to ApiError at top‑level apply()
  - [ ] Edit `cargo/switchyard/src/api/mod.rs`
    - [ ] In `Switchyard::apply(...)`, call `apply::run(...)` then inspect the `ApplyReport`. If `mode==Commit` and `report.errors` contains the lock message (or add a structured flag via `apply::lock::acquire`), return `Err(ApiError::LockingTimeout(...))` instead of `Ok(report)`.
    - [ ] Alternatively, plumb a structured `LockingFailed` error up from `apply::lock::acquire` via a new return type; keep surface signature stable by mapping to `Result`.
  - [ ] Ensure `apply.attempt` and `apply.result` E_LOCKING events remain emitted (they already are in `apply/lock.rs`).

- [ ] Verify tests
  - [ ] Un‑ignore and run `requirements::lockmanager_required_production::req_l4_lockmanager_required_production`.

---

## 8) partial-restoration-facts

- [ ] Emit rollback planning facts (dry‑run friendly)
  - [ ] Edit `cargo/switchyard/src/api/mod.rs`
    - [ ] In `plan_rollback_of(...)`, construct an `AuditCtx` similarly to `prune_backups()` and emit a `StageLogger::rollback()` event with `{ planning: true, executed: report.executed.len() }` before returning the plan.
  - [ ] Edit `cargo/switchyard/src/api/apply/summary.rs`
    - [ ] When apply failed and rollback occurred, add a boolean `partial_restoration` or `degraded` field (best‑effort) and ensure it shows up in the summary facts.

- [ ] Verify tests
  - [ ] Un‑ignore and run `requirements::partial_restoration_facts::req_r5_partial_restoration_facts`.

---

## 9) smoke-invariants

- [ ] Map smoke failure to ApiError at top‑level apply()
  - [ ] Edit `cargo/switchyard/src/api/mod.rs`
    - [ ] After `apply::run(...)`, when `mode==Commit` and `report.errors` contains `"smoke"`, return `Err(ApiError::SmokeFailed)`.
  - [ ] Ensure auto‑rollback still happens via `apply/mod.rs` on smoke failure (already implemented).

- [ ] Verify tests
  - [ ] Un‑ignore and run `oracles::smoke_invariants::smoke_invariants`.

---

## Cross‑cutting: test visibility and notes parity

- [ ] Ensure all preflight notes strings reflect test expectations (include keywords like `"mount"`, `"allowed by policy"`, etc.).
- [ ] Keep `production_preset()` strict and do not weaken production gates; adjust only `Policy::default()` or tests as needed.
- [ ] Update `cargo/switchyard/BUGS.md` with short "Fixed" notes once each item is closed.

---

## Execution order (recommended)

- [ ] 1) Provenance emissions (preflight+apply)
- [ ] 2) Locking/Smoke error mapping at `Switchyard::apply`
- [ ] 3) Mount notes + source trust default parity for baseline preflight
- [ ] 4) Backup tag handling (empty/coreutils)
- [ ] 5) Rollback planning facts
- [ ] 6) Rescue exec/min semantics
- [ ] 7) Final doc updates and close BUGS
