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
