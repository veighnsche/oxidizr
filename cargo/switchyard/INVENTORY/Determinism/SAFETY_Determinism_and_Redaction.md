# Determinism and redaction

- Category: Safety
- Maturity: Silver

## Summary

Deterministic IDs and ordering with stable timestamps in DryRun, plus redaction of volatile fields to enable golden tests and reproducible logs.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Stable IDs (UUIDv5) for reproducibility | `cargo/switchyard/src/types/ids.rs::{plan_id, action_id}` |
| Deterministic ordering of outputs | Preflight rows sorted by `(path, action_id)` |
| Redaction yields golden-friendly artifacts | `logging/redact.rs::{TS_ZERO, redact_event}` applied in helpers |

| Cons | Notes |
| --- | --- |
| Requires disciplined use of helpers | Facts must go through `logging/audit.rs` to ensure redaction |
| Some fields remain environment-sensitive | Paths and filesystem metadata may vary across hosts |

## Behaviors

- Generates stable `plan_id`/`action_id` using UUIDv5 from content and a fixed namespace.
- Sorts preflight rows deterministically (e.g., by path and action_id) before export.
- In DryRun, timestamps are zeroed via `TS_ZERO` and volatile fields are redacted.
- Ensures facts emitted from all stages go through redaction-aware helpers.

## Implementation

- IDs: `cargo/switchyard/src/types/ids.rs::{plan_id, action_id}` with UUIDv5 under namespace `constants::NS_TAG`.
- Redaction: `cargo/switchyard/src/logging/redact.rs::{TS_ZERO, ts_for_mode, redact_event}`.
- Sorting: preflight rows sorted by `(path, action_id)`.

## Wiring Assessment

- All stage facts go through `logging/audit.rs` helpers which apply redaction in dry-run.
- Apply selects timestamp via `ts_for_mode()`.
- Conclusion: wired correctly.

## Evidence and Proof

- Tests for `redact_event` and facts presence in `api.rs::tests`.

## Feature Analytics

- Complexity: Low. Helper functions and stable IDs.
- Risk & Blast Radius: Medium; incorrect use of helpers can leak non-determinism into goldens.
- Performance Budget: Negligible overhead for redaction and UUIDv5.
- Observability: All emitted facts carry redacted timestamps in DryRun; events validate against Audit v2 schema.
- Test Coverage: Unit tests for redaction; gaps: property tests for UUID stability and cross-run determinism.
- Determinism & Redaction: Core purpose; DryRun forces `TS_ZERO`.
- Policy Knobs: None directly; determinism interacts with DryRun mode.
- Exit Codes & Error Mapping: N/A; determinism does not map to exit codes.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: None specific; relies on standard library.
- DX Ergonomics: Centralized helpers reduce caller burden.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| N/A | — | Determinism/redaction controlled by stage mode (DryRun/Commit) and helper usage |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| N/A | — | No direct mapping |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| All stage events | `schema_version`, `ts` (zero in DryRun), `plan_id`, `path?`, `dry_run` | Audit v2 (`SPEC/audit_event.v2.schema.json`) |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/logging/redact.rs` | redact tests | zeroed timestamps; field masking |
| `src/api.rs` | facts presence tests | envelope fields present |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Redaction helpers and stable IDs exist | DryRun timestamps zeroed | Unit tests | None | Additive |
| Silver (current) | Deterministic sorting; enforced helper usage in stages | Stable outputs; redaction applied | Unit + integration | Inventory docs | Additive |
| Gold | Schema validation + goldens for determinism | Validated redaction and IDs across runs | Goldens + CI gates | CI gates | Additive |
| Platinum | Formalized determinism properties | Proved invariants, platform stability | Property tests | Continuous compliance |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [ ] Policy knobs documented reflect current `policy::Policy` (N/A)
- [x] Error mapping and `exit_code` coverage verified (N/A)
- [x] Emitted facts fields listed and schema version up to date
- [ ] Determinism parity (DryRun vs Commit) verified in tests
- [ ] Goldens added/updated and CI gates green
- [x] Preflight YAML or JSON Schema validated (planned) (where applicable)
- [ ] Cross-filesystem or degraded-mode notes reviewed (if applicable)
- [ ] Security considerations reviewed; redaction masks adequate
- [ ] Licensing impact considered (deps changed? update licensing inventory)

## Gaps and Risks

- No schema validation on facts; redaction policy could broaden.

## Next Steps to Raise Maturity

- Validate emitted events against Audit v2 schema in tests.
- Add property tests for UUIDv5 stability.

## Related

- SPEC v1.1 determinism requirements.
- `cargo/switchyard/src/logging/audit.rs`.
