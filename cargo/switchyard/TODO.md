# Switchyard TODO

This document tracks the near-term and medium-term tasks for the `switchyard` crate.
It is structured for day-to-day engineering, with explicit code pointers, acceptance
criteria, and suggested test coverage. Use this as the single source of truth for
work planning within the crate.

Last updated: 2025-09-14

## Conventions

- Use feature branches with focused PRs and crisp commit messages.
- Keep public API breaking changes grouped under a single coordinated refactor (when planned), otherwise prefer additive changes.
- Add tests alongside code under `cargo/switchyard/tests/` and update BDD features under `cargo/switchyard/SPEC/features/` when behavior changes.
- Ensure `cargo check`, unit/integration tests, BDD tests (`--features bdd`), and trybuild tests are green locally before PR.

---

## Immediate Priorities (P0)

- [ ] BDD attestation summary is missing
  - Symptom: `Then attestation fields (sig_alg, signature, bundle_hash, public_key_id) are present` fails in `SPEC/features/observability_audit.feature`.
  - Code: `src/api/apply/mod.rs` → `summary::ApplySummary::attestation()`; `src/api/apply/summary.rs`; `src/adapters/attest.rs`.
  - Acceptance: On successful Commit apply with an Attestor configured, summary `apply.result` contains `attestation` object with the 4 fields; related BDD scenario passes.
  - Tests: `tests/steps/attestation_steps.rs` scenario passes; add a unit test for `build_attestation_fields()` resilience.

- [ ] Implement missing BDD step glue for SPEC coverage (see section below)
  - Add step modules and glue for: rollback, safety preconditions, thread safety.
  - Acceptance: No "Step doesn't match any function" failures when running `cargo test -p switchyard --features bdd --test bdd`.

- [ ] Add cucumber CLI filtering to the BDD runner (optional dev UX)
  - File: `tests/bdd_main.rs`.
  - Goal: Allow `--tags` and scenario filters to speed up local runs.
  - Acceptance: Running the bdd binary directly (or through `cargo test`) honors cucumber CLI flags.

---

## BDD Coverage Parity (SPEC ⇒ Steps)

Ensure every scenario in `SPEC/features/` has step glue under `tests/steps/`. Items below enumerated by feature file.

### Rollback (`SPEC/features/rollback.feature`)

- [ ] Given a plan with three actions A, B, C where B will fail
  - Implement builder that creates three `EnsureSymlink` actions under temp root; force failure for B (e.g., forbid target path via `policy.scope.forbid_paths`, or simulate via overrides if applicable).
- [ ] When I apply the plan in Commit mode
  - Reuse existing apply step or add alias.
- [ ] Then the engine rolls back A in reverse order automatically
  - Inspect emitted facts for `rollback` and `rollback.summary`; assert reverse order and partial restoration state visibility.
- [ ] And emitted facts include partial restoration state if any rollback step fails
  - Ensure per-action facts or summary indicate failures during rollback.
- [ ] Given a plan that replaces a symlink then restores it
  - Construct forward plan then `plan_rollback_of()` and apply twice; verify topology.
- [ ] When I apply the plan and then apply a rollback plan twice
- [ ] Then the final link/file topology is identical to the prior state
  - Verify with filesystem inspection under temp root.

Step file: `tests/steps/rollback_steps.rs` (new).

### Safety Preconditions (`SPEC/features/safety_preconditions.feature`)

- [ ] Given a candidate path containing .. segments or symlink escapes
  - Implement using `SafePath::from_rooted()` with invalid candidate; assert rejection.
- [ ] When I attempt to construct a SafePath
- [ ] Then SafePath normalization rejects the path as unsafe
- [ ] Given the target filesystem is read-only or noexec or immutable
  - Already implemented via forbidding root; confirm reuse.
- [ ] When I attempt to apply a plan / Then operations fail closed with a policy violation error
  - Implemented.
- [ ] Given a source file that is not root-owned or is world-writable
  - Create temp file and `chmod 0o666` (or change owner when feasible); rely on ownership oracle or policy logic.
- [ ] Then preflight fails closed unless an explicit policy override is present
- [ ] Given strict_ownership=true policy / And a target not package-owned per oracle
  - Enable `policy.risks.ownership_strict = true`; set a dummy oracle that reports not owned.
- [ ] Then preflight fails closed
- [ ] Given the policy requires preserving owner, mode, timestamps, xattrs, ACLs, and caps
  - Set `policy.durability.preservation = RequireBasic` and simulate unsupported preservation via `fs::meta::detect_preservation_capabilities` outcome.
- [ ] Then preflight stops with a fail-closed decision unless an explicit override is set
- [ ] Given a backup sidecar v2 with payload present / When I restore under policy requiring sidecar integrity / Then the engine verifies payload hash and fails restore on mismatch
  - Create backup via a previous apply path, then tamper with sidecar/payload before restore.

Step file: `tests/steps/safety_preconditions_steps.rs` (new). May also extend `bdd_support/` with helper creators.

### Thread Safety (`SPEC/features/thread_safety.feature`)

- [ ] Given the Switchyard core types / Then they are Send + Sync for safe use across threads
  - Add a compile-time assertion helper (e.g., `fn assert_send_sync<T: Send + Sync>() {}`) and use it with key types (`Switchyard<_,_>`, `SafePath`, etc.). This can be in a test module.
- [ ] Given two threads invoking apply() concurrently / And a LockManager is configured / When both apply() calls run / Then only one mutator proceeds at a time under the lock
  - Similar to `tests/steps/locks_steps.rs::when_two_apply_overlap`, but assert mutual exclusion via facts or outcomes (only one success at a time).

