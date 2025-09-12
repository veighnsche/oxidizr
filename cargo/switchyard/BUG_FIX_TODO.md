# BUG_FIX_TODO

Date: 2025-09-12
Scope: `cargo/switchyard/`
Related: `BUG.md`, `docs/CLEAN_CODE.md`

This plan proposes elegant, low-risk fixes aligned with our Clean Code principles to address verified issues in `BUG.md`. Each section covers architectural decisions, a concrete technical plan, feasibility/complexity, and tests/acceptance.

## Clean Code anchors (applied)

- __Honest error handling__: no `unwrap`, categorical errors, early returns.
- __Side-effect isolation__: move platform-specific FS checks behind a small interface.
- __Types encode invariants__: explicit types for mount capability and decisions.
- __Fail closed, loudly__: ambiguous environment → structured STOP with remediation.
- __Observability__: consistent fields and naming in facts; determinism preserved.

---

## 1) Filesystem mount verification (REQ-S2)

Problem: `ensure_mount_rw_exec()` parses `/proc/self/mounts`. If parsing/canonicalization fails, it returns `Ok(())` (silent pass). Gating/preflight also hard-code a secondary `/usr` check.

Goal: Verify the actual mount of the target path, fail closed when ambiguous, and remove ad hoc hard-coding.

### Architecture

Introduce a small mount inspection module with a clear contract and test seam:

- New module: `src/fs/mount.rs`.
- Type: `MountFlags { read_only: bool, no_exec: bool }`.
- Trait: `MountInspector` with `fn flags_for(&self, path: &Path) -> Result<MountFlags, MountError>`.
- Prod impl: `ProcStatfsInspector` using `rustix` `statfs`/`statvfs` on the target (or its parent when needed), with `/proc/self/mounts` as a fallback.
- Test impl: `MockMountInspector` to simulate edge cases without privileged mounts.

API used by policy checks:

- `pub fn ensure_rw_exec(inspector: &impl MountInspector, path: &Path) -> Result<(), PolicyError>`
  - Fail when `read_only` or `no_exec`.
  - If inspection fails (no definitive answer), return `PolicyError::FsAmbiguous` (STOP).

Integration:

- Replace calls to `crate::preflight::ensure_mount_rw_exec(...)` with `policy::checks::ensure_mount_rw_exec(...)` that delegates to the inspector.
- Provide a crate-internal singleton `INSPECTOR: ProcStatfsInspector` or inject via `Switchyard` if we want full explicit dependencies (see `docs/CLEAN_CODE.md` §2, §6). Minimal-change path: internal module-level prod inspector + tests using the trait.
- Remove or policy-gate the `/usr` hard-coded check. If needed, add `Policy::extra_mount_checks: Vec<PathBuf>` with default empty, and loop over those paths using the same inspector.

### Technical plan

- Add `src/fs/mount.rs`:
  - Define `MountFlags`, `MountError`, `MountInspector` trait.
  - Implement `ProcStatfsInspector`:
    - Prefer `rustix::fs::statfs`/`statvfs` on `path` or its parent.
    - Interpret `f_flags` for `ST_RDONLY` and `MNT_NOEXEC` (platform-gated; unit-test on Linux).
    - Fallback: parse `/proc/self/mounts` selecting longest prefix, same as today; map to flags.
    - On any ambiguity (I/O error, unknown platform or missing signals), return `MountError::Unknown`.
- Update `src/preflight.rs`:
  - Replace `ensure_mount_rw_exec` body to call the inspector. Rename to `ensure_mount_rw_exec` (keep API) but delegate to `fs::mount::ensure_rw_exec(&INSPECTOR, path)`.
- Update call sites:
  - `src/api/preflight.rs` and `src/policy/gating.rs`: drop explicit `/usr` check or guard it behind `Policy::extra_mount_checks`.
- Add `Policy` extension (optional but recommended):
  - `pub extra_mount_checks: Vec<PathBuf>` default empty. Iterate in preflight/gating if present.

### Feasibility & complexity

- Complexity: Medium. New module + small refactor. No behavior changes for healthy systems; fixes fail-closed on ambiguity.
- Dependencies: `rustix` already in use.
- Risks: Platform flags mapping. Mitigate with feature gates and fallbacks. Keep current parser as fallback.

