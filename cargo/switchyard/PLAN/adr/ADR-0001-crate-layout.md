# ADR Template

- Title: Crate layout and module boundaries
- Status: Accepted
- Date: 2025-09-10

## Context

We need a clear crate structure that enforces separation of concerns: orchestration vs low-level FS ops vs safety preflight. The SPEC mandates SafePath-only mutations, TOCTOU-safe sequences, determinism, audit facts, and adapters for environment-specific behavior.

## Decision

- Keep a single library crate `switchyard` with modules:
  - `api` (orchestration; `plan`, `preflight`, `apply`, `plan_rollback_of`)
  - `fs_ops` (TOCTOU-safe primitives; atomic rename; EXDEV fallback; backup/restore)
  - `preflight` (safety and policy gating)
  - `symlink` (safe symlink operations)
  - `types` (SafePath, Plan, PlanInput, reports; planning-only for now)
- Public surface lives in `lib.rs` and re-exports from `api` and `types`.
- Adapters defined as traits under `api` or `adapters` module (OwnershipOracle, LockManager, PathResolver, Attestor, SmokeTestRunner).

## Consequences

+ Clear boundaries for testing and ownership.
+ Aligns with SPEC sections 3.1–3.3 and requirement groupings.
- Requires careful dependency direction to keep `fs_ops` free of higher-level policy.

## Links

- `cargo/switchyard/PLAN/10-architecture-outline.md`
- `cargo/switchyard/SPEC/SPEC.md` §§ 3, 4, 5