Step file: `tests/steps/thread_safety_steps.rs` (new).

---

## Observability & Schema v2

- [ ] Preflight summary must include `summary_error_ids` on failures (done), keep parity.
  - Files: `src/api/preflight/mod.rs` (already emits on failure); maintain contract.
- [ ] Apply summary should include `duration_ms`, `perf` fields, and conditionally `severity=warn` when fsync_ms exceeds threshold (already present).
  - Files: `src/api/apply/summary.rs`, `src/api/apply/executors/ensure_symlink.rs`.
- [ ] Determinism in DryRun
  - Ensure timestamps are zeroed and volatile fields redacted (`src/logging/redact.rs`).
  - Verify equality in BDD redaction checks.

---

## Locking & Concurrency

- [ ] Ensure `apply.attempt` is emitted with `lock_backend`, `lock_wait_ms`, `lock_attempts` (done, keep parity).
  - Files: `src/api/apply/lock.rs`.
- [ ] Emit E_LOCKING summary on acquisition failure with early return (done).
- [ ] Respect `allow_unlocked_commit` when no lock manager; emit WARN attempt (done).
- [ ] Add explicit BDD step for "Only one mutator proceeds under lock" (see Thread Safety section).

---

## Policy & Preflight

- [ ] Gate on rescue profile availability; prefer per-instance Overrides over env (done in tests).
  - Files: `src/policy/rescue.rs`, `src/api/preflight/mod.rs`, `src/api/overrides.rs`.
- [ ] Ownership oracle integration for strict ownership gating
  - Files: `src/adapters/ownership/` and API wiring.
  - Tests: add an oracle stub in tests to confirm `ownership_strict` policies are enforced.
- [ ] Preservation capability detection drives STOP when required (done in preflight); add tests for unsupported env.

---

## Filesystem Safety & Atomicity

- [ ] Maintain TOCTOU-safe sequence in swaps via `open parent O_DIRECTORY|O_NOFOLLOW → *at → renameat → fsync(parent)`.
  - Files: `src/fs/swap.rs`, `src/fs/atomic.rs`.
- [ ] Backup creation and restore round-trip correctness under `fs/backup` and `fs/restore`.
  - Add tests for hash preservation in sidecar v2 when policy requires integrity.
- [ ] Cross-filesystem handling (EXDEV) and degraded fallback knob via policy + overrides.

---

## Attestation

- [ ] Ensure `apply.result` summary includes `attestation` on success when attestor configured.
  - Files: `src/api/apply/mod.rs`, `src/api/apply/summary.rs`, `src/adapters/attest.rs`.
  - Tests: `tests/steps/attestation_steps.rs`, plus unit test for signing failure path (should not panic; no attestation attached).

---

## Retention & Prune

- [ ] Keep `prune.result` parity: include `path`, `backup_tag`, `retention_*`, `pruned_count`, `retained_count`.
  - Files: `src/api/mod.rs::prune_backups()`.
  - Tests: BDD already asserts presence; add edge-case tests for zero pruned/retained.

---

## Error Taxonomy & Exit Codes

- [ ] Maintain stable mapping of error IDs and exit codes
  - Files: `src/api/errors.rs` and `src/api/apply/summary.rs` (smoke vs policy mapping).
  - Tests: Add coverage for `E_EXDEV`, `E_POLICY`, `E_LOCKING`, `E_SMOKE` mapping in summaries.

---

## API Surface & Builder

- [ ] Keep `Switchyard::builder` as default construction path; public re-exports stable in `src/lib.rs`.
- [ ] Overrides are per-instance; prefer not to rely on process-global env in library logic.
- [ ] Consider adding a typed `ApiError` to more public methods (work-in-progress).

---

## Observability Sinks & Redaction

- [ ] Ensure `AuditSink` and `FactsEmitter` remain minimal and JSONL-friendly (`src/logging/facts.rs`).
- [ ] Redaction masks secrets and volatile identifiers; confirm masking for `provenance.helper` and attestation fields in redacted outputs.

---

## Determinism & Reproducibility

- [ ] UUIDv5 derivation rules for `plan_id` and deterministic identifiers are stable (`src/types/ids.rs`).
- [ ] Dry-run timestamps are zero (`TS_ZERO`); Commit uses real timestamps.
- [ ] BDD redaction-based equality tests remain green.

---

## CI & Dev Experience

- [ ] Add a CI job to run BDD: `cargo test -p switchyard --features bdd --test bdd` (matrix opt-in).
- [ ] Keep trybuild tests current (`tests/trybuild/`).
- [ ] Ensure rust-toolchain is compatible; keep clippy level sensible.

---

## Documentation

- [ ] Update `SPEC/SPEC.md` when behavior changes (fields, error mapping, stages).
- [ ] Add a short "How to run BDD" section in `README.md` of the crate:
  - `cargo test -p switchyard --features bdd --test bdd`
- [ ] Cross-link `SPEC/features/` and step files for contributors.

---

## Nice-to-haves / Backlog

- [ ] Enhanced perf telemetry: per-phase timers beyond hash/backup/swap.
- [ ] Optional tracing feature to emit spans aligned with audit facts.
- [ ] Ownership oracle: pluggable backends and richer provenance.
- [ ] Sidecar integrity: integrate signature verification path when available.

---

## Execution Checklist (maintainers)

1. Implement missing step glue and re-run BDD until zero unmatched steps remain.
2. Fix attestation summary emission; re-run observability-audit scenarios.
3. Add CLI filtering to BDD runner for dev speed.
4. Land CI job for BDD behind a feature flag.
5. Keep SPEC and tests in lockstep for any behavior changes.
