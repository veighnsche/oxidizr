# Switchyard CODE_REVIEW_TODO

Date: 2025-09-12
Scope: `cargo/switchyard/`
References: `docs/CLEAN_CODE.md`, `cargo/switchyard/SPEC/SPEC.md`

This TODO is the authoritative plan to bring the Switchyard crate in full alignment with our Clean Code principles and SPEC v1.1. It prioritizes safety, maintainability, modularity, and observability. Items are grouped by theme and ordered by priority. Checked items are already implemented in this branch.

---

## P0 — Non-breaking hardening (ready to execute)

- [x] Replace panic-prone `CString::new(...).unwrap()` in production code
  - Files: `src/fs/atomic.rs`, `src/fs/symlink.rs`
  - Result: map NUL errors to `io::ErrorKind::InvalidInput`; propagate with `?`.
- [x] Add `#[must_use]` to important result types
  - Files: `src/types/report.rs` → `PreflightReport`, `ApplyReport`
- [x] Establish lint guardrails (non-test code)
  - File: `src/lib.rs`
  - Effects: deny `clippy::unwrap_used`, `clippy::expect_used`; warn `clippy::all,cargo,pedantic`.
- [x] Update review with concrete plan and metrics
  - File: `cargo/switchyard/CLEAN_CODE_REVIEW.md` (2025-09-12 addendum)
- [ ] Surface `dry_run` in emitted facts (observability)
  - Files: `src/api/audit.rs` (use `AuditMode.dry_run`), `src/api/preflight.rs`, `src/api/apply.rs`
  - Acceptance: every fact includes a `dry_run` boolean when applicable.
- [ ] Resolve current dead_code warnings (no behavior change)
  - `Switchyard.audit` (use for operator summaries or remove)
  - `AuditMode.dry_run` (use as above)
  - `emit_preflight_fact`, `emit_summary` (either call or remove; prefer calling)

## P0 — Module refactors (no behavior change)

- [ ] Split `src/fs/symlink.rs` (607 LOC) into focused modules under `src/fs/`
  - [ ] `fs/backup.rs`
    - `backup_path_with_tag`
    - `has_backup_artifacts`
    - `find_latest_backup_and_sidecar`
    - `BackupSidecar` + `read_sidecar`/`write_sidecar`
  - [ ] `fs/restore.rs`
    - `restore_file` and helpers
  - [ ] `fs/swap.rs` (optional)
    - `replace_file_with_symlink` (mechanism façade that calls `atomic` + `backup`)
  - [ ] Keep `fs/atomic.rs` as-is (TOCTOU-safe primitives)
  - [ ] Keep or move `is_safe_path` to a small `fs/paths.rs` utility
  - [ ] Wire `fs/mod.rs` re-exports; update call sites
  - Acceptance: tests pass; public API unchanged; LOC per file < ~300 where practical.

- [ ] Extract helpers in `src/api/apply.rs` (529 LOC)
  - [ ] `api/apply/gating.rs` → `enforce_policy_gates(...)`
  - [ ] `api/apply/handlers.rs` → `apply_symlink_action(...)`, `apply_restore_action(...)`
  - [ ] `api/apply/audit_emit.rs` → small helpers to build per-action audit maps
  - [ ] Keep `api/apply.rs` as façade delegating to submodules
  - Acceptance: no behavior change; functions <= ~80 LOC; tests pass.

- [ ] Tidy `src/api/preflight.rs` (286 LOC)
  - Note: non-mutating detectors already live in `src/preflight.rs`.
  - [ ] Extract row-building helpers (e.g., provenance/preservation assembly) into `api/preflight/report.rs`
  - [ ] Ensure stable ordering remains explicit and documented

## P1 — Observability and SPEC conformance polish

- [ ] Enrich facts with:
  - [ ] `dry_run` field on all stages
  - [ ] Explicit `degraded` reasons (EXDEV) when known
  - [ ] Lock metrics (`lock_wait_ms`) consistently at summary
- [ ] Locking tests: bounded wait and `E_LOCKING` mapping
  - Add integration test asserting timeout behavior and facts emission
- [ ] CI: clippy and fmt for this crate
  - GitHub Actions: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTFLAGS="-Dwarnings" cargo check`

## P1 — Error taxonomy and API clarity

- [ ] Expand `crate::types::errors::ErrorKind`
  - Add granular variants: `AtomicSwap`, `Fsync`, `Cstring`, `BackupSidecar`, `Xdev`, `OwnershipCheck`
  - Attach structured fields (path, syscall, errno) where possible
  - Ensure `ApiError` mapping and `ErrorId`/exit codes remain stable

## P2 — Rescue and environment hardening

- [ ] `rescue::verify_rescue_tools_with_exec_min` → internal `Result<RescueOk, RescueError>`
  - Keep public façade as `bool` for now
  - Add short timeouts for executable probes; avoid hangs on corrupt PATH
  - Emit audit fields for rescue profile selection

## P2 — Public surface and docs

- [ ] Tighten public API surface with `pub(crate)` where feasible
- [ ] Add concise module docs for `types/*` on invariants and usage
- [ ] Consider optional `tracing` spans around `plan`/`preflight`/`apply`/`rollback` with `plan_id`

---

## Mapping guide (who/what moves where)

- From `fs/symlink.rs` → `fs/backup.rs`:
  - `BackupSidecar`, `read_sidecar`, `write_sidecar`, `backup_path_with_tag`, `find_latest_backup_and_sidecar`, `has_backup_artifacts`
- From `fs/symlink.rs` → `fs/restore.rs`:
  - `restore_file` and subordinate helpers
- From `fs/symlink.rs` → `fs/swap.rs` (optional):
  - `replace_file_with_symlink` (mechanism that calls `fs/backup` + `fs/atomic`)
- Keep `fs/atomic.rs` unchanged (already modular and <100 LOC)
- Preflight row assembly helpers → `api/preflight/report.rs`
- Apply per-action execution and gating → `api/apply/handlers.rs`, `api/apply/gating.rs`

Acceptance criteria: all tests green; public API unchanged; per-file LOC and cyclomatic complexity lowered; code structure matches `docs/CLEAN_CODE.md` principles.

---

## Done in this branch

- [x] Removed `unwrap()` uses for `CString` in production FS code
- [x] Added `#[must_use]` to critical result types
- [x] Enabled clippy guardrails (non-test) and pedantic warnings
- [x] Fixed minor warning (unused `mut`) in sidecar path helper
- [x] Updated review document with metrics and plan

---

## Notes and risks

- Splitting `fs/symlink.rs` touches widely-used helpers. Proceed in small PRs:
  1) Introduce new modules + re-exports (no moves) →
  2) Move types/functions gradually with compiler assistance →
  3) Update imports and run tests between each step.
- Keep behavior identical; changes are mechanical.
- After refactor, consider promoting clippy warnings to `-D warnings` in CI.
