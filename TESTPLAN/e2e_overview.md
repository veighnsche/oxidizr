# Switchyard Library E2E Test Plan — Overview

This document defines the end-to-end test planning approach for the Switchyard library API, targeting high-confidence coverage of behavior, interactions, and invariants under realistic environments.

References to code are to the crate layout under `cargo/switchyard/`.

## Objectives

- Validate the public API surface in `src/api/` as used via the standard builder pattern (`Switchyard::builder`) with realistic adapter configurations.
- Prove atomicity, durability, rollback, and policy gating invariants across pairwise combinations of options, escalating to 3-wise for high-risk axes.
- Exercise negative/error paths deterministically (fixed seeds/time; stable expectations via redaction).
- Keep mapping auditable: every scenario explicitly maps to axes/levels, preconditions, and oracles.

## Out of Scope

- CLI behavior, argument parsing, and shell UX are explicitly out-of-scope. The focus is the Rust library API as exposed by `src/api/mod.rs`.
- Non-public/internal modules unless they influence public behavior (they inform oracles/invariants but are not direct test targets).

## Glossary (project-terminology aligned)

- API: Public entry points in `src/api/` (e.g., `Switchyard::plan`, `preflight`, `apply`, `plan_rollback_of`, `prune_backups`).
- Option (Axis/Level): An option is any input that can vary (policy knob, mode, adapter presence). An axis is a named option; levels are its allowed values.
- Environment: External execution context variables (filesystem flavor, permissions, path shapes, EXDEV conditions, parallelism, disk pressure, unicode/long paths, etc.).
- State: The filesystem and system state relevant to a scenario (pre-existing links, backups, ownership, rescue tools availability), plus the `Policy` used.
- Oracle: Deterministic observation used to assert behavior (return values, `ApplyReport`, `PreflightReport`, filesystem diffs, emitted facts via `logging::StageLogger`).
- Invariant: Implementation-agnostic claim that must always hold (e.g., atomic rename for swaps; newest backup is never pruned when within limits; no temp files on failure).
- Determinism: Stability of oracles across runs. For facts, DryRun uses `TS_ZERO` via `logging/redact.rs`; event fields not covered by contracts are ignored.

## API Under Test (summary)

- `Switchyard::plan(&self, PlanInput) -> Plan` (see `src/api/plan.rs` and `src/types/plan.rs`).
- `Switchyard::preflight(&self, &Plan) -> Result<PreflightReport, ApiError>` (see `src/api/preflight/`).
- `Switchyard::apply(&self, &Plan, ApplyMode) -> Result<ApplyReport, ApiError>` (see `src/api/apply/`).
- `Switchyard::plan_rollback_of(&self, &ApplyReport) -> Plan` (see `src/api/rollback.rs`).
- `Switchyard::prune_backups(&self, &types::safepath::SafePath) -> Result<types::PruneResult, ApiError>`.
- `types::safepath::SafePath::from_rooted(root, candidate) -> Result<SafePath>` (constructor contract and boundary checks).
- Construction axes via `ApiBuilder` and `Switchyard::with_*` methods (lock manager, ownership oracle, attestor, smoke runner, lock timeout).

## Coverage Goals

- Pairwise coverage across all axes per function by default.
- Escalate to 3-wise coverage for High-risk axes and historically interaction-driven failures:
  - EXDEV policy × Locking × ApplyMode
  - SmokePolicy × Smoke runner presence × ApplyMode
  - Ownership strictness × OwnershipOracle presence × target ownership
  - Preservation policy × FS capabilities × Restore flow
  - Best-effort-restore × Backup presence × ApplyMode
- Boundary coverage overlay: at least one dedicated scenario per boundary level per axis (min/max/empty/huge/invalid where applicable).
- Negative cases: type/contract violations (invalid `SafePath`), policy stops, locking timeouts, smoke failures, attestation errors, missing backups.

## Determinism & Reproducibility

- Use `ApplyMode::DryRun` for log-based oracles when feasible; facts redaction yields `ts="1970-01-01T00:00:00Z"` and removes volatile fields per `logging/redact.rs`.
- When `ApplyMode::Commit` is required, assert only on stable outputs and FS state; ignore volatile audit fields (e.g., `event_id`).
- Use isolated temp roots for all filesystem state and construct `SafePath` via `SafePath::from_rooted`.
- Record seeds/time in scenario headers; prefer DryRun for timing-independent oracles.

## Risk-Based Expansions (examples)

