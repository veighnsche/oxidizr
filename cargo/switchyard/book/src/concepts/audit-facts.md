# Audit Facts and Redaction

- Schema v2: envelope (`schema_version`, `ts`, `plan_id`, `run_id`, `event_id`).
- Redaction zeros timestamps and masks secrets for canon comparison.
- Use `StageLogger` facade for emission.

Citations:
- `cargo/switchyard/src/logging/audit.rs`
- `cargo/switchyard/src/logging/redact.rs`
- `cargo/switchyard/SPEC/audit_event.v2.schema.json`

What’s recorded (highlights)
- Stages: `plan`, `preflight` (rows + summary), `apply.attempt`, `apply.result`, `rollback` (and summary), `prune.result`.
- Hashes: for each mutated file, `hash_alg=sha256` and both `before_hash` and `after_hash` are emitted in `apply.result`.
- Locking telemetry: `apply.attempt` includes `lock_backend`, `lock_wait_ms`, and `lock_attempts` (approximate).
- Summary error chains: summaries (preflight/apply/rollback) include `summary_error_ids` listing specific→general error identifiers (e.g., `E_SMOKE`, `E_LOCKING`, `E_POLICY`).
- Attestation: on apply success (when an `Attestor` is configured), events include signature fields and `bundle_hash`.

Determinism & redaction
- Dry-run and Commit facts must be byte-identical after redaction. Timestamps are zeroed and volatile fields masked.
- Preflight rows are deterministically ordered by `(path, action_id)`.

Integration tips
- Prefer the `StageLogger` facade for all emissions to ensure consistent redaction and schema shape.
- Validate JSONL facts against `SPEC/audit_event.v2.schema.json` in CI to prevent drift.

Inventory cross‑links
- `INVENTORY/65_Observability_Audit_and_Logging.md`
- `INVENTORY/70_Observability_Facts_Schema_Validation.md`
