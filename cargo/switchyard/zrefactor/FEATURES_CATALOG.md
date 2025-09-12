# Switchyard Features Catalog (exhaustive, current as of v0.1.0)

Scope: Library crate `cargo/switchyard/` (public API and implemented behaviors). This catalog lists what the library offers today, with references to sources in the repository for each capability.

---

## Top‑level summary

- Safe, deterministic, auditable apply engine for filesystem changes (primarily atomic symlink replacement) with backup/restore and fail‑closed policy gating.
- Structured facts (JSON) and audit events with stage semantics and error taxonomy.
- Deterministic planning (UUIDv5 IDs), dry‑run parity via redaction, and bounded locking with observability.
- Extensible adapters for locking, ownership, attestation, path resolution, and smoke tests.

---

## 1) Filesystem Safety and Operations

- Atomic symlink replacement with backup/sidecar
  - Implementation: `src/fs/backup.rs`, `src/fs/restore.rs`; atomic rename via `rustix` (`openat`/`renameat`/`fsync`).
  - Behavior: creates a backup payload and a JSON sidecar (meta) next to the target; supports restore and idempotent retries.
  - Tests: `tests/restore_invertible_roundtrip.rs`, `tests/error_atomic_swap.rs`.

- Cross‑filesystem degraded fallback (EXDEV)
  - When `allow_degraded_fs=true`, falls back to unlink + `symlinkat` while emitting `degraded=true`.
  - Tests: `tests/error_exdev.rs`.

- Idempotent restore logic with sidecar guidance
  - Recognizes `prior_kind` (`file` | `symlink` | `none`) and optionally `mode`/`prior_dest`.
  - Integrity check with SHA‑256 when present in sidecar; best‑effort behavior can be policy‑controlled.
  - Tests: `tests/restore_invertible_roundtrip.rs`, `tests/error_restore_failed.rs`.

- Backup pruning API
  - Public API: `Switchyard::prune_backups(&SafePath)` prunes by count/age with safety rules (never delete newest).
  - Emits `prune.result` facts. Emitted fields include: `path`, `backup_tag`, `retention_count_limit`, `retention_age_limit_ms`, `pruned_count`, `retained_count`. On failure includes `error`, `error_id`, and `exit_code`.
  - Source: `src/api.rs::prune_backups`, `src/fs/backup.rs`.
  - Tests: `tests/prune_backups.rs`.

- Preservation intent signaling
  - Fields: `preservation { owner, mode, timestamps, xattrs, acls, caps }` + `preservation_supported` in preflight rows.
  - Source: `SPEC/audit_event.schema.json`, preflight module.
  - Tests: `tests/preflight_preservation_required.rs`.

- YAML export of preflight results
  - Source: `src/preflight/yaml.rs`.
  - Tests: `tests/preflight_yaml.rs`, `tests/preflight_yaml_golden.rs`.

---

## 2) Policy and Preflight Gating

- Policy knobs (flat, current)
  - Examples: `require_rescue`, `require_preservation`, `strict_ownership`, `force_untrusted_source`, `allow_degraded_fs`, `override_preflight`, `extra_mount_checks`, `require_lock_manager`, `allow_unlocked_commit`, `require_smoke_in_commit`, `backup_tag`, `retention_count_limit`, `retention_age_limit`.
  - Source: `src/policy/config.rs`.

- Gating checks in preflight/apply
  - Mount flags (`rw+exec`), immutable files, SUID/SGID risks, strict ownership, scope (`allow_roots`/`forbid_paths`), rescue presence and exec bits, preservation support.
  - Sources: `src/api/preflight/mod.rs`, `src/preflight/checks.rs`.
  - Tests: `tests/preflight_suid_sgid.rs`, `tests/preflight_preservation_required.rs`, `tests/rescue_preflight.rs`.

- Fail‑closed behavior
  - Unless `override_preflight=true`, apply aborts when preflight gates STOP.
  - Tests: `tests/error_policy.rs`, `tests/preflight_summary_error_id.rs`.

