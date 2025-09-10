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

- Public API: functions exposed by `api.rs` (as planned below in 3.1 of `SPEC.md`).
- Audit event schema: `SPEC/audit_event.schema.json` (validation in CI).

## Failure Domains

- Filesystem boundaries and cross-device moves (support degraded mode when configured).
- Locking and concurrency around file swaps.

## Planned Public API (planning only)

Planned signatures per `SPEC.md §3.1` (no code yet):

```rust
fn plan(input: PlanInput) -> Plan;
fn preflight(plan: &Plan) -> PreflightReport;
fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport;
fn plan_rollback_of(report: &ApplyReport) -> Plan;
```

All mutating inputs carry `SafePath` (see SafePath notes below).

## Core Types (planning)

- `PlanInput`: normalized intent (targets, providers, policy flags). All path fields are `SafePath`.
- `Plan`: ordered actions with deterministic `plan_id` (UUIDv5) and per-action `action_id` (UUIDv5).
- `ApplyMode`: `DryRun` (default) | `RealRun`.
- `PreflightReport`: sequence of entries per `SPEC/preflight.yaml`.
- `ApplyReport`: summary + per-step facts (JSON) matching `SPEC/audit_event.schema.json`.
- `SafePath`: constructed via `SafePath::from_rooted(root, candidate)`; rejects `..` after normalization; used by all mutating APIs.

## Error Model & Exit Codes (planning)

- Library error enum maps to stable categories; CI and facts use stable identifiers:
  - `E_POLICY` → `policy_violation`
  - `E_OWNERSHIP` → `ownership_error`
  - `E_LOCKING` → `lock_timeout`
  - `E_ATOMIC_SWAP` → `atomic_swap_failed`
  - `E_EXDEV` → `exdev_fallback_failed`
  - `E_BACKUP_MISSING` → `backup_missing`
  - `E_RESTORE_FAILED` → `restore_failed`
  - `E_SMOKE` → `smoke_test_failed`
- Facts include `exit_code` per `SPEC/error_codes.toml` where applicable.

## Adapters & Boundaries

As per `SPEC.md §3.2` (planning interfaces):

- `OwnershipOracle` — package ownership checks.
- `LockManager` — bounded lock acquisition; emits `lock_wait_ms` in facts.
- `PathResolver` — resolves binaries to `SafePath`.
- `Attestor` — signs apply bundles (ed25519) and returns attestation fields.
- `SmokeTestRunner` — executes minimal smoke suite and reports failures.

## Configuration & Policy (planning)

- Builder-style configuration for `PlanInput` and `Adapters` bundle.
- Defaults are conservative:
  - `ApplyMode::DryRun` by default (REQ-C1).
  - `strict_ownership=true`; `allow_degraded_fs=false` unless explicitly set.
  - `disable_auto_rollback=false` unless explicitly set.
- Policy gates enforced in `preflight.rs`; fail-closed on violations (REQ-C2).

## Cross-cutting Concerns

- Determinism (REQ-D1, REQ-D2): UUIDv5 IDs; timestamp redaction in dry-run; stable field ordering for facts and preflight.
- Observability (REQ-O1..O7, REQ-VERS1): JSONL facts per step; schema v1; secret masking; provenance completeness.
- Safety (REQ-API1, REQ-TOCTOU1, REQ-S1..S5): SafePath-only mutations; TOCTOU-safe syscall sequence; preservation capability gating.
- Filesystems (REQ-F1..F3): EXDEV degraded fallback with policy and telemetry.

## Concurrency & Locking

- `LockManager` required in production (REQ-L4); bounded wait with timeout (REQ-L3).
- Without a `LockManager` (dev/test), concurrent `apply()` is UNSUPPORTED; emit WARN fact (REQ-L2).
- Core types are `Send + Sync`; only one mutator proceeds under lock (REQ-L1, Thread-safety §14).

## Testing & Evidence (planning)

- Test taxonomy: unit (fs_ops, safepath), integration (api orchestration), property (AtomicReplace, IdempotentRollback), BDD (SPEC/features/*.feature), golden fixtures for facts.
- Minimal smoke suite executed via `SmokeTestRunner` post-apply (REQ-H1..H3).
- CI gates: determinism, safety preconditions, audit schema validation, zero-SKIP golden fixture checks.

## Open Questions (updated)

- Final public API versioning strategy (semver) and stability policy.
- Boundaries between `api.rs` orchestration and lower-level modules; trait object vs generics for adapters.
- Attestation key management (dev/test keys vs prod integration) and SBOM-lite format.
