# Switchyard Feature TODO

Generated: 2025-09-12

This file consolidates all feature proposals (AI1–AI4) into an actionable, prioritized plan. Each item includes why it’s good, impact, risk, feasibility, complexity, and scope. Paths reference the `cargo/switchyard/` crate unless otherwise noted.

## Index of Workstreams

- [P0 — Security and Recovery First](#p0--security-and-recovery-first)
  - SafePath enforcement at mutating boundaries
  - SUID/SGID preflight gate
  - Hardlink breakage preflight check
  - Backup/sidecar durability (fsync + *at)
  - Backup sidecar integrity (payload hash)
  - Prune backups (retention)
- [P1 — Observability and Operability](#p1--observability-and-operability)
  - Summary error IDs in summaries
  - Error taxonomy chain + E_OWNERSHIP
  - Performance telemetry aggregation
  - Lock fairness telemetry
- [P2 — Fidelity, Robustness, and Developer Experience](#p2--fidelity-robustness-and-developer-experience)
  - Extended preservation tiers
  - Immutable detection reliability
  - Public FS atom restriction completion
  - Production preset adapter docs
  - Deprecation program wrap-up


## P0 — Security and Recovery First

- SafePath enforcement at mutating boundaries (Phase 1 → Phase 2)
  - Priority: P0
  - Why it’s good: Enforces traversal/TOCTOU safety via types, aligning with SPEC v1.1 mandates for mutating APIs.
  - Impact: Medium–High (API safety; phase 2 is a breaking change when removing raw `&Path`).
  - Risk: Downstream reliance on raw `&Path`. We will make a direct breaking change.
  - Feasibility: Medium
  - Complexity: 3–4
  - Scope:
    - Change mutator APIs to accept `&SafePath` exclusively; remove `&Path` variants.
    - Files: `src/fs/swap.rs::replace_file_with_symlink`, `src/fs/restore.rs::{restore_file, restore_file_prev}`, high-level callsites under `src/api/**`.
    - Emit `safepath_validation=success|failure` in `src/api/apply/mod.rs` `apply.attempt` facts.
    - Docs: Update `DOCS/analysis/CLI_INTEGRATION_GUIDE.md`; update SPEC § Public Interfaces.
  - Dependencies: SPEC doc update for SafePath-only requirement.

  - Action Plan (no backcompat required)
    - [x] Change mutator signatures to `&SafePath` only
      - [x] `src/fs/swap.rs::replace_file_with_symlink(&SafePath, &SafePath, ...)`
      - [x] `src/fs/restore.rs::restore_file(&SafePath, ...)`
      - [x] `src/fs/restore.rs::restore_file_prev(&SafePath, ...)`
      - [x] `src/fs/mod.rs` re-exports compile after signature changes
    - [x] Update all call sites to pass `SafePath`
      - [x] `src/api/apply/handlers.rs` (EnsureSymlink handler)
        - [x] Add field to attempt extra: `"safepath_validation": "success"` at the emit where we already set `action_id` and `path`.
        - [x] Change call to `crate::fs::replace_file_with_symlink(source, target, dry, api.policy.allow_degraded_fs, &api.policy.backup_tag)` (drop `.as_path()` calls).
        - [x] Keep hashing with `sha256_hex_of(&source.as_path())` and `resolve_symlink_target(&target.as_path())` (unchanged).
      - [x] `src/api/apply/handlers.rs` (RestoreFromBackup handler)
        - [x] Add field to attempt extra: `"safepath_validation": "success"`.
        - [x] If `capture_restore_snapshot` remains a path-based helper, keep `create_snapshot(&target.as_path(), ...)` for now.
        - [x] Change calls to `crate::fs::restore_file_prev(target, ...)` or `crate::fs::restore_file(target, ...)` (drop `.as_path()`).
      - [x] `src/api/apply/mod.rs` (rollback loops + smoke failure rollback)
        - [x] In rollback on failure: change `fs::restore_file(&target.as_path(), …)` to `fs::restore_file(&target, …)` in both loops (initial failure and smoke rollback).
        - [x] Ensure `lock_backend` telemetry remains intact (recently added).
      - [x] Grep checklist (update all SafePath call sites)
        - [x] `grep -R "restore_file\(&" src | grep -v safe` and convert to pass `&SafePath`.
        - [x] `grep -R "replace_file_with_symlink\(&" src` and ensure `&SafePath` is passed.
    - [x] Emit safepath telemetry
      - [x] Add `"safepath_validation": "success"` to `emit_apply_attempt(...)` in both handlers
    - [x] Tests
      - [x] Update unit tests in `src/fs/swap.rs` to construct `SafePath` (partially done: `atomic_swap_creates_symlink_pointing_to_source`, `replace_and_restore_roundtrip`)
      - [x] Update any direct `restore_file(...)` calls in tests to pass `SafePath`
      - [x] Add negative tests for `SafePath::from_rooted` rejecting escaping paths
        - [x] Add test in `src/types/safepath.rs` module tests or new `tests/safepath_negative.rs` with cases: `../etc`, absolute outside root.
      - [x] Grep and convert tests referencing old signatures
        - [x] `grep -R "replace_file_with_symlink(\s*&[a-zA-Z].*Path" cargo/switchyard/tests src | sed -n 'p'`
        - [x] `grep -R "restore_file(\s*&[a-zA-Z].*Path" cargo/switchyard/tests src | sed -n 'p'`
    - [ ] Docs
      - [ ] Add SafePath code examples to `DOCS/analysis/CLI_INTEGRATION_GUIDE.md`
      - [ ] Note SafePath-only mutators in public API section; no deprecation window needed
    - [x] Acceptance
      - [x] All public mutators accept only `&SafePath`
      - [x] All call sites compile; unit/integration tests pass
      - [x] apply.attempt facts include `safepath_validation`

- SUID/SGID preflight gate
  - Priority: P0
  - Why it’s good: Prevents privilege escalation risk by default.
  - Impact: High (security) and additive.
  - Risk: False positives. Mitigation: policy override with clear notes.
  - Feasibility: High
  - Complexity: 1
  - Scope: Add `check_suid_sgid_risk()` in `src/preflight/checks.rs`; integrate into `src/api/preflight/mod.rs`; policy `allow_suid_sgid_mutation` (default false in production presets) in `src/policy/config.rs`.
  - Dependencies: SPEC proposal for SUID/SGID preflight gate.

  - Implementation Steps
    - [x] Implement `check_suid_sgid_risk(path: &Path) -> std::io::Result<bool>`
      - [x] On Unix, use `std::fs::symlink_metadata(path)` and `std::os::unix::fs::MetadataExt::mode()`; risk if `(mode & 0o6000) != 0`.
      - [x] For symlinks, consider target metadata (best-effort) or record note `"suid_sgid unresolved for symlink"`.
    - [x] Integrate into preflight
      - [x] In `src/api/preflight/mod.rs`, for both actions, call `check_suid_sgid_risk(target)`.
      - [x] If risk and `!policy.allow_suid_sgid_mutation`, push STOP `"suid/sgid risk"` and add note.
      - [x] If risk and allowed, push WARN note `"suid/sgid risk allowed by policy"`.
    - [x] Policy wiring
      - [x] Add `allow_suid_sgid_mutation: bool` to `Policy` (default false; true only when explicitly set); include in defaults.
      - [x] Ensure apply gating parity via `policy::gating`.
    - [x] Tests
      - [x] Unit: mark a file SUID in tmpdir and assert STOP vs WARN based on policy.
      - [x] Integration: preflight summary includes STOP on risk with `error_id=E_POLICY`.

- Hardlink breakage preflight check
  - Priority: P0
  - Why it’s good: Avoids silent hardlink breakage and data inflation.
  - Impact: Medium–High; additive.
  - Risk: Blocking legitimate use. Mitigation: policy override + clear messaging.
  - Feasibility: High
  - Complexity: 2
  - Scope: Add `check_hardlink_hazard()` in `src/preflight/checks.rs`, integrate in `src/api/preflight/mod.rs`, and add policy `allow_hardlink_breakage` (default false) in `src/policy/config.rs`.
  - Implementation Steps
    - [x] Implement `check_hardlink_hazard(path: &Path) -> std::io::Result<bool>`
      - [x] Use `std::fs::symlink_metadata(path)` and on Unix get `nlink` via `std::os::unix::fs::MetadataExt::nlink()`; hazard = `nlink > 1`.
      - [x] If metadata read fails, return `Ok(false)` and let caller add a note.
    - [x] Integrate into preflight
      - [x] In `src/api/preflight/mod.rs`, for both `EnsureSymlink` and `RestoreFromBackup`, call `check_hardlink_hazard(target)`.
      - [x] If hazard and `!policy.allow_hardlink_breakage`, push STOP reason and add note `"hardlink risk"`.
      - [x] If hazard and allowed, push WARN note `"hardlink risk allowed by policy"`.
    - [x] Policy wiring
      - [x] Add `allow_hardlink_breakage: bool` to `Policy` (default false; true only if explicitly set).
      - [x] Ensure gating mirrors preflight behavior when `override_preflight=false`.
    - [x] Tests
      - [x] Unit: create two hardlinks to the same inode in a tmpdir and assert detection.
      - [x] Integration: preflight on a hardlinked target STOPs by default and WARNs when policy allows.

- Backup/sidecar durability (fsync + *at)
  - Priority: P0
  - Why it’s good: Survives crashes and ensures durable backups and metadata.
  - Impact: High (safety); additive.
  - Risk: Performance hit from fsync. Mitigation: policy `require_backup_durability` default true; can relax.
  - Feasibility: High
  - Complexity: 2
  - Scope: Ensure `create_snapshot()` and `write_sidecar()` use dir handles + `*at` syscalls and `fsync_parent_dir()`; emit `backup_durable` in `apply` facts; add policy in `src/policy/config.rs`.
  - Implementation Steps
    - [ ] `src/fs/backup.rs::create_snapshot`
      - [ ] Confirm usage of `open_dir_nofollow` + `openat` (present) and ensure parent dir is fsynced after payload and sidecar creation.
      - [ ] After writing payload and sidecar, call `fsync_parent_dir(target)`.
    - [ ] `src/fs/backup.rs::write_sidecar`
      - [ ] After writing JSON, call `File::sync_all()` on the file handle.
      - [ ] Ensure parent directory is fsynced via `fsync_parent_dir` (compute parent from `sc_path`).
    - [ ] Policy + facts
      - [ ] Add `require_backup_durability: bool` to `Policy` (default true).
      - [ ] Emit `backup_durable=true` in `apply.attempt` or per-action facts when durability path taken; `false` when policy opts out.
    - [ ] Tests
      - [ ] Extend existing snapshot tests to assert sidecar/payload exist and are flushed (mock or rely on success path; crash-sim left for later).
      - [ ] Add a test toggling policy to ensure `backup_durable` field flips.

- Backup sidecar integrity (payload hash)
  - Priority: P0
  - Why it’s good: Detects tampering/corruption; improves restore reliability.
  - Impact: High; additive with sidecar v2 (coexists with v1).
  - Risk: Performance overhead. Mitigation: SHA-256, policy `require_sidecar_integrity` default true in production.
  - Feasibility: Medium
  - Complexity: 3
  - Scope: Extend `src/fs/backup.rs::BackupSidecar` and sidecar write path with `payload_hash`; verify in `src/fs/restore.rs::restore_file()`; emit `sidecar_integrity_verified` fields; add policy knob in `src/policy/config.rs`.
  - Dependencies: SPEC for sidecar v2 schema.
  - Implementation Steps
    - [ ] Schema evolution
      - [ ] Add `payload_hash: Option<String>` to `BackupSidecar` (v2 marker: `schema: "backup_meta.v2"`).
      - [ ] Write v2 sidecar for new snapshots; continue reading v1.
    - [ ] Compute & store hash
      - [ ] In `create_snapshot()` when payload exists, compute SHA-256 of payload and set `payload_hash`.
    - [ ] Verify on restore
      - [ ] In `restore_file()`, if sidecar has `payload_hash` and payload present, recompute and compare; if mismatch and policy requires integrity, return error (map to `E_BACKUP_MISSING` or new `E_BACKUP_TAMPERED`).
      - [ ] Emit `sidecar_integrity_verified=true/false` in per-action facts.
    - [ ] Policy wiring
      - [ ] Add `require_sidecar_integrity: bool` (default true in production presets).
    - [ ] Tests
      - [ ] Unit: create snapshot, then modify payload; restore should fail when policy requires integrity and pass when disabled.
      - [ ] Integration: round-trip success with intact payload.

- Prune backups (retention)
  - Priority: P0
  - Why it’s good: Prevents unbounded disk usage; essential lifecycle function.
  - Impact: High; additive.
  - Risk: Wrong deletions. Mitigation: pairwise deletes, strong tests (never delete last), optional dry-run.
  - Feasibility: High
  - Complexity: 3
  - Scope: Add `Switchyard::prune_backups(&SafePath) -> Result<PruneResult>` in `src/api.rs`; core in `src/fs/backup.rs`; add `prune.result` emission in `src/logging/audit.rs`.
  - Dependencies: SPEC for retention knobs/invariants.
  - Implementation Steps
    - [ ] Policy knobs
      - [ ] Add `retention_count_limit: Option<usize>` and `retention_age_limit: Option<std::time::Duration>` to `Policy`.
    - [ ] Core pruning logic (`src/fs/backup.rs`)
      - [ ] Enumerate artifacts for a target/tag via existing discovery helpers.
      - [ ] Sort by timestamp descending; retain newest; apply count/age filters for deletion set.
      - [ ] Delete pairs atomically (payload + sidecar) and fsync parent directory.
    - [ ] Public API (`src/api.rs`)
      - [ ] Add `Switchyard::prune_backups(&self, target: &SafePath) -> Result<PruneResult, ApiError>`; call core logic.
    - [ ] Facts (`src/logging/audit.rs`)
      - [ ] Emit `prune.result` with `target_path`, `policy_used`, `pruned_count`, `retained_count`.
    - [ ] Tests
      - [ ] Unit: selection logic (count and age); never delete the newest backup.
      - [ ] Integration: create N backups, run prune, verify counts and that pairs are deleted together.

## P1 — Observability and Operability

- Summary error IDs in summaries
  - Priority: P1
  - Why it’s good: Better analytics/alerts: surface specific and general error classes.
  - Impact: Medium; additive schema field.
  - Risk: Consumer drift. Mitigation: CHANGELOG + docs.
  - Feasibility: High
  - Complexity: 3
  - Scope: Add `summary_error_ids` to `preflight.summary`, `apply.result`, `rollback.summary`; update `SPEC/audit_event.schema.json`; ensure preserved in `src/logging/redact.rs`.
  - Implementation Steps
    - [ ] Schema: add optional `summary_error_ids: [string]` to `SPEC/audit_event.schema.json`.
    - [ ] Emitters: in `logging/audit.rs::emit_summary_extra` callers, include chain in the `extra` field on failures.
    - [ ] Redaction: ensure `redact_event` preserves `summary_error_ids`.
    - [ ] Tests: extend JSON schema test to accept and validate the new field across summaries.
    - [ ] Apply summary wiring (`src/api/apply/mod.rs`)
      - [ ] At the final decision block where `fields` is constructed (search for `let mut fields = json!({` near the end), insert `summary_error_ids` when `decision == "failure"`.
      - [ ] Populate using a helper (see Error taxonomy task) or a local vec like `vec![id_str(ErrorId::E_POLICY)]` plus specific causes if known (e.g., `E_SMOKE`, `E_LOCKING`).
    - [ ] Preflight summary wiring (`src/api/preflight/mod.rs`)
      - [ ] Where `emit_summary_extra(&ctx, "preflight", decision, extra)` is called, add `summary_error_ids` into `extra` on failure, e.g., `["E_POLICY"]`.
    - [ ] Rollback summary (if/when a summary event exists)
      - [ ] Mirror the approach used for apply/preflight when emitting a rollback summary.
    - [ ] Grep checklist
      - [ ] `grep -R "emit_summary_extra\(" src` and ensure each summary path can include `summary_error_ids` on failures.
      - [ ] `grep -R "summary_error_ids" cargo/switchyard` to verify all additions are covered and tests reference the field.

- Error taxonomy chain + E_OWNERSHIP surfacing
  - Priority: P1
  - Why it’s good: Normalized routing (E_OWNERSHIP) while retaining specifics.
  - Impact: Medium; additive.
  - Risk: Helper plumbing churn; mitigate by central helpers.
  - Feasibility: Medium
  - Complexity: 3
  - Scope: Add helpers in `src/types/errors.rs` and wire into `src/api/errors.rs` + `apply` handlers.
  - Implementation Steps
    - [ ] Add `error_ids_chain(e: &Error) -> Vec<&'static str>` helpers mapping internal errors to public `ErrorId` strings.
    - [ ] In apply/preflight summaries, compute chain and populate `summary_error_ids` (ties to previous task).
    - [ ] Ensure ownership-related errors co-emit `E_OWNERSHIP` in the chain alongside specific root causes.
    - [ ] Tests: unit test chain mapping; integration test facts include both `E_OWNERSHIP` and specific codes.
    - [ ] Implement chain helper location
      - [ ] Prefer `src/api/errors.rs` for mapping from `ErrorId` to stable string via `id_str`, and build chains based on context (e.g., `E_POLICY` + specific stage error like `E_EXDEV`).
      - [ ] Optionally add `src/types/errors.rs` helper for internal `types::errors::Error` → `ErrorId` mapping.
    - [ ] Apply handler mapping (`src/api/apply/handlers.rs`)
      - [ ] On failure paths (search for `emit_apply_result(..., "failure", ...)`), add a `summary_error_ids` sibling when building summary at the end (or pass along a per-action chain if desired later).
    - [ ] Verification
      - [ ] `grep -R "ErrorId::" src/api/apply` and list all emit points; ensure chain logic accounts for `E_EXDEV`, `E_ATOMIC_SWAP`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`, etc.

- Performance telemetry aggregation
  - Priority: P1
  - Why it’s good: Visibility into IO hotspots; detect regressions.
  - Impact: Medium; additive.
  - Risk: Small overhead; low-cost timers mitigate.
  - Feasibility: High
  - Complexity: 2
  - Scope: Capture durations in `src/fs/{atomic,backup,meta}.rs`; aggregate into `perf` object in `src/api/apply/mod.rs` (and rollback if useful).
  - Implementation Steps
    - [ ] Add timing captures (Instant::now/elapsed) in `atomic_symlink_swap`, `create_snapshot`, `sha256_hex_of`.
    - [ ] Thread timings back to apply handlers via return tuples or a small struct.
    - [ ] In `apply/mod.rs`, aggregate into `perf` object on the final summary.
    - [ ] Tests: assert `perf` object exists and contains plausible non-zero values in commit mode.
    - [ ] Minimal invasive path
      - [ ] In `src/api/apply/handlers.rs`, wrap calls with `Instant::now()` → elapsed and accumulate: `hash_ms_total`, `backup_ms_total`, `swap_ms_total`.
      - [ ] Add a small `PerfAgg` local struct `{ hash_ms: u64, backup_ms: u64, swap_ms: u64 }` and bubble into `apply/mod.rs` via a per-action vector or summarize per action and aggregate later.
    - [ ] Summary emission (`src/api/apply/mod.rs`)
      - [ ] Extend `fields` with `"perf": { "hash_ms": X, "backup_ms": Y, "swap_ms": Z }` on success/failure.
    - [ ] Grep checklist
      - [ ] `grep -R "sha256_hex_of\(" src` and instrument call sites.
      - [ ] `grep -R "create_snapshot\(" src` and instrument around it.

- Lock fairness telemetry
  - Priority: P1
  - Why it’s good: Diagnoses contention/backoff patterns.
  - Impact: Low–Medium; additive.
  - Risk: Minimal.
  - Feasibility: High
  - Complexity: 1–2
  - Scope: Emit `lock_attempts` in `apply.attempt` and track in `src/adapters/lock/file.rs`.
  - Implementation Steps
    - [ ] In `adapters/lock/file.rs`, count attempts in the polling loop and return attempts via a side channel or track in `apply/mod.rs` by incrementing per retry.
    - [ ] Emit `lock_attempts` in the initial `apply.attempt` and/or at failure path.
    - [ ] Tests: contention test with two threads shows `lock_attempts > 1` for the second contender.
    - [ ] Minimal, no-trait-change option
      - [ ] Infer attempts as `attempts = 1 + (lock_wait_ms.unwrap_or(0) / LOCK_POLL_MS)` and include in facts. Document it as approximate.
    - [ ] Full option (trait change)
      - [ ] Extend `LockManager` to optionally report attempts (e.g., new method or return a struct). Implement in `FileLockManager`.
      - [ ] Wire attempts into apply attempt emission.

## P2 — Fidelity, Robustness, and Developer Experience

- Extended preservation tiers
  - Priority: P2
  - Why it’s good: Full metadata fidelity (owner, timestamps, xattrs, ACLs, caps) where supported.
  - Impact: High (fidelity); additive (sidecar v2 evolution).
  - Risk: Platform variance; mitigate with capability detection and graceful degradation.
  - Feasibility: Medium
  - Complexity: 3
  - Scope: Add `PreservationTier` and policy in `src/policy/config.rs`; extend backup/restore logic and sidecar schema; emit `preservation_applied` status.
  - Dependencies: SPEC sidecar v2 schema.
  - Implementation Steps
    - [ ] Policy: add `enum PreservationTier { Basic, Extended, Full }` and `policy.preservation_tier`.
    - [ ] Backup: capture fields based on tier and `detect_preservation_capabilities`.
    - [ ] Restore: apply fields best-effort; record per-field success in `preservation_applied`.
    - [ ] Tests: tiered round-trip scenarios; graceful degradation asserts.
    - [ ] Sidecar v2 fields (examples)
      - [ ] For Extended: include `uid`, `gid`, `mtime_sec`, `mtime_nsec` when supported.
      - [ ] For Full: include `xattrs` map and capability set (platform-gated), kept optional in sidecar.
    - [ ] Emission
      - [ ] Add `preservation_applied` object to per-action apply facts with booleans per field.

- Immutable bit detection reliability
  - Priority: P2
  - Why it’s good: Works in minimal container/base images without external tools.
  - Impact: Medium–High (preflight robustness); additive.
  - Risk: Platform-specific ioctl behavior; mitigate via `nix` feature flag and fallbacks.
  - Feasibility: Medium
  - Complexity: 3
  - Scope: Enhance `src/preflight/checks.rs::check_immutable()` to use `FS_IOC_GETFLAGS` ioctl, fallback to `lsattr`; add policy `allow_unreliable_immutable_check` in `src/policy/config.rs`; include detection method in facts.
  - Implementation Steps
    - [ ] Feature flag optional `nix` usage for ioctl; implement ioctl path guarded by `cfg(unix)`.
    - [ ] Fallback sequence: ioctl → lsattr → unknown.
    - [ ] Add `immutable_detection_method` and `immutable_check` fields in preflight facts.
    - [ ] Tests: simulate lack of ioctl support; ensure graceful fallback.
    - [ ] Ioctl details
      - [ ] Use `nix::ioctl_read!` or direct libc binding for `FS_IOC_GETFLAGS` on an open fd; check `FS_IMMUTABLE_FL` bit.
      - [ ] If ioctl fails (ENOTTY/EPERM), fallback to `lsattr` as today.

- Public FS atom restriction completion
  - Priority: P2
  - Why it’s good: Removes footguns from public surface.
  - Impact: Medium; phased restriction.
  - Risk: Hidden consumer reliance; mitigate with deprecation window and compile-fail tests.
  - Feasibility: Medium
  - Complexity: 2
  - Scope: Change re-exports to `pub(crate)` or remove in `src/fs/mod.rs` immediately; add trybuild compile-fail tests.
  - Implementation Steps
    - [ ] Change `pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};` to private or remove now; keep only high-level helpers public.
    - [ ] Add `trybuild` tests asserting crate users cannot import these atoms.
    - [ ] Update docs and Migration Guide with intended imports.
    - [ ] Grep checklist
      - [ ] `grep -R "pub use .*atomic_symlink_swap" src` and remove or privatize.
      - [ ] `grep -R "open_dir_nofollow\(" src` to ensure no example/docs expose it.

- Production preset adapter configuration docs
  - Priority: P2
  - Why it’s good: Reduces E_LOCKING/E_SMOKE surprises for new users.
  - Impact: Medium (DX); additive.
  - Risk: Docs drift; mitigate with doctests.
  - Feasibility: High
  - Complexity: 1
  - Scope: Add Rustdoc examples to `src/policy/config.rs::production_preset()` showing minimal `FileLockManager` and `SmokeTestRunner` setup; enable doctests in CI.
  - Implementation Steps
    - [ ] Add Rustdoc example showing building `Switchyard` with `FileLockManager` + `DefaultSmokeRunner`.
    - [ ] Ensure doctest compiles with `--features file-logging` minimal set.
    - [ ] Add CI job to run doc tests.
    - [ ] Example skeleton
      - [ ] Include `Switchyard::new(JsonlSink::default(), JsonlSink::default(), Policy::production_preset())
            .with_lock_manager(Box::new(FileLockManager::new(PathBuf::from("/tmp/lock"))))
            .with_smoke_runner(Box::new(DefaultSmokeRunner::default()));`

<!-- Deprecation program removed (no back-compat required). -->

---

### Cross-Cutting Implementation Notes

- Schema/SPEC sequencing: land SPEC and schema updates (sidecar v2, `summary_error_ids`, retention knobs) first or concurrently behind flags to minimize churn.
- Tests: add compile-fail tests for restricted atoms; expand schema-validation tests and end-to-end perf/locking tests.
- Migration docs: focus on current API usage; no deprecation timelines needed.
- CI: keep the hermetic test guard and changelog enforcement; consider adding cargo-public-api diff checks for breaking changes.

## CI / Tooling Tasks

<!-- cargo-public-api guard removed (back-compat enforcement not required). -->

- Add trybuild compile-fail tests for restricted FS atoms
  - [ ] Create `tests/trybuild/` with small crates that attempt to `use switchyard::fs::atomic_symlink_swap` etc.
  - [ ] Add a test harness `tests/trybuild.rs` that runs `trybuild::TestCases::new().compile_fail("tests/trybuild/*.rs");`.
  - [ ] Wire into CI test job.

- Add doc tests job
  - [ ] Ensure `cargo test --doc -p switchyard` runs in CI.
  - [ ] Keep examples minimal and feature-gated where needed (e.g., `file-logging`).

- Facts schema verification in CI
  - [ ] Ensure `cargo/switchyard/tests/audit_schema.rs` runs on CI (already covered by unit tests).
  - [ ] Optionally add a dedicated step that parses a small set of sample facts and validates against `SPEC/audit_event.schema.json` using `jsonschema`.