- Atomicity under EXDEV: `apply` with cross-filesystem target and `ExdevPolicy::{Fail,DegradedFallback}`.
- Locking: Required locking with/without `LockManager`; bounded wait behavior; early failure mapping to `E_LOCKING` (see `src/api/apply/mod.rs`).
- Smoke enforcement: `SmokePolicy::Require{auto_rollback}` with and without a runner; verify rollback and `E_SMOKE` mapping.
- Ownership strictness: `ownership_strict=true` with/without `OwnershipOracle`; policy stops vs allowed.
- Prune retention policy: `retention_count_limit`/`retention_age_limit` boundaries and tag scoping.

## What Each Document Provides

- `api_option_inventory.md`: Canonical list of API functions, their axes/levels, boundaries, and constraints.
- `combinatorial_model.json`: Machine-readable axes, levels, constraints, and risk overrides per function.
- `test_selection_matrix.md`: Strategy (pairwise/3-wise/boundaries/negative) and a human-readable selection table.
- `environment_matrix.md`: Environment axes, base sets, and mapping of scenarios to envs (blow-up control justification).
- `oracles_and_invariants.md`: Observable oracles and invariants per function and scenario class.
- `traceability.md`: Function × Axis × Level → Scenario IDs with coverage proof.
- `flakiness_and_repro.md`: Determinism policy, retry/quarantine protocol, proof a failure isn’t a race.
- `scheduling_and_cost.md`: Estimated test count, parallelization, wall clock targets, CI tiers (Bronze/Silver/Gold/Platinum).

## State Models

- Happy flow (symlink switch):
  - Preconditions: target may be file/symlink/absent.
  - Sequence: plan → preflight(ok) → apply(EnsureSymlink) with `atomic_symlink_swap` → success.
  - Illegal jumps: apply with `override_preflight=false` when preflight would STOP → should fail early.
- Restore flow:
  - Preconditions: backup snapshot(s) exist with sidecar; optional integrity hash.
  - Sequence: plan (RestoreFromBackup) → preflight (annotates backup presence) → apply(restore_file | restore_file_prev) → success.
  - Recovery path: if failure after partial execution and `capture_restore_snapshot=true`, rollback uses previous snapshot.
- Rollback flow:
  - Preconditions: `ApplyReport.executed` populated.
  - Sequence: plan_rollback_of(report) → preflight → apply → state convergence to pre-apply.

Coverage policy: at least one test per edge (see `test_selection_matrix.md` → E2E-APPLY-009/016, E2E-ROLLBACK-001..003).

## Risk Register (mapped to scenarios)

- Data loss during swap (non-atomic window)
  - Mitigation: atomic rename; degraded path only with EXDEV fallback. Scenarios: E2E-APPLY-005 (degraded), E2E-APPLY-019 (fail on EXDEV), E2E-APPLY-022 (crash injection).
- Backup integrity mismatch
  - Mitigation: sidecar payload hash; best-effort enforcement per policy. Scenarios: E2E-APPLY-020 (integrity disabled), E2E-APPLY-016 (no capture snapshot), E2E-APPLY-009.
- Locking and concurrency hazards
  - Mitigation: required locking with bounded wait. Scenarios: E2E-APPLY-003 (no manager), E2E-APPLY-015 (contention timeout), E2E-APPLY-002 (happy).
- Ownership/provenance policy violations
  - Mitigation: preflight gating and optional oracle. Scenarios: E2E-PREFLIGHT-002 (strict without oracle), E2E-APPLY-017 (strict with oracle).
- Smoke health regressions post-apply
  - Mitigation: require smoke with optional auto-rollback. Scenarios: E2E-APPLY-011/012/004.
- Preservation capability gaps
  - Mitigation: gate with `RequireBasic`. Scenarios: E2E-PREFLIGHT-003.
- ENOSPC / I/O faults
  - Mitigation: fail safe, no temp files left. Scenarios: E2E-APPLY-014.

## Review & Sign-off Checklist

- [x] Every public function listed; parameters represented as axes with levels (see `api_option_inventory.md`).
- [x] Boundaries covered at least once (see `test_selection_matrix.md`).
- [x] Pairwise achieved; 3-wise escalations justified (see `combinatorial_model.json` and selection tags).
- [x] Constraints documented with rationale (see `api_option_inventory.md`).
- [x] Negative tests cover contract violations and common OS error codes (`E_LOCKING`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`).
- [x] Environment matrix defined; rotation policy controls blow-up (see `environment_matrix.md`).
- [x] Oracles/invariants are precise and implementation-agnostic (`oracles_and_invariants.md`).
- [x] Traceability matrix has no unmapped (function, axis, level) (`traceability.md`).
- [x] Seeds and determinism policy specified (`flakiness_and_repro.md`).
- [x] Tiers assigned; CI slicing feasible (`scheduling_and_cost.md`).
