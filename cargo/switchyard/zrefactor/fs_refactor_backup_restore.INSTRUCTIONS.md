# FS Backup/Restore Refactor — Actionable Steps (breaking)

> STATUS: Not landed in src/. `src/fs/backup.rs` (~17k) and `src/fs/restore.rs` (~30k) remain monoliths; no split modules exist yet. Keep PRs refactor-only.

Do these changes. Update call sites; remove legacy files at the end.

1) Create modules under `src/fs/backup/`

- Files to add:
  - `src/fs/backup/mod.rs` (re-exports; public entry)
  - `src/fs/backup/snapshot.rs` (create_snapshot, backup_path_with_tag, has_backup_artifacts)
  - `src/fs/backup/sidecar.rs` (BackupSidecar, read/write, sidecar_path_for_backup)
  - `src/fs/backup/index.rs` (find_latest_backup_and_sidecar, find_previous_backup_and_sidecar)
- Extract functions from `src/fs/backup.rs` into the above files.
- Update `src/fs/mod.rs` to `pub mod backup;` and re-export the public functions.
- Acceptance: `cargo check` passes; unit tests compile in their new locations.

2) Create modules under `src/fs/restore/`

- Files to add:
  - `src/fs/restore/mod.rs` (public entry)
  - `src/fs/restore/types.rs` (SnapshotSel, RestoreOptions, PriorKind)
  - `src/fs/restore/selector.rs`
  - `src/fs/restore/idempotence.rs`
  - `src/fs/restore/integrity.rs`
  - `src/fs/restore/steps.rs`
  - `src/fs/restore/engine.rs` (restore_impl entry used by public functions)
- Extract logic from `src/fs/restore.rs` into the above files.
- Update `src/fs/mod.rs` to `pub mod restore;` and re-export the public functions you keep or rename.
- Acceptance: `cargo check` passes; public API calls build after import updates.

3) Update public API and imports

- If keeping names: `restore_file`, `restore_file_prev` delegate to `engine::restore_impl(sel, opts)`.
- If renaming: provide new names using `SnapshotSel`/`RestoreOptions`; update call sites.
- Update all imports across the crate to the new module paths.
- Acceptance: `grep -R "use crate::fs::backup"` and `use crate::fs::restore` resolve everywhere; `cargo test` passes.

4) Move and add tests

- Move unit tests from old files into their corresponding modules under `#[cfg(test)]`.
- Add focused tests for:
  - index selection (latest/previous)
  - idempotence short-circuit
  - integrity verify (ok/mismatch)
  - steps: file bytes + mode, symlink, absent, legacy rename
- Acceptance: new unit tests pass locally; integration tests updated to new API where changed.

5) Small extractions

- Extract repeated idempotence check into `restore::idempotence::is_idempotent`.
- Extract `open_dir_nofollow + renameat + fsync` into `restore::steps::legacy_rename` and reuse.
- Extract file-bytes restore (including mode set) into `restore::steps::restore_file_bytes`.
- Acceptance: code duplication reduced; helpers used in all branches.

6) CI and cleanup

- Add module docs and rustdoc on public functions.
- /// remove this file: `src/fs/backup.rs` (after full extraction and re-exports in place)
- /// remove this file: `src/fs/restore.rs` (after full extraction and re-exports in place)
- Acceptance: `grep -R "src/fs/backup.rs"` returns 0 in code references; same for `restore.rs`.

- Remove internal re-export lines for low-level atoms in `src/fs/mod.rs` (e.g., `pub(crate) use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};`). Call these via `fs::atomic` from inside the crate instead of re-exporting at the module root.
- Acceptance: `rg -n "^\s*pub\(crate\)\s+use\s+atomic::\{" cargo/switchyard/src/fs/mod.rs -S` returns 0.

---

## Meta

- Scope: Split monolithic FS backup/restore into cohesive submodules; update public facade
- Status: Breaking allowed (pre-1.0)
- Index: See `zrefactor/README.md`

## Related

- Cohesion targets and FS layout: `zrefactor/responsibility_cohesion_report.md`
- API call sites touching FS: `zrefactor/api_refactor.INSTRUCTIONS.md`
- Logging facade used by stages: `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Removal planning and registry: `zrefactor/backwards_compat_removals.md`, `zrefactor/removals_registry.md`
