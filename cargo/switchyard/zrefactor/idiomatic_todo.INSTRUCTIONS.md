# Idiomatic Module/Layout Refactors — Actionable Steps

Perform these refactors to make the module graph idiomatic and remove legacy shims.

1) Make `src/api` an idiomatic directory module

- Move: `src/api.rs` → `src/api/mod.rs`
- In `src/api/mod.rs`, declare:
  - `mod apply;`
  - `pub mod errors;`
  - `mod plan;`
  - `mod preflight;`
  - `mod rollback;`
- Remove all `#[path = "api/..."]` attributes.
- Acceptance: `cargo check` and `cargo test` pass; `rg -n "#\[path\]" cargo/switchyard/src/api -S` returns 0.

2) Remove unused compatibility file

- /// remove this file: `src/policy/checks.rs` (already removed from module graph)
- Acceptance: `cargo check` passes.

3) Remove adapters legacy shim

- Files: `src/adapters/mod.rs`, tests using `switchyard::adapters::lock_file::FileLockManager`
- Update call sites to import `switchyard::adapters::FileLockManager`.
- Delete shim block:

  ```rust
  pub mod lock_file { pub use super::lock::file::*; }
  ```

- Acceptance: `rg -n "adapters::lock_file::FileLockManager" cargo/switchyard/src cargo/switchyard/tests -S` returns 0; `cargo test` passes.

4) Convert API leaf modules to directory modules (optional but recommended)

- Convert:
  - `src/api/errors.rs` → `src/api/errors/mod.rs`
  - `src/api/plan.rs` → `src/api/plan/mod.rs`
  - `src/api/rollback.rs` → `src/api/rollback/mod.rs`
- Acceptance: `cargo check` passes; no `#[path]` used.

5) Add module-level docs to key modules

- Files: `src/api/apply/mod.rs`, `src/api/preflight/mod.rs`, `src/api/plan/mod.rs`, `src/api/rollback/mod.rs`, `src/preflight.rs`
- Add one-paragraph summary of responsibilities and relationships at the top.
- Acceptance: docs exist and match current behavior.

6) Tighten visibilities

- Prefer `pub(crate)` for low-level FS atoms and internal helpers.
- Example: stop re-exporting `open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir` publicly in `src/fs/mod.rs`.
- Acceptance: `cargo check` passes; intended public API remains available.

7) Extract common syscall patterns into helpers

- Files: `src/fs/backup.rs`, `src/fs/restore.rs`, `src/fs/swap.rs`, `src/fs/atomic.rs`
- Factor repeated `open_dir_nofollow` + `renameat`/`unlinkat` sequences into helpers (e.g., `renameat_same_dir(dirfd, old, new)`).
- Acceptance: reduced duplication; behavior unchanged.

8) Deterministic backup naming in tests

- File: `src/fs/backup.rs`
- Introduce `Clock` trait with default `SystemClock`; allow tests to inject a fixed clock.
- Acceptance: tests assert backup names deterministically without directory scans.

9) Restore observability parity

- Files: `src/fs/restore.rs`, `src/api/apply/handlers.rs`
- Return `RestoreStats { fsync_ms: u64 }` or similar; include in emitted facts.
- Acceptance: emitted facts include restore duration; optional FSYNC warnings.

10) CI guardrails

- Add grep checks for:
  - lingering `#[path]` usage under `src/api/`
  - `adapters::lock_file::` path
  - public re-exports of `open_dir_nofollow`/`atomic_symlink_swap`/`fsync_parent_dir`

11) Verification

- After each change set:
  - Run `cargo check` and `cargo test -p switchyard`
  - Grep for removed paths and shims as above
  - Update docs as needed

/// remove this file: `zrefactor/idiomatic_todo.md` (narrative; replaced by this instructions file)
