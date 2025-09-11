# (Archived) Backlog & Requirements Mapping

This is the end‑to‑end implementation backlog for the `switchyard` crate, derived from:

- `cargo/switchyard/SPEC/SPEC.md` and `SPEC/requirements.yaml`
- `cargo/switchyard/PLAN/*` (architecture, delivery plan, quality gates)
- Current code under `cargo/switchyard/src/`

Legend:

- [x] Done
- [ ] TODO (not implemented)
- [~] Partial (some parts present, needs completion)

## Cyclical Development Process

We follow a tight build–test–document cycle to avoid plan/spec drift:

- Implement: Small, verifiable increments with unit tests.
- Test: `cargo test -p switchyard` must stay green; add tests for new behaviors.
- Document: Update PLAN (impl or ADR) and append immutable SPEC_UPDATE_####.md entries for any normative change.
- Sync Review: Ensure PLAN, SPEC_UPDATEs, ADRs, and this TODO remain consistent; link items where relevant.
- Repeat: Iterate with small scope until the milestone feature is complete.

## Documentation Sync Policy

- SPEC baseline (`SPEC/SPEC.md`) remains stable; normative changes are appended as immutable `SPEC/SPEC_UPDATE_####.md` files.
- PLAN remains the living design: `PLAN/impl/*` for implementation notes; decisions recorded in `PLAN/adr/ADR-*.md`.
- Each TODO item that changes behavior should reference the relevant SPEC_UPDATE and ADR once accepted.
- Doc Sync Checklist (run per PR):
  - SPEC_UPDATE added/updated for normative behavior.
  - PLAN impl notes updated for approach and API surfaces.
  - ADR added/updated for decisions (numbered, dated, status Accepted/Proposed).
  - TODO updated to reflect current state; cross-links added.
  - Tests updated to reflect and verify behavior.

## 1) Crate Scaffolding & Module Layout

- [x] Library crate scaffolding and module tree present
  - Files: `Cargo.toml`, `src/lib.rs`, `src/api.rs`, `src/preflight.rs`, `src/rescue.rs`, `src/fs/*`, `src/types/*`, `src/adapters/*`, `src/logging/*`, `src/policy/*`
- [x] Public modules exported via `src/lib.rs`
- [~] Top‑level crate docs and README for the crate (usage, safety model, adapter model)
  - README added (`cargo/switchyard/README.md`); module/crate rustdocs pending

## 2) Public API Surface (SPEC §3.1)

- [~] `plan(input: PlanInput) -> Plan` exists and delegates to `src/api/plan.rs` (basic link/restore expansion)
- [~] `preflight(plan: &Plan) -> PreflightReport` exists and delegates to `src/api/preflight.rs` (basic checks)
- [~] `apply(plan: &Plan, mode: ApplyMode) -> ApplyReport` exists and delegates to `src/api/apply.rs` (symlink swap + restore)
- [~] `plan_rollback_of(report: &ApplyReport) -> Plan` implemented via `src/api/rollback.rs` (basic inverse for symlink actions)
- [x] All path‑carrying fields in mutating API use `SafePath` (see `src/types/plan.rs`)

## 3) SafePath & TOCTOU Safety (SPEC §2.3, §3.3)

- [x] `SafePath` type rejects `..` and escaping roots (`src/types/safepath.rs`)
- [~] All mutating entry points accept `SafePath` (API uses `SafePath` but `fs/*` operate on `&Path` internally)
- [x] TOCTOU‑safe syscall sequence
  - Present: `openat` (via `rustix`), `symlinkat`/`unlinkat`, `renameat`, `fsync(parent)` (`src/fs/atomic.rs`)
  - All mutations use capability-style directory handles and the *at APIs.
- [~] Unit/property tests for `SafePath` normalization and negative cases beyond `rejects_dotdot`

## 4) Atomicity & Rollback Engine (SPEC §2.1, §2.2)

- [x] Atomic symlink swap exists (`atomic_symlink_swap`) and backup/restore flows (`src/fs/symlink.rs`) with unit tests
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
- [x] Strict target ownership via `OwnershipOracle` when `policy.strict_ownership` (REQ‑S4)
- [~] Preservation capability gating: detect whether ownership/mode/timestamps/xattrs/ACLs/caps can be preserved and STOP if policy requires but unsupported (REQ‑S5)
  - Detection + facts emission implemented (`preservation{}`, `preservation_supported`); STOP enforced when `Policy.require_preservation=true`.
- [ ] Emit structured preflight diff rows per `SPEC/preflight.yaml` (and ensure dry‑run byte identity)
  
