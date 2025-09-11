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