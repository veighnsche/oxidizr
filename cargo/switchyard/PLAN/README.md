# Switchyard Implementation Planning (No Code)

This folder contains planning artifacts for the implementation phase of `cargo/switchyard`. It does not include any Rust implementation; it captures structure, signatures, and pseudocode to enable a high-level design review.

Related references:

- SPEC: `cargo/switchyard/SPEC/SPEC.md`, `requirements.yaml`, `features/`, `audit_event.schema.json`, `preflight.yaml`, `error_codes.toml`
- Planning: `cargo/switchyard/PLAN/10-architecture-outline.md`, `20-spec-traceability.md`, `30-delivery-plan.md`, `40-quality-gates.md`

Documents in this folder:

- `00-structure.md` — proposed crate folder/module structure
- `05-migration.md` — planned structure migration steps (planning only)
- `10-types-traits.md` — planned types, enums, and traits with purpose
- `15-policy-and-adapters.md` — policy flags and the `Adapters` bundle (SPEC §2.5, §2.8, §3.2)
- `20-adapters.md` — adapter trait contracts and responsibilities
- `25-safepath.md` — SafePath constructors, invariants, and TOCTOU-safe usage (SPEC §3.3)
- `30-errors-and-exit-codes.md` — error taxonomy mapping to stable exit codes (SPEC §6)
- `35-determinism.md` — UUIDv5 strategy, normalization, stable ordering (SPEC §2.7, §13)
- `40-facts-logging.md` — facts schema, redaction, ordering, attestation/provenance (SPEC §2.4, §5, §13)
- `45-preflight.md` — preflight checks, gating rules, YAML schema mapping (SPEC §4, §2.3)
- `50-locking-concurrency.md` — LockManager semantics, bounded wait, WARN behavior (SPEC §2.5, §14)
- `55-operational-bounds.md` — fsync ≤50ms, plan size bounds, telemetry (SPEC §9)
- `60-rollback-exdev.md` — rollback strategy, EXDEV degraded fallback, partial restoration (SPEC §2.2, §2.10)
- `65-rescue.md` — rescue profile and fallback toolset verification (SPEC §2.6)
- `70-pseudocode.md` — high-level pseudocode for core functions and flows (no code)
- `80-testing-mapping.md` — test strategy and fixture layout mapped to SPEC
- `12-api-module.md` — API module responsibilities and planned split of `src/api.rs` into `src/api/`

Review instructions: see `cargo/switchyard/PLAN/80-review-instructions.md`.
