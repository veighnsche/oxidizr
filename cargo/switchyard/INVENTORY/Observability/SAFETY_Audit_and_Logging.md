# Audit and logging (Minimal Facts v1)

- Category: Safety
- Maturity: Silver

## Summary

Central helpers emit stage facts with a stable envelope (schema_version, ts, plan_id, path, dry_run) to a `FactsEmitter`. Redaction applied as configured.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Centralized emission ensures uniform schema/envelope | `cargo/switchyard/src/logging/audit.rs` helpers used by all stages |
| Redaction applied consistently | `logging/redact.rs` used by helpers; DryRun → `TS_ZERO` |
| Pluggable sinks | `logging/facts.rs::{FactsEmitter, AuditSink, JsonlSink}` |

| Cons | Notes |
| --- | --- |
| No schema validation yet | See `SAFETY_Facts_Schema_Validation.md` gap |
| File sink is minimal (no rotation) | `INFRA_JSONL_File_Logging.md` lists gaps |

## Behaviors

- Emits `apply.attempt`, `apply.result`, per-action results, and rollback step/summary events.
- Attaches envelope fields and stage-specific extras (perf, lock info, error IDs).
- Applies redaction policy in DryRun (e.g., timestamps zeroed, volatile fields masked).
- Sends structured events to configured sinks (in-memory test emitter, JSONL sinks).

## Implementation

- Core: `cargo/switchyard/src/logging/audit.rs` (`AuditCtx`, `emit_*` helpers).
- Sinks: `cargo/switchyard/src/logging/facts.rs::{FactsEmitter, AuditSink, JsonlSink}` and optional `FileJsonlSink` under `file-logging` feature.

## Wiring Assessment

- `api/plan`, `api/preflight`, `api/apply`, and rollback all invoke audit helpers.
- `Switchyard` constructed with a `FactsEmitter` and an `AuditSink`.
- Conclusion: wired correctly.

## Evidence and Proof

- `api.rs::tests` capture facts into a test emitter and assert required fields and consistency.

## Feature Analytics

- Complexity: Low-Medium. Central helpers + sink interfaces.
- Risk & Blast Radius: High observability value; mis-emission affects all analytics; mitigated by centralization.
- Performance Budget: Minimal overhead for serialization and emission; acceptable for CLI workflows.
- Observability: Primary mechanism; emits Minimal Facts v1 envelope + extras.
- Test Coverage: Tests exist in `api.rs`; gaps: schema validation and golden facts.
- Determinism & Redaction: DryRun redaction applied in helpers.
- Policy Knobs: None directly; redaction controlled by mode.
- Exit Codes & Error Mapping: Carried in facts; mapping lives in `api/errors.rs`.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: None.
- DX Ergonomics: Simple helper API reduces per-callsite complexity.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| N/A | — | Emission controlled by stage flows; no policy flag |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| See `SAFETY_Exit_Codes.md` | — | Facts carry `error_id`/`exit_code` when available |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.attempt` | envelope + lock info | Minimal Facts v1 |
| `apply.result` | envelope + perf, error ids | Minimal Facts v1 |
| `rollback.step`/`rollback.summary` | envelope + rollback data | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/api.rs` | facts emission tests | envelope fields and stage emissions |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Central helpers; basic sinks | Facts present for stages | Unit/integration | None | Additive |
| Silver (current) | Redaction applied; consistent envelope | Stable shape; stage coverage | Tests + inventory | Inventory docs | Additive |
| Gold | Schema validation + goldens | Validated facts across runs | CI schema + goldens | CI gates | Additive |
| Platinum | Extended sinks, rotation, durability | Operationally robust logging | System tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields listed and schema version up to date
- [ ] Schema validated in CI
- [ ] Goldens added/updated and CI gates green

## Gaps and Risks

- No schema validation; file-backed sink is feature-gated and minimal.

## Next Steps to Raise Maturity

- Add JSON Schema validation in tests; expand sink options.

## Related

- SPEC v1.1 (Minimal Facts schema and redaction policy).
