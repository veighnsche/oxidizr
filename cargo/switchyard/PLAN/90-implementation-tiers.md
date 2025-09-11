# Switchyard Implementation Tiers (Consolidated Planning)

This document consolidates tiered maturity (Bronze → Silver → Gold → Platinum) for core Switchyard modules and processes. It also provides a current maturity assessment based on code in `src/` and existing plans/specs.

References

- SPEC: `cargo/switchyard/SPEC.md`
- Planning: `PLAN/30-errors-and-exit-codes.md`, `PLAN/35-determinism.md`, `PLAN/40-facts-logging.md`, `PLAN/45-preflight.md`, `PLAN/50-locking-concurrency.md`, `PLAN/60-rollback-exdev.md`, `PLAN/65-rescue.md`, `PLAN/80-testing-mapping.md`
- Implementation: `src/api/{preflight,apply}.rs`, `src/api/audit.rs`, `src/api/errors.rs`, `src/adapters/*`, `src/policy/config.rs`

---

## 1) Preflight

Bronze

- Emit one row per action per `SPEC/preflight.yaml`; deterministic ordering (sort by `path`, then `action_id`).
- Gating: check `/usr` mount `rw+exec`, target mount `rw+exec`, immutability (chattr +i), allowed roots, forbidden paths.
- Compute `policy_ok` as the conjunction of gating checks (with explicit overrides only via policy).
- Emit per‑action preflight facts with `current_kind`, `planned_kind`, `policy_ok`, `notes`, and preservation fields; plus a summary fact indicating decision.
- Evidence: unit tests for each gating dimension; redaction parity in facts; planning alignment with `PLAN/45-preflight.md`.

Silver

- Ownership: when `strict_ownership=true`, require package ownership via `OwnershipOracle`; emit provenance `{uid,gid,pkg}` where available.
- Source trust: verify link source; `force_untrusted_source=true` permits with WARN and note.
- Preservation probes: detect support (owner, mode, timestamps, xattrs, ACLs, caps) and record `preservation` + `preservation_supported`.
- `apply()` refuses when any action has `policy_ok=false` unless `override_preflight=true`; emit `E_POLICY` and `exit_code`.
- Evidence: golden for a negative scenario (forbidden path), deterministic YAML diff, and apply refusal.

Gold

- Golden fixtures for positive/negative scenarios; byte‑identical after redaction for stable fields.
- CI gate on curated preflight scenarios; artifact upload on diff.
- Coverage report enumerating enforced policy dimensions and capability probes.

Platinum

- Cross‑platform and cross‑filesystem coverage for preservation probes; documented differences and policy toggles.
- Schema versioning/migration with dual‑emit if necessary; deprecation notes.
- Performance budgets for preflight latency with measurements recorded in facts.

---

## 2) Apply

Bronze

- TOCTOU‑safe sequence: open parent `O_DIRECTORY|O_NOFOLLOW` → `openat` final component → stage → `renameat` → `fsync(parent)` within bound.
- DryRun parity: emit identical (redacted) facts with `TS_ZERO`; no side effects.
- Emit per‑action `apply.attempt` and `apply.result` minimal facts; include `action_id`, `path`, and decision.
- On first failure, attempt rollback of executed actions; record partial restoration in report and facts.

Silver

- Enforce preflight (`policy_ok=false` refuses unless override) prior to mutation in Commit mode; emit `E_POLICY` + `exit_code_for`.
- Map swap errors to `E_ATOMIC_SWAP`; map EXDEV to `E_EXDEV` and honor `allow_degraded_fs`; include `degraded=true` when used.
- Optional bounded locking via `LockManager`; on timeout emit `E_LOCKING` with `lock_wait_ms` and abort.
- Ensure provenance fields on results (after_hash/before_hash, hash_alg) and deterministic ordering/IDs.

Gold

- Golden fixtures for representative apply/rollback paths (success, swap failure, EXDEV degraded, restore failure).
- CI gate for curated apply scenarios; property tests on rollback idempotency.
- Contention tests validate bounded wait and error mapping to `E_LOCKING` with stable exit codes.

Platinum

- Performance budgets recorded per action (`duration_ms`, fsync bound warn threshold); guidance for tuning.
- Degraded mode paths tested across filesystems; policy forbids respected; facts include `degraded=true`.
- Optional attestation in summary on success (signature, bundle hash) recorded in facts.

---

## 3) Rollback

Bronze

- Restore the most recent backup matching `backup_tag`; location adjacent to target; preserve owner/mode where supported.
- Emit `rollback.attempt` and `rollback.result` per step; record partial restoration and guidance notes when incomplete.

Silver

- Deterministic candidate selection policy documented (most recent same-tag); clear notes when backup missing.
- Preserve timestamps; attempt xattrs/ACLs/caps when supported; record preservation support in facts.

Gold

- Goldens for (a) success, (b) missing backup → `E_BACKUP_MISSING`, (c) restore failure → `E_RESTORE_FAILED`.
- CI gate for curated rollback scenarios; exit codes verified.

Platinum

- Cross‑filesystem restoration behavior validated; degraded policy interactions documented.
- Retention/cleanup guidance with safe pruning and audit trail; operator checklist.

---

## 4) Backup & Restore

Bronze

- Create `.basename.<backup_tag>.<unix_millis>.bak` adjacent to target to maximize atomicity; create parent dirs as needed.
- Preserve owner/mode; record backup path and tag in apply facts.

Silver

