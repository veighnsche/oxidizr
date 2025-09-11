# Switchyard TODO (Implementation & SPEC Conformance)

This is the end‑to‑end implementation backlog for the `switchyard` crate, derived from:

- `cargo/switchyard/SPEC/SPEC.md` and `SPEC/requirements.yaml`
- `cargo/switchyard/PLAN/*` (architecture, delivery plan, quality gates)
- Current code under `cargo/switchyard/src/`

Legend:

- [x] Done
- [ ] TODO (not implemented)
- [~] Partial (some parts present, needs completion)

---

## Implementation Status Summary

- Core mechanics exist (atomic symlink swap, backups, SafePath, basic preflight).
- New since last update:
  - Deterministic IDs module (`types/ids.rs`) with UUIDv5 `plan_id`/`action_id`; wired into `api::apply` for execution flow.
  - Transactional `apply()` with reverse-order rollback for previously executed `EnsureSymlink` steps and `plan_rollback_of()` (basic inverse for symlink actions).
  - `ApplyReport` extended to include `plan_uuid`, `rolled_back`, and `rollback_errors` for downstream audit/rollback planning.

## Milestones & Next Sprint

- Milestone M1 (in progress):
  - Harden transactional engine invariants and tests.
  - Complete TOCTOU sequence with `openat` for final component in `src/fs/*`.
  - Minimal facts emission (schema v1 core fields) including `plan_id`/`action_id`.

- Next Sprint Focus:
  - Implement `openat` path in `fs/atomic.rs`/`fs/symlink.rs` to meet REQ‑TOCTOU1.
  - Emit minimal facts (stage, decision, schema_version, plan_id/action_id, path) and add unit tests.
  - Add unit tests for rollback and SafePath negatives.

## 1) Crate Scaffolding & Module Layout

- [x] Library crate scaffolding and module tree present
  - Files: `Cargo.toml`, `src/lib.rs`, `src/api.rs`, `src/preflight.rs`, `src/rescue.rs`, `src/fs/*`, `src/types/*`, `src/adapters/*`, `src/logging/*`, `src/policy/*`
- [x] Public modules exported via `src/lib.rs`
- [ ] Top‑level crate docs and README for the crate (usage, safety model, adapter model)

## 2) Public API Surface (SPEC §3.1)

- [~] `plan(input: PlanInput) -> Plan` exists in `src/api.rs` (basic link/restore expansion)
- [~] `preflight(plan: &Plan) -> PreflightReport` exists with basic checks (`src/preflight.rs`, `src/api.rs`)
- [~] `apply(plan: &Plan, mode: ApplyMode) -> ApplyReport` exists (symlink swap + restore)
- [~] `plan_rollback_of(report: &ApplyReport) -> Plan` implemented (basic inverse for symlink actions) and exported
- [x] All path‑carrying fields in mutating API use `SafePath` (see `src/types/plan.rs`)

## 3) SafePath & TOCTOU Safety (SPEC §2.3, §3.3)

- [x] `SafePath` type rejects `..` and escaping roots (`src/types/safepath.rs`)
- [~] All mutating entry points accept `SafePath` (API uses `SafePath` but `fs/*` operate on `&Path` internally)
- [~] TOCTOU‑safe syscall sequence
  - Present: `open_dir_nofollow`, `renameat`, `fsync(parent)` (`src/fs/atomic.rs`)
  - Missing: `openat` on final component for symlink create/replace; enforce sequence consistently across all mutations
- [ ] Unit/property tests for `SafePath` normalization and negative cases beyond `rejects_dotdot`

## 4) Atomicity & Rollback Engine (SPEC §2.1, §2.2)

- [~] Atomic symlink swap exists (`atomic_symlink_swap`) and backup/restore flows (`src/fs/symlink.rs`)
- [~] Transactional apply engine with all‑or‑nothing semantics
  - Auto reverse‑order rollback on mid‑plan failure (REQ‑R4)
  - Idempotent rollback (REQ‑R3) with property tests
  - Partial restoration state captured if rollback fails (REQ‑R5)
- [ ] `ApplyReport` extended to capture granular results and rollback status needed by `plan_rollback_of`

## 5) Preflight Preconditions & Policy Gating (SPEC §2.3)

- [~] Implemented checks:
  - mount `rw` and not `noexec` (best‑effort via `/proc/self/mounts`) (REQ‑S2)
  - immutable bit via `lsattr -d` (best‑effort) (REQ‑S2)
  - source ownership/world‑writable gating with `policy.force_untrusted_source` (REQ‑S3)
  - `allow_roots` and `forbid_paths` enforcement (policy)
- [ ] Strict target ownership via `OwnershipOracle` when `policy.strict_ownership` (REQ‑S4)
- [ ] Preservation capability gating: detect whether ownership/mode/timestamps/xattrs/ACLs/caps can be preserved and STOP if policy requires but unsupported (REQ‑S5)
- [ ] Emit structured preflight diff rows per `SPEC/preflight.yaml` (and ensure dry‑run byte identity)

## 6) Cross‑Filesystem & Degraded Mode (SPEC §2.10)

