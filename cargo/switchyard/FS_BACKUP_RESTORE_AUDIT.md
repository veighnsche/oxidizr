# Switchyard Backup/Restore Inventory and Preflight Structure

Date: 2025-09-12

This document catalogs all backup/restore-related functions and explains the relationship between the top-level `preflight.rs` and the API-layer `api/preflight.rs`/`api/preflight/`. It also proposes refactor opportunities.

## Summary Counts

- Core filesystem backup operation: 1
  - `fs::backup::create_snapshot()`
- Core filesystem restore operations: 2
  - `fs::restore::restore_file()`
  - `fs::restore::restore_file_prev()`
- Backup/sidecar helpers and queries: 6
  - `fs::backup::backup_path_with_tag()`
  - `fs::backup::find_latest_backup_and_sidecar()`
  - `fs::backup::find_previous_backup_and_sidecar()`
  - `fs::backup::has_backup_artifacts()`
  - `fs::backup::sidecar_path_for_backup()`
  - `fs::backup::read_sidecar()` / `fs::backup::write_sidecar()`
- API-layer restore handler: 1
  - `api::apply::handlers::handle_restore()` (calls the filesystem restore ops and emits audit facts)

Notes:

- `fs::swap::replace_file_with_symlink()` is not itself a backup function, but it triggers `create_snapshot()` before swapping to preserve prior state.
- Integration/unit tests exist under `src/fs/*` and `tests/` that cover success and error cases (e.g., `error_restore_failed.rs`, `error_exdev.rs`).

## Function Inventory (with paths)

- `cargo/switchyard/src/fs/backup.rs`
  - `backup_path_with_tag(target, tag) -> PathBuf` (public)
  - `create_snapshot(target, tag) -> io::Result<()>` (public) — the main backup operation; creates payload (.bak) and sidecar (.bak.meta.json) for file/symlink/none topologies
  - `has_backup_artifacts(target, tag) -> bool` (public) — checks presence of backup payload and/or sidecar
  - `find_latest_backup_and_sidecar(target, tag) -> Option<(Option<PathBuf>, PathBuf)>` (pub(crate))
  - `find_previous_backup_and_sidecar(target, tag) -> Option<(Option<PathBuf>, PathBuf)>` (pub(crate))
  - `sidecar_path_for_backup(backup) -> PathBuf` (pub(crate))
  - `read_sidecar(sc_path) -> io::Result<BackupSidecar>` (pub(crate))
  - `write_sidecar(backup, &BackupSidecar) -> io::Result<()>` (pub(crate))
  - `BackupSidecar` schema struct (serde Serialize/Deserialize)

- `cargo/switchyard/src/fs/restore.rs`
  - `restore_file(target, dry_run, force_best_effort, backup_tag) -> io::Result<()>` (public)
  - `restore_file_prev(target, dry_run, force_best_effort, backup_tag) -> io::Result<()>` (public)
  - Both support sidecar-driven restores for file/symlink/none kinds, idempotent short-circuiting, and legacy rename fallback when sidecar missing.

- `cargo/switchyard/src/fs/swap.rs`
  - `replace_file_with_symlink(source, target, dry_run, allow_degraded, backup_tag) -> io::Result<(bool, u64)>`
    - Calls `create_snapshot()` for file/symlink/none cases prior to mutation; performs atomic swap and returns whether EXDEV degraded path was used and duration.

- `cargo/switchyard/src/api/apply/handlers.rs`
  - `handle_restore(api, tctx, pid, act, idx, dry) -> (Option<Action>, Option<String>)` (crate-private)
    - API-level orchestration handler for `Action::RestoreFromBackup`.
    - Optionally takes a fresh snapshot when `policy.capture_restore_snapshot` is true, then prefers restoring from the “previous” snapshot; falls back to “latest” on NotFound.
    - Emits structured audit facts and error IDs (`E_BACKUP_MISSING`, `E_RESTORE_FAILED`).

## Where they are used

- `replace_file_with_symlink()` calls `create_snapshot()` before swapping to preserve state.
- `api::preflight::run()` uses `fs::has_backup_artifacts()` to annotate preflight rows for `Action::RestoreFromBackup`.
- `api::apply::handlers::handle_restore()` calls `fs::restore_file_prev()` or `fs::restore_file()` based on policy.

## Preflight modules: purpose and layering

- `cargo/switchyard/src/preflight.rs` (crate-level helpers)
  - Provides low-level checks used by policy and preflight: `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust`.
  - Provides `to_yaml(report)` exporter to render SPEC-aligned YAML from `PreflightReport` rows.

- `cargo/switchyard/src/api/preflight.rs` (API stage orchestrator)
  - Orchestrates the preflight stage over a `Plan`: runs policy gating, preservation probes, ownership checks, and per-action row emission.
  - Emits a preflight summary fact and returns a `PreflightReport` with stable ordering for YAML export (via `preflight::to_yaml()`).
  - Depends on `api/preflight/rows.rs` for row-building and audit emission details.

- `cargo/switchyard/src/policy/checks.rs`
  - Currently a compatibility shim that re-exports the check functions from `crate::preflight` (so callers can migrate):
    - `pub use crate::preflight::{check_immutable, check_source_trust, ensure_mount_rw_exec};`

