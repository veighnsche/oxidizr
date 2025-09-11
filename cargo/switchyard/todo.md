# Switchyard Production Gaps — Implementation Plan (TODO)

Source: `cargo/switchyard/GAPS.md` and SPEC v1.1 requirements. This plan decomposes each production blocker into concrete code changes, tests, CI gates, and documentation updates. It references the current code layout under `cargo/switchyard/src/`, tests in `cargo/switchyard/tests/`, and planning/spec materials under `cargo/switchyard/PLAN/` and `cargo/switchyard/SPEC/`.

Guiding constraints (from SPEC v1.1): SafePath for all mutating APIs; TOCTOU-safe sequence; deterministic IDs and redact/TS_ZERO for dry-run; production LockManager with bounded wait; minimal smoke suite with auto-rollback; EXDEV degraded mode and telemetry; golden fixtures; schema versioning/migration; thread-safety.

---

## Index of production gaps and owners

- REQ‑R2/REQ‑R3 Rollback topology + idempotency — Owner: Core API (apply/rollback, fs)  
- REQ‑L4 LockManager required in production — Owner: API (apply), Policy  
- REQ‑RC1 Rescue profile availability — Owner: Preflight/Policy, `src/rescue.rs`  
- REQ‑H3 Health verification part of commit — Owner: API (apply), Policy  
- REQ‑F3 Filesystem support verified — Owner: Tests/Orchestrator, Preflight  
- REQ‑CI2/REQ‑CI3 CI zero‑SKIP + golden diff gate — Owner: CI/Docs

Milestone target: 1 sprint. See per‑gap estimates.

---

## 1) REQ‑R2 Restore exact topology (and metadata) — Blocker

Objective: On rollback, the prior on‑disk topology is restored exactly:

- If the target was a regular file, we restore a regular file with prior bytes and mode.
- If it was a symlink, we restore a symlink to the prior destination.
- If it was absent, rollback leaves it absent.

Today’s state:

- `src/fs/symlink.rs::replace_file_with_symlink()` backs up regular files as data and symlinks as a symlink copy, but `restore_file()` treats the backup as a plain file rename back into place.
- No explicit recording of prior_kind = {file, symlink, none}. Tests cover a single replace/restore roundtrip but not all permutations.

Design changes:

- Sidecar metadata: Write a JSON sidecar next to each backup to capture prior topology and minimal attributes required for exact restore. File name: `.<basename>.<backup_tag>.<millis>.bak.meta.json`.
  - Schema v1: `{ "schema":"backup_meta.v1", "prior_kind":"file|symlink|none", "prior_dest": "string?", "mode": "octal-string" }`.
  - We do not store xattrs/ACLs/capabilities in v1 (tracked in audit checklist as future hardening). Mode is preserved as today; UID/GID are not modified by library (documented).
- Backup creation:
  - If target is symlink, keep existing symlink‑to‑symlink backup and write sidecar with `prior_kind=symlink` and `prior_dest` set to the symlink target (resolved relative to parent for portability).
  - If target is regular file, keep the byte copy and write sidecar with `prior_kind=file` and `mode` from `symlink_metadata().permissions().mode()`.
  - If target is absent, write an empty tombstone file named as backup and sidecar with `prior_kind=none`.
- Restore logic:
  - `src/fs/symlink.rs::restore_file()` loads the sidecar (by selecting the matching backup basename). Behavior by `prior_kind`:
    - `file`: ensure `renameat` of backup bytes back to `target`; set mode from sidecar.
    - `symlink`: create/remove and then atomically place a symlink to `prior_dest` (use capability handle + `atomic_symlink_swap()` with a temp staging link under the same parent).
    - `none`: ensure the `target` is removed; no backup rename performed.
  - After successful restore, keep the sidecar (to support idempotency; see R3) and remove only the backup data when applicable.

Code changes:

- `src/fs/symlink.rs`
  - Expand `backup_path_with_tag()` helpers to also compute sidecar path.
  - Update `replace_file_with_symlink()` to always emit sidecar JSON before mutation.
  - Update `restore_file()` to branch on sidecar and perform exact restore per prior_kind; add TOCTOU‑safe operations via parent dir FD.
- `src/api/apply.rs`
  - No change required for call sites (they already call `restore_file()`), but enrich apply facts to include `before_kind` and `after_kind` when cheap to compute.
- `SPEC/requirements.yaml`, `SPEC/SPEC.md` and `SPEC_CHECKLIST.md`
  - Mark R2 as implemented once tests pass; add sidecar description to SPEC §2.2 (Rollback).

Tests:

- `src/fs/symlink.rs` unit tests:
  - file→symlink replace then restore (existing) — assert regular file restored with same bytes and mode.
  - symlink→symlink retarget then restore — assert symlink restored pointing to previous dest.
  - none→symlink then restore — assert target absent after restore.
  - sidecar integrity test: sidecar remains after restore.
- `tests/smoke_rollback.rs` integration: cover all three prior_kind permutations with golden facts.

Acceptance criteria:

- All permutations pass locally and in container runner.
- Golden fixtures updated in `tests/golden/` and traceability updated for R2.

Estimate: 1.0–1.5 days.

Risks/Mitigations:

- Multiple backups per target: we continue selecting the latest by timestamp (already implemented). Sidecars keyed to the same basename avoids mismatch.
- Cross‑fs (EXDEV) on restore symlink: use existing degraded path logic for staging and atomic appearance.

Feasibility & Complexity:

- Feasibility: High — primitives exist for backup/restore and TOCTOU-safe ops.
- Complexity: Medium — add sidecar write/read and restore branching, plus tests.
- Evidence:
  - Current restore is a simple rename of backup → target (topology-agnostic): `src/fs/symlink.rs::restore_file()`.
  - Symlink and file backups already created: `src/fs/symlink.rs::replace_file_with_symlink()`.
  - Kind detection and symlink resolution helpers: `src/api/fs_meta.rs::{kind_of, resolve_symlink_target}`.
  - Audit schema allows additional fields (no additionalProperties=false): `SPEC/audit_event.schema.json`.
- Blockers: None.

---

## 2) REQ‑R3 Idempotent rollback — Blocker

Objective: Rollback may be invoked multiple times (manually or via chained apply/rollback cycles) without error and without mutating already‑restored state.

Design changes:

- Make `restore_file()` idempotent by short‑circuiting when current state matches `prior_kind` from the sidecar, even if the backup payload is no longer present (e.g., already consumed on first restore):
  - If `prior_kind=file` and `target` is a regular file, return Ok.
  - If `prior_kind=symlink` and `target` is a symlink to `prior_dest`, return Ok.
  - If `prior_kind=none` and `target` is absent, return Ok.
- If short‑circuit conditions are not met and the backup payload is missing:
  - Return `NotFound` only when `force_best_effort=false` and state is not already consistent. Otherwise, return Ok to preserve best‑effort semantics.
- Keep sidecar after successful restore to enable idempotent checks.

Code changes:

- `src/fs/symlink.rs::restore_file()` implement idempotent checks before performing rename/create.
- `src/api/apply.rs` rollback loop already tolerates per‑step failures; we will enrich rollback telemetry: when short‑circuit occurs, emit `emit_rollback_step(..., decision="success", reason="idempotent_noop")`.

Tests:

- Unit: call `restore_file()` twice in a row for each prior_kind; second invocation is a no‑op.
- Integration: `tests/smoke_rollback.rs` run end‑to‑end twice and assert no additional errors; golden facts include `reason=idempotent_noop` on second run.

Acceptance criteria:

- R3 flipped to complete in `SPEC_CHECKLIST.md`.

Estimate: 0.5–1.0 days (builds on R2).

Risks/Mitigations:

- Sidecar drift if a third‑party modifies target between runs. We detect divergence and perform restore if possible; otherwise we return appropriate error ids for visibility.

Feasibility & Complexity:

- Feasibility: High — incremental to R2 (sidecar presence enables idempotent checks).
- Complexity: Low–Medium — conditional no-op checks + telemetry reason; unit/integration tests.
- Evidence:
  - Missing-backup behavior currently returns NotFound unless best-effort: `src/fs/symlink.rs::restore_file()`.
  - Rollback loop already tolerates per-step failures and emits facts: `src/api/apply.rs` (rollback sections).
- Blockers: None.

---

## 3) REQ‑L4 LockManager required in production — Blocker

Objective: In production commits, a `LockManager` must be present; absence is an error (not just a warning). Bounded wait and timeout behavior remains unchanged; failing to acquire a lock maps to `E_LOCKING` and includes `lock_wait_ms`.

Design changes:

- Policy toggle: add `Policy.require_lock_manager` (default true). In dev/test, disable via policy for convenience.
- Enforcement in `apply.rs`:
  - Before emitting the initial success `apply.attempt`, check for `api.lock`. If absent and `mode=Commit` and `require_lock_manager=true`, emit an `apply.attempt` with `decision=failure`, `error_id=E_LOCKING`, `exit_code=30`, and return an `ApplyReport` without mutating state.
  - When lock manager is present, retain current bounded wait and fact `lock_wait_ms`.

Code changes:

- `src/policy/config.rs` (or equivalent): add `require_lock_manager: bool` with `Default = true`.
- `src/api/apply.rs`: implement the early failure path (Commit only).
- `SPEC/SPEC.md` §2.5 and `SPEC_CHECKLIST.md`: mark L4 implemented and document policy flag for dev/test override.

Tests:

- `tests/locking_timeout.rs`: add a new case that runs Commit with no lock manager and asserts failure `E_LOCKING` and exit code mapping.
- Golden facts updated to include `no_lock_manager` only as a WARN in dry-run/dev and as ERROR in production when required.

Acceptance criteria:

- L4 checked off in checklist; CI green.

Estimate: 0.5 days.

Feasibility & Complexity:

- Feasibility: High — reuse existing E_LOCKING path and facts; add a policy flag and early-fail.
- Complexity: Low — add `Policy.require_lock_manager` and a short check in `apply.rs` for Commit.
- Evidence:
  - Missing policy flag today: `src/policy/config.rs` has no `require_lock_manager`.
  - Lock timeout path already emits `E_LOCKING`, `lock_wait_ms`: `src/api/apply.rs` (lock acquire block) and `tests/locking_timeout.rs`.
  - Stable error-id/exit-code mapping exists: `src/api/errors.rs`.
- Blockers: None.

---

## 4) REQ‑RC1 Rescue profile available — Blocker

Objective: A verifiable rescue profile is available before commits: either BusyBox or a minimal GNU tool subset on `PATH`. When `Policy.require_rescue=true`, `apply()` must refuse to commit if verification fails.

Today’s state:

- `src/rescue.rs::verify_rescue_tools()` checks for BusyBox or ≥6/10 GNU tools; `apply.rs` already gates when `policy.require_rescue` is true.

Hardening plan (minimal to unblock):

- Expand verification to also ensure executability (e.g., via `access(X_OK)` or spawning `--help` with a very small timeout) when running in containerized tests.
- Record `rescue_profile` details in `plan/preflight` facts: which toolset passed.
- Document operational expectations and how to configure `PATH` in constrained images.

Code changes:

- `src/rescue.rs`: add an optional `verify_exec` sub‑check behind a small timeout (non‑fatal if environment forbids exec in tests; controlled via policy `rescue_exec_check` default false in tests, true in production).
- `src/api/preflight.rs` (or emit from `apply.rs` gating): include `rescue_profile` facts when `require_rescue=true`.

Tests:

- `tests/rescue_preflight.rs`: extend with a case where we clear `PATH` and assert STOP when `require_rescue=true`.
- Orchestrator YAML: add a derivative image that intentionally lacks locales/tools to validate failure mode (expected‑fail in matrix where appropriate).

Acceptance criteria:

- RC1 checked off; docs updated.

Estimate: 0.5 days.

Risks/Mitigations:

- Minimal derivative Docker images may lack even `--help`; keep the exec check policy‑gated.

Feasibility & Complexity:

- Feasibility: High — verification and gating already wired; minimal hardening and facts enrichment.
- Complexity: Low–Medium — small enhancements in `src/rescue.rs` and preflight/apply facts.
- Evidence:
  - Toolset verification function exists: `src/rescue.rs::verify_rescue_tools()`.
  - Preflight STOP when `require_rescue=true`: `src/api/preflight.rs`.
  - Apply gating includes rescue check before mutation: `src/api/apply.rs`.
- Blockers: None.

---

## 5) REQ‑H3 Health verification is part of commit — Blocker

Objective: Commits must include health verification (smoke tests). Failure invokes auto‑rollback unless explicitly disabled by policy; success is required for a successful commit.

Design changes:

- Policy toggle: `Policy.require_smoke_in_commit` (default true in production).
- Enforcement:
  - If `mode=Commit` and `require_smoke_in_commit=true` and no `SmokeTestRunner` is provided, treat as failure:
    - Emit final `apply.result` with `error_id=E_SMOKE` (or `E_POLICY` if we want strict policy semantics) and perform auto‑rollback unless `disable_auto_rollback=true`.
  - If runner is present and returns error, behavior remains: rollback executed unless disabled; ensure summary carries `E_SMOKE` and exit code mapping.

Code changes:

- `src/policy/config.rs`: add flag with sensible defaulting.
- `src/api/apply.rs`: after action loop and before attestation, enforce presence/success of health verification under the policy.
- Docs: `SPEC/SPEC.md §11`, `SPEC_CHECKLIST.md` mark H3 complete; `PLAN/80-testing-mapping.md` updated.

Tests:

- `tests/smoke_rollback.rs`: add case for Commit with `require_smoke_in_commit=true` and missing runner -> failure + optional rollback.
- Golden fixtures updated.

Acceptance criteria:

- H3 checked off; CI green.

Estimate: 0.5 days.

Feasibility & Complexity:

- Feasibility: High — smoke runner path and E_SMOKE mapping already present.
- Complexity: Low — add `Policy.require_smoke_in_commit` and enforce presence/success in Commit.
- Evidence:
  - Smoke failure triggers auto-rollback and emits `E_SMOKE`: `src/api/apply.rs` and `tests/smoke_rollback.rs`.
  - Error mapping present: `src/api/errors.rs` (E_SMOKE → 80).
- Blockers: None.

---

## 6) REQ‑F3 Supported filesystems verified — Blocker

Objective: Demonstrate Switchyard works on a representative set of filesystems (at least ext4 and one additional such as btrfs or xfs) using the EXDEV/degraded path as applicable.

Plan:

- Add end‑to‑end tests that run the symlink swap + restore cycle on loop‑mounted filesystems within the Docker container (privileged job in the test orchestrator):
  - Create sparse files, `mkfs.ext4`, `mkfs.btrfs` (or `mkfs.xfs` when available), mount them at `/mnt/fs_under_test`, and run a small Rust test binary (or use the library test with `#[ignore]` + YAML runner) to execute `replace_file_with_symlink()` and `restore_file()` there.
  - Capture facts and ensure `degraded` flag is correctly set on EXDEV fallbacks and that operations succeed.
- Emit filesystem type in facts for visibility during testing (from `preflight` using `/proc/mounts` or `statfs`).

Code changes:

- Optional: `src/api/fs_meta.rs` helper to fetch fs type for diagnostics; include in apply facts when available.

Tests/Orchestrator:

- Add YAML tasks under `tests/` or `test-orch/container-runner` to perform the mount lifecycle (requires root inside the container). Use the existing orchestrator (`test-orch/`) to run these steps safely and in isolation.
- Mark as mandatory in CI matrix for at least ext4; btrfs/xfs can be additional but should be part of a scheduled job if flaky.

Acceptance criteria:

- F3 checked off; fixture evidence captured; docs updated (`SPEC/features/fs.md` if needed).

Estimate: 1.0–1.5 days depending on orchestrator plumbing.

Risks/Mitigations:

- Docker base image may lack `mkfs.*`; ensure the container runner installs needed packages (`e2fsprogs`, `btrfs-progs`, `xfsprogs`) for the test phase only.

Feasibility & Complexity:

- Feasibility: Medium — product code is ready; infra changes needed for privileged mounts.
- Complexity: High — requires privileged container runs and additional packages in the image plus CI/orchestrator wiring.
- Evidence:
  - Current Dockerfile lacks mkfs tool packages: `test-orch/docker/Dockerfile`.
  - Host orchestrator `docker run` args lack `--privileged`/cap adds: `test-orch/host-orchestrator/dockerutil/run_args.go`.
  - GH Actions workflow does not run the containerized suite; only cargo and golden diff: `.github/workflows/ci.yml`.
- Blockers:
  - GH-hosted runners typically disallow privileged containers. Mitigate via self-hosted runners or make FS matrix non-blocking in GH CI.

---

## 7) REQ‑CI2 / REQ‑CI3 Zero‑SKIP + Golden diff gate — Blocker

Objective: CI must:

