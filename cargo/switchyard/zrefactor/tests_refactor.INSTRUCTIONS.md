# Tests Refactor — Actionable Instructions

Goal: Better organization, faster iteration, and clearer ownership across test layers. Keep smoke assertions inline, move heavy flows to integration tests, and keep E2E scenarios in the repo root `tests/` with a tidy index.

Scope

- Crate-level tests for `cargo/switchyard/` (unit, integration, golden, schema, trybuild).
- Repository E2E scenarios under repo root `tests/` (numbered scenario folders using the orchestrator).
- Inline `#[cfg(test)]` modules inside `src/**`.

Outcomes

- Clear taxonomy and folder layout.
- Minimal inline tests in `src/**` (smoke-only).
- Integration tests grouped by domain (locking, preflight, apply, audit, fs, prune, provenance).
- Top-level E2E scenarios documented and indexed.

---

## 1) Test taxonomy and when to use what

- Unit (inline): `#[cfg(test)]` inside modules
  - Use for pure helpers and small logic branches (under ~30 lines per test file). No filesystem layout or long flows.
  - Examples: tiny helpers in `types/`, calc functions, error-id mapping.

- Integration (crate-level): `cargo/switchyard/tests/*.rs`
  - Use for API flows that touch filesystem or multiple modules (locking, smoke, preflight/apply, schema validation, prune).
  - Group by domain with subfolders (see Layout below).

- Golden tests (crate-level): `cargo/switchyard/tests/golden/**`
  - Canonical JSON and YAML fixtures; compare via redaction.

- Schema tests (crate-level): `audit_event` and related
  - Validate emitted events against `SPEC/audit_event.schema.json` (and future v2).

- Trybuild (crate-level): `cargo/switchyard/tests/trybuild/**`
  - Compile-time examples; API surface guarantees.

- E2E scenarios (repo root): `tests/NN-name/`
  - System-level behavior under container/orchestrator; keep slow paths and distro matrices here.

---

## 2) Target Layout (crate-level)

```
cargo/switchyard/tests/
  common.rs                 # shared helpers (TestEmitter, temp roots, asserts)
  audit/
    audit_schema.rs
    provenance_presence.rs
    preflight_summary_error_id.rs
    summary_error_ids_ownership.rs
  locking/
    lock_wait_fact.rs
    lock_attempts.rs
    locking_required.rs
    locking_timeout.rs
    locking_stage_parity.rs
  preflight/
    preflight_preservation_required.rs
    preflight_suid_sgid.rs
    preflight_yaml.rs
    preflight_yaml_golden.rs
  apply/
    public_api.rs
    smoke_required.rs
    smoke_rollback.rs
    perf_aggregation.rs
    attestation_apply_success.rs
    error_policy.rs
    error_atomic_swap.rs
    error_exdev.rs
    error_restore_failed.rs
  fs/
    restore_invertible_roundtrip.rs
    prune_backups.rs
  trybuild/
    ...
  golden/
    ...
```

Notes

- Convert existing flat files into the above subdirectories by domain.
- Add `mod common;` at the top of each test file to import helpers. A shared `tests/common.rs` already exists with `TestEmitter`, `TestAudit`, and `with_temp_root()`.

---

## 3) Inline tests policy (src/**)

- Keep only smoke-size tests that do not require complex FS setup.
- Move heavy tests from `src/api.rs` (e.g., `rollback_reverts_first_action_on_second_failure`) to `cargo/switchyard/tests/apply/rollback_reverts_first_action.rs`.
- Keep tiny invariants and small helper tests inline (e.g., error ID mapping table sanity).

Acceptance

- `rg -n "mod tests\s*\{" cargo/switchyard/src | xargs -I{} sh -c 'awk "NR==FNR{c++} END{if(c>200) exit 1}" {} /dev/null'` → no inline test module exceeds ~200 loc. (Rule of thumb)
- Heavy FS apply/rollback tests live in integration tests.

---

## 4) Repo root E2E scenarios (tests/)

- Keep numbered scenarios under `tests/NN-name/` (already present) with a schema’d `task.yaml`.
- Ensure `tests/README.md` documents orchestration usage:
  - How to run the orchestrator and any matrix toggles.
  - Infra requirements (locales, privileges) and known skips.
- Create `tests/INDEX.md` listing all scenarios with a one-line description and tags.

Acceptance

- Every `tests/NN-name/` contains `task.yaml` with keys: `name`, `description`, `tags: [..]`, `steps: [...]`.
- `rg -n "^name:|^description:|^tags:|^steps:" tests/**/task.yaml` finds all four keys.

Additional acceptance greps (crate tests):

- All crate integration tests import helpers: `rg -n "^mod common;" cargo/switchyard/tests/*.rs | wc -l` matches the count of non-helper test files (exclude `common.rs`, `trybuild.rs`).
- Domain grouping complete: no top-level test files remain outside `cargo/switchyard/tests/{locking,preflight,apply,fs,audit}/` apart from `common.rs`, `trybuild.rs`, `README.md`, and golden fixtures.

---

## 5) Shared helpers and stability

