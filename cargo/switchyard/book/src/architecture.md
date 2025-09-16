# Architecture Overview

This is a quick tour of the major modules and how they collaborate to deliver safe, atomic, and reversible swaps.

> Note: This guide focuses on reading the code with a safety lens. For normative requirements, see `cargo/switchyard/SPEC/SPEC.md`.

## Layered Map

- api/
  - `ApiBuilder`, `Switchyard`, and orchestration of `plan`, `preflight`, `apply`, `plan_rollback_of`, and `prune_backups`.
- fs/
  - Atomic swap, backup creation, sidecar metadata, restore engine, and retention (prune) logic.
- policy/
  - Safety knobs and presets (rescue, durability, risks, governance, apply flow).
- adapters/
  - Integration points: LockManager, OwnershipOracle, Attestor, PathResolver, SmokeTestRunner.
- logging/
  - StageLogger facade, JSON facts, redaction, and sinks.
- types/
  - Data types (Plan, Action, SafePath, IDs, reports, error enums).

## Data and Control Flow

1. Build a `Switchyard` via `ApiBuilder` with policy and adapters.
2. Create a `Plan` from `PlanInput` using typed `SafePath` values.
3. Run `preflight` to gate environment and policy (rescue, ownership, preservation, mounts, etc.).
4. Run `apply` in DryRun or Commit mode.
   - Emit `apply.attempt` with locking details.
   - Perform atomic swap with backup and sidecar.
   - Emit `apply.result` with before/after hashes and provenance.
   - On failure, perform automatic reverse-order rollback and emit `rollback` facts.
5. Optionally run `prune_backups` under retention policy.

## Safety Anchors in Code

- `types::SafePath` — the only path type accepted by mutating APIs.
- `fs::*` — TOCTOU-safe sequences and durability (`fsync(parent)` after rename).
- `policy::*` — conservative defaults; fail-closed on critical differences.
- `logging::audit` — schema v2 facts, redaction, attestation support.

## Example: One-Stop Construction

```rust
use switchyard::api::{ApiBuilder, Switchyard};
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;

let api: Switchyard<_, _> = ApiBuilder::new(JsonlSink::default(), JsonlSink::default(), Policy::production_preset())
    .with_lock_timeout_ms(500)
    .build();
```

## Reading Tips

- Start in `src/api/` to see stage orchestration and event emission.
- Cross-reference to `fs/backup` and `fs/restore` for durability and rollback logic.
- Review `SPEC/traceability.md` to see requirement coverage mapped to scenarios.