- Fail when any test is skipped (Zero‑SKIP gate).
- Run golden diff: produce canonical JSONL facts and compare byte‑for‑byte against committed fixtures.

Plan:

- Golden diff:
  - Ensure tests that produce canon are run with `GOLDEN_OUT_DIR` set to a temp dir, then compare to `cargo/switchyard/tests/golden/` using a stable order (e.g., `sort` + `diff -u`).
  - Add a Makefile target `switchyard-golden` to simplify local updates.
- Zero‑SKIP:
  - Rust tests seldom “skip”; for YAML/task suites, detect `SKIP` markers in logs and fail the job when present.
  - Add a CI step that greps the test output for `SKIP`/`XFAIL` semantics and enforces the policy.
- Wire into `.github/workflows/ci.yml` job for the crate:
  - Run `cargo test -p switchyard -- --nocapture`.
  - Run the golden generator tests with `GOLDEN_OUT_DIR` and compare to repo fixtures.
  - Add matrix entries as appropriate.

Artifacts/Docs:

- Update `docs/` and `cargo/switchyard/README.md` to describe how to update golden fixtures and the zero‑SKIP policy.
- Update `SPEC_CHECKLIST.md` once gates are live.

Estimate: 0.5–1.0 days.

Risks/Mitigations:

- Golden drift: require explicit `GOLDEN_UPDATE=1` (or a make target) to regenerate and commit new fixtures.

Feasibility & Complexity:

- Feasibility: High (CI3 golden diff already integrated); Medium for Zero‑SKIP if YAML runner must be enforced in GH.
- Complexity: Low–Medium — golden is done; zero‑SKIP requires parsing runner logs or integrating `test-orch` in GH.
- Evidence:
  - Golden diff job present: `.github/workflows/ci.yml` (golden-fixtures job) + `test_ci_runner.py` scenarios.
  - GH workflow does not run the Docker orchestrator; zero‑SKIP for YAML is currently outside GH.
- Blockers:
  - Enforcing zero‑SKIP for YAML suites in GH requires Docker privileged access or self-hosted runners.

---

## Cross‑cutting updates

- Traceability: Update `SPEC/traceability.md` to point from REQ IDs to tests and fixtures added here.
- Error IDs & exit codes: Ensure new paths use existing `ErrorId` mapping in `src/api/errors.rs` (`E_LOCKING`, `E_POLICY`, `E_SMOKE`, etc.). Update `SPEC/error_codes.toml` if a new mapping is needed.
- Docs: Align glossary entries (`DOCS/GLOSSARY.md`) for new terms like backup sidecar.
- Concurrency: No changes needed; `Send + Sync` remains satisfied as we only add sidecar read/write under the same parent dir capability.

---

## Delivery plan & sequencing

1) R2 Topology + sidecar (1.0–1.5d)  
2) R3 Idempotent restore using sidecar (0.5–1.0d)  
3) L4 Production lock enforcement (0.5d)  
4) H3 Health‑as‑commit (0.5d)  
5) RC1 Rescue hardening (0.5d)  
6) F3 Filesystems verification (1.0–1.5d)  
7) CI2/CI3 Gates (0.5–1.0d)

Total: ~4.5–6.5 days net engineering.

---

## Definition of Done per gap

- Code merged with tests.
- Golden fixtures updated and diff gate passing.
- `SPEC_CHECKLIST.md` items checked off and SPEC text updated where needed.
- Traceability links updated.
- CI green across matrix.

---

## Pointers (code & docs)

- API: `src/api/apply.rs`, `src/api/rollback.rs`, `src/api/preflight.rs`, `src/api/fs_meta.rs`
- FS helpers: `src/fs/symlink.rs`, `src/fs/atomic.rs`
- Policy: `src/policy/`
- Rescue: `src/rescue.rs`
- Errors: `src/api/errors.rs`, `SPEC/error_codes.toml`
- SPEC: `SPEC/SPEC.md`, `SPEC/requirements.yaml`, `SPEC/traceability.md`, `SPEC_CHECKLIST.md`
- PLAN: `PLAN/*.md` (notably `45-preflight.md`, `50-locking-concurrency.md`, `60-rollback-exdev.md`, `80-testing-mapping.md`)
- Tests: `tests/*.rs`, goldens under `tests/golden/`
- Orchestrator: `test-orch/` (Docker runner) for FS matrix and env‑level tests
