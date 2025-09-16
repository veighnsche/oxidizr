# Audit Event Schema (v2)

- Envelope fields: `schema_version`, `ts`, `plan_id`, `run_id`, `event_id`.
- Stages: `plan`, `preflight` (rows + summary), `apply.attempt`, `apply.result`, `rollback` (and summary), `prune.result`.

Citations:
- `SPEC/audit_event.v2.schema.json`
- `src/logging/audit.rs`
