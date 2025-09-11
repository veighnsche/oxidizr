# Proposed Crate Structure (Planning Only)

This is a planning artifact for `cargo/switchyard`. No Rust implementation is included here. Paths and modules are proposed for high-level review.

## Folder / Module Tree

- `cargo/switchyard/`
  - `src/`
    - `lib.rs` — crate root; re-exports public API and core types
    - `api.rs` — orchestration entrypoints (`plan`, `preflight`, `apply`, `plan_rollback_of`)
    - `preflight.rs` — environment and policy gating; preflight diff generation
    - `rescue.rs` — rescue profile checks and fallback toolset verification (GNU/BusyBox)
    - `types/`
      - `mod.rs` — type exports
      - `plan.rs` — `PlanInput`, `Plan`, `Action`, `ApplyMode`
      - `safepath.rs` — `SafePath`
      - `report.rs` — `PreflightReport`, `ApplyReport`
      - `errors.rs` — library error enum mapping SPEC taxonomy
    - `adapters/`
      - `mod.rs` — adapter trait exports
      - `ownership.rs` — `OwnershipOracle`
      - `lock.rs` — `LockManager`, `LockGuard`
      - `path.rs` — `PathResolver`
      - `attest.rs` — `Attestor`
      - `smoke.rs` — `SmokeTestRunner`
    - `logging/`
      - `facts.rs` — JSONL fact builders (schema v1)
      - `redact.rs` — secret masking and timestamp redaction
      - `provenance.rs` — provenance fields and policy
    - `determinism/`
      - `ids.rs` — UUIDv5 `plan_id`/`action_id`
      - `ordering.rs` — stable field ordering; golden determinism
    - `fs/`
      - `atomic.rs` — rename flow, fsync bounds, symlink replacement entrypoints
      - `exdev.rs` — degraded fallback (copy+sync+rename)
      - `backup.rs` — backup/restore of previous targets
      - `symlink.rs` — helpers specific to symlink creation/replacement under TOCTOU-safe sequence
    - `policy/`
      - `config.rs` — policy flags & defaults (dry-run, strict ownership, degraded)
      - `flags.rs` — CLI/config parsing (if ever exposed by consumers)

- `SPEC/` (already present)
- `PLAN/` (this planning)
  - `impl/` (this folder)

## Notes

- Mutating APIs accept `SafePath` only (REQ-API1).
- All mutations must follow TOCTOU-safe sequence (REQ-TOCTOU1).
- Production requires `LockManager` with bounded wait (REQ-L1..L4).
- Facts follow `SPEC/audit_event.schema.json` (REQ-O1..O7, REQ-VERS1).
- EXDEV fallback per `ADR-0011-exdev-degraded-mode.md`.
- Rescue profile always available; preflight verifies fallback toolset on PATH (GNU or BusyBox) (REQ-RC1..RC3).

### Implementation Notes (Rustix & Unsafe Ban)

- System calls are performed via `rustix`, not `libc` nor raw FDs. We use:
  - `openat` with `OFlags::DIRECTORY | OFlags::NOFOLLOW` for parent handles.
  - `symlinkat`, `renameat`, and `unlinkat` for final-component operations.
  - Capability-style API: operations are expressed relative to an `OwnedFd` for the parent directory; no ambient absolute path traversal.
- The crate has `#![forbid(unsafe_code)]` at `src/lib.rs`. Any attempt to introduce `unsafe` will fail the build.
- The FS layer records `fsync(parent)` timings and surfaces telemetry to the API for WARN-on-breach of the ≤50ms bound (REQ-BND1).

## Placeholder Migration Mapping (planning)

Current placeholder files under `src/` will be reorganized into the proposed modules as follows. This planning intentionally does not constrain itself to the current placeholder layout.

- `src/fs_ops.rs` → `src/fs/atomic.rs` (atomic rename flow), `src/fs/backup.rs` (backup/restore), `src/fs/exdev.rs` (cross-fs fallback)
- `src/symlink.rs` → `src/fs/symlink.rs` (symlink-specific helpers used by `fs/atomic.rs`)
- `src/preflight.rs` → keep filename or evolve into `src/preflight.rs` + potential submodules if it grows (schema remains in SPEC)
- `src/api.rs` → remains orchestration but delegates to the new `fs/*`, `logging/*`, `determinism/*`, and `adapters/*` modules