### Backup Tagging (Multi-CLI)

- [x] Policy includes `backup_tag` (default `"switchyard"`); CLIs set their own tag to isolate backups.
- [x] Backup naming `.basename.<backup_tag>.<unix_millis>.bak`; restore selects latest matching tag.

## 6) Cross‑Filesystem & Degraded Mode (SPEC §2.10)

- [~] EXDEV fallback path: safe copy + fsync + rename strategy (REQ‑F1)
  - Implemented degraded fallback for symlink replace in `fs/atomic.rs::atomic_symlink_swap()` using `symlinkat` directly when `renameat` returns `EXDEV` and policy allows.
  - Remaining: broader mutation coverage and explicit copy+fsync path where applicable.
- [x] Policy hook `allow_degraded_fs`; record `degraded=true` in facts when used; fail when disallowed (REQ‑F2)
  - Per‑action `apply.result` includes `degraded=true` when fallback used.
- [ ] Acceptance coverage for ext4/xfs/btrfs/tmpfs semantics (REQ‑F3)

## 7) Locking & Concurrency (SPEC §2.5, §14)

- [~] Integrate `LockManager` with bounded wait; record `lock_wait_ms`; timeout → `E_LOCKING` (REQ‑L1..L3)
  - Added `LockManager::acquire_process_lock(timeout_ms)` and optional wiring in `api::apply()`.
  - Emits `apply.attempt` failure with `error_id=E_LOCKING`, `exit_code=30`, and records `lock_wait_ms` on timeout.
  - Emits `apply.attempt` success with `lock_wait_ms` when acquired.
- [x] Emit WARN fact when no `LockManager` (dev/test only) (REQ‑L2)
- [~] Ensure core types and engine are `Send + Sync` and document thread‑safety (SPEC §14)
  - Adapter traits now `Send + Sync` where applicable.

## 8) Determinism & Redactions (SPEC §2.7)

- [x] Deterministic `plan_id` and `action_id` (UUIDv5 over normalized inputs/namespace) (REQ‑D1)
- [x] Dry‑run redactions pinned; dry‑run facts byte‑identical to real‑run after redaction (REQ‑D2)
  - Redaction policy extended to remove timings, severity, degraded, content hashes, and mask attestation fields for diffing.
- [x] Stable ordering of plan actions and fact emission streams

## 9) Observability & Audit (SPEC §2.4, §5, §13)

- [~] Traits exist: `FactsEmitter`, `AuditSink`, `JsonlSink` (stubs)
- [~] Emit structured facts for every step, validating against `SPEC/audit_event.schema.json` (REQ‑O1, REQ‑O3, REQ‑VERS1)
  - Minimal facts emitted via centralized `src/api/audit.rs` for `plan`, `preflight`, `apply.lock`, `apply.attempt`, `apply.result` with `schema_version`, `plan_id`, `action_id` (per‑action), and `path`.
- [~] Compute and record `before_hash`/`after_hash` (sha256) for mutated files (REQ‑O5) — implemented for symlink ensure; remaining mutations pending
- [~] Signed attestation per apply bundle via `Attestor` (ed25519) (REQ‑O4)
  - Implemented bundle construction (JSON), `bundle_hash` (sha256), and `public_key_id`; signature included when `Attestor` provided.
- [ ] Secret masking policy and implementation across all sinks (REQ‑O6)
- [ ] Provenance completeness fields populated (origin/helper/uid/gid/pkg/env_sanitized) (REQ‑O7)
- [~] Golden fixtures for plan, preflight, apply, rollback facts; CI diff gate (SPEC §12)
  - Implemented canonical (canon) arrays for selected stages with a blocking CI golden diff gate; raw JSONL optional and not part of strict diffing.

## 10) Conservatism & Modes (SPEC §2.8)

- [x] `ApplyMode` defaults to `DryRun` (REQ‑C1)
- [ ] Explicit operator approval path for side effects where relevant (library API + top‑level app integration guidance)
- [ ] Fail‑closed behavior on critical compatibility violations unless policy overrides (REQ‑C2) — ensure coverage of all critical checks

## 11) Health Verification & Auto‑Rollback (SPEC §2.9, §11)

- [~] Integrate smoke tests post‑apply with minimal suite (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date) (REQ‑H1)
  - `Switchyard` accepts optional `SmokeTestRunner`; when provided, runs post‑apply in Commit mode.
- [~] Auto‑rollback on smoke failure unless explicitly disabled by policy (REQ‑H2, H3)
  - Implemented with `policy.disable_auto_rollback` gating; emits rollback facts per step.