- Enforce `backup_tag` segregation across tools; preserve timestamps; attempt xattrs/ACLs/caps.
- Record preservation capabilities and outcomes in facts for observability.

Gold

- Goldens asserting naming pattern, presence, and restore coupling; consistent `E_*` mapping for failures.

Platinum

- Retention policy (count/age) and safe cleanup; degraded‑mode aware backups across filesystems; performance budgets.

---

## 5) Locking & Concurrency

Bronze

- No‑op/default locking acceptable in dev/test; single‑process assumption documented in README/PLAN.
- Emit WARN attempt fact when no `LockManager` is configured.

Silver

- Bounded wait with configurable timeout; return `E_LOCKING` on timeout; include `lock_wait_ms` and policy in facts.
- Lock scope defined (per process vs per target) and justified; deadlock avoidance strategy documented.

Gold

- Contention tests simulate competing processes; golden for timeout path; CI verifies stability of facts.

Platinum

- Record queue/hold time metrics in facts; guidance for tuning; HA/multi‑host story documented or explicitly non‑goal.

---

## 6) Determinism

Bronze

- UUIDv5 `plan_id`/`action_id` from normalized inputs; stable ordering for rows and facts; `TS_ZERO` timestamp zeroing.
- Normalize input ordering and remove non-deterministic iteration from core paths.

Silver

- Redact volatile fields (timestamps, durations where needed) and mask secrets; normalize PATH/locale or record as provenance.
- Parity tests for DryRun vs Commit facts for covered scenarios.

Gold

- CI gate on determinism goldens; property tests for stable ID derivation and ordering.

Platinum

- Strict reproducibility mode (fail build if non-determinism detected); schema/version migration preserving determinism.

---

## 7) Audit & Logging

Bronze

- Emit facts for all stages with `schema_version=1` to a JSONL sink; TS_ZERO applied in DryRun; append-only writes.
- Include `plan_id`, `action_id` where applicable; consistent `stage/decision/severity` fields.

Silver

- Ensure `error_id` and `exit_code` at covered failure sites; broaden redaction with unit tests.
- Include provenance in facts (subject to redaction); ensure schema validation (JSON Schema) in tests.

Gold

- Golden fixtures for facts across plan/preflight/apply/rollback; CI gate on curated set; diff artifacts uploaded.
- Schema version pinned; migration notes and dual-emit strategy documented when evolving.

Platinum

- Multiple sinks supported with consistent redaction; optional attestations captured in summary facts.
- Secret-masking guarantees documented with tests and fuzzing where applicable.

---

## 8) Policy Enforcement

Bronze

- Parse and surface policy flags end-to-end; default `override_preflight=false` (fail-closed).
- Notes in preflight facts reflect active policy dimensions.

Silver

- Preflight computes `policy_ok` across all gates; `apply()` refuses when `policy_ok=false` unless overridden explicitly.
- Emit `E_POLICY` with `exit_code` and specific notes identifying the violated dimension.

Gold

- Goldens for stops (forbidden path, outside allowed roots) and overrides; deterministic outputs.
- Coverage report enumerating enforced policy flags and evidence locations.

Platinum

- Versioning of policy schema; migration and deprecation guidance; lints for dangerous combinations (e.g., broad `allow_roots` + `override_preflight`).

---

## 9) Smoke Tests (Health Verification)

Bronze

- Define `SmokeTestRunner` trait and minimal default suite (e.g., `true`, shell probe); no auto‑rollback.
- Emit audit of runner invocation and results (even if not enforced).

Silver

- Integrate runner post‑apply; failures trigger `E_SMOKE` and auto‑rollback when policy allows; record outputs (redacted).

Gold

- Pluggable suites by provider; goldens for failure cases; machine‑readable reports uploaded in CI.

Platinum

- Tunable timeouts/parallelism; curated suite per environment; record runner provenance and environment.

---

## 10) Golden Fixtures

Bronze

- At least one stable golden (dry‑run); documented regeneration and review process; redaction masks volatile fields.

Silver

- Add negative goldens (policy stop, locking) and apply/rollback basics; non‑blocking CI check for presence and validity.

Gold

- Blocking CI gate on curated golden set; machine-readable diff artifacts uploaded for failures.

Platinum

- Coverage expanded across preflight/apply/rollback/smoke/degrade; traceability report linking REQ-* ↔ tests ↔ goldens.

---

## 11) Rescue Profile

Bronze

- PATH scan for required rescue binaries (GNU core or BusyBox); preflight summary note indicates status.

Silver

- Policy extension `require_rescue`: preflight fails closed if unmet; facts list missing binaries and backup symlinks.

Gold

- Verify rescue symlinks and a minimal command health check; golden for missing rescue scenario.

Platinum

- Extended checks (initramfs access, recovery shell availability) and documented operator drills; record verification time.

---

## 12) Exit Codes

Bronze

- Provisional `error_id`; `exit_code` optional except where already stable (locking timeout → 30); placeholder mapping exists.
- Document incomplete mapping with `// INCOMPLETE` and tests asserting shape only.

Silver

- Implement `exit_code_for()` for curated subset: `E_LOCKING`, `E_POLICY`, `E_GENERIC`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_SMOKE`.
- Ensure facts at those failure sites include `error_id` and `exit_code`; document mapping table in code with Covered/Deferred markers.

Gold

- Expand coverage to all core paths exercised by tests; goldens for error scenarios; CI gate on curated subset.

Platinum

- Finalize full mapping in SPEC (`SPEC/error_codes.toml`); versioned table and migration notes; traceability to tests.

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

