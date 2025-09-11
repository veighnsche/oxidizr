# Switchyard Implementation Tiers (Consolidated Planning)

This document consolidates tiered maturity (Bronze → Silver → Gold → Platinum) for core Switchyard modules and processes. It also provides a current maturity assessment based on code in `src/` and existing plans/specs.

References

- SPEC: `cargo/switchyard/SPEC.md`
- Planning: `PLAN/30-errors-and-exit-codes.md`, `PLAN/35-determinism.md`, `PLAN/40-facts-logging.md`, `PLAN/45-preflight.md`, `PLAN/50-locking-concurrency.md`, `PLAN/60-rollback-exdev.md`, `PLAN/65-rescue.md`, `PLAN/80-testing-mapping.md`
- Implementation: `src/api/{preflight,apply}.rs`, `src/api/audit.rs`, `src/api/errors.rs`, `src/adapters/*`, `src/policy/config.rs`

---

## 1) Preflight

Bronze

- Emit one row per action per `SPEC/preflight.yaml`; deterministic ordering.
- Check mount `rw+exec`, immutability, allowed/forbidden roots; compute `policy_ok`; emit per‑action + summary facts.

Silver

- Add strict ownership, source trust, and preservation capability probes; policy controls enforcement; `apply()` refuses when `policy_ok=false` unless overridden.

Gold

- Golden fixtures for positive/negative scenarios; byte‑identical after redaction; CI gate for curated set.

Platinum

- Cross‑platform capability coverage; schema versioning/migration; performance budgets and measurements.

---

## 2) Apply

Bronze

- TOCTOU‑safe sequence: open parent `O_DIRECTORY|O_NOFOLLOW` → `openat` → `renameat` → `fsync(parent)`; DryRun parity; minimal facts.
- Rollback attempt on failure; record partial restoration.

Silver

- Enforce preflight (`policy_ok=false` refuses unless override); map failures to `E_POLICY`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_GENERIC`.
- Optional bounded locking with `E_LOCKING`; provenance in facts; deterministic ordering & IDs.

Gold

- Golden fixtures for representative apply/rollback paths; CI gate; contention tests.

Platinum

- Performance budgets in facts; degraded mode (EXDEV) coverage; optional attestation on success.

---

## 3) Rollback

Bronze

- Restore last backup for tag; preserve owner/mode where supported; emit rollback facts; note partial restoration.

Silver

- Enforce `backup_tag`; deterministic candidate selection; preserve timestamps and attempt xattrs/ACLs/caps; record support.

Gold

- Golden fixtures for success/partial/missing backup; error ids/exit codes consistent.

Platinum

- Cross‑fs aware flows; retention/cleanup guidance with audit trail.

---

## 4) Backup & Restore

Bronze

- Create `.basename.<backup_tag>.<unix_millis>.bak` adjacent to target; preserve owner/mode; audit notes.

Silver

- Tag discipline; timestamps; attempt xattrs/ACLs/caps; record preservation support.

Gold

- Goldens for naming/presence/restore coupling; consistent error mapping.

Platinum

- Retention/cleanup policy; degraded‑mode aware backups; performance budgets.

---

## 5) Locking & Concurrency

Bronze

- No‑op/default locking acceptable in dev/test; single‑process assumption documented.

Silver

- Production `LockManager` with bounded wait; on timeout emit `E_LOCKING` with `lock_wait_ms` and abort.

Gold

- Concurrency tests simulate contention; golden fixture for locking failure.

Platinum

- Queue/hold time metrics; tuning guidance; multi‑host story documented or explicitly out of scope.

---

## 6) Determinism

Bronze

- UUIDv5 `plan_id`/`action_id` from normalized inputs; stable ordering; `TS_ZERO` timestamp zeroing.

Silver

- Broader redaction and normalization of environment inputs; DryRun vs Commit parity tests.

Gold

- CI gate on determinism goldens; property tests for ordering/IDs.

Platinum

- Strict reproducibility mode; schema/version migration that preserves determinism.

---

## 7) Audit & Logging

Bronze

- Facts for all stages with `schema_version=1` to JSONL sink; basic redaction (TS_ZERO in DryRun).

Silver

- `error_id` everywhere it applies; exit codes for covered set; redaction policy documented and tested; provenance (redacted) included.

Gold

- Golden fixtures for facts; schema version pinned; migration notes; CI gate on curated set.

Platinum

- Multiple sinks (file/stderr/syslog/custom) with consistent redaction; optional attestations on success.

---

## 8) Policy Enforcement

Bronze

- Parse and carry policy flags (`allow_roots`, `forbid_paths`, `strict_ownership`, `force_untrusted_source`, `allow_degraded_fs`, `disable_auto_rollback`, `backup_tag`, `override_preflight`).

Silver

- Preflight computes `policy_ok`; `apply()` refuses when `policy_ok=false` unless overridden; `E_POLICY` with details.

Gold

- Golden scenarios for policy stops/overrides; deterministic outputs; coverage report.

Platinum

- Policy versioning; deprecation paths; lints/guardrails for dangerous combinations.

---

## 9) Smoke Tests (Health Verification)

Bronze

- Define `SmokeTestRunner` and minimal suite; no auto‑rollback.

Silver

- Post‑apply integration; on failure and `!disable_auto_rollback`, trigger auto‑rollback and emit `E_SMOKE`.

Gold

- Pluggable suites; goldens for failure; machine‑readable reports.

Platinum

- Tunables (timeouts/parallelism); curated per environment; provenance recorded.

---

## 10) Golden Fixtures

Bronze

- At least one stable golden (dry‑run); regeneration instructions; redaction masking of volatile fields.

Silver

- Negative goldens (policy stop, locking) and apply/rollback basics; non‑blocking CI check.

Gold

- Blocking CI gate on curated golden set; diff artifacts uploaded.

Platinum

- Coverage across preflight/apply/rollback/smoke/degrade; traceability linking goldens ↔ requirements.

---

## 11) Rescue Profile

Bronze

- PATH scan for required rescue binaries; preflight summary note.

Silver

- Optional `require_rescue` policy gates; facts list missing binaries and backup symlinks.

Gold

- Verify rescue symlinks and minimal command health; golden for missing rescue.

Platinum

- Extended checks (e.g., initramfs access) and operator drills; metrics for verification time.

---

## 12) Exit Codes

Bronze

- Provisional `error_id`; `exit_code` optional except where already stable (locking timeout → 30).

Silver

- Implement `exit_code_for()` for curated subset: `E_LOCKING`, `E_POLICY`, `E_GENERIC`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_SMOKE`.
- Ensure facts at those failure sites include `error_id` and `exit_code`.

Gold

- Expand coverage to all core paths exercised by tests; goldens for error scenarios; CI gate.

Platinum

- Finalize full mapping in SPEC; versioned table; traceability to tests.

---

# Current Maturity Assessment (as of 2025-09-11)

Basis: implementation review and tests.

- Preflight: Silver
  - Evidence: `src/api/preflight.rs` emits per‑action facts with `policy_ok`, ownership checks under `strict_ownership`, allowed/forbidden roots, immutability, mount `rw+exec`, preservation probe fields; summary fact. Enforcement wired via `apply()`.

- Apply: Silver
  - Evidence: `src/api/apply.rs` enforces preflight (emits `E_POLICY` with `exit_code_for`), per‑action attempt/result facts, degraded path reporting, provenance hashing, optional attestation, smoke hook, rollback on failure.

- Rollback: Silver
  - Evidence: `apply.rs` restores backups on failure; emits rollback step facts; maps `E_BACKUP_MISSING`/`E_RESTORE_FAILED`.

- Backup & Restore: Silver
  - Evidence: backup tag usage (`policy.backup_tag`), adjacent backups; preservation attempts logged via preflight capability probe; restore path used in rollback and on demand.

- Locking & Concurrency: Silver (with adapter), Bronze (without adapter)
  - Evidence: bounded wait and `E_LOCKING` with `lock_wait_ms` when `LockManager` is configured; warn when absent.

- Determinism: Silver
  - Evidence: UUIDv5 IDs, TS_ZERO redaction; tests compare DryRun vs Commit apply.result decisions after redaction (`tests/sprint_acceptance-0001.rs`).

- Audit & Logging: Silver
  - Evidence: JSONL sink, schema version, redaction via `TS_ZERO`, `error_id/exit_code` at covered sites; product‑side migration to structured helper completed.

- Policy Enforcement: Silver
  - Evidence: `Policy` in `src/policy/config.rs`; enforcement in preflight/apply; `override_preflight` default off (fail‑closed); `allow_roots/forbid_paths/strict_ownership` applied.

- Smoke Tests: Bronze→Silver
  - Evidence: `apply.rs` integrates optional `SmokeTestRunner`; on failure triggers auto‑rollback when not disabled and emits `E_SMOKE`. Adapter presence determines effective tier.

- Golden Fixtures: Bronze
  - Evidence: tests produce canon when `GOLDEN_OUT_DIR` set; SPEC calls for zero‑SKIP gate but CI blocking not yet established.

- Rescue Profile: Bronze
  - Evidence: Planning present (`PLAN/65-rescue.md`); no enforced gating in code yet.

- Exit Codes: Silver
  - Evidence: `exit_code_for()` used at multiple sites (`E_POLICY`, `E_LOCKING`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`).

---

## Next Steps to Reach Gold (high‑level)

- Preflight/Apply: add curated golden fixtures and enable a blocking CI gate for selected scenarios.
- Locking: add contention tests and a golden for `E_LOCKING` path.
- Determinism: property tests for ordering/IDs; CI gate on parity.
- Audit: pin schema version in tests; migration notes; broaden redaction tests.
- Golden Fixtures: adopt zero‑SKIP gating with artifact upload.
- Rescue: implement `require_rescue` policy and verification; negative golden.
- Smoke: formalize adapter, default minimal suite, and failure goldens.

---

If you want, I can remove the per‑module `*-tiers.md` files now that this consolidated plan exists, or keep them temporarily until this file is reviewed.