- Introduce `cargo/switchyard/tests/common.rs` with:
  - TestEmitter/TestAudit (or feature-gated from `logging/test-utils` when added).
  - `with_temp_root()` to create rooted SafePaths.
  - Canon JSON/YAML helpers with redaction applied.
- Avoid ad-hoc helpers duplicated across tests.

Acceptance

- `rg -n "TestEmitter" cargo/switchyard/tests | wc -l` decreases as helpers are centralized.

---

## 6) Execution and CI

- Crate tests
  - `cargo test -p switchyard` runs unit+integration+golden+schema+trybuild.
- E2E scenarios
  - Provide a `make test-e2e` or doc command to invoke the orchestrator.
- Feature matrix
  - Add CI job to compile tests under optional features (serde-reports, jsonl-file-sink, test-utils, etc.).

Acceptance

- CI shows two lanes: `switchyard-tests` and `e2e-scenarios`.

---

## 7) Planned Moves (examples)

Record in `zrefactor/removals_registry.md` and execute as part of batched changes:

- move: cargo/switchyard/tests/locking_timeout.rs -> cargo/switchyard/tests/locking/locking_timeout.rs
- move: cargo/switchyard/tests/lock_wait_fact.rs -> cargo/switchyard/tests/locking/lock_wait_fact.rs
- move: cargo/switchyard/tests/lock_attempts.rs -> cargo/switchyard/tests/locking/lock_attempts.rs
- move: cargo/switchyard/tests/locking_required.rs -> cargo/switchyard/tests/locking/locking_required.rs
- move: cargo/switchyard/tests/locking_stage_parity.rs -> cargo/switchyard/tests/locking/locking_stage_parity.rs
- move: cargo/switchyard/tests/preflight_preservation_required.rs -> cargo/switchyard/tests/preflight/preflight_preservation_required.rs
- move: cargo/switchyard/tests/preflight_suid_sgid.rs -> cargo/switchyard/tests/preflight/preflight_suid_sgid.rs
- move: cargo/switchyard/tests/preflight_yaml.rs -> cargo/switchyard/tests/preflight/preflight_yaml.rs
- move: cargo/switchyard/tests/preflight_yaml_golden.rs -> cargo/switchyard/tests/preflight/preflight_yaml_golden.rs
- move: cargo/switchyard/tests/public_api.rs -> cargo/switchyard/tests/apply/public_api.rs
- move: cargo/switchyard/tests/smoke_required.rs -> cargo/switchyard/tests/apply/smoke_required.rs
- move: cargo/switchyard/tests/smoke_rollback.rs -> cargo/switchyard/tests/apply/smoke_rollback.rs
- move: cargo/switchyard/tests/perf_aggregation.rs -> cargo/switchyard/tests/apply/perf_aggregation.rs
- move: cargo/switchyard/tests/attestation_apply_success.rs -> cargo/switchyard/tests/apply/attestation_apply_success.rs
- move: cargo/switchyard/tests/error_policy.rs -> cargo/switchyard/tests/apply/error_policy.rs
- move: cargo/switchyard/tests/error_atomic_swap.rs -> cargo/switchyard/tests/apply/error_atomic_swap.rs
- move: cargo/switchyard/tests/error_exdev.rs -> cargo/switchyard/tests/apply/error_exdev.rs
- move: cargo/switchyard/tests/error_restore_failed.rs -> cargo/switchyard/tests/apply/error_restore_failed.rs
- move: cargo/switchyard/tests/restore_invertible_roundtrip.rs -> cargo/switchyard/tests/fs/restore_invertible_roundtrip.rs
- move: cargo/switchyard/tests/prune_backups.rs -> cargo/switchyard/tests/fs/prune_backups.rs
- move: cargo/switchyard/tests/audit_schema.rs -> cargo/switchyard/tests/audit/audit_schema.rs
- move: cargo/switchyard/tests/provenance_presence.rs -> cargo/switchyard/tests/audit/provenance_presence.rs
- move: cargo/switchyard/tests/preflight_summary_error_id.rs -> cargo/switchyard/tests/audit/preflight_summary_error_id.rs
- move: cargo/switchyard/tests/summary_error_ids_ownership.rs -> cargo/switchyard/tests/audit/summary_error_ids_ownership.rs

Inline test move:

- move: cargo/switchyard/src/api.rs (test `rollback_reverts_first_action_on_second_failure`) -> cargo/switchyard/tests/apply/rollback_reverts_first_action.rs

---

## 8) Housekeeping and Docs

- Add `cargo/switchyard/tests/README.md` explaining domains, how to run, and common helpers.
- Add `tests/README.md` and `tests/INDEX.md` at repo root; include infra notes and tags.

---

## 9) Acceptance Gates (overall)

- `cargo test -p switchyard` passes.
- `rg -n "mod tests\s*\{" cargo/switchyard/src | wc -l` decreases or stays small; heavy flows moved out.
- Each crate test subfolder contains at least one focused test; no orphaned files.
- Repo root `tests/` folders each contain a valid `task.yaml` and are indexed in `tests/INDEX.md`.

---

## Related

- Cohesion and greps: `zrefactor/responsibility_cohesion_report.md`
- Features overview: `zrefactor/FEATURES_CATALOG.md`
- UX refactor plan: `zrefactor/features_ux_refactor.PROPOSAL.md`