Conclusion: these modules are not doing “parallel” duplicate work; they are layered:

- `preflight.rs` hosts generic checks and YAML export.
- `api/preflight.rs` is the API’s preflight stage.
- `api/preflight/rows.rs` is a private helper for the API preflight stage’s row construction.
- `policy/checks.rs` is a transitional re-export to decouple policy code from the old `crate::preflight` namespace.

## Refactor opportunities

1) Deduplicate restore logic

- Problem: `restore_file` and `restore_file_prev` are near-identical except for how they pick the backup pair.
- Proposal: factor a private `do_restore(target, dry_run, force_best_effort, pair: Option<(Option<PathBuf>, PathBuf)>)` and/or a small enum `Which { Latest, Previous }` passed into a single `restore_file_with(target, dry_run, force, tag, which)`.
- Benefit: reduces code duplication, eases maintenance, and ensures identical bugfixes/features for both paths.

2) Clarify preflight module boundaries

- Problem: `policy/checks.rs` re-exports `crate::preflight::*`, which is a transitional shim and can be confusing.
- Proposal A (minimal): move the three checks into `policy/checks.rs` proper and have `preflight.rs` import from there. Keep `preflight::to_yaml()` where it is but document clearly.
- Proposal B (clean layering): create `src/preflight/mod.rs` with `checks.rs` and `yaml.rs`. Make `api/preflight.rs` depend on `preflight::checks::*`. Remove the shim in `policy/checks.rs` after call sites migrate.
- Benefit: removes name duplication and clarifies ownership of checks (policy) vs API-stage orchestration.

3) Sidecar schema and constants

- Problem: string literals like `"backup_meta.v1"` appear inline.
- Proposal: centralize schema version and keys in a `fs::backup::schema` module with constants and a typed newtype for `BackupTag`.
- Benefit: avoids string drift and allows easier schema evolution/migration.

4) Deterministic/testable backup naming

- Problem: `backup_path_with_tag()` uses `SystemTime::now()`, which makes timestamped names non-deterministic in tests and complicates reproducibility.
- Proposal: inject a `Clock` trait (or parameter) with a default to `SystemTime`, and use a fixed clock in tests. Alternatively, allow the policy to carry an optional `timestamp_override` for dry-run or test builds.
- Benefit: predictable fixtures and easier to assert backup file names.

5) Uniform duration metrics for restore

- Problem: restore paths don’t surface `fsync` duration (unlike `replace_file_with_symlink()` which returns `(bool, u64)` and feeds FSYNC warnings).
- Proposal: return a `RestoreStats { fsync_ms: u64 }` from restore operations, and have `handle_restore()` include duration in emitted facts (plus FSYNC WARN gating if desired).
- Benefit: observability parity between apply and restore.

6) Extract common path operations

- Problem: repeated sequences for `open_dir_nofollow` + `CString` + `renameat`/`unlinkat` appear in multiple places across backup/restore/swap.
- Proposal: extract small helpers in `fs::atomic` for common patterns (e.g., `renameat_same_dir(dirfd, old_name, new_name)`) to reduce error-prone boilerplate.
- Benefit: smaller, clearer code, less chance of subtle differences.

7) Strengthen invariants around symlink restore

- Idea: when `prior_kind == "symlink"`, add an extra validation step to compare canonicalized targets as already done, but also record whether the original was relative/absolute and restore in the same form (where possible) to preserve topology aesthetics.

8) Tests coverage

- Add explicit tests for `restore_file_prev()` success and NotFound fallback behavior (API handler already does the fallback, but direct FS tests would help).
- Add tests asserting `has_backup_artifacts()` behavior for the three topologies (file/symlink/none).

9) Naming polish

- Consider renaming `preflight.rs` to `preflight_checks.rs` or creating `preflight/yaml.rs` to make intent obvious and reduce the appearance of duplication with `api/preflight.rs`.

## Quick answers to the original questions

- How many backup and restore methods?
  - 1 core backup operation (`create_snapshot`), 2 core restore operations (`restore_file`, `restore_file_prev`).
  - 6 additional backup/sidecar helpers and queries (plus the sidecar read/write helpers), and 1 API-layer restore handler.
- Why do we have `src/preflight.rs`, `src/api/preflight.rs`, and `src/api/preflight/`?
  - They are layered, not redundant: `preflight.rs` hosts generic checks and YAML exporting; `api/preflight.rs` orchestrates the API preflight stage; `api/preflight/rows.rs` contains internal row-construction logic for that stage. `policy/checks.rs` is a temporary re-export shim.

## Suggested next steps

- PR1: Factor restore duplication (`restore_file*`) and add unit tests for both variants through the new shared path.
- PR2: Move check functions into `policy/checks.rs` proper and update imports; restrict `preflight.rs` to YAML exporting (or split into `preflight/{checks,yaml}.rs`). Remove re-export shim after migration.
- PR3: Centralize sidecar schema constants and add a typed `BackupTag`.
- PR4: Optional: introduce `Clock` abstraction for `backup_path_with_tag()` to improve testability.
- PR5: Optional: emit restore durations and FSYNC warnings in API facts for observability parity.
