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

## Round 1 Peer Review (AI 2, 2025-09-12 15:06 +02:00)

**Claims Verified:**
- ✅ Preflight module structure: `src/api/preflight/mod.rs` exists (moved from single file)
- ✅ Apply module structure: `src/api/apply/mod.rs` exists (moved from single file)  
- ✅ Preflight checks split: `src/preflight/checks.rs` and `src/preflight/yaml.rs` exist
- ✅ No `src/policy/checks.rs` found - shim has been removed as claimed
- ❌ **Correction needed**: `src/api.rs` still exists as a file, not moved to `src/api/mod.rs` yet

**Key Citations:**
- File system shows `src/api/preflight/mod.rs` and `src/api/apply/mod.rs` exist
- `src/preflight.rs:7-10` uses `#[path]` delegation to submodules
- `src/api.rs` still exists as 9002-byte file, not moved to directory structure

**Summary of Edits:** Most claimed refactoring is complete, but the high-priority item "Make `src/api.rs` a module directory" remains pending. Updated status to reflect current state.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:06 +02:00

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Stable, idiomatic module paths for consumers and docs
  - Assumption (from doc): API modules follow directory-module conventions and avoid `#[path]` shims.
  - Reality (evidence): `src/api.rs` still exists as a single file delegator with `#[path]` attributes to `api/*` (see file presence and size in repo, 9002 bytes). This creates drift from the recommended structure and complicates navigation and Rustdocs.
  - Gap: Documentation and contributor expectations assume `src/api/mod.rs`; references can go stale and IDE tooling loses some affordances.
  - Mitigations: Perform the planned move of `src/api.rs` → `src/api/mod.rs` and replace `#[path]` with `mod ...;` declarations. Update imports and docs.
  - Impacted users: Contributors and downstream readers relying on conventional module layout and docs.
  - Follow-ups: Execute High-priority item in this doc; add a CI grep check to detect lingering `#[path]` usage under `src/api/`.

- Invariant: Legacy shims are removed before the next minor release
  - Assumption (from doc): `policy::checks` shim is gone; remaining shims will be cleaned up.
  - Reality (evidence): `src/policy/checks.rs` is absent as expected; however, `adapters::lock_file::*` shim still exists (`src/adapters/mod.rs` lines 6–9) to preserve an older import path.
  - Gap: Public shim risks prolonging duplicate import paths and documentation confusion.
  - Mitigations: Deprecate `adapters::lock_file::*` in Rustdoc now, and remove after one minor version; add a linter/grep gate in CI to prevent new usages; update samples.
  - Impacted users: Integrators importing legacy path; internal docs/refs.
  - Follow-ups: Track deprecation window in RELEASE_AND_CHANGELOG_POLICY.md; remove shim when window expires.

- Invariant: Public surface is minimal; low-level FS atoms are not consumer-facing
  - Assumption (from analysis): Low-level FS atoms are Internal-only.
  - Reality (evidence): `src/fs/mod.rs` publicly re-exports `open_dir_nofollow`, `atomic_symlink_swap`, and `fsync_parent_dir` (lines 9–15). These are footguns for consumers and bypass `SafePath` type.
  - Gap: External callers could misuse low-level atoms and violate TOCTOU invariants.
  - Mitigations: Mark these re-exports as `pub(crate)` and provide high-level `replace_file_with_symlink`/`restore_file` only; if removal is breaking, first deprecate with clear Rustdocs and changelog.
  - Impacted users: Power users calling low-level APIs directly.
  - Follow-ups: Coordinate with RELEASE_AND_CHANGELOG_POLICY.md to outline deprecation timeline; add a compile-fail doc test demonstrating intended usage.

- Invariant: Deterministic backup naming in tests
  - Assumption (from doc): Tests can rely on stable backup names.
  - Reality (evidence): `backup_path_with_tag()` uses `SystemTime::now()` (`src/fs/backup.rs` lines 18–23), producing non-deterministic names; tests scan directory to find latest.
  - Gap: Flaky assertions/golden fixtures possible around timestamp ordering.
  - Mitigations: Introduce a `Clock` trait with a default `SystemClock`; allow tests to inject a fixed clock for deterministic names.
  - Impacted users: Test authors and CI stability.
  - Follow-ups: Implement the proposed trait and refactor callers; update tests and docs accordingly.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00
