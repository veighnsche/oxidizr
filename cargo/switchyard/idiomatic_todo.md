# Idiomatic Cleanups and Refactors: TODO

Generated: 2025-09-12

This document tracks the remaining idiomatic Rust module/layout cleanups and a few code-quality improvements. Items are grouped by status with concrete file paths and acceptance criteria.

## Completed in this branch

- [x] Preflight module made idiomatic
  - Moved `src/api/preflight.rs` → `src/api/preflight/mod.rs`
  - Referenced `rows.rs` via `mod rows;` (no more `#[path]`)
  - Updated `src/api.rs` to point to `api/preflight/mod.rs`
  - Old file removed (by user): `src/api/preflight.rs`

- [x] Apply module made idiomatic
  - Moved `src/api/apply.rs` → `src/api/apply/mod.rs`
  - Internal modules included via `mod audit_fields; mod handlers;`
  - Updated `src/api.rs` to point to `api/apply/mod.rs`
  - Old file removed (by user): `src/api/apply.rs`

- [x] Preflight checks split and shim removed
  - Created `src/preflight/checks.rs` and `src/preflight/yaml.rs`
  - Updated `src/preflight.rs` to delegate and re-export
  - Updated all imports to `crate::preflight::checks::*`
  - Removed `policy::checks` from module tree (`src/policy/mod.rs`); `cargo check` passes

## High-priority (proposed next)

- [ ] Make `src/api.rs` a module directory
  - Action: Move `src/api.rs` → `src/api/mod.rs`
  - Replace all `#[path = "api/..." ]` attributes with idiomatic `mod ...;`
  - After move, submodules should be declared:
    - `mod apply;`
    - `pub mod errors;`
    - `mod plan;`
    - `mod preflight;`
    - `mod rollback;`
  - Acceptance: `cargo check` and `cargo test` pass; no `#[path]` in `src/api/mod.rs`.

- [ ] Physically delete unused compatibility file
  - File: `src/policy/checks.rs`
  - Note: already removed from the module graph; delete to avoid confusion
  - Acceptance: `cargo check` passes.

- [ ] Deprecate top-level rescue re-export
  - File: `src/lib.rs`
  - Change: Annotate `pub use policy::rescue;` with `#[deprecated = "use switchyard::policy::rescue"]`
  - Acceptance: builds with a deprecation warning for any in-tree use; document in CHANGELOG for next major removal.

- [ ] Remove adapters legacy shim after migration
  - Files: `src/adapters/mod.rs`, tests using `switchyard::adapters::lock_file::FileLockManager`
  - Steps:
    - Update tests and in-tree code to import `switchyard::adapters::FileLockManager`
    - Remove the shim block:

      ```rust
      pub mod lock_file { pub use super::lock::file::*; }
      ```

  - Acceptance: `grep -R "adapters::lock_file::FileLockManager"` returns 0; `cargo test` passes.

## Medium-priority cleanup

- [ ] Consistent directory modules for API leaf files (optional)
  - Convert single-file modules if we expect future growth:
    - `src/api/errors.rs` → `src/api/errors/mod.rs`
    - `src/api/plan.rs` → `src/api/plan/mod.rs`
    - `src/api/rollback.rs` → `src/api/rollback/mod.rs`
  - Acceptance: `cargo check` passes; no `#[path]` needed.

- [ ] Module-level docs for key modules
  - Files: `src/api/apply/mod.rs`, `src/api/preflight/mod.rs`, `src/api/plan.rs`, `src/api/rollback.rs`, `src/preflight.rs`
  - Add a clear, single-paragraph summary at top of each file to document responsibilities and relationships.
  - Acceptance: docs exist and match current behavior.

- [ ] Tighten visibilities
  - Review public items and prefer `pub(crate)` where feasible (to minimize surface area):
    - Example: fields in `fs::backup::BackupSidecar`
    - Example: `fs::backup::backup_path_with_tag` (if not intended for external consumers)
  - Acceptance: `cargo check` passes; no breaking changes to intended public API.

## Engineering improvements (optional but valuable)

- [ ] Extract common syscall patterns into helpers
  - Files: `src/fs/backup.rs`, `src/fs/restore.rs`, `src/fs/swap.rs`, `src/fs/atomic.rs`
  - Factor repeated `open_dir_nofollow` + `CString` + `renameat` / `unlinkat` sequences into small helpers (e.g., `renameat_same_dir(dirfd, old, new)`).
  - Acceptance: reduced duplication; behavior unchanged.

- [ ] Deterministic backup naming for tests
  - File: `src/fs/backup.rs` (`backup_path_with_tag`)
  - Introduce a `Clock` trait with a default `SystemClock`; allow tests to inject a fixed clock for deterministic names.
  - Acceptance: tests can assert backup names deterministically without relying on filesystem scans.

- [ ] Restore observability parity
  - Files: `src/fs/restore.rs`, `src/api/apply/handlers.rs`
  - Return a `RestoreStats { fsync_ms: u64 }` (or surface metrics) from restore operations and include in emitted facts (similar to `replace_file_with_symlink`).
  - Acceptance: emitted facts include restore duration; optional FSYNC warnings when over threshold.

## Tracking and verification

- After each change set:
  - Run `cargo check` and `cargo test -p switchyard`
  - Grep for old paths when removing shims:
    - `grep -R "policy::checks::" src/` (already 0)
    - `grep -R "adapters::lock_file::"` (should be 0 after migration)
  - Update docs:
    - `REEXPORTS_AND_FACADES.md`
    - `BACKWARDS_COMPAT_SHIMS.md`
