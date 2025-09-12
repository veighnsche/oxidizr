# Switchyard Clean Code TODO

Date: 2025-09-12
Scope: `cargo/switchyard/`
References: `docs/CLEAN_CODE.md`, `cargo/switchyard/SPEC/SPEC.md`

## Status legend

- [x] done
- [ ] pending
- [~] in progress

## P0 — Non-breaking hardening (ready)

- [x] Replace panic-prone `CString::new(...).unwrap()` in production FS code
  - Files: `src/fs/atomic.rs`, `src/fs/swap.rs`, `src/fs/restore.rs`
- [x] Add `#[must_use]` to result types: `PreflightReport`, `ApplyReport`
- [x] Include `dry_run` in all facts via `api/audit.rs`
- [x] Add degraded telemetry details: `degraded_reason: "exdev_fallback"`
- [x] Include `lock_wait_ms` in apply summary facts
- [x] Operator logs via `api.audit.log(...)` at start/end and on errors

## P0 — Module refactors

- [x] Split FS responsibilities
  - [x] `fs/backup.rs` (sidecar + backup helpers)
  - [x] `fs/restore.rs` (restore orchestrator)
  - [x] `fs/swap.rs` (replace_file_with_symlink orchestrator)
  - [x] `fs/paths.rs` (is_safe_path)
  - [x] Deprecate/remove `fs/symlink.rs` (now deleted)
- [~] Extract `apply.rs` helpers
  - [x] `api/apply/gating.rs` → `enforce_policy_gates` (errors_for)
  - [x] `api/apply/handlers.rs` → action handlers
  - [x] `api/apply/audit_emit.rs` → per-action field/provenance helpers
- [x] Tidy `preflight.rs`
  - [x] Extract row-building helpers `api/preflight/report.rs`

## P1 — Observability and SPEC polish

- [x] Rescue check internal `Result` API (bool façade preserved)
- [ ] Add lock contention test asserting bounded wait and `lock_wait_ms` present
- [ ] CI: add clippy -D warnings and rustfmt checks specifically for switchyard (if not already in workflow)

## P1 — Error taxonomy & API clarity

- [ ] Expand `types/errors.rs` with granular kinds and structured fields (path, syscall, errno)

## P2 — Public surface and docs

- [ ] Tighten `pub(crate)` where feasible
- [ ] Add concise module docs for `types/*`
- [ ] Consider `tracing` spans around `plan`/`preflight`/`apply`/`rollback`

---

## Notes

- Public API remains stable: `plan`, `preflight`, `apply`, `plan_rollback_of`.
- Refactors were mechanical with tests green after each step.
- Next focus: finish `apply.rs` split (gating/handlers/audit helpers), then preflight `report.rs` extraction.
