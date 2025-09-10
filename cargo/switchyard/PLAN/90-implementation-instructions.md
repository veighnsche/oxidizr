# Implementation Instructions (Switchyard)

This document is a top-level, practical guide to implementing the Switchyard crate from the current planning package. It consolidates structure, order of operations, SPEC alignment, and CI evidence expectations.

Read these first:

- Plan structure: `PLAN/impl/00-structure.md`
- Migration: `PLAN/impl/05-migration.md`
- Phase-by-phase steps: `PLAN/impl/07-implementation-steps.md`
- SPEC: `cargo/switchyard/SPEC/SPEC.md` (requirements and schemas)

---

## What to Build First (Minimal Skeleton)

1. Create the module tree per `impl/00-structure.md` with empty modules and re-exports.
2. Add type stubs: `types/{plan,safepath,report,errors}` and `ApplyMode`.
3. Add adapter traits as empty interfaces under `adapters/*`.
4. Add logging structs and a JSONL sink stub (`logging/*`).
5. Ensure the crate builds (no behavior).

Exit: empty, compilable crate with modules.

---

## Core Foundations (Lock In Invariants)

1. Implement `SafePath` (REQ-API1, REQ-S1, REQ-TOCTOU1) per `impl/25-safepath.md`.
2. Implement `ErrorKind` and mapping to `SPEC/error_codes.toml` per `impl/30-errors-and-exit-codes.md`.
3. Implement deterministic IDs and ordering utilities per `impl/35-determinism.md`.
4. Implement `Fact` composition, redaction, and deterministic emission per `impl/40-facts-logging.md`.

Exit: unit tests for SafePath and basic facts serialization pass.

---

## Boundaries & Policy

1. Define adapter traits (`OwnershipOracle`, `LockManager`, `PathResolver`, `Attestor`, `SmokeTestRunner`) per `impl/20-adapters.md`.
2. Implement `policy/config.rs` with conservative defaults (REQ-C1,C2). See `impl/15-policy-and-adapters.md`.
3. Provide initial `Mock`/`No*` adapters for development and tests (e.g., `NoLockManager`).

Exit: crate compiles with trait-only adapters; policy available to API.

---

## Preflight & Rescue

1. Implement `rescue.rs` to verify fallback toolset and rescue profile per `impl/65-rescue.md`.
2. Implement `preflight.rs` per `impl/45-preflight.md`, producing `PreflightReport` entries conforming to `SPEC/preflight.yaml`.
3. Add YAML serialization and a CI check to validate preflight output against the schema.

Exit: deterministic preflight output with gating.

---

## Filesystem Engine (Atomicity, Rollback, EXDEV)

1. Implement `fs/atomic.rs` (TOCTOU-safe rename + fsync within 50ms), `fs/symlink.rs`, `fs/backup.rs`, and `fs/exdev.rs` per `impl/60-rollback-exdev.md` and `impl/55-operational-bounds.md`.
2. Integrate SHA-256 `before_hash`/`after_hash` into facts (REQ-O5).
3. Implement reverse-order rollback and partial restoration facts.

Exit: unit and property tests for atomicity and rollback pass locally.

---

## API Orchestration, Locking & Attestation

1. Implement API functions per `impl/70-pseudocode.md`.
2. Integrate `LockManager` bounded wait with `lock_wait_ms`; emit WARN in dev when missing (REQ-L2..L4). See `impl/50-locking-concurrency.md`.
3. Call `Attestor` post-apply and include signing info in final facts (REQ-O4).
4. Record `duration_ms` around rename+fsync (REQ-BND1).

Exit: end-to-end apply flow works with mocks/stubs, facts emitted.

---

## Health Verification & CI Evidence

1. Implement `SmokeTestRunner` adapter and integrate auto-rollback on failure (REQ-H1..H3).
2. Establish golden fixtures (plan, preflight, apply, rollback) and enforce zero-SKIP gate.
3. Run `SPEC/tools/traceability.py` and validate schemas in CI.

Exit: CI green with planned gates.

---

## Migration & Cleanup

1. Move logic from placeholders into the planned modules per `impl/05-migration.md`.
2. Remove placeholder modules once parity is achieved.
3. Finalize ADRs and polish docs.

Exit: planned structure realized; ready for tagged pre-release.

---

## Quick Checklist

- SafePath-only mutations and TOCTOU sequence implemented (REQ-API1, REQ-TOCTOU1).
- Deterministic UUIDv5 IDs and stable, redacted facts (REQ-D1, REQ-D2, REQ-VERS1).
- LockManager integration with bounded wait and WARN in dev (REQ-L1..L4).
- EXDEV degraded fallback with `degraded=true` facts and policy gating (REQ-F1..F2).
- Smoke suite and auto-rollback wired (REQ-H1..H3).
- Golden fixtures and zero-SKIP gate enforced in CI.
