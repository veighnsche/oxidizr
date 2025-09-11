# Switchyard Sprint 01 Plan (Active)

Duration: 2 weeks
Team: 5 AI engineers (parallel lanes)
Drift Guard: Stop scope expansion if doc-sync risks appear (SPEC_UPDATE/ADR/PLAN must remain in lockstep)

Sprint Theme: Determinism, Observability, Policy Gating, Locking, and Acceptance Foundations

Objectives

- Determinism & Redactions (EPIC-3): Dry-run facts byte-identical to commit after redaction; UUIDv5 IDs; stable ordering.
- Observability & Audit (EPIC-4): Schema validation, golden fixtures, provenance, and hashing scaffolding.
- Policy & Preflight (EPIC-5): OwnershipOracle integration and preservation probes.
- Locking & Concurrency (EPIC-6): Reference LockManager with timeout + telemetry and tests.
- Acceptance & CI (EPIC-8): EXDEV matrix via test-orch and doc-sync checker.

Stories & Tasks (parallelizable lanes)

1) Determinism & Redactions (Owner: Dev-A)
   - Implement `logging/redact.rs` with policies: timestamp zeroing for DryRun; secret masking placeholders.
   - Replace `TS_ZERO` usage with redactor-injected timestamps for Commit; ensure DryRun == Commit after redaction.
   - Stable ordering for facts and plan actions; add sorting where required.
   - Tests: prove DryRun and Commit facts match after redaction; unit tests for redactor.
   - Docs: SPEC_UPDATE_0002 (Redaction policy & determinism) + ADR (Redaction strategy, tradeoffs).

2) Observability & Audit (Owner: Dev-B)
   - Add `before_hash`/`after_hash` (sha256) capture around mutations.
   - Complete minimal schema v1; add JSON schema validation in tests.
   - Golden fixtures for plan, preflight, apply, rollback; simple harness to diff.
   - Provenance fields scaffolding (uid/gid/env_sanitized placeholders); secret-masking call points.
   - Docs: SPEC_UPDATE_0003 (Facts schema and hashing) + PLAN impl updates.

3) Policy & Preflight Gating (Owner: Dev-C)
   - Integrate `OwnershipOracle` for strict target ownership checks; enrich errors and facts.
   - Preservation probes: detect ability to preserve mode/owner/timestamps/xattrs/ACLs/caps; gate by policy.
   - Emit structured preflight diff rows per SPEC (begin with a minimal subset); ensure dry-run byte identity.
   - Tests: negative/positive coverage for ownership and preservation.
   - Docs: PLAN impl note; ADR if policy defaults change.

4) Locking & Concurrency (Owner: Dev-D)
   - Provide a reference `LockManager` with timeout; return E_LOCKING and `lock_wait_ms` telemetry.
   - Wire into `api::apply()` under policy; WARN when absent in dev/test (already present, expand tests).
   - Tests: timeout path and success path; facts include `lock_wait_ms`.
   - Docs: PLAN impl note; update SPEC_UPDATE_0002/3 if needed.

5) Acceptance & CI Gates (Owner: Dev-E)
   - Add EXDEV acceptance matrix via `test-orch/` (ext4/xfs/btrfs/tmpfs) invoking the library.
   - Add doc-sync checker: require SPEC_UPDATE present for normative changes; CI stub.
   - Prepare zero-SKIP policy and golden diff gate plan (stub CI config + checklist).
   - Docs: PLAN impl note; update SPEC CI section if needed.

Deliverables

- Code: redactor, hashing, schema validation hooks, ownership checks, preservation probes, LockManager reference, acceptance harness.
- Docs: SPEC_UPDATE_0002, SPEC_UPDATE_0003; PLAN impl updates; ADR for Redaction.
- Tests: unit tests for all above; golden fixtures baseline; acceptance jobs invoked locally.

Acceptance Criteria

- `cargo test -p switchyard` green; new tests added for each story.
- Golden fixtures produced and validated locally; schema validation wired in tests.
- Demonstrate DryRun==Commit after redaction for a representative plan.
- Ownership and preservation gating effective and covered by tests.
- Locking tests passing with timeout and telemetry.
- EXDEV acceptance runs at least on one FS locally and documents steps for the rest.
- SPEC_UPDATEs and ADR merged; PLAN impl notes updated; Doc Sync Checklist completed.

Notes

- Stop adding scope if SPEC/PLAN/code drift risk appears; open a new SPEC_UPDATE number only when necessary.
- Any additional scope must be tied to an epic and have clear tests/doc updates.