- [ ] EXDEV fallback path: safe copy + fsync + rename strategy (REQ‑F1)
- [ ] Policy hook `allow_degraded_fs`; record `degraded=true` in facts when used; fail when disallowed (REQ‑F2)
- [ ] Acceptance coverage for ext4/xfs/btrfs/tmpfs semantics (REQ‑F3)

## 7) Locking & Concurrency (SPEC §2.5, §14)

- [ ] Integrate `LockManager` with bounded wait; record `lock_wait_ms`; timeout → `E_LOCKING` (REQ‑L1..L3)
- [ ] Emit WARN fact when no `LockManager` (dev/test only) (REQ‑L2)
- [ ] Ensure core types and engine are `Send + Sync` and document thread‑safety (SPEC §14)

## 8) Determinism & Redactions (SPEC §2.7)

- [~] Deterministic `plan_id` and `action_id` (UUIDv5 over normalized inputs/namespace) (REQ‑D1)
- [ ] Dry‑run redactions pinned (timestamps zeroed/expressed as deltas); dry‑run facts byte‑identical to real‑run after redaction (REQ‑D2)
- [ ] Stable ordering of plan actions and fact emission streams

## 9) Observability & Audit (SPEC §2.4, §5, §13)

- [~] Traits exist: `FactsEmitter`, `AuditSink`, `JsonlSink` (stubs)
- [ ] Emit structured facts for every step, validating against `SPEC/audit_event.schema.json` (REQ‑O1, REQ‑O3, REQ‑VERS1)
- [ ] Compute and record `before_hash`/`after_hash` (sha256) for every mutated file (REQ‑O5)
- [ ] Signed attestation per apply bundle via `Attestor` (ed25519) (REQ‑O4)
- [ ] Secret masking policy and implementation across all sinks (REQ‑O6)
- [ ] Provenance completeness fields populated (origin/helper/uid/gid/pkg/env_sanitized) (REQ‑O7)
- [ ] Golden JSONL fixtures for plan, preflight, apply, rollback facts; CI diff gate (SPEC §12)

## 10) Conservatism & Modes (SPEC §2.8)

- [x] `ApplyMode` defaults to `DryRun` (REQ‑C1)
- [ ] Explicit operator approval path for side effects where relevant (library API + top‑level app integration guidance)
- [ ] Fail‑closed behavior on critical compatibility violations unless policy overrides (REQ‑C2) — ensure coverage of all critical checks

## 11) Health Verification & Auto‑Rollback (SPEC §2.9, §11)

- [~] Trait exists: `SmokeTestRunner`
- [ ] Integrate smoke tests post‑apply with minimal suite (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date) (REQ‑H1)
- [ ] Auto‑rollback on smoke failure unless explicitly disabled by policy (REQ‑H2, H3)

## 12) Rescue Profile (SPEC §2.6)

- [ ] Maintain/verify rescue profile (backup symlink set) always available (REQ‑RC1)
- [ ] Preflight verifies at least one functional fallback path/toolset on PATH (GNU or BusyBox) (REQ‑RC2, RC3)
- [ ] Replace `rescue::verify_rescue_tools()` stub with real checks and facts

## 13) Policy Model

- [~] Policy struct present with `allow_roots`, `forbid_paths`, `strict_ownership`, `force_untrusted_source`, `force_restore_best_effort` (`src/policy/config.rs`)
- [ ] Defaults and builder API for policy; documentation of semantics
- [ ] Policy options for degraded FS, auto‑rollback disable, provenance requirements, secret masking, etc.

## 14) Error Model & Taxonomy (SPEC §6)

- [~] Basic error types exist (`src/types/errors.rs`)
- [ ] Map errors to SPEC exit codes / stable identifiers (E_POLICY, E_LOCKING, etc.)
- [ ] Enrich error contexts; plumb through to facts

## 15) Filesystem Ops Hardening

- [ ] Use `openat` + `O_NOFOLLOW` for final component operations to fully meet TOCTOU sequence (REQ‑TOCTOU1)
- [ ] Bound `fsync(parent)` timing ≤ 50ms after rename; record telemetry (REQ‑BND1)
- [ ] Preserve and/or gate ownership/mode/timestamps/xattrs/ACLs/caps as required by policy (ties to REQ‑S5)

## 16) Adapters & Integration Contracts (SPEC §3.2)

- [~] Traits defined: `OwnershipOracle`, `LockManager`, `PathResolver`, `Attestor`, `SmokeTestRunner`
- [ ] Provide reference implementations/mocks and adapter wiring in `api::apply`
- [ ] Steps contract mapping (see `SPEC/features/steps-contract.yaml`) to adapter calls

## 17) Testing & CI (SPEC §8, §12)

- [ ] Unit tests for `fs/*`, `types/*`, policy/preflight, determinism IDs, locking behavior
- [ ] BDD features wired up with `cargo/bdd-runner` and golden fixtures
- [ ] Property tests for invariants: `AtomicReplace`, `IdempotentRollback`
- [ ] CI gates: zero‑SKIP, schema validation, golden diffs, traceability report generation (`SPEC/tools/traceability.py`)

