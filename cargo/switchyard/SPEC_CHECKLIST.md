# SPEC Requirement Checklist

Below maps each requirement from `SPEC/requirements.yaml` to current status.

- [x] REQ‑A1 Atomic crash‑safety — atomic symlink swap via openat/symlinkat/renameat + fsync; unit tests present
- [~] REQ‑A2 No broken/missing path visible — covered by atomic swap; needs invariants/tests
- [~] REQ‑A3 All‑or‑nothing per plan — reverse‑order rollback on failure for applied symlink actions; restore inverse TBD

- [~] REQ‑R1 Rollback reversibility — inverse plan for executed symlink ensures; restore inverse skipped
- [ ] REQ‑R2 Restore exact topology — extend backup/restore and tests
- [ ] REQ‑R3 Idempotent rollback — property tests and logic
- [x] REQ‑R4 Auto reverse‑order rollback — implemented in engine (apply.rs)
- [x] REQ‑R5 Partial restoration facts — rollback steps emitted with success/failure and recorded

- [x] REQ‑S1 Safe paths only — `SafePath` enforced for API inputs; additional validation in fs layer
- [~] REQ‑S2 Reject unsupported FS states — mount flags + immutability checks exist; harden and make authoritative
- [~] REQ‑S3 Source ownership gating — checks present (world‑writable, root‑owned) with force override; facts emitted
- [~] REQ‑S4 Strict target ownership — `OwnershipOracle` integrated when policy.strict_ownership=true
- [~] REQ‑S5 Preservation capability gating — probes emitted in preflight; gating policy pending

- [x] REQ‑O1 Structured fact for every step — plan, preflight, apply.attempt/result, rollback emitted
- [~] REQ‑O2 Dry‑run facts identical to real‑run — timestamps zeroed; volatile fields redacted; identity not enforced
- [~] REQ‑O3 Versioned, stable facts schema — schema_version=1 present; JSON Schema validation pending
- [x] REQ‑O4 Signed attestations — `Attestor` integrated on successful commit with signature bundle
- [x] REQ‑O5 Before/after hashes per mutation — sha256 computed for symlink ensure
- [~] REQ‑O6 Secret masking — basic redactions implemented (provenance/attestation fields); policy hardening pending
- [~] REQ‑O7 Provenance completeness — minimal provenance emitted; completeness to be expanded

- [x] REQ‑L1 Single mutator — LockManager integration present
- [x] REQ‑L2 Warn when no lock manager — `apply.attempt` emits `no_lock_manager:true`
- [~] REQ‑L3 Bounded lock wait with timeout — enforced; on timeout emits `E_LOCKING`; `lock_wait_ms` not captured on error path
- [ ] REQ‑L4 LockManager required in production — policy + docs

- [ ] REQ‑RC1 Rescue profile available — maintain/verify backup symlink set
- [ ] REQ‑RC2 Verify fallback path — preflight checks
- [ ] REQ‑RC3 Fallback toolset on PATH — verify GNU/BusyBox presence

- [x] REQ‑D1 Deterministic IDs (UUIDv5) — implemented for plan_id/action_id
- [x] REQ‑D2 Redaction‑pinned dry‑run — implemented (TS_ZERO + redactor)

- [x] REQ‑C1 Dry‑run by default — `ApplyMode::default()` = DryRun
- [~] REQ‑C2 Fail‑closed on critical violations — partial; preflight produces stops but apply does not enforce

- [~] REQ‑H1 Minimal smoke suite — trait integrated; external runner integration pending
- [x] REQ‑H2 Auto‑rollback on smoke failure — implemented (unless disabled by policy)
- [ ] REQ‑H3 Health verification is part of commit — enforce

- [~] REQ‑F1 EXDEV fallback preserves atomic visibility — degraded non‑atomic fallback implemented with telemetry
- [x] REQ‑F2 Degraded mode policy & telemetry — `allow_degraded_fs` + `degraded` fact
- [ ] REQ‑F3 Supported filesystems verified — acceptance tests

- [x] REQ‑TOCTOU1 TOCTOU‑safe syscall sequence — `open_dir_nofollow` + `*at` + `renameat` + parent fsync
- [~] REQ‑BND1 fsync within 50ms — recorded and WARN‑flagged when exceeded; no hard enforcement
- [ ] REQ‑CI1 Golden fixtures existence — produce
- [ ] REQ‑CI2 Zero‑SKIP gate — CI config
- [ ] REQ‑CI3 Golden diff gate — CI config
- [x] REQ‑VERS1 Facts carry `schema_version` — emit