---

## Sprint 01 Status Update — 2025-09-11

Summary: Good progress across Determinism, Observability, and Locking. Minimal schema v1 facts are flowing; redaction policy implemented; a reference file-based lock manager added. Planning artifacts updated to track the `api/` module split.

Progress

- __Determinism__: 
  - Implemented `logging/redact.rs` with `TS_ZERO`, `ts_for_mode()`, and `redact_event()` to normalize timestamps and mask volatile fields (duration, lock_wait_ms, attestation bundle IDs) for DryRun vs Commit comparison.
  - Stabilized action ordering in `api::plan()` by sorting actions by kind and `SafePath.rel()`.
  - Added tests to assert schema validation and DryRun==Commit event equivalence after redaction.

- __Observability & Audit__:
  - Emitted Minimal Facts v1 across `plan`, `preflight`, `apply.attempt`, `apply.result`, and `rollback` with `schema_version=1`.
  - Added integrity scaffolding: optional `hash_alg=sha256`, `before_hash`, `after_hash` for symlink replacements.
  - Added provenance placeholders: `{uid, gid, env_sanitized, helper:""}` to `apply.result`.
  - Validated facts against `SPEC/audit_event.schema.json` via unit tests.

- __Locking & Concurrency__:
  - Verified `adapters/lock.rs` is the trait definition and added `adapters/lock_file.rs` as the concrete reference `FileLockManager` using `fs2` file locks.
  - Integrated bounded wait in `api::apply()` with configurable `lock_timeout_ms` (default 5000 ms) and `lock_wait_ms` telemetry.
  - Added unit tests for the file-lock manager’s timeout and success paths.

- __Planning Artifacts__:
  - Created `PLAN/12-api-module.md` documenting the responsibilities of an `api/` module and a no-behavior-change split plan for `src/api.rs`.
  - Updated `PLAN/00-structure.md` to reflect the planned `api/` module layout (`mod.rs`, `plan.rs`, `preflight.rs`, `apply.rs`).
  - Updated `PLAN/README.md` to include the new API planning document.

Decisions (recorded)

- __Timestamp policy__: Commit mode uses real RFC3339 timestamps; DryRun uses `TS_ZERO`. Redaction normalizes both to `TS_ZERO` for diffing (SPEC §2.7).
- __Locking default__: Require a `LockManager` in production; default `lock_timeout_ms = 5000`. In dev/test, WARN fact when lock manager not provided.
- __Schema scope__: Minimal Facts v1 fields are required now; integrity/provenance optional fields are present where available and omitted otherwise to satisfy schema (`null` avoided).
- __Stable ordering__: Actions within a plan are deterministically ordered before computing IDs and emitting facts.
- __API split__: Proceed with an internal refactor to `src/api/` once current feature work stabilizes; no behavioral changes expected (tracked in `PLAN/12-api-module.md`).

Blockers / Risks

- __Ownership & Preservation__: Strict `OwnershipOracle` integration is stubbed at the trait level; richer provenance (pkg origin) and preservation probes (mode/owner/timestamps/xattrs/ACLs/caps) are not yet implemented.
- __Attestation bundle__: The final `apply.result` attestation includes a placeholder bundle hash and public key id; needs full bundle construction and hashing.
- __Golden fixtures / CI gate__: JSONL golden generation and byte-for-byte gate not wired in CI yet; local tests validate schema and redaction only.
- __Smoke tests__: `SmokeTestRunner` adapter exists but no default commands; auto-rollback path covered but test assets pending.
- __EXDEV acceptance__: Cross-filesystem matrix not yet exercised from this crate; integration with `test-orch/` needed.

Next Steps (target before sprint end)

- __Ownership & Preservation__ (EPIC-5):
  - Implement `OwnershipOracle` default adapter (pkg/UID/GID) and emit preflight failures with structured facts.
  - Add preservation capability detection and `preservation{}` facts; gate by policy.

- __Attestation__ (EPIC-4):
  - Build the attestation bundle (plan, facts digest) and compute `bundle_hash` (sha256). Wire an injected `Attestor` key id.

- __Golden & CI__ (EPIC-8):
  - Add a golden fixtures generator and byte-identical diff gate; ensure DryRun vs Commit equivalence post-redaction.

- __API Module Refactor__:
  - Execute the split of `src/api.rs` into `src/api/` per `PLAN/12-api-module.md` (no behavior change) once tests remain green for two runs.