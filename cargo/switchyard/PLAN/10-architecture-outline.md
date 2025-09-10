# Architecture Outline

## Modules (current inventory)

- `src/lib.rs`: crate entry/export surface.
- `src/api.rs`: high-level operations API (orchestration layer).
- `src/fs_ops.rs`: safe filesystem operations, atomic swaps.
- `src/preflight.rs`: safety preconditions and environment checks.
- `src/symlink.rs`: symlink creation/replacement with safety guarantees.

## Responsibilities

- Enforce SafePath and TOCTOU-safe patterns (open parent O_DIRECTORY|O_NOFOLLOW → openat → renameat → fsync parent).
- Emit audit events per `SPEC/audit_event.schema.json`.
- Offer deterministic behaviors required by SPEC features.

## Interfaces & Data Contracts

- Public API: functions exposed by `api.rs` (to be finalized in SPEC traceability).
- Audit event schema: `SPEC/audit_event.schema.json` (validation in CI).

## Failure Domains

- Filesystem boundaries and cross-device moves (support degraded mode when configured).
- Locking and concurrency around file swaps.

## Open Questions

- Final public API surface area and versioning strategy.
- Boundaries between `api.rs` and lower-level modules.
