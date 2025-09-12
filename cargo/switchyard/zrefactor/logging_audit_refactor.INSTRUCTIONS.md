# Logging/Audit Refactor — Actionable Steps (breaking)

Do the following changes. No shims; remove legacy helpers.

1. Create/update modules under `src/logging/`
   - Edit `src/logging/audit.rs`:
     - Add: `StageLogger`, `EventBuilder`, `Stage` (Plan, Preflight, ApplyAttempt, ApplyResult, Rollback), `Decision` (Success, Failure, Info).
     - Enforce envelope in one place (schema_version, ts, plan_id, path, dry_run) and call `redact_event()`.
     - Remove or make private the legacy `emit_*` helpers.
   - Edit `src/logging/mod.rs` to re-export: `StageLogger`, `EventBuilder`, `Stage`, `Decision`, `FactsEmitter`, `AuditSink`, `JsonlSink`, `redact` helpers.
   - Leave `src/logging/facts.rs` and `src/logging/redact.rs` intact.

2. Replace all audit emissions at call sites
   - In `src/api/preflight/{mod.rs,rows.rs}`: replace ad-hoc JSON + `emit_*` with the builder facade:
     - `slog.preflight().action(..).path(..).current_kind(..).planned_kind(..).policy_ok(..).provenance(..).notes(..).preservation(..).preservation_supported(..).emit()`.
     - Field parity with existing code:
       - Per-row: `action_id`, `path`, `current_kind`, `planned_kind`.
       - Extended: `policy_ok`, `provenance`, `notes[]`, `preservation`, `preservation_supported`.
       - Row-only (RestoreFromBackup): `restore_ready` boolean when backup artifacts are present.
   - In `src/api/apply/{mod.rs,handlers.rs}`:
     - `.apply_attempt().decision(..).lock_backend(..).lock_wait_ms(..).lock_attempts(..).emit()`.
     - `.apply_result().decision(..).lock_backend(..).lock_wait_ms(..).perf(..).attestation(..).error_id(..).exit_code(..).summary_error_ids(..).emit()`.
     - Field parity with existing code:
       - `apply.attempt`: `lock_backend`, `lock_wait_ms` (nullable), `lock_attempts`.
       - `apply.result` per-action: include `action_id`, `path` when mapping gating failures.
       - `apply.result` summary: `lock_backend`, `lock_wait_ms`, `perf{hash_ms,backup_ms,swap_ms}`, optional `attestation{sig_alg,signature,bundle_hash,public_key_id}`; on failure include `error_id`, `exit_code`, and `summary_error_ids`.
   - In `src/api/plan.rs`: `.plan().action(..).path(..).emit()` where applicable.
   - In `src/api.rs::prune_backups`: `.prune_result().path(..).backup_tag(..).retention_count_limit(..).retention_age_limit_ms(..).pruned_count(..).retained_count(..).error_id(..).exit_code(..).emit()`.

3. Centralize and restrict emission
   - Only code under `src/logging/` may call `FactsEmitter::emit(..)` or construct audit JSON payloads.
   - Outside `src/logging/` use the typed facade exclusively.

4. CI guardrails (grep-based)
   - Forbid `audit::emit_` outside `src/logging/`.
   - Forbid direct `FactsEmitter::emit` outside `src/logging/`.

5. Tests (keep existing TestEmitter usage)
   - Unit: verify envelope, decision mapping, and redaction via `StageLogger`.
   - Integration: golden tests to assert emitted JSON for representative flows.
   - `tests/backup_durable_flag.rs` remains valid — events still arrive via `FactsEmitter`.

6. Cleanups
   - /// remove this file if present: `src/api/telemetry.rs` (renamed/obsolete; all audit via `src/logging/`).
   - Delete legacy `emit_*` in `src/logging/audit.rs` after migrations in this PR.

---

## Meta

- Scope: Centralize audit/logging behind a typed facade; migrate call sites
- Status: Breaking allowed (pre-1.0)
- Index: See `zrefactor/README.md`

## Related

- API migration away from direct emissions: `zrefactor/api_refactor.INSTRUCTIONS.md`
- Preflight/apply usage of facade: `zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`
- Policy-owned gating (drives fields added to events): `zrefactor/policy_refactor.INSTRUCTIONS.md`
- Cohesion/guardrails: `zrefactor/responsibility_cohesion_report.md`
- Removal plan and registry: `zrefactor/backwards_compat_removals.md`, `zrefactor/removals_registry.md`
