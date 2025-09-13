# Audit and logging (Minimal Facts v1)

- Category: Safety
- Maturity: Silver

## Summary

Central helpers emit stage facts with a stable envelope (schema_version, ts, plan_id, path, dry_run) to a `FactsEmitter`. Redaction applied as configured.

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

## Gaps and Risks

- No schema validation; file-backed sink is feature-gated and minimal.

## Next Steps to Raise Maturity

- Add JSON Schema validation in tests; expand sink options.

## Related

- SPEC v1.1 (Minimal Facts schema and redaction policy).
