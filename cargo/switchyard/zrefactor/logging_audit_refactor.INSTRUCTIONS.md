# Logging/Audit Refactor — Actionable Steps (breaking)

> STATUS: Not landed in src/ (as of 2025-09-12 23:16:50 +02:00). `src/logging/audit.rs` still exposes `emit_*` helpers and no `StageLogger` facade; API call sites use legacy helpers. Keep PRs refactor-only.

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
     - `slog.preflight().action(..).path(..).current_kind(..).planned_kind(..).policy_ok(..).provenance(..).notes(..).preservation(..).emit()`.
   - In `src/api/apply/{mod.rs,handlers.rs}`:
     - `.apply_attempt().decision(..).extra(..).emit()` and `.apply_result().decision(..).extra(..).emit()`.
   - In `src/api/plan.rs`: `.plan().action(..).path(..).emit()` where applicable.

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
