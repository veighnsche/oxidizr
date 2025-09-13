# BDD Adapter — Remaining TODOs

This document tracks the remaining work to fully wire the BDD adapter so that all Gherkin features under `cargo/switchyard/SPEC/features/` execute against the Switchyard Rust library with 100% step coverage and green results.

See reference guide: `cargo/switchyard/SPEC/features/bdd_adapter_guide.md`

## World design and discipline

- Finalize the `World` fields and lifecycle:
  - `facts`, `audit`, `policy`, `plan`, `preflight`, `apply_report`, temp root (`TempDir`/`PathBuf`), `lock_path`.
  - Optional handles: `Attestor`, `LockManager`, `OwnershipOracle`, `SmokeTestRunner`.
- Document and enforce test FS discipline:
  - All filesystem operations occur under a temporary directory.
  - All paths must be `SafePath::from_rooted` under the temp root.
  - Never touch system paths in scenarios.

## Sinks and logging

- Document the in-memory `CollectingEmitter` (FactsEmitter) and `CollectingAudit` (AuditSink).
- Provide instructions to use file-backed JSONL sinks behind `--features file-logging` for debugging.

## Step mapping (align to steps-contract.yaml)

- Add a canonical table mapping Given/When/Then regex → Switchyard API calls and assertions.
  - Plan, Preflight, Apply (Commit/DryRun) flows.
  - Locking, Rescue, Attestation, Retention/Prune, Rollback, Safety Preconditions, Thread Safety, Operational Bounds.
- Cover synonyms/aliases used across features to ensure complete match to `steps-contract.yaml`.

## Implement missing step definitions

### Retention / Prune

- "Given a target with multiple backup artifacts"
- "Given eligible backups older than retention limits"
- "Given a prune operation completed"
- Assert `prune.result` facts: `pruned_count`, `retained_count`, limits, `backup_tag`.

### Rollback

- Generate 3-action plan A, B (forced failure), C; assert automatic reverse-order rollback.
- Validate emitted `rollback` per-action and `rollback.summary` facts; ensure `summary_error_ids` ordering and partial-restore notes when applicable.

### Safety Preconditions

- `SafePath` rejection (.. segments and symlink escapes).
- Read-only / noexec / immutable gating checks (preflight).
- Source trust policy, strict ownership via `FsOwnershipOracle`.
- Preservation policy `RequireBasic` STOP behavior.
- Backup sidecar v2 presence and payload hash verification.

### Thread Safety

- Assert core types are `Send`/`Sync`.
- Two concurrent `apply()` calls:
  - With `LockManager`: only one mutator proceeds; the other times out with `E_LOCKING`.
  - Without `LockManager` in dev/test: WARN fact is emitted (`no_lock_manager`).

### Operational Bounds

- Assert post-rename fsync timing:
  - within threshold: `FSYNC_WARN_MS` not exceeded.
  - exceeding threshold: `severity=warn` present in `apply.result`.

## Attestation

- Support both Then phrasings:
  - "attestation fields (sig_alg, signature, bundle_hash, public_key_id) are present"
  - "an attestation is attached to the apply.result summary fact …"
- Negative tests: no attestation on apply failure or DryRun.

## Observability identity stabilization

- Ensure apply.result per-action DryRun vs Commit events are byte-identical after redaction:
  - Build a plan that yields non-noop mutations in both modes.
  - Compare redacted per-action facts by `action_id` after normalizing (`run_id`, `event_id`, `seq`, `switchyard_version`).

## Error taxonomy

- Map and assert error_id → exit_code expectations and summary chains:
  - `E_LOCKING` (30), `E_EXDEV`, `E_ATOMIC_SWAP`, `E_RESTORE_FAILED`, `E_POLICY`, `E_SMOKE`, etc.
- Validate `summary_error_ids` ordering from specific → general on summary events.

## Golden fixtures (if required by SPEC)

- Wire validation against `cargo/switchyard/tests/golden/` for plan, preflight, apply, rollback.
- Document how to add/update fixtures and keep redaction stable.

## Determinism

- Enforce TS_ZERO in DryRun; document redaction normalization.
- Capture any random/volatile sources with fixed seeds.
- Simulation toggles:
  - `SWITCHYARD_FORCE_EXDEV`, `SWITCHYARD_FORCE_RESCUE_OK`.

## Traceability to requirements

- Ensure every scenario’s `@REQ-*` tags map to `SPEC/requirements.yaml`.
- Provide a checklist or script stub to verify tag coverage against the requirements index.

## CI integration

- Command to run: `cargo test -p switchyard --features bdd`
- Guardrails:
  - No skipped scenarios (except explicitly tagged `@xfail`).
  - Harness-less cucumber binary registered in `Cargo.toml` (`[[test]] name = "bdd"; harness = false`).
- Tips for running subsets and debugging failures.

## Runbook

- How to run subsets of features by path or tag.
- Enabling verbose output and logging to file sinks.
- How to add new step definitions aligned with `steps-contract.yaml`.
- Debugging schema failures, redaction diffs, and error taxonomy issues.
