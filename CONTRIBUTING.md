# Contributing Guide

This project operates under a Clean Code & Safety posture for high‑risk system utilities. Please read:

- `CLEAN_CODE.md`
- `PROJECT_PLANS/README.md` (Streams A–E)
- `AUDIT_AND_LOGGING.md`
- `AUDIT_CHECKLIST.md`

## Authoritative Modules & Reuse Rules

Single authorities are enforced. Do not re‑implement these concerns elsewhere:

- Filesystem link/backup/restore and atomic swap:
  - Authority: `src/symlink/ops.rs`
  - Callers must use: `replace_file_with_symlink(...)`, `restore_file(...)`
  - Pointer flip (planned): `rename_active_pointer(...)` will be added here
- Structured audit JSONL and human logs:
  - Authority: `src/logging/{init.rs,audit.rs}`
  - Callers must use: `audit_event_fields(...)` or `audit_op(...)`
  - Do not create new tracing layers/sinks outside `logging/init.rs`
- System/package/PATH operations:
  - Authority: `src/system/worker/*.rs`
  - Callers must use: `Worker::which(...)` for PATH lookups (no direct `which::which`)
  - Repository/AUR policy and lock‑wait UX live in `src/system/worker/packages.rs`
- Experiment orchestration and planning:
  - Authority: `src/experiments/*` and `src/experiments/util.rs`
  - Preflight and plan rendering must reuse discovery + util renderer
- State/reporting and progress UI:
  - Authority: `src/state/`, `src/ui/progress.rs`

## Refactor Policy (Pre‑Release)

- Breaking changes are allowed until the first public release.
- Backwards compatibility and shims may be removed to simplify the codebase.

## Coding Standards (short form)

- Follow `CLEAN_CODE.md` for architecture, error handling, logging, filesystem safety, testing, and review.
- New features should be additive and respect single authorities. If you need a new capability, extend the authority rather than creating a parallel path.
- Use `tracing` for logs; prefer `audit_event_fields` with `AuditFields` for audit entries.
- No `unwrap`/`expect` in library code; return rich `Error` variants.
- Tests: unit on pure logic; integration with tmp roots; E2E via `test-orch/`.

## Guardrails (CI‑enforced)

The CI pipeline enforces the following rules:

1) PATH lookup centralization

- Only `src/system/worker/fs_ops.rs` may reference `which::which`.
- All PATH lookups use `Worker::which(...)`.

2) Single logging sink

- No `tracing_subscriber::fmt` layer setup outside `src/logging/init.rs`.

3) Single symlink implementation

- No direct `std::os::unix::fs::symlink` calls outside `src/symlink/ops.rs`.

Violations will fail CI. If you need to change an authority or temporarily relax a guardrail, coordinate in your PR description with a plan to restore the rule.