### Tests & acceptance

- Unit tests for inspector using `MockMountInspector` to simulate `ro` and `noexec`.
- Integration tests: ensure STOP when inspector returns `Unknown`.
- Regression: same pass/fail semantics on normal systems; no silent pass on ambiguous.

---

## 2) EXDEV degraded mode telemetry (REQ-F2 alignment)

Problem: On EXDEV failure with `allow_degraded_fs=false`, we emit `E_EXDEV` but no explicit fields like `degraded=false` or reason marker.

Goal: Consistent observability regardless of success/failure.

### Architecture

- Keep `ErrorId::E_EXDEV`. Augment `apply.result` with stable fields:
  - `degraded: false` when EXDEV fallback not allowed.
  - `degraded_reason: "exdev_fallback"` on both success (fallback used) and failure (fallback disallowed).
  - Optionally, add `error_detail: "exdev_fallback_failed"` to disambiguate in analytics.

### Technical plan

- Edit `src/api/apply/handlers.rs::handle_ensure_symlink` failure branch:
  - When mapping EXDEV, add the fields above into `extra` before emitting `apply.result`.
- Add a small helper to normalize fields population for success/failure paths.

### Feasibility & complexity

- Complexity: Low.
- Risk: Very low; schema already allows extra fields.

### Tests & acceptance

- Add/adjust tests to assert presence of `degraded` and `degraded_reason` in both branches.

---

## 3) Locking defaults (REQ-L4 safer-by-default)

Problem: `Policy::default()` sets `allow_unlocked_commit=true`. Commit without a `LockManager` proceeds with a WARN.

Goal: Safer default without breaking production preset.

### Architecture

- Flip default: `allow_unlocked_commit=false` in `Policy::default()`.
- Keep `production_preset()` unchanged (it already requires locking).
- Document change and how to opt out for dev ergonomics.

### Technical plan

- Edit `src/policy/config.rs` default impl: set `allow_unlocked_commit: false`.
- Update any tests that rely on current default by explicitly setting `allow_unlocked_commit=true` or using `production_preset()`.
- Update README/SPEC to reflect default.

### Feasibility & complexity

- Complexity: Low.
- Risk: Some unit tests may need flag adjustments.

### Tests & acceptance

- Tests asserting WARN on missing lock in Commit should now expect early STOP unless flag set.

---

## 4) Rollback invertibility for Restore (REQ-R2/R3)

Problem: `RestoreFromBackup` is non-invertible; inverse plan omits it.

Goal: Make a restore invertible with minimal API churn.

### Architecture

- Phase 1 (Docs): Explicitly document non-invertibility in SPEC and user docs.
- Phase 2 (Snapshot): Before performing a restore, capture a "pre-restore snapshot" using the same backup/sidecar mechanism used by `replace_file_with_symlink()`.
  - Implementation choice: capture snapshot under the same `backup_tag` as the target so it becomes the latest backup. This makes inversion a normal `RestoreFromBackup`.
  - Policy: add `capture_restore_snapshot: bool` (default true) to gate the behavior if needed.
  - Idempotence: snapshot-then-restore is safe to repeat; sidecar/timestamps are already designed for retries.

### Technical plan

- Extract backup+sidecar creation from `src/fs/swap.rs` into a shared helper `fs::backup::create_snapshot(target, backup_tag)` that:
  - If target is a file: copies bytes and mode, writes sidecar `{prior_kind:"file", mode, prior_dest:None}`.
  - If target is a symlink: creates a symlink backup pointing to current dest, writes sidecar `{prior_kind:"symlink", prior_dest:Some(dest)}`.
  - If target absent: creates tombstone backup and sidecar `{prior_kind:"none"}`.
- Add new function in `src/fs/backup.rs` (or `snapshot.rs`) and refactor `replace_file_with_symlink()` to reuse it, eliminating duplicate logic.
- Update `src/api/apply/handlers.rs::handle_restore`:
  - When `!dry` and `policy.capture_restore_snapshot`, call `create_snapshot(target, &policy.backup_tag)` before `restore_file()`.
- Update `src/api/rollback.rs::inverse`:
  - Now invert `RestoreFromBackup` → `RestoreFromBackup` (relying on the most recent pre-restore snapshot).
  - If policy disables snapshot, keep current behavior (skip inversion) and set an explanatory note in the plan or logs.

