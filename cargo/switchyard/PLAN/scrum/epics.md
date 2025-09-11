# Switchyard Epics

This document enumerates the epics for Switchyard and ties them to SPEC requirements, PLAN notes, and the TODO backlog.

Format per epic:

- Objective
- Scope (In/Out)
- SPEC mapping (REQ-*)
- Deliverables & Definition of Done (DoD)
- Dependencies
- Risks & Mitigations

---

## EPIC-1: Safe Filesystem Engine (Rustix & TOCTOU)

- Objective: Provide a safe, race-free filesystem mutation engine using rustix and capability-style handles, enforcing the normative TOCTOU sequence with fsync bounds.
- Scope:
  - In: rustix-only syscalls; `#![forbid(unsafe_code)]`; open parent O_DIRECTORY|O_NOFOLLOW; *at calls; record fsync timings; EXDEV degraded policy.
  - Out: high-level policy decisions unrelated to FS mechanics.
- SPEC: REQ-TOCTOU1, REQ-BND1, REQ-F1/F2, §14.1 Safe Rust.
- DoD:
  - All mutating paths use parent dir handles and *at calls.
  - Unit tests pass; fsync duration emitted in apply.result with WARN on breach.
  - SPEC_UPDATE_0001 documents the normative behavior.
- Dependencies: none.
- Risks: platform quirks — Mitigation: rustix abstractions; acceptance coverage later.

## EPIC-2: Backup/Restore & Rollback

- Objective: Robust backup and restore logic with multi-CLI isolation and transactional rollback.
- Scope:
  - In: backup tag (`Policy.backup_tag`), unique timestamped backups, latest-by-tag restore; rollback for symlink replaces; plan_rollback_of.
  - Out: long-term backup retention policies.
- SPEC: §3.1 API, §2.1/2.2 Atomicity & Rollback, Backup Tagging (SPEC_UPDATE_0001).
- DoD:
  - Tag-filtered backups; restore picks latest for tag.
  - Reverse-order rollback for executed actions; tests for roundtrip.
  - ADR-0013 accepted; Design doc in DOCS/.
- Dependencies: EPIC-1.
- Risks: directory scan performance — Mitigation: colocated backups; straightforward filters.

## EPIC-3: Determinism & Redactions

- Objective: Ensure dry-run facts are byte-identical to commit after redaction; stable ordering and deterministic IDs.
- Scope: redaction policy; `sha256` before/after; stable ordering; UUIDv5 plan/action IDs; provenance fields.
- SPEC: REQ-D1, REQ-D2, §13 Observability.
- DoD: redactor implemented; tests prove dry==commit after redaction; schema validated.
- Dependencies: EPIC-4.
- Risks: field drift — Mitigation: schema + golden fixtures.

## EPIC-4: Observability & Audit

- Objective: Complete minimal facts schema v1 and validation.
- Scope: `FactsEmitter`, `AuditSink`, per-step facts with schema_version; schema validation; golden fixtures; attestation hooks.
- SPEC: REQ-O1..O7, REQ-VERS1.
- DoD: facts validated against schema; golden fixtures in CI; attestation placeholder wired; provenance redacted.
- Dependencies: none.

## EPIC-5: Policy & Preflight Gating

- Objective: Enforce safety preconditions via policy and preflight.
- Scope: mount flags, immutability, ownership gating (`OwnershipOracle`), preservation capability probes; allow_roots/forbid_paths.
- SPEC: §2.3 Safety Preconditions, REQ-S1..S5.
- DoD: preflight emits structured facts with reasons; policy toggles effective; tests for negative/positive cases.
- Dependencies: EPIC-4.

## EPIC-6: Locking & Concurrency

- Objective: Single-mutator enforcement with bounded waits and useful telemetry.
- Scope: `LockManager` integration; timeout → `E_LOCKING`; `lock_wait_ms` facts; WARN when missing in dev/test.
- SPEC: §2.5, §14; REQ-L1..L4.
- DoD: reference LockManager; tests for timeout and success; facts include `lock_wait_ms`.
- Dependencies: EPIC-4.

## EPIC-7: Rescue Profile

- Objective: Guarantee a minimal rescue toolset is always available and verified.
- Scope: verify GNU/BusyBox presence; backup symlink set; preflight checks; facts.
- SPEC: §2.6; REQ-RC1..RC3.
- DoD: rescue verification implemented with tests; documented in PLAN and SPEC update if needed.
- Dependencies: EPIC-5.

## EPIC-8: Acceptance & CI Gates

- Objective: Confidence via acceptance tests and CI gates aligned with SPEC.
- Scope: EXDEV matrix across filesystems (LXD/Docker); zero-SKIP gate; golden diff gate; traceability report.
- SPEC: §12 CI; REQ-CI1..CI3; REQ-F3.
- DoD: acceptance jobs green; CI gates enforced; traceability script updated.
- Dependencies: EPIC-1,2,3,4.

## EPIC-9: Developer Docs & Adapters

- Objective: First-class docs and reference adapters for integrators.
- Scope: crate-level docs; module docs; adapters reference impls; examples.
- SPEC: §3.2 Adapters documentation; §14 Thread-safety.
- DoD: docs published; examples run; adapters mocks and sample impls.
- Dependencies: cross-epic.
