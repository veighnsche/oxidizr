# Audit Schema (v2)

Envelope fields:
- `schema_version`, `ts`, `plan_id`, `run_id`, `event_id`, `switchyard_version`, `dry_run`, `seq`

Stages:
- `plan`, `preflight` (rows + `preflight.summary`), `apply.attempt`, `apply.result`, `rollback`, `rollback.summary`, `prune.result`.

Redaction:
- Zero `ts` in canon; remove timings/severity; mask secret fields; keep `lock_wait_ms` and `degraded` for assertions.

Citations:
- `cargo/switchyard/SPEC/audit_event.v2.schema.json`
- `cargo/switchyard/src/logging/audit.rs`
- `cargo/switchyard/src/logging/redact.rs`
