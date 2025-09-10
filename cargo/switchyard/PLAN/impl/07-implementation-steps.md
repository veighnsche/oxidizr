# Step-by-Step Implementation Plan (From Plan to Code)

This document provides a concrete, incremental sequence to implement the Switchyard crate according to the planned modular structure and SPEC. It assumes the placeholder files in `src/` are not normative and will be migrated per `impl/05-migration.md`.

References:

- Structure: `impl/00-structure.md`, `impl/05-migration.md`
- SPEC: `cargo/switchyard/SPEC/SPEC.md`, `requirements.yaml`
- Key planning docs: `impl/25-safepath.md`, `impl/30-errors-and-exit-codes.md`, `impl/35-determinism.md`, `impl/40-facts-logging.md`, `impl/45-preflight.md`, `impl/50-locking-concurrency.md`, `impl/55-operational-bounds.md`, `impl/60-rollback-exdev.md`, `impl/65-rescue.md`, `impl/70-pseudocode.md`

---

## Phase 0 — Scaffold the Module Tree (No Behavior)

1. Create empty modules under `src/` as per `impl/00-structure.md`:
   - `types/{plan.rs,safepath.rs,report.rs,errors.rs}`
   - `adapters/{ownership.rs,lock.rs,path.rs,attest.rs,smoke.rs}`
   - `logging/{facts.rs,redact.rs,provenance.rs}`
   - `determinism/{ids.rs,ordering.rs}`
   - `fs/{atomic.rs,backup.rs,exdev.rs,symlink.rs}`
   - `policy/config.rs`, `preflight.rs`, `rescue.rs`, `api.rs`, `lib.rs`
2. Wire `lib.rs` to re-export public API and types (empty structs/enums, feature-gated if needed).
3. Build to ensure the new module skeleton compiles (use `todo!()` or minimal stubs).

Exit: crate builds with stubs only.

---

## Phase 1 — Core Foundations

1. SafePath (REQ-API1, REQ-S1, REQ-TOCTOU1)
   - Implement `types/safepath.rs` per `impl/25-safepath.md` (constructors, normalization).
   - Unit tests for normalization, `..` rejection, root escape.
2. Errors & Exit Codes (SPEC §6)
   - Implement `types/errors.rs` with `ErrorKind` and helpers per `impl/30-errors-and-exit-codes.md`.
   - Map to `SPEC/error_codes.toml`.
3. Determinism (REQ-D1, REQ-D2)
   - Implement `determinism/ids.rs` for UUIDv5 plan/action IDs.
   - Implement `determinism/ordering.rs` utilities for stable ordering.
4. Facts & Logging (REQ-O1..O7, REQ-VERS1)
   - Implement `logging/facts.rs` with a `Fact` builder and JSONL sink stub.
   - Implement `logging/redact.rs` for dry-run timestamp redactions and masking.
   - Implement `logging/provenance.rs` (struct and masking hooks).

Exit: Unit tests pass for SafePath; basic serialization compiles for Facts.

---

## Phase 2 — Adapters & Policy Contracts

1. Adapters (interfaces only): `adapters/*`
   - Define traits per `impl/20-adapters.md`.
   - Provide a simple `Noop` or `Mock` impls for tests.
2. Policy defaults
   - Implement `policy/config.rs` with `PolicyFlags` and conservative defaults (REQ-C1,C2).
3. Locking contract
   - Define `LockManager` interface and a `NoLockManager` dev/test stub (emits WARN).

Exit: crate compiles with trait-only adapters; policy available to API layer.

---

## Phase 3 — Preflight & Rescue

1. Rescue checks
   - Implement `rescue.rs` basics (probe symlink set, PATH fallback toolset) per `impl/65-rescue.md`.
2. Preflight
   - Implement `preflight.rs` per `impl/45-preflight.md` to generate `PreflightReport` entries.
   - Include rescue verification gating (REQ-RC2), ownership checks, FS flags, preservation caps (stubs acceptable initially).
3. Schema alignment
   - Add YAML serialization for preflight rows and a CI step to validate against `SPEC/preflight.yaml`.

Exit: preflight builds and can emit deterministic rows in tests.

---

## Phase 4 — Filesystem Engine (Atomicity, Rollback, EXDEV)

1. FS primitives
   - Implement `fs/atomic.rs` with TOCTOU-safe parent opens, `rename` + `fsync(parent)` (REQ-TOCTOU1, REQ-BND1).
   - Implement `fs/symlink.rs` helpers used by atomic flow.
   - Implement `fs/backup.rs` for backup/restore.
   - Implement `fs/exdev.rs` fallback with policy gating and `degraded=true` facts (REQ-F1..F2).
2. Hashes & integrity
   - Integrate SHA-256 before/after hashes into facts (REQ-O5).
3. Rollback
   - Implement reverse-order rollback per `impl/60-rollback-exdev.md`.

Exit: unit tests covering `AtomicReplace` and initial integration tests for swap/rollback pass locally.

---

## Phase 5 — API Orchestration & Locking

1. API wiring
   - Implement `api.rs` functions per `impl/70-pseudocode.md` using the modules above.
   - Honor `ApplyMode::DryRun` default (REQ-C1).
2. Locking
   - Integrate `LockManager` bounded wait and record `lock_wait_ms` (REQ-L3); emit WARN when none (REQ-L2).
3. Attestation
   - Wire `Attestor` to sign the post-apply bundle and include fields in final facts (REQ-O4).
4. Operational bounds
   - Capture `duration_ms` around rename+fsync (REQ-BND1) per `impl/55-operational-bounds.md`.

Exit: end-to-end apply flow works with mocks/stubs; facts emitted.

---

## Phase 6 — Health Verification & CI Evidence

1. Smoke tests
   - Implement `SmokeTestRunner` adapter; in dev, provide a mock that runs a tiny suite.
   - Integrate into `apply()` with auto-rollback unless disabled (REQ-H1..H3).
2. Golden fixtures & gating
   - Establish golden fixtures for plan, preflight, apply, rollback.
   - CI jobs for schema validation and golden diffs (zero-SKIP).
3. Traceability
   - Run `SPEC/tools/traceability.py` in CI and verify `SPEC/traceability.md` coverage.

Exit: CI green with planned gates; minimal smoke suite passing.

---

## Phase 7 — Migration from Placeholders

1. Move code from placeholders per `impl/05-migration.md` mapping.
2. Replace direct calls with adapter-based boundaries.
3. Remove obsolete placeholder modules once parity is reached.

Exit: final layout matches planned structure; placeholders retired.

---

## Phase 8 — Hardening & ADR Finalization

1. Finalize ADRs (determinism namespace, EXDEV policy, error strategy, attestation keys).
2. Property tests for `IdempotentRollback` and `AtomicReplace`.
3. Performance passes and bounds monitoring.
4. Documentation polish and examples.

Exit: ready for tagged pre-release.