- Deterministic preflight rows
  - One row per action with deterministic ordering; includes `current_kind`, `planned_kind`, provenance/preservation fields where applicable.
  - Source: `src/api/preflight/mod.rs`.

---

## 3) Apply and Runtime Workflows

- Locking with bounded wait
  - Adapters trait: `LockManager` with `FileLockManager` implementation.
  - Emits `lock_wait_ms` and derived `lock_attempts` in `apply.attempt` events.
  - Timeout surfaces `E_LOCKING` and exit codes.
  - Sources: `src/adapters/lock/*`, `src/api/apply/mod.rs`.
  - Tests: `tests/locking_timeout.rs`, `tests/lock_wait_fact.rs`, `tests/lock_attempts.rs`, `tests/locking_required.rs`, `tests/locking_stage_parity.rs`.

- Smoke tests and auto‑rollback
  - Optional `SmokeTestRunner` runs after Commit; on failure emits `E_SMOKE` and rolls back unless disabled.
  - Sources: `src/adapters/smoke/*`, `src/api/apply/mod.rs`.
  - Tests: `tests/smoke_required.rs`, `tests/smoke_rollback.rs`.

- Apply rollback on failure
  - Reverse‑ordered rollback of earlier actions; report includes `rolled_back` flag.
  - Source: `src/api.rs::plan_rollback_of` + apply handlers.
  - Test: `src/api.rs` unit test `rollback_reverts_first_action_on_second_failure`.

- Performance metrics aggregation
  - Aggregates per‑action timing into a summary `perf` object in `apply.result` (hash_ms, backup_ms, swap_ms).
  - Sources: `src/api/apply/mod.rs`.
  - Tests: `tests/perf_aggregation.rs`.

- Attestation bundle scaffolding
  - Optional attestation on `apply.result` summary with `{sig_alg, signature, bundle_hash, public_key_id}`.
  - Sources: `src/adapters/attest/*`, `src/api/apply/mod.rs`.
  - Tests: `tests/attestation_apply_success.rs`.

---

## 4) Logging, Facts, and Audit Schema

- Structured JSON facts for all stages
  - Stages: `plan`, `preflight`, `apply.attempt`, `apply.result`, `rollback`, `rollback.summary`, `prune.result`.
  - Common envelope: `schema_version`, `ts`, `plan_id`, `action_id?`, `decision`, `severity?`, `degraded?`, `dry_run`.
  - Source: `SPEC/audit_event.schema.json`, `src/logging/audit.rs`.
  - Tests: `tests/audit_schema.rs`.

- Error taxonomy and summary fields
  - Fields: `error_id`, `exit_code`, `error_detail`, `summary_error_ids` on summaries.
  - Sources: `SPEC/SPEC.md` (Error taxonomy), `src/api/errors.rs`.
  - Tests: `tests/preflight_summary_error_id.rs`, `tests/summary_error_ids_ownership.rs`.

- Provenance and preservation
  - `provenance` object and `preservation` object included where applicable.
  - Source: `SPEC/audit_event.schema.json`; validated in tests.

- Redaction layer for DryRun==Commit parity
  - Removes volatile fields and masks secrets; helper `redact_event()`.
  - Sources: `src/logging/redact.rs`, used by `src/logging/audit.rs`.

- Sinks and emitters
  - Traits: `FactsEmitter`, `AuditSink` with a built‑in `JsonlSink` (opt‑in file logging).
  - Source: `src/logging/facts.rs` (`file-logging` feature gated for FileJsonlSink persistence).

---

## 5) Determinism and Types

- Deterministic IDs
  - `plan_id` and `action_id` use UUIDv5; stable across DryRun/Commit and runs with the same plan.
  - Source: `src/api/plan.rs`, `src/api/apply/mod.rs`.

- Dry‑run timestamp parity
  - DryRun timestamps are zeroed in canon; `dry_run=true` included in facts.
  - Sources: `src/logging/audit.rs`, tests inside `src/api.rs`.

