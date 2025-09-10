# SPEC → Code → Test Traceability (Seed)

This file maps SPEC features and schema to code modules and tests. Initially a seed; to be refined as implementation proceeds.

| SPEC Artifact | Description | Code Modules (initial guess) | Tests/Evidence |
|---|---|---|---|
| `features/api_toctou.feature` | TOCTOU-safe sequences | `fs_ops.rs`, `api.rs` | unit + integration + BDD |
| `features/atomic_swap.feature` | Atomic file swaps | `fs_ops.rs` | unit + BDD |
| `features/conservatism_ci.feature` | Fail-closed CI policy | `preflight.rs`, `api.rs` | policy tests + BDD |
| `features/determinism_attestation.feature` | Deterministic behaviors | `api.rs`, `preflight.rs` | golden logs + BDD |
| `features/locking_rescue.feature` | Locking and rescue behavior | `fs_ops.rs` | unit + BDD |
| `features/observability.feature` | Audit/observability | `api.rs` | schema validation + BDD |
| `features/operational_bounds.feature` | Operational limits | `preflight.rs` | unit + BDD |
| `features/safety_preconditions.feature` | Preconditions & SafePath | `preflight.rs`, `fs_ops.rs` | negative BDDs + unit |
| `audit_event.schema.json` | Audit schema | `api.rs` | CI schema validation |

Next: expand each row with concrete functions, test file names, and CI job links.
