# Switchyard Spec Traceability Report

## Summary
Total requirements: 44
MUST/MUST_NOT: 43
Covered MUST/MUST_NOT: 43
Uncovered MUST/MUST_NOT: 0

## Uncovered MUST/MUST_NOT Requirements
- None

## Coverage Matrix (Requirement → Scenarios)
- REQ-A1 — Atomic crash-safety
  - atomic_swap.feature :: Enable and rollback
- REQ-A2 — No broken or missing path visible
  - atomic_swap.feature :: Enable and rollback
- REQ-A3 — All-or-nothing per plan
  - atomic_swap.feature :: Automatic rollback on mid-plan failure
- REQ-API1 — SafePath-only for mutating APIs
  - api_toctou.feature :: Mutating public APIs require SafePath
- REQ-BND1 — fsync within 50ms of rename
  - operational_bounds.feature :: fsync within 50ms after rename
- REQ-C1 — Dry-run by default
  - conservatism_ci.feature :: Dry-run is the default mode
- REQ-C2 — Fail-closed on critical violations
  - conservatism_ci.feature :: Fail-closed on critical violations
- REQ-CI1 — Golden fixtures existence
  - conservatism_ci.feature :: CI gates for golden fixtures and zero-SKIP
- REQ-CI2 — Zero-SKIP gate
  - conservatism_ci.feature :: CI gates for golden fixtures and zero-SKIP
- REQ-CI3 — Golden diff gate
  - conservatism_ci.feature :: CI gates for golden fixtures and zero-SKIP
- REQ-D1 — Deterministic IDs
  - determinism_attestation.feature :: Deterministic UUIDv5 plan and action IDs
- REQ-D2 — Redaction-pinned dry-run
  - observability.feature :: Dry-run facts are byte-identical to real-run facts after redaction
- REQ-F1 — EXDEV fallback preserves atomic visibility
  - atomic_swap.feature :: Cross-filesystem EXDEV fallback
- REQ-F2 — Degraded mode policy & telemetry
  - atomic_swap.feature :: Cross-filesystem EXDEV fallback
- REQ-F3 — Supported filesystems verified in acceptance tests
  - (no scenarios)
- REQ-H1 — Minimal smoke suite
  - atomic_swap.feature :: Smoke test failure triggers rollback
- REQ-H2 — Auto-rollback on smoke failure
  - atomic_swap.feature :: Smoke test failure triggers rollback
- REQ-H3 — Health verification is part of commit
  - atomic_swap.feature :: Smoke test failure triggers rollback
- REQ-L1 — Single mutator
  - locking_rescue.feature :: Bounded locking in production
- REQ-L2 — Warn when no lock manager
  - locking_rescue.feature :: No LockManager in dev/test emits WARN
- REQ-L3 — Bounded lock wait with timeout
  - locking_rescue.feature :: Bounded locking in production
- REQ-L4 — LockManager required in production
  - locking_rescue.feature :: Bounded locking in production
- REQ-O1 — Structured fact for every step
  - observability.feature :: Every step emits a structured fact conforming to schema v1
- REQ-O2 — Dry-run facts identical to real-run
  - observability.feature :: Dry-run facts are byte-identical to real-run facts after redaction
- REQ-O3 — Versioned, stable facts schema
  - observability.feature :: Every step emits a structured fact conforming to schema v1
- REQ-O4 — Signed attestations per apply bundle
  - determinism_attestation.feature :: Signed attestation per apply bundle
- REQ-O5 — Before/after hashes for each mutated file
  - observability.feature :: Before/after hashes are recorded for mutated files
- REQ-O6 — Secret masking across all sinks
  - observability.feature :: Secret masking is enforced across all sinks
- REQ-O7 — Provenance completeness
  - observability.feature :: Provenance fields are complete
- REQ-R1 — Rollback reversibility
  - atomic_swap.feature :: Enable and rollback
- REQ-R2 — Restore exact topology
  - atomic_swap.feature :: Enable and rollback
- REQ-R3 — Idempotent rollback
  - atomic_swap.feature :: Enable and rollback
- REQ-R4 — Auto reverse-order rollback on failure
  - atomic_swap.feature :: Automatic rollback on mid-plan failure
- REQ-R5 — Partial restoration facts on rollback error
  - atomic_swap.feature :: Automatic rollback on mid-plan failure
- REQ-RC1 — Rescue profile available
  - locking_rescue.feature :: Rescue profile and fallback toolset verified
- REQ-RC2 — Verify fallback path
  - locking_rescue.feature :: Rescue profile and fallback toolset verified
- REQ-RC3 — Fallback toolset on PATH
  - locking_rescue.feature :: Rescue profile and fallback toolset verified
- REQ-S1 — Safe paths only
  - safety_preconditions.feature :: SafePath rejects escaping paths
- REQ-S2 — Reject unsupported filesystem states
  - safety_preconditions.feature :: Fail on unsupported filesystem state
- REQ-S3 — Source ownership gating
  - safety_preconditions.feature :: Source ownership gating
- REQ-S4 — Strict target ownership
  - safety_preconditions.feature :: Strict package ownership for targets
- REQ-S5 — Preservation capability gating
  - safety_preconditions.feature :: Preservation capability gating in preflight
- REQ-TOCTOU1 — TOCTOU-safe syscall sequence
  - api_toctou.feature :: TOCTOU-safe syscall sequence is normative
- REQ-VERS1 — Facts carry schema_version
  - observability.feature :: Every step emits a structured fact conforming to schema v1