## 18) Documentation

- [ ] Crate‑level docs: guarantees, safety model, adapter interfaces, examples
- [ ] Module‑level docs for `fs`, `preflight`, `api`, `types`, `adapters`, `logging`, `policy`
- [ ] Update `PLAN/20-spec-traceability.md` as implementation proceeds

---

# SPEC Requirement Checklist

Below maps each requirement from `SPEC/requirements.yaml` to current status.

- [~] REQ‑A1 Atomic crash‑safety — atomic symlink swap present; needs full engine + tests
- [~] REQ‑A2 No broken/missing path visible — covered by atomic swap; needs invariants/tests
- [ ] REQ‑A3 All‑or‑nothing per plan — implement transactional engine + auto‑rollback

- [ ] REQ‑R1 Rollback reversibility — implement rollback plan/apply
- [ ] REQ‑R2 Restore exact topology — extend backup/restore and tests
- [ ] REQ‑R3 Idempotent rollback — property tests and logic
- [ ] REQ‑R4 Auto reverse‑order rollback — engine feature
- [ ] REQ‑R5 Partial restoration facts — emit on rollback error

- [~] REQ‑S1 Safe paths only — `SafePath` exists and used; extend tests/coverage
- [~] REQ‑S2 Reject unsupported FS states — mount flags + immutability checks exist; harden and make authoritative
- [ ] REQ‑S3 Source ownership gating — present but enrich and fact‑backed
- [ ] REQ‑S4 Strict target ownership — integrate `OwnershipOracle`
- [ ] REQ‑S5 Preservation capability gating — implement probes + policy

- [ ] REQ‑O1 Structured fact for every step — implement emitter
- [ ] REQ‑O2 Dry‑run facts identical to real‑run — determinism + redactions
- [ ] REQ‑O3 Versioned, stable facts schema — validate against JSON schema
- [ ] REQ‑O4 Signed attestations — integrate `Attestor`
- [ ] REQ‑O5 Before/after hashes per mutation — implement sha256
- [ ] REQ‑O6 Secret masking — implement policy + redactor
- [ ] REQ‑O7 Provenance completeness — populate fields

- [ ] REQ‑L1 Single mutator — LockManager integration
- [ ] REQ‑L2 Warn when no lock manager — emit WARN fact
- [ ] REQ‑L3 Bounded lock wait with timeout — timeout → `E_LOCKING`, record `lock_wait_ms`
- [ ] REQ‑L4 LockManager required in production — policy + docs

- [ ] REQ‑RC1 Rescue profile available — maintain/verify backup symlink set
- [ ] REQ‑RC2 Verify fallback path — preflight checks
- [ ] REQ‑RC3 Fallback toolset on PATH — verify GNU/BusyBox presence

- [ ] REQ‑D1 Deterministic IDs (UUIDv5) — implement
- [ ] REQ‑D2 Redaction‑pinned dry‑run — implement

- [x] REQ‑C1 Dry‑run by default — `ApplyMode::default()` = DryRun
- [ ] REQ‑C2 Fail‑closed on critical violations — ensure comprehensive gating

- [ ] REQ‑H1 Minimal smoke suite — integrate runner
- [ ] REQ‑H2 Auto‑rollback on smoke failure — implement
- [ ] REQ‑H3 Health verification is part of commit — enforce

- [ ] REQ‑F1 EXDEV fallback preserves atomic visibility — implement
- [ ] REQ‑F2 Degraded mode policy & telemetry — implement
- [ ] REQ‑F3 Supported filesystems verified — acceptance tests

- [ ] REQ‑TOCTOU1 TOCTOU‑safe syscall sequence — complete with `openat`
- [ ] REQ‑BND1 fsync within 50ms — enforce/record
- [ ] REQ‑CI1 Golden fixtures existence — produce
- [ ] REQ‑CI2 Zero‑SKIP gate — CI config
- [ ] REQ‑CI3 Golden diff gate — CI config
- [ ] REQ‑VERS1 Facts carry `schema_version` — emit

---

## Cross‑References (where to implement)

- Core orchestration: `src/api.rs`
- Preconditions/policy: `src/preflight.rs`, `src/policy/config.rs`
- Filesystem atomic ops: `src/fs/atomic.rs`, `src/fs/symlink.rs`
- Types & models: `src/types/*`
- Logging/Audit: `src/logging/*`, `SPEC/audit_event.schema.json`
- Adapters: `src/adapters/*`
- Rescue: `src/rescue.rs`
- SPEC artifacts: `SPEC/*`, `SPEC/features/*`, `SPEC/tools/traceability.py`
- PLAN tracking: `PLAN/*`, `PLAN/TODO.md`

---

## Notes

- Current code is a placeholder scaffold that partially exercises atomic symlink swapping and simple preflight checks. The majority of SPEC requirements are intentionally left TODO for the implementation phase.
- The test orchestrator (`test-orch/`) and BDD runner (`cargo/bdd-runner/`) are available for acceptance testing; wire switchyard once core features are implemented.
