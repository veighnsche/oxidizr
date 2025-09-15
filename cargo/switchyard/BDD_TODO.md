# TODO.md — BDD Step Coverage & Failures

## P0 — Must Fix Before RC

- [ ] Change all mutating APIs (e.g. `prune_backups`) to take `&SafePath` instead of `PathBuf`.
- [ ] Fix unwraps in EXDEV degraded path handling → return classified error (`E_EXDEV`) and preserve atomic copy+rename fallback.
- [ ] Fix unwraps in LockManager absence → emit WARN fact with classification instead of panic.
- [ ] Align DryRun vs Commit per-action `apply.result` facts (ensure identical after redaction).
- [ ] Add step defs for:
  - Then the engine automatically rolls back A in reverse order.
  - Then the engine performs reverse-order rollback of any executed actions.
  - And at least one smoke command will fail with a non-zero exit.
  - And a configured SmokeTestRunner.
  - And auto_rollback is enabled.
  - Then the operation fails closed unless an explicit policy override is set.
  - And another process holds the lock (bounded wait → timeout + metrics).
  - And a rescue profile remains available for recovery.

## P1 — Strongly Recommended (Can Slip With Docs)

- [ ] Add step def: target path currently resolves to providerA/ls.
- [ ] Wire determinism_attestation.feature step defs (plan_id, action_id, signed attestation).
- [ ] Ensure EXDEV disallowed path → classified error_id=E_EXDEV with exit_code=50.

## P2 — Optional For RC, Tag As @postrc

- [ ] Deduplicate redundant determinism/attestation coverage vs schema v2 audit tests.
- [ ] Defer extended atomicity visibility checks if already covered elsewhere.

## Housekeeping

- [ ] Remove or implement unused step imports in tests (FileLockManager, Value, schema, AuditSink, FactsEmitter).
- [ ] Remove dead code in `bdd_support/env.rs` (`set_var_scoped`).
- [ ] Remove unused `World` methods (`run_preflight_capture`, `enable_smoke`) or wire to scenarios.