### Feasibility & complexity

- Complexity: Medium. Code already has nearly all primitives; main effort is factorization and plumbing.
- Risk: Low/medium. Touches symlink/file backup logic—covered by existing round-trip tests; add restore-specific tests.

### Tests & acceptance

- New tests:
  - `restore_invertible_roundtrip`: apply a restore, then compute `inverse()` and apply; expect exact prior topology.
  - Dry-run behavior: snapshot suppressed; inversion behavior documented.
  - Policy toggle off: inverse skips with a note.

---

## 5) Facts schema and SPEC updates

- Add fields to `apply.result` for EXDEV failure (`degraded=false`, `degraded_reason="exdev_fallback"`, optional `error_detail`).
- Ensure preflight row fields like `preservation_supported` are documented in v1 schema.
- Document restore invertibility policy and snapshotting behavior.

### Technical plan

- Update `SPEC/SPEC.md` and traceability.
- Update `DOCS/GOLDEN_FIXTURES.md` if fixtures include new fields.

### Feasibility & complexity

- Complexity: Low.

### Tests & acceptance

- Golden fixtures update to include new fields where applicable.

---

## Work breakdown (checklist)

- [ ] Mount inspector module (`src/fs/mount.rs`), trait, and prod impl using `rustix`.
- [ ] Refactor `ensure_mount_rw_exec` to use inspector; policy-gate extra roots.
- [ ] Remove/convert `/usr` hard-coded checks to policy-driven list.
- [ ] Adjust `apply/handlers.rs` to emit EXDEV failure fields.
- [ ] Flip `Policy::default().allow_unlocked_commit=false`; fix affected tests.
- [ ] Factor out snapshot helper and use from both swap and restore paths.
- [ ] Add `Policy::capture_restore_snapshot` (default true) and wire in `handle_restore`.
- [ ] Update `rollback::inverse` to invert restore when snapshots enabled.
- [ ] SPEC/docs updates for fields and invertibility.
- [ ] Tests: unit (inspector, snapshot), integration (invertible restore), telemetry assertions.

## Acceptance criteria

- REQ-S2: Fails closed when mount info is ambiguous; checks actual target mount; no reliance on hard-coded `/usr`.
- REQ-F2: EXDEV failure facts include `degraded=false` and `degraded_reason`.
- REQ-L4: Default policy prevents unlocked commit unless explicitly allowed.
- REQ-R2/R3: Restores are invertible when `capture_restore_snapshot=true`; documented otherwise.
- Observability/Determinism: Schema envelope retained; new fields documented; TS_ZERO and stable IDs unaffected.

---

# WORK ORDER: Codify Error IDs and Exit Codes (Switchyard)

Context

- Authoritative mapping lives in `src/api/errors.rs` and `SPEC/error_codes.toml`.
- Catalog for developers and reviewers: `ER_CAT.md` and `ER_CAT.json`.
- Current status is Silver-tier for exit codes; several sites already emit `error_id`/`exit_code` in `apply` and per-action handlers.

Goals

- Ensure every covered failure pathway emits the correct `error_id` and `exit_code`, deterministically.
- Keep mapping centralized in `src/api/errors.rs` and documented in `ER_CAT.md`/`SPEC`.
- Provide tests and goldens to prevent regressions.

Deliverables

- Updated code paths emitting `error_id`/`exit_code` consistently for Silver scope.
- `ER_CAT.json` annotated with `maps_to_error_id` and `exit_code` (or null for warnings).
- Unit tests and golden fixtures asserting presence and correctness of `error_id` and `exit_code`.
- Documentation updates where applicable (README, SPEC references).

Work Items (Silver-tier)

1) Apply-stage locking and attempts

- [ ] Verify `apply` emits `E_LOCKING` (30) on lock acquisition error with `lock_wait_ms` in `apply.attempt` (already implemented in `src/api/apply.rs`).
- [ ] Verify Commit without lock (when required) maps to `E_LOCKING` (30) and fails.
- [ ] Ensure dry-run and allowed-unlocked paths only warn (no exit code).

2) Apply-stage policy gating → E_POLICY (10)

