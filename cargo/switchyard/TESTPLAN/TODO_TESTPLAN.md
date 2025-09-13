# TODO — E2E Test Implementation from TESTPLAN

Generated: 2025-09-13
Source: TESTPLAN/*

This TODO file is machine-derived from TESTPLAN; do not hand-edit. Re-run the generator to update.

Summary: 147 total tasks
Priorities: P0 = Bronze (pre-merge), P1 = Silver (daily), P2 = Gold (nightly), P3 = Platinum (weekly/soak)

---

## API Option Coverage — Scenarios by Function (from test_selection_matrix.md)

### plan()

- [x] (P0) Implement E2E-PLAN-001 — Plan empty inputs; expect `Plan.actions=[]` and redacted facts TS_ZERO (test_selection_matrix.md E2E-PLAN-001; api_option_inventory.md §plan; traceability.md plan.link_count=min:0, restore_count=min:0; env: Base-1). Verify stable ordering and zero actions.
- [x] (P1) Implement E2E-PLAN-002 — Deterministic sorting for mixed actions; random input order seed=42 (test_selection_matrix.md E2E-PLAN-002; api_option_inventory.md §plan; traceability.md plan.many:10, restore=one, input_order=random; env: Base-1). Assert sorted by kind then target.rel.
- [x] (P0) Implement E2E-PLAN-003 — Duplicate targets preserved (no dedupe) (test_selection_matrix.md E2E-PLAN-003; api_option_inventory.md §plan; traceability.md duplicate_targets=true; env: Base-1). Assert both actions present.
- [ ] (P3) Implement E2E-PLAN-004 — Huge plan size (1000 links) performance and determinism (test_selection_matrix.md E2E-PLAN-004; api_option_inventory.md §plan; traceability.md link_count=huge:1000; env: Base-4). Assert ordering and run-time within budget.
- [x] (P1) Implement E2E-PLAN-005 — Many restores sorting (test_selection_matrix.md E2E-PLAN-005; api_option_inventory.md §plan; traceability.md restore_count=many:10; env: Base-1). Assert deterministic ordering.
- [x] (P0) Implement E2E-PLAN-006 — Single link trivial plan (test_selection_matrix.md E2E-PLAN-006; api_option_inventory.md §plan; env: Base-1). Assert one EnsureSymlink.

### preflight()

- [x] (P1) Implement E2E-PREFLIGHT-001 — Rescue required but unavailable → STOP; assert summary `error_id=E_POLICY`, `exit_code=10`, `summary_error_ids` includes `E_POLICY` (test_selection_matrix.md E2E-PREFLIGHT-001; api_option_inventory.md §rescue; traceability.md rescue.require=true; REQ-E2, REQ-RC2; env: Base-1).
- [x] (P1) Implement E2E-PREFLIGHT-002 — Ownership strict without oracle → STOP; assert ownership mentioned and `summary_error_ids` includes `E_OWNERSHIP` (test_selection_matrix.md E2E-PREFLIGHT-002; api_option_inventory.md §risks.ownership_strict; REQ-S4; env: Base-1).
- [x] (P1) Implement E2E-PREFLIGHT-003 — Preservation required but unsupported → STOP (test_selection_matrix.md E2E-PREFLIGHT-003; api_option_inventory.md §durability.preservation; REQ-S5; env: Base-1 with unsupported path shim). Assert STOP reason mentions preservation unsupported.
- [x] (P0) Implement E2E-PREFLIGHT-004 — Rescue not required baseline (test_selection_matrix.md E2E-PREFLIGHT-004; api_option_inventory.md §rescue; env: Base-1). Assert `ok=true`.
- [x] (P1) Implement E2E-PREFLIGHT-005 — Exec check with min_count=100 → STOP (test_selection_matrix.md E2E-PREFLIGHT-005; api_option_inventory.md §rescue.exec_check/min_count; REQ-RC2; env: Base-1). Assert STOP and message.
- [ ] (P1) Implement E2E-PREFLIGHT-006 — Extra mount checks (5) (test_selection_matrix.md E2E-PREFLIGHT-006; api_option_inventory.md §apply.extra_mount_checks; env: Base-2). Assert notes present.
- [ ] (P0) Implement E2E-PREFLIGHT-007 — Long backup tag annotation (test_selection_matrix.md E2E-PREFLIGHT-007; api_option_inventory.md §backup.tag; env: Base-1). Assert tag carried in rows.
- [x] (P0) Implement E2E-PREFLIGHT-008 — One extra mount check (test_selection_matrix.md E2E-PREFLIGHT-008; api_option_inventory.md §apply.extra_mount_checks; env: Base-1). Assert note present.
- [x] (P0) Implement E2E-PREFLIGHT-009 — Empty backup tag baseline (test_selection_matrix.md E2E-PREFLIGHT-009; api_option_inventory.md §backup.tag; env: Base-1). Assert ok.
- [x] (P0) Implement E2E-PREFLIGHT-010 — Exec check disabled baseline (test_selection_matrix.md E2E-PREFLIGHT-010; api_option_inventory.md §rescue.exec_check; env: Base-1). Assert ok.
- [x] (P0) Implement E2E-PREFLIGHT-011 — Coreutils tag baseline (test_selection_matrix.md E2E-PREFLIGHT-011; api_option_inventory.md §backup.tag; env: Base-1). Assert tag carried.

### apply()

- [x] (P0) Implement E2E-APPLY-001 — DryRun symlink ensure; assert `ApplyReport.errors=[]`, per-action facts TS_ZERO, after_kind="symlink" (test_selection_matrix.md E2E-APPLY-001; api_option_inventory.md §mode/locking/exdev/override_preflight; REQ-D2; env: Base-1).
- [x] (P1) Implement E2E-APPLY-002 — Commit happy path; assert executed matches plan, summary has no error_id (test_selection_matrix.md E2E-APPLY-002; api_option_inventory.md §locking Required; env: Base-1). Capture attestation if configured.
- [x] (P1) Implement E2E-APPLY-003 — Locking required, no manager → `E_LOCKING` with `exit_code=30`; no FS mutation (test_selection_matrix.md E2E-APPLY-003; api_option_inventory.md §locking/lock_manager/timeout; REQ-L1, REQ-L3; env: Base-1).
- [x] (P1) Implement E2E-APPLY-004 — Smoke required, runner absent → `E_SMOKE`, `rolled_back=false` (auto_rollback=false) (test_selection_matrix.md E2E-APPLY-004; api_option_inventory.md §smoke; REQ-H3; env: Base-1).
- [x] (P2) Implement E2E-APPLY-005 — EXDEV degraded fallback used; assert `degraded=true` (test_selection_matrix.md E2E-APPLY-005; api_option_inventory.md §exdev; REQ-F2; env: Base-3 with `SWITCHYARD_FORCE_EXDEV=1`).
- [ ] (P1) Implement E2E-APPLY-006 — Ownership strict gates apply via preflight (override_preflight=false) (test_selection_matrix.md E2E-APPLY-006; api_option_inventory.md §override_preflight; REQ-S4, REQ-C2; env: Base-1). Assert early failure.
- [x] (P2) Implement E2E-APPLY-007 — Attestation present on success; assert `attestation{sig_alg, signature, bundle_hash, public_key_id}` (test_selection_matrix.md E2E-APPLY-007; api_option_inventory.md §attestor; REQ-O4; env: Base-3).
- [x] (P1) Implement E2E-APPLY-008 — Best-effort restore tolerates missing backup; assert no `E_BACKUP_MISSING` (test_selection_matrix.md E2E-APPLY-008; api_option_inventory.md §best_effort_restore; env: Base-1).
- [x] (P1) Implement E2E-APPLY-009 — Capture restore snapshot enables inverse (test_selection_matrix.md E2E-APPLY-009; api_option_inventory.md §capture_restore_snapshot; env: Base-1). Verify `plan_rollback_of` later produces `RestoreFromBackup`.
- [ ] (P1) Implement E2E-APPLY-010 — Lock timeout high; assert wait_ms <= timeout (test_selection_matrix.md E2E-APPLY-010; api_option_inventory.md §lock_timeout_ms; REQ-L3, REQ-L5; env: Base-2 with rival-holder).
- [x] (P2) Implement E2E-APPLY-011 — Smoke runner present and fails → auto-rollback=true; assert `rolled_back=true`, `error_id=E_SMOKE` (test_selection_matrix.md E2E-APPLY-011; api_option_inventory.md §smoke_outcome; REQ-H2; env: Base-3).
- [ ] (P2) Implement E2E-APPLY-012 — Smoke runner present and ok; assert `rolled_back=false` (test_selection_matrix.md E2E-APPLY-012; api_option_inventory.md §smoke; REQ-H1; env: Base-3).
- [ ] (P1) Implement E2E-APPLY-013 — Attestation signing error tolerated; attestation omitted (test_selection_matrix.md E2E-APPLY-013; api_option_inventory.md §attestation_outcome; env: Base-2).
- [ ] (P3) Implement E2E-APPLY-014 — ENOSPC during backup/restore path; assert failure with generic policy error; no partials (test_selection_matrix.md E2E-APPLY-014; env: Base-4 low-disk).
- [ ] (P2) Implement E2E-APPLY-015 — Lock contention timeout → `E_LOCKING`; bounded wait (test_selection_matrix.md E2E-APPLY-015; api_option_inventory.md §lock_timeout_ms; REQ-L3; env: Base-3 with rival-holder).
- [ ] (P1) Implement E2E-APPLY-016 — Restore without capture snapshot (test_selection_matrix.md E2E-APPLY-016; api_option_inventory.md §capture_restore_snapshot=false; env: Base-1). Verify rollback plan behavior.
- [ ] (P1) Implement E2E-APPLY-017 — Ownership strict with oracle present; assert provenance present (test_selection_matrix.md E2E-APPLY-017; api_option_inventory.md §ownership_oracle; REQ-S4, REQ-O7; env: Base-2).
- [ ] (P1) Implement E2E-APPLY-018 — DryRun ignores smoke; success, no E_SMOKE (test_selection_matrix.md E2E-APPLY-018; api_option_inventory.md §mode=DryRun, smoke=Require; REQ-H3 bound to Commit only; env: Base-1).
- [x] (P2) Implement E2E-APPLY-019 — EXDEV with Fail policy → `E_EXDEV` (test_selection_matrix.md E2E-APPLY-019; api_option_inventory.md §exdev=Fail; REQ-F2; env: Base-3 `SWITCHYARD_FORCE_EXDEV=1`).
- [ ] (P2) Implement E2E-APPLY-020 — Sidecar integrity disabled tolerates tamper (test_selection_matrix.md E2E-APPLY-020; api_option_inventory.md §durability.sidecar_integrity=false; REQ-S6; env: Base-3).
- [x] (P0) Implement E2E-APPLY-021 — Backup durability flag off → facts `backup_durable=false` (test_selection_matrix.md E2E-APPLY-021; api_option_inventory.md §durability.backup_durability=false; env: Base-1).
- [ ] (P3) Implement E2E-APPLY-022 — Crash between backup and rename; on rerun system converges; no tmp artifacts remain (test_selection_matrix.md E2E-APPLY-022; oracles_and_invariants.md §apply invariants; REQ-A1, REQ-A3; env: Base-4 crash-injection).

### plan_rollback_of()

- [x] (P0) Implement E2E-ROLLBACK-001 — Invert only symlink actions (test_selection_matrix.md E2E-ROLLBACK-001; api_option_inventory.md §plan_rollback_of; env: Base-1). Assert inverse plan content.
- [x] (P1) Implement E2E-ROLLBACK-002 — Invert previous restore when snapshot captured (test_selection_matrix.md E2E-ROLLBACK-002; env: Base-1). Assert `RestoreFromBackup` present.
- [ ] (P1) Implement E2E-ROLLBACK-003 — Mixed executed actions inversion (test_selection_matrix.md E2E-ROLLBACK-003; env: Base-1). Assert mixed inversions.

### prune_backups()

- [ ] (P1) Implement E2E-PRUNE-001 — Count limit min=0; newest retained (test_selection_matrix.md E2E-PRUNE-001; api_option_inventory.md §retention_count_limit; REQ-PN1; env: Base-1). Assert counts match.
{{ ... }}
- [ ] (P1) Implement E2E-PRUNE-009 — Age limit max=365d (test_selection_matrix.md E2E-PRUNE-009; env: Base-1 seeded). Assert pruning >365d.
- [ ] (P1) Implement E2E-PRUNE-010 — Age limit min=1s (test_selection_matrix.md E2E-PRUNE-010; env: Base-1 seeded). Assert pruning >1s.

### SafePath::from_rooted()

- [x] (P0) Implement E2E-SAFEPATH-001 — Reject dotdot (invalid) (test_selection_matrix.md E2E-SAFEPATH-001; api_option_inventory.md §SafePath.candidate_form; REQ-S1; env: Base-1).
- [x] (P0) Implement E2E-SAFEPATH-002 — Accept absolute inside root (test_selection_matrix.md E2E-SAFEPATH-002; REQ-API1; env: Base-1).
- [x] (P0) Implement E2E-SAFEPATH-003 — Reject absolute outside root (test_selection_matrix.md E2E-SAFEPATH-003; REQ-S1; env: Base-1).
- [x] (P0) Implement E2E-SAFEPATH-004 — Relative normal (test_selection_matrix.md E2E-SAFEPATH-004; env: Base-1). Assert Ok.
- [x] (P0) Implement E2E-SAFEPATH-005 — Curdir normalization (test_selection_matrix.md E2E-SAFEPATH-005; env: Base-1). Assert `.` components removed.
- [x] (P0) Implement E2E-SAFEPATH-006 — Root not absolute (invalid) (test_selection_matrix.md E2E-SAFEPATH-006; env: Base-1). Assert panic or error.
- [ ] (P0) Implement E2E-SAFEPATH-007 — Unsupported component invalid (test_selection_matrix.md E2E-SAFEPATH-007; env: Base-1). Assert error.
- [x] (P0) Implement E2E-SAFEPATH-008 — Empty candidate (test_selection_matrix.md E2E-SAFEPATH-008; env: Base-1). Assert Ok within root.
- [x] (P0) Implement E2E-SAFEPATH-009 — Unicode segments (test_selection_matrix.md E2E-SAFEPATH-009; env: Base-1). Assert Ok.
- [x] (P0) Implement E2E-SAFEPATH-010 — Short path (3 segs) (test_selection_matrix.md E2E-SAFEPATH-010; env: Base-1). Assert Ok.
- [ ] (P2) Implement E2E-SAFEPATH-011 — Long path (255 bytes) (test_selection_matrix.md E2E-SAFEPATH-011; env: Base-3). Assert Ok.
- [ ] (P3) Implement E2E-SAFEPATH-012 — Huge path (4096 bytes) (test_selection_matrix.md E2E-SAFEPATH-012; env: Base-4). Assert Ok or documented error.

---

{{ ... }}

- [ ] (P0) Implement pairwise combination generator for all functions using `TESTPLAN/combinatorial_model.json`; export scenario manifests with seeds (pairwise=4242) and map to E2E IDs (combinatorial_model.json; test_selection_matrix.md; e2e_overview.md Determinism).
- [ ] (P2) Implement 3-wise generator for High-risk axes and merge with curated additions (seed=314159) (combinatorial_model.json; selection strategy).
- [ ] (P0) Implement scenario harness: constructs temp roots, builds `SafePath`, applies policy knobs, runs plan/preflight/apply; records redacted facts and FS state for assertions (flakiness_and_repro.md; oracles_and_invariants.md).

---

## Environment Rotation & Runners (from environment_matrix.md)

- [ ] (P0) Implement EnvRunner Base-1 (CI quick): ext4-like tmpfs, same-fs, normal disk, single-thread; unicode off; short paths; file lock manager present (environment_matrix.md Base-1).
- [ ] (P1) Implement EnvRunner Base-2 (Daily): Base-1 + unicode + long paths + rival lock holder (environment_matrix.md Base-2).
- [ ] (P2) Implement EnvRunner Base-3 (Nightly): EXDEV simulated (`SWITCHYARD_FORCE_EXDEV=1`), low-disk for selected, tampered backup, relative symlinks, deep nesting, suid/sgid where permitted (environment_matrix.md Base-3).
- [ ] (P3) Implement EnvRunner Base-4 (Weekly/Platinum): huge path lengths, xfs/btrfs where available, crash injection around swap/restore, parallel stress (environment_matrix.md Base-4; REQ-F3).
- [ ] (P2) Implement helper: RivalLockHolder to simulate contention with bounded hold times (environment_matrix.md Parallelism/Contention; used by E2E-APPLY-010/015; REQ-L3).
- [ ] (P2) Implement helper: ExdevSimulator via env `SWITCHYARD_FORCE_EXDEV=1` (environment_matrix.md Cross-filesystem; used by E2E-APPLY-005/019; REQ-F2).
- [ ] (P3) Implement helper: DiskPressureInjector for ENOSPC (environment_matrix.md Disk space; used by E2E-APPLY-014).
- [ ] (P3) Implement helper: CrashInjector at swap/restore steps (environment_matrix.md Crash/kill points; used by E2E-APPLY-022; REQ-A1).

---

## Oracles & Invariants Assertions (from oracles_and_invariants.md)

- [ ] (P0) Plan: assert deterministic sorting (kind then target.rel), stable `action_id` per SPEC (plan_id/action_id derivation) (oracles_and_invariants.md §plan; REQ-D1).
- [ ] (P0) Preflight: assert rows sorted by (path, action_id), and summary `error_id/exit_code` mapping for failures (oracles_and_invariants.md §preflight; REQ-E2).
- [ ] (P0) Apply: per-action facts emitted; summary contains `summary_error_ids` chain on failure (oracles_and_invariants.md §apply; REQ-O1, REQ-O8).
- [ ] (P0) EnsureSymlink success: target becomes symlink to source; parent dir fsynced (oracles_and_invariants.md §apply; fs/atomic.rs; REQ-TOCTOU1).
- [ ] (P0) Restore success: restores prior state from sidecar; payload hash checked when integrity enabled; parent fsynced (oracles_and_invariants.md §apply; REQ-S6).
- [ ] (P0) Locking invariant: Required+no manager → early E_LOCKING and no FS mutation (oracles_and_invariants.md §apply; REQ-L1/L3).
- [ ] (P2) Smoke invariants: missing/fail → E_SMOKE; auto-rollback per policy (oracles_and_invariants.md §apply; REQ-H2/H3).
- [ ] (P2) EXDEV invariants: DegradedFallback → `degraded=true`; Fail → `E_EXDEV` (oracles_and_invariants.md §apply; REQ-F2).
- [ ] (P1) Best-effort restore invariant: tolerate missing payload when enabled (oracles_and_invariants.md §apply).
- [ ] (P0) Redaction invariant: DryRun facts TS_ZERO and volatile fields removed; Commit comparisons ignore volatile fields (oracles_and_invariants.md §Determinism; REQ-D2).
- [ ] (P0) Rollback: inverse plan derived from executed actions; pure function (oracles_and_invariants.md §rollback; REQ-R1/R3).
- [ ] (P1) Prune: newest never deleted; payload+sidecar removed; parent fsynced (oracles_and_invariants.md §prune; REQ-PN1/PN2/PN3).
- [ ] (P0) SafePath: invariants across candidate forms; `rel` normalized (oracles_and_invariants.md §safepath; REQ-S1).
- [ ] (P3) Bounds: record `fsync_ms` and assert ≤50ms where applicable (best-effort, non-flaky threshold) (oracles_and_invariants.md; REQ-BND1).

---

## Requirements Coverage (from SPEC/requirements.yaml)

- [ ] (P3) REQ-A1 Atomic crash-safety — cover with E2E-APPLY-022; assert no tmp artifacts and convergence (requirements.yaml REQ-A1; test_selection_matrix.md E2E-APPLY-022).
- [ ] (P2) REQ-A2 No broken/missing path visible — cover with E2E-APPLY-005/019; assert no intermediate broken link visible (requirements.yaml REQ-A2; test_selection_matrix.md E2E-APPLY-005/019).
- [ ] (P2) REQ-A3 All-or-nothing — use E2E-APPLY-011 failure mid-plan + auto-rollback; assert no visible partials (requirements.yaml REQ-A3; test_selection_matrix.md E2E-APPLY-011).
- [ ] (P1) REQ-R1 Rollback reversibility — use E2E-ROLLBACK-001/002/003; assert exact inverse actions (requirements.yaml REQ-R1; test_selection_matrix.md E2E-ROLLBACK-001..003).
- [ ] (P1) REQ-R2 Restore exact topology — validate symlink/file topology matches before state on rollback (requirements.yaml REQ-R2; E2E-ROLLBACK-002/003).
- [ ] (P1) REQ-R3 Idempotent rollback — run rollback twice; assert stable state (requirements.yaml REQ-R3; E2E-ROLLBACK-001..003).
- [ ] (P1) REQ-R4 Auto reverse-order rollback — assert reverse order on first failure (requirements.yaml REQ-R4; E2E-APPLY-011).
- [ ] (P1) REQ-R5 Partial restoration facts on rollback error — induce rollback error; assert summary fields present (requirements.yaml REQ-R5; apply/rollback summary).
- [ ] (P0) REQ-S1 Safe paths only — cover via E2E-SAFEPATH-001/003 and positive cases (requirements.yaml REQ-S1; test_selection_matrix.md SAFEPATH set).
- [ ] (P3) REQ-S2 Reject unsupported FS states — simulate read-only/noexec/immutable target; assert fail-closed (requirements.yaml REQ-S2; preflight/apply gates).
- [ ] (P0) REQ-S3 Source ownership gating — add fixture ensuring source root-owned/not world-writable by policy; assert gating (requirements.yaml REQ-S3; preflight checks).
- [ ] (P0) REQ-S4 Strict target ownership — cover with E2E-PREFLIGHT-002 and E2E-APPLY-017 (requirements.yaml REQ-S4).
- [ ] (P1) REQ-S5 Preservation capability gating — cover with E2E-PREFLIGHT-003 (requirements.yaml REQ-S5).
- [ ] (P1) REQ-S6 Backup sidecar integrity — cover with E2E-APPLY-020 (requirements.yaml REQ-S6).
- [ ] (P1) REQ-PF1 Preflight YAML dry-run parity — export YAML for dry-run and real preflight, assert byte-identical (requirements.yaml REQ-PF1; preflight YAML exporter).
- [ ] (P0) REQ-O1 Structured fact for every step — assert all stages emit facts (requirements.yaml REQ-O1; logging/audit; E2E-APPLY-001/002).
- [ ] (P0) REQ-O2 Dry-run facts identical to real-run — via redaction equality (requirements.yaml REQ-O2; E2E-APPLY-001 vs Commit).
- [ ] (P0) REQ-O3 Versioned stable facts schema — assert `schema_version` present (v2) (requirements.yaml REQ-O3; StageLogger events).
- [ ] (P1) REQ-O4 Signed attestations — assert attestation fields present when configured (requirements.yaml REQ-O4; E2E-APPLY-007).
- [ ] (P0) REQ-O5 Before/after hashes — assert `before_hash`/`after_hash` recorded (requirements.yaml REQ-O5; E2E-APPLY-001/002).
- [ ] (P0) REQ-O6 Secret masking — assert redaction masks secrets in provenance/attestation (requirements.yaml REQ-O6; logging/redact.rs tests).
- [ ] (P1) REQ-O7 Provenance completeness — assert uid/gid/pkg present when oracle provided (requirements.yaml REQ-O7; E2E-APPLY-017).
- [ ] (P0) REQ-O8 Summary error chain — assert `summary_error_ids` on failures (requirements.yaml REQ-O8; E2E-PREFLIGHT-001, E2E-APPLY-011).
- [ ] (P0) REQ-E1 Stable error identifiers — assert `error_id` present on failures (requirements.yaml REQ-E1; multiple scenarios).
- [ ] (P0) REQ-E2 Preflight summary exit-code mapping — assert `error_id=E_POLICY`, `exit_code=10` (requirements.yaml REQ-E2; E2E-PREFLIGHT-001).
- [ ] (P0) REQ-L1 Single mutator — assert only one Commit mutates at a time under lock (requirements.yaml REQ-L1; E2E-APPLY-002/015).
- [ ] (P0) REQ-L2 Warn when no lock manager — run Commit with Optional+no manager; assert WARN fact (requirements.yaml REQ-L2; add assert in apply attempt). If absent, file follow-up issue.
- [ ] (P0) REQ-L3 Bounded lock wait with timeout — assert lock_wait_ms <= timeout and `E_LOCKING` on timeout (requirements.yaml REQ-L3; E2E-APPLY-010/015).
- [ ] (P1) REQ-L4 LockManager required in production — ensure production preset enforces Required; assert early fail when absent (requirements.yaml REQ-L4; Policy::production_preset).
- [ ] (P1) REQ-L5 Lock attempts metric — assert approx lock_attempts present in `apply.attempt` (requirements.yaml REQ-L5; E2E-APPLY-010/015).
- [ ] (P0) REQ-RC1 Rescue profile available — assert backups always retained for restore paths in scope (requirements.yaml REQ-RC1; backup presence checks).
- [ ] (P0) REQ-RC2 Verify fallback path — assert preflight verifies functional fallback (requirements.yaml REQ-RC2; E2E-PREFLIGHT-001/005).
- [ ] (P0) REQ-RC3 Fallback toolset on PATH — simulate PATH; assert at least one binary set present/exec (requirements.yaml REQ-RC3; preflight tooling shim).
- [ ] (P0) REQ-D1 Deterministic IDs — assert UUIDv5 derivation for plan_id/action_id stable over runs (requirements.yaml REQ-D1; plan tests).
- [ ] (P0) REQ-D2 Redaction-pinned dry-run — assert redacted dry-run facts equal real-run (requirements.yaml REQ-D2; E2E-APPLY-001 vs Commit redacted).
- [ ] (P0) REQ-C1 Dry-run by default — assert default ApplyMode=DryRun in builder or API surface (requirements.yaml REQ-C1; api surface tests).
- [ ] (P1) REQ-C2 Fail-closed on critical violations — assert STOP without override_preflight (requirements.yaml REQ-C2; E2E-APPLY-006).
- [ ] (P1) REQ-H1 Minimal smoke suite — assert default runner validates symlink destinations (requirements.yaml REQ-H1; adapters::DefaultSmokeRunner; E2E-APPLY-012).
- [ ] (P2) REQ-H2 Auto-rollback on smoke failure — assert rollback executed (requirements.yaml REQ-H2; E2E-APPLY-011).
- [ ] (P1) REQ-H3 Health verification part of commit — assert missing runner triggers error in Commit (requirements.yaml REQ-H3; E2E-APPLY-004).
- [ ] (P2) REQ-F1 EXDEV fallback preserves atomic visibility — assert fallback path uses safe sequence (best-effort) (requirements.yaml REQ-F1; E2E-APPLY-005).
- [ ] (P2) REQ-F2 Degraded mode policy & telemetry — assert degraded flag or error by policy (requirements.yaml REQ-F2; E2E-APPLY-005/019).
- [ ] (P3) REQ-F3 Supported filesystems verified — schedule xfs/btrfs/tmpfs acceptance (requirements.yaml REQ-F3; environment_matrix.md Base-4).
- [ ] (P0) REQ-API1 SafePath-only for mutating APIs — assert mutating APIs accept SafePath (requirements.yaml REQ-API1; compile-time/type tests).
- [ ] (P0) REQ-TOCTOU1 TOCTOU-safe syscall sequence — assert behavior consistent with open_dir_nofollow→openat→renameat→fsync (requirements.yaml REQ-TOCTOU1; fs/atomic.rs behavior).
- [ ] (P1) REQ-PN1 Newest backup retained — assert in prune tests (requirements.yaml REQ-PN1; E2E-PRUNE-001).
- [ ] (P1) REQ-PN2 Prune deletes payload+sidecar and fsyncs parent — assert deletions and fsync (requirements.yaml REQ-PN2; E2E-PRUNE-001..010).
- [ ] (P1) REQ-PN3 Prune emits result summary — assert `prune.result` fact fields (requirements.yaml REQ-PN3; prune tests).
- [ ] (P3) REQ-BND1 fsync within 50ms — assert `fsync_ms <= 50` best-effort with non-flaky tolerance (requirements.yaml REQ-BND1; E2E-APPLY-* swap facts).
- [ ] (P0) REQ-CI1 Golden fixtures existence — generate golden JSON fixtures for plan, preflight, apply, rollback (requirements.yaml REQ-CI1; CI gate).
- [ ] (P0) REQ-CI2 Zero-SKIP gate — fail CI when any test is SKIP (requirements.yaml REQ-CI2; CI config).
- [ ] (P0) REQ-CI3 Golden diff gate — fail CI on any non-identical fixture diff (requirements.yaml REQ-CI3; golden-diff tooling).
- [ ] (P0) REQ-VERS1 Facts carry schema_version — assert schema_version=v2 present in facts (requirements.yaml REQ-VERS1; logging schemas).
- [ ] (P1) REQ-T1 Core types are Send+Sync — compile-time assertions for Plan and apply engine (requirements.yaml REQ-T1; static_assertions).
- [ ] (P1) REQ-T2 Single mutator under lock across threads — concurrent apply calls; only one mutates (requirements.yaml REQ-T2; E2E-APPLY-002/015).

---

## Flakiness & Repro/Determinism (from flakiness_and_repro.md)

- [ ] (P0) Ensure every scenario uses temp roots and `SafePath::from_rooted` (flakiness_and_repro.md; applies across suites).
- [ ] (P0) Ensure DryRun assertions compare redacted facts; Commit ignores volatile fields (flakiness_and_repro.md; logging/redact.rs).
- [ ] (P0) Record seeds and environment toggles per scenario artifact (flakiness_and_repro.md; test harness).
- [ ] (P2) Implement retry/quarantine protocol hooks; quarantine flaky scenarios with tags (flakiness_and_repro.md; CI integration).

---

## Scheduling & Cost / CI Tiers (from scheduling_and_cost.md)

- [ ] (P0) Define Bronze suite: minimal pairwise for plan/preflight/apply(DryRun) + minimal boundaries; time ≤2m (scheduling_and_cost.md; CI pipeline).
- [ ] (P1) Define Silver suite: full pairwise + selected 3-wise; boundary overlays; Base-1 & Base-2 rotation; time ≤8m (scheduling_and_cost.md).
- [ ] (P2) Define Gold suite: 3-wise High/Medium + negative suites; EXDEV/lock contention; Base-1 & Base-3; time ≤20m (scheduling_and_cost.md).
- [ ] (P3) Define Platinum suite: soak + fault injection + rare envs (xfs/btrfs, huge paths); Base-4; time ≤40m (scheduling_and_cost.md).