- `SafePath` for all mutating operations
  - Prevents path traversal; anchors operations under a caller‑controlled root.
  - Source: `src/types/safepath.rs`.

- Reports and plan types
  - `Plan`, `PlanInput`, `PreflightReport`, `ApplyReport`, `ApplyMode`.
  - Source: `src/types/*` and `src/api.rs` surface.

---

## 6) Adapters and Extensibility

- Locking
  - Trait: `LockManager`; default: `adapters::lock::file::FileLockManager`.
  - Source: `src/adapters/lock/*`.

- Ownership
  - Trait: `OwnershipOracle`; default: `adapters::ownership::fs::FsOwnershipOracle`.
  - Source: `src/adapters/ownership/*`.

- Path resolution
  - Module: `src/adapters/path/*` (helpers for root‑relative file system ops alongside `SafePath`).

- Attestation
  - Trait/module: `src/adapters/attest/*` (optional attestation support).

- Smoke tests
  - Trait: `SmokeTestRunner` and helpers under `src/adapters/smoke/*`.

- Emission
  - Traits: `FactsEmitter`, `AuditSink` (bring your own sinks or use `JsonlSink`).

---

## 7) Testing and Conformance

- Unit and integration tests
  - Under `tests/*` covering schema validation, locking paths, preflight gates, provenance presence, prune, smoke flows, error mappings, and performance metrics.

- Golden fixtures
  - Some golden files under `tests/golden/*`; environment variable hooks for canon output.

- Trybuild and compile‑time examples
  - `dev-dependencies`: `trybuild`; see `tests/trybuild/*`.

- CI/quality posture
  - `#![forbid(unsafe_code)]` and explicit error handling; schema compliance tests.

---

## 8) Cargo Features

- `file-logging`
  - Enables a file‑backed JSONL sink (`FileJsonlSink`) for facts/audit persistence.
  - Source: `Cargo.toml [features]`, `src/logging/facts.rs` guarded `#[cfg(feature = "file-logging")]`.

---

## 9) Public API Surface (summary)

- `Switchyard::new(facts, audit, policy)` with builder‑style setters:
  - `.with_lock_manager(Box<dyn LockManager>)`
  - `.with_ownership_oracle(Box<dyn OwnershipOracle>)`
  - `.with_attestor(Box<dyn Attestor>)`
  - `.with_smoke_runner(Box<dyn SmokeTestRunner>)`
  - `.with_lock_timeout_ms(u64)`
- Planning and execution:
  - `.plan(PlanInput) -> Plan`
  - `.preflight(&Plan) -> Result<PreflightReport, ApiError>`
  - `.apply(&Plan, ApplyMode) -> Result<ApplyReport, ApiError>`
  - `.plan_rollback_of(&ApplyReport) -> Plan`
  - `.prune_backups(&SafePath) -> Result<PruneResult, ApiError>`
- Source: `src/api.rs` (facade delegating to `src/api/*`).

---

## 10) Known limitations (non‑features; contextual)

- Policy is flat (booleans) in current implementation; central evaluator and grouped types are planned in `zrefactor/policy_refactor.INSTRUCTIONS.md`.
- Logging facade (StageLogger/EventBuilder) is planned; today orchestrators still use `emit_*` helpers.
- FS backup/restore are monolithic files pending split into cohesive submodules per `zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`.
- No `tracing` spans; observability via facts/audit only.

---

## References

- README (features, concepts): `cargo/switchyard/README.md`
- API surface and traits: `cargo/switchyard/src/api.rs`, `cargo/switchyard/src/adapters/*`, `cargo/switchyard/src/types/*`
- Filesystem operations: `cargo/switchyard/src/fs/backup.rs`, `cargo/switchyard/src/fs/restore.rs`
- Preflight helpers: `cargo/switchyard/src/preflight/*`
- Logging schema and sinks: `cargo/switchyard/SPEC/audit_event.schema.json`, `cargo/switchyard/src/logging/*`
- Tests (capabilities coverage): `cargo/switchyard/tests/*`
