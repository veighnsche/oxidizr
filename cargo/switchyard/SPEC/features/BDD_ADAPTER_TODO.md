# BDD Adapter — Remaining TODOs

This document tracks the remaining work to fully wire the BDD adapter so that all Gherkin features under `cargo/switchyard/SPEC/features/` execute against the Switchyard Rust library with 100% step coverage and green results.

See reference guide: `cargo/switchyard/SPEC/features/bdd_adapter_guide.md`

## World design and discipline

- [x] Finalize the `World` fields and lifecycle:
  - `facts`, `audit`, `policy`, `plan`, `preflight`, `apply_report`, temp root (`TempDir`/`PathBuf`), `lock_path` implemented in `tests/bdd_world/mod.rs`.
  - Optional handles implemented where needed: `Attestor`, `LockManager`, `SmokeTestRunner` wired via steps. `OwnershipOracle` pending product integration.
- [x] Enforce test FS discipline in code:
  - All filesystem operations routed under a temporary directory and via `SafePath::from_rooted` helpers.
  - Never touches system paths in scenarios.
- [ ] Document the discipline in this SPEC (usage notes, examples).

## Sinks and logging

- [ ] Document the in-memory `CollectingEmitter` (FactsEmitter) and `CollectingAudit` (AuditSink).
- [ ] Provide instructions to use file-backed JSONL sinks behind `--features file-logging` for debugging.

## Step mapping (align to steps-contract.yaml)

- [ ] Add a canonical table mapping Given/When/Then regex → Switchyard API calls and assertions.
  - Plan, Preflight, Apply (Commit/DryRun) flows.
  - Locking, Rescue, Attestation, Retention/Prune, Rollback, Safety Preconditions, Thread Safety, Operational Bounds.
- [ ] Cover synonyms/aliases used across features to ensure complete match to `steps-contract.yaml`.

## Implement missing step definitions

### Retention / Prune

- [x] "Given a target with multiple backup artifacts"
- [x] "Given eligible backups older than retention limits"
- [x] "Given a prune operation completed"
- [x] Assert `prune.result` facts: `pruned_count`, `retained_count`, limits, `backup_tag`.

### Rollback

- [ ] Generate 3-action plan A, B (forced failure), C; assert automatic reverse-order rollback.
- [ ] Validate emitted `rollback` per-action and `rollback.summary` facts; ensure `summary_error_ids` ordering and partial-restore notes when applicable.

### Safety Preconditions

- [x] `SafePath` rejection (.. segments and symlink escapes).
- [x] Read-only / noexec / immutable gating checks (policy stops apply under forbidden root).
- [ ] Source trust policy, strict ownership via `FsOwnershipOracle`.
- [ ] Preservation policy `RequireBasic` STOP behavior.
- [ ] Backup sidecar v2 presence and payload hash verification.

### Thread Safety

- [ ] Assert core types are `Send`/`Sync`.
- [x] Two concurrent `apply()` calls:
  - [x] With `LockManager`: only one mutator proceeds; the other times out with `E_LOCKING` and emits `lock_wait_ms`/`lock_attempts`.
  - [x] Without `LockManager` in dev/test: WARN fact is emitted (`no_lock_manager`).

### Operational Bounds

- [x] Assert post-rename fsync timing step definitions present:
  - [x] within threshold recorded via `duration_ms`.
  - [x] exceeding threshold: `severity=warn` present in `apply.result`.

## Attestation

- [x] Support both Then phrasings:
  - [x] "attestation fields (sig_alg, signature, bundle_hash, public_key_id) are present"
  - [x] "an attestation is attached to the apply.result summary fact …"
- [ ] Negative tests: no attestation on apply failure or DryRun.

## Observability identity stabilization

- [x] Ensure apply.result per-action DryRun vs Commit events are byte-identical after redaction:
  - [x] Build a plan that yields non-noop mutations in both modes.
  - [x] Compare redacted per-action facts by `action_id` after normalizing (`run_id`, `event_id`, `seq`, `switchyard_version`).

## Error taxonomy

- [ ] Map and assert error_id → exit_code expectations and summary chains:
  - `E_LOCKING` (30) covered; expand to `E_EXDEV`, `E_ATOMIC_SWAP`, `E_RESTORE_FAILED`, `E_POLICY`, `E_SMOKE`, etc.
- [x] Validate `summary_error_ids` ordering from specific → general on summary events.

## Golden fixtures (if required by SPEC)

- [ ] Wire validation against `cargo/switchyard/tests/golden/` for plan, preflight, apply, rollback.
- [ ] Document how to add/update fixtures and keep redaction stable.

## Determinism

- [ ] Enforce/verify TS_ZERO in DryRun; document redaction normalization.
- [ ] Capture any random/volatile sources with fixed seeds.
- [x] Simulation toggles:
  - [x] `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`.

## Traceability to requirements

- [ ] Ensure every scenario’s `@REQ-*` tags map to `SPEC/requirements.yaml`.
- [ ] Provide a checklist or script stub to verify tag coverage against the requirements index.

## CI integration

- [x] Command to run: `cargo test -p switchyard` (BDD enabled by default feature).
- [x] Guardrails:
  - [x] No skipped scenarios (runner uses `fail_on_skipped`).
  - [x] Harness-less cucumber test registered in `Cargo.toml` (`[[test]] name = "bdd"; harness = false`).
- [ ] Tips for running subsets and debugging failures.

## Runbook

- [ ] How to run subsets of features by path or tag.
- [ ] Enabling verbose output and logging to file sinks.
- [ ] How to add new step definitions aligned with `steps-contract.yaml`.
- [ ] Debugging schema failures, redaction diffs, and error taxonomy issues.

---

## Summary (what remains to be done)

- Rollback steps and assertions for 3-action plans, including `rollback` and `rollback.summary` facts.
- Safety preconditions: ownership oracle, strict ownership policy, preservation STOP, backup sidecar v2 checks.
- Thread safety: explicit step asserting core types are `Send`/`Sync`.
- Attestation: negative tests (no attestation on failure or DryRun).
- Error taxonomy: broaden mappings beyond `E_LOCKING` to cover all documented errors.
- Determinism: explicitly verify TS_ZERO in DryRun and document normalization/seeding.
- Step mapping table aligned to `steps-contract.yaml` and documentation for sinks/runbook.
- Golden fixtures wiring (if required by SPEC) and traceability checks against requirements index.
