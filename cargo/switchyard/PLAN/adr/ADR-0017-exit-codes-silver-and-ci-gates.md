# ADR-0015 — Exit Codes Silver Coverage and CI Gate Staging

Status: Accepted
Date: 2025-09-11
Authors: Switchyard Team

## Context

The Switchyard SPEC defines an error taxonomy mapped to stable exit codes. The implementation progresses by tiers (Bronze → Silver → Gold), with Silver delivering a curated subset of error coverage end-to-end and Gold introducing stronger CI enforcement.

## Decision

- Exit Code Mapping (Silver)
  - Covered identifiers and codes at Silver tier:
    - `E_POLICY` → 10
    - `E_LOCKING` → 30 (bounded wait timeout emits `lock_wait_ms`)
    - `E_ATOMIC_SWAP` → 40
    - `E_EXDEV` → 50
    - `E_BACKUP_MISSING` → 60
    - `E_RESTORE_FAILED` → 70
    - `E_SMOKE` → 80
  - Facts MUST include `error_id` and `exit_code` for the covered sites at both per-action result and summary (where applicable).
  - Additional IDs (e.g., granular filesystem or ownership errors) remain at Bronze until tests exist.

- CI Gate Staging
  - Introduce a non-blocking traceability job that runs `SPEC/tools/traceability.py` and uploads artifacts.
  - Golden diff gates remain non-blocking for this crate initially; they will become blocking when curated scenarios are stabilized.
  - Schema validation runs in unit tests; promotion to a dedicated CI job is deferred until the Golden gate is stable.

## Rationale

- The curated Silver set captures the most impactful failures, allowing deterministic testing and stable facts.
- Non-blocking CI traceability provides visibility without slowing iteration. Once stable, we can flip Golden gates to blocking.

## Consequences

- Tests and goldens must assert `error_id`/`exit_code` presence for the Silver set.
- Lock timeout facts must include `lock_wait_ms` (masked in redacted canon), and tests should accept redaction for canon diffs.
- Documentation (SPEC_UPDATE) must reflect policy surfaces (`require_rescue`, `require_preservation`) and degraded symlink semantics.

## Alternatives Considered

- Immediate full mapping: rejected due to brittleness and missing coverage.
- Blocking CI gates immediately: rejected to avoid flakiness while scenarios stabilize.

## References

- `SPEC/SPEC.md` §§ 2.5, 6, 12
- `SPEC/error_codes.toml`
- `src/api/errors.rs`, `src/api/apply.rs`
- `.github/workflows/ci.yml` traceability job