## 12) Rescue Profile (SPEC §2.6)

- [ ] Maintain/verify rescue profile (backup symlink set) always available (REQ‑RC1)
- [x] Preflight verifies at least one functional fallback path/toolset on PATH (GNU or BusyBox) (REQ‑RC2, RC3)
  - Minimal deterministic PATH-based verification (BusyBox present OR ≥6/10 GNU tools), no command execution; gated by `Policy.require_rescue`.
- [~] Replace `rescue::verify_rescue_tools()` stub with real checks and facts
  - Basic verification implemented; extend facts/notes and adapter-based probes in a follow-up.

## 13) Policy Model

- [~] Policy struct present with `allow_roots`, `forbid_paths`, `strict_ownership`, `force_untrusted_source`, `force_restore_best_effort` (`src/policy/config.rs`)
- [ ] Defaults and builder API for policy; documentation of semantics
- [ ] Policy options for degraded FS, auto‑rollback disable, provenance requirements, secret masking, etc.

## 14) Error Model & Taxonomy (SPEC §6)

- [~] Basic error types exist (`src/types/errors.rs`)
- [~] Map errors to SPEC exit codes / stable identifiers (E_POLICY, E_LOCKING, etc.) — scaffold `ErrorId` + `exit_code_for()` in `src/api/errors.rs`; partial emission (locking + generic) in facts
- [ ] Enrich error contexts; plumb through to facts

## 15) Filesystem Ops Hardening

- [x] Use `openat` + `O_NOFOLLOW` for final component operations to fully meet TOCTOU sequence (REQ‑TOCTOU1)
- [ ] Bound `fsync(parent)` timing ≤ 50ms after rename; record telemetry (REQ‑BND1)
- [ ] Preserve and/or gate ownership/mode/timestamps/xattrs/ACLs/caps as required by policy (ties to REQ‑S5)

## 16) Adapters & Integration Contracts (SPEC §3.2)

- [~] Traits defined: `OwnershipOracle`, `LockManager`, `PathResolver`, `Attestor`, `SmokeTestRunner`
- [ ] Provide reference implementations/mocks and adapter wiring in `api::apply`
- [ ] Steps contract mapping (see `SPEC/features/steps-contract.yaml`) to adapter calls

## 17) Testing & CI (SPEC §8, §12)

- [~] Unit tests for `fs/*`, `types/*`, policy/preflight, determinism IDs, locking behavior
  - Added unit tests for atomic swap and restore roundtrip; added minimal facts emission test in `api`.
- [ ] BDD features wired up with `cargo/bdd-runner` and golden fixtures
- [ ] Property tests for invariants: `AtomicReplace`, `IdempotentRollback`
- [~] CI gates: zero‑SKIP, schema validation, golden diffs, traceability report generation (`SPEC/tools/traceability.py`)
  - Golden diffs: implemented (blocking job over all scenarios) ✅
  - Schema validation: integrated in tests ✅
  - Zero‑SKIP: enforcement policy documented; wired per-suite as next step ◻️
  - Traceability artifact: implemented via non‑blocking job (uploads report) ✅

## 18) Documentation

- [ ] Crate‑level docs: guarantees, safety model, adapter interfaces, examples
- [ ] Module‑level docs for `fs`, `preflight`, `api`, `types`, `adapters`, `logging`, `policy`
- [ ] Update `PLAN/20-spec-traceability.md` as implementation proceeds
  - [x] Updated PLAN impl docs: `impl/00-structure.md` (rustix & unsafe ban notes) and `impl/15-policy-and-adapters.md` (backup_tag policy).
  - [x] SPEC updated via immutable `SPEC/SPEC_UPDATE_0001.md` (safe Rust policy with rustix/capability-style handles, TOCTOU sequence & fsync bound, EXDEV degraded mode, backup_tag).

### Doc Sync Matrix (Current)

- SPEC Updates: `SPEC/SPEC_UPDATE_0001.md` (Accepted)
- PLAN Impl Notes: `PLAN/impl/00-structure.md`, `PLAN/impl/15-policy-and-adapters.md`
- ADRs: `PLAN/adr/ADR-0013-backup-tagging.md` (Accepted)
- Design Docs: `DOCS/backup-restore-design.md` (Draft)

## 19) Safety & Implementation

- [x] Migrate FS layer to `rustix` and eliminate `unsafe`/`libc` in crate.
- [~] Record and emit fsync timing per mutation; WARN on >50ms with `severity=warn`.
- [x] Schema validation for facts integrated in tests; golden canon compared in CI.

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