- [ ] Confirm gating failures in `apply.rs` map to `E_POLICY` with `exit_code=10` in `apply.result` (already implemented).
- [ ] Add/verify backpressure so these failures appear in final decision and report.

3) Per-action failures in handlers

- [ ] Ensure symlink swap failures map: EXDEV → `E_EXDEV` (50); other IO → `E_ATOMIC_SWAP` (40). File: `src/api/apply/handlers.rs`.
- [ ] Ensure restore failures map: NotFound → `E_BACKUP_MISSING` (60); other IO → `E_RESTORE_FAILED` (70). File: `src/api/apply/handlers.rs`.
- [ ] Confirm emitted per-action `apply.result` includes `error_id` and `exit_code` (already implemented), plus provenance and hashes.

4) Smoke tests summary mapping

- [ ] When post-apply smoke fails or required runner is missing, ensure final `apply.result` summary includes `error_id=E_SMOKE` (80) and `exit_code` (implemented in `src/api/apply.rs`).
- [ ] Consider adding the best culprit `error_id` to final summary for non-smoke failures (optional; keep determinism).

5) Preflight stage emissions

- [ ] Keep per-row preflight as-is but add `error_id=E_POLICY` and `exit_code=10` to the preflight summary fact when `stops` is non-empty. File: `src/api/preflight.rs` (emit via `emit_summary_extra`).
- [ ] Do not emit exit codes for warning-only items (e.g., untrusted source allowed by policy); ensure they remain warnings.

6) Catalog JSON enrichment

- [ ] Update `ER_CAT.json` entries to include `maps_to_error_id` and `exit_code` fields where applicable; leave `exit_code=null` for warnings.
- [ ] Add a lightweight JSON schema comment or validator in tests to ensure well-formedness.

7) Tests and goldens

- [ ] Add/extend tests under `cargo/switchyard/tests/` to assert presence and correctness of `error_id` and `exit_code`:
  - locking timeout (attempt fact has `E_LOCKING`/30 and `lock_wait_ms`)
  - policy gating failure maps to `E_POLICY`/10
  - atomic swap failure (`E_ATOMIC_SWAP`/40) and EXDEV (`E_EXDEV`/50)
  - restore NotFound (`E_BACKUP_MISSING`/60) and generic restore failure (`E_RESTORE_FAILED`/70)
  - smoke fail or missing runner maps to `E_SMOKE`/80 in summary
- [ ] Where applicable, write canon to `GOLDEN_OUT_DIR` and hook into `test_ci_runner.py` coverage artifacts.

8) Documentation

- [ ] Update `ER_CAT.md` if new rows or clarifications are added.
- [ ] Ensure `README.md` “Locking and Exit Codes (Silver Tier)” stays accurate.
- [ ] Cross-check `SPEC/error_codes.toml` remains aligned with `src/api/errors.rs` mapping.

Nice-to-have / Future (Gold/Platinum)

- [ ] Introduce direct `E_OWNERSHIP` (20) emission for strict ownership failures; today these map to `E_POLICY` by design at Silver.
- [ ] Expand coverage to all representative failure sites across plan/preflight/apply/rollback; gate via goldens in CI.
- [ ] Add per-row preflight `error_id` annotations (not just summary) once taxonomy stabilizes.
- [ ] Emit `error_id` at final summary for non-smoke failures using a deterministic selection policy (e.g., first failure; doc this rule).
- [ ] Publish versioned table in SPEC and lock changes via ADR once Platinum is targeted.

Acceptance Criteria

- All Silver-tier failure sites deterministically emit the correct `error_id` and `exit_code`.
- Tests cover each mapped `ErrorId`, and goldens are stable under redaction.
- `ER_CAT.md` and `ER_CAT.json` are consistent, machine-checkable, and referenced from README.

References

- Mapping: `src/api/errors.rs`, `SPEC/error_codes.toml`
- Emissions: `src/api/apply.rs`, `src/api/apply/handlers.rs`, `src/api/preflight.rs`
- Catalogs: `ER_CAT.md`, `ER_CAT.json`
- Plans/ADRs: `DOCS/EXIT_CODES_TIERS.md`, `PLAN/30-errors-and-exit-codes.md`, `PLAN/adr/ADR-0014-exit-codes-deferral.md`
