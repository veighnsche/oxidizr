# Switchyard TODO_2 — Comprehensive Backlog (2025-09-11)

Derived from a full sweep of:

- `cargo/switchyard/SPEC/SPEC.md`, `SPEC/requirements.yaml`, `SPEC/features/*`, `SPEC/audit_event.schema.json`
- Planning docs under `cargo/switchyard/PLAN/*` (including `12-api-module.md`, `45-preflight.md`, `50-locking-concurrency.md`, `60-rollback-exdev.md`, `90-implementation-tiers.md`)
- Current implementation under `cargo/switchyard/src/*` and tests in `cargo/switchyard/tests/*`
- Existing backlog in `TODO.md` and `SPEC_CHECKLIST.md`

Legend:

- [x] Done
- [~] Partial / Silver-tier
- [ ] TODO

---

## 0) State Snapshot (summary)

- API split complete (`src/api/{plan,preflight,apply,rollback,fs_meta,errors}.rs`) with centralized audit emission (`src/api/audit.rs`).
- Preflight emits per-action rows with `policy_ok`, `provenance` (when oracle is present), and `preservation{}` + `preservation_supported`.
- Apply enforces fail-closed by default unless `Policy.override_preflight=true`; emits Silver-tier `error_id/exit_code` at covered failure sites.
- Locking adapter (`FileLockManager`) implemented; failure path emits `E_LOCKING` with `lock_wait_ms` (apply.attempt).
- Hashing and attestation scaffolding present; redaction policy implemented for DryRun vs Commit parity.
- Golden canon fixtures and schema validation integrated in tests; non-blocking CI traceability job added; golden gate staging remains.
- Rescue profile: minimal PATH-based verification implemented with `Policy.require_rescue` gating in preflight/apply.

---

## 1) Public API & Types

- [x] Split `src/api.rs` into submodules; keep façade delegators
- [~] `ApiError` taxonomy scaffolded; `ErrorId` → `exit_code` mapping Silver subset
- [ ] Crate-level docs + module docs (fs, preflight, api, types, adapters, logging, policy)
- [ ] Policy builder ergonomics and full docs (defaults, semantics, examples)

Action items

- [ ] Add crate-level documentation with usage examples and safety model
- [ ] Add module-level rustdoc for `api`, `fs`, `preflight`, `types`, `adapters`, `logging`, `policy`
- [ ] Introduce `Policy::builder()` with sane defaults and examples in docs

---

## 2) Preflight (SPEC §4, §2.3)

- [x] Emits per-action rows with `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`
- [~] Emits `provenance{uid,gid,pkg}` when an `OwnershipOracle` is configured
- [x] Emits `preservation{}` and `preservation_supported`; policy STOP wired when `Policy.require_preservation=true`
- [ ] Optional output: materialize `PreflightReport` to YAML conforming to `SPEC/preflight.yaml` for fixtures/artifacts

Action items

- [x] Enforce `Policy.require_preservation` STOP path end-to-end (already recorded in rows)
- [ ] Add unit tests for negative cases per gate (ro/noexec/immutable/ownership/roots/forbid_paths)
- [ ] Provide `preflight::to_yaml(&PreflightReport)` helper (pure) for producing spec-aligned YAML when needed

---

## 3) Apply & Rollback (SPEC §2.1, §2.2)

- [x] TOCTOU-safe sequence for symlink replacement (`fs/atomic.rs` + `fs/symlink.rs`)
- [~] Error mapping for per-action failures: `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`
- [~] Reverse-order rollback on failure; partial restoration facts emitted; idempotency not yet property-tested
- [ ] Broaden mapping and ensure every failure site emits `error_id/exit_code` consistently (summary and per-action)

Action items

- [ ] Property test: `IdempotentRollback` (apply failure → rollback twice → same FS state)
- [ ] Property test: `AtomicReplace` (no visible broken/missing path within same FS) — assert via simulated race and metadata checks
- [ ] Ensure summary `apply.result` carries `error_id/exit_code` for all failure modes, not only smoke

---

## 4) Cross-Filesystem & Degraded Mode (SPEC §2.10)

- [~] EXDEV degraded path for symlink replacement implemented (non-atomic fallback); facts include `degraded=true`
- [ ] Document/align Gherkin around symlink semantics (copy+fsync+rename does not apply to symlinks; we unlink+symlink)
- [ ] Acceptance: test matrix (ext4/xfs/btrfs/tmpfs) remains external to this crate

Action items

- [ ] Add explicit note in PLAN/SPEC clarifying degraded symlink swap semantics vs regular file copy path; update feature text
- [ ] Add unit test asserting `degraded=true` is emitted when EXDEV is simulated and policy allows

---

## 5) Locking & Concurrency (SPEC §2.5)

- [~] `FileLockManager` with bounded wait and tests; WARN fact when no lock manager
- [x] On timeout, we emit `apply.attempt` failure with `E_LOCKING`; `lock_wait_ms` captured on the error path

Action items

- [x] Capture and emit `lock_wait_ms` on the timeout failure path (SPEC_CHECKLIST gap) in `src/api/apply.rs`
- [ ] Add golden fragment asserting `E_LOCKING` with `exit_code=30` and presence of `lock_wait_ms` when available

---

## 6) Determinism & Redaction (SPEC §2.7)

- [x] UUIDv5 `plan_id`/`action_id` (see `src/types/ids.rs`)
- [x] Redaction policy ensuring DryRun == Commit parity for canonicalized events (tests in `tests/sprint_acceptance-0001.rs`)
- [x] Extend redaction/masking coverage with unit tests for known volatile fields

Action items

- [ ] Add unit test that asserts canon of plan & preflight events are byte-identical DryRun vs Commit (we currently test apply.result parity)
- [ ] Consider masking additional provenance fields beyond `helper` (policy-driven masking list)

---

## 7) Observability & Audit (SPEC §2.4, §5, §13)

- [~] Minimal Facts v1 emitted at all stages; schema validation in tests; per-action hashing on symlink swap
- [~] Attestation bundle created and signed when an `Attestor` is provided; redacted fields masked for determinism
- [~] Provenance completeness (origin/helper/uid/gid/pkg/env_sanitized) across all relevant facts
- [ ] Secret masking policy enforcement across all sinks (beyond current helper masking)

Action items

- [x] Extend apply-level provenance to include `uid/gid/pkg` when an oracle is present.
- [ ] Populate `origin` and stable `helper` via adapters where available; ensure `env_sanitized=true` across stages
- [ ] Add policy-driven secret-mask list; ensure `redact_event()` removes/masks configured keys consistently
- [ ] Add schema validation tests for added provenance fields (allowing omission when N/A)

---

## 8) Smoke Tests (SPEC §11)

- [~] `SmokeTestRunner` trait implemented; default runner is a no-op; apply integrates auto-rollback on failure
- [x] Provide a minimal deterministic default smoke suite (deterministic checks; no clock/locale dependence) — default runner validates that targets resolve to sources for `EnsureSymlink` actions

Action items

- [x] Implement `DefaultSmokeRunner` with deterministic symlink resolution checks (no external commands)
- [ ] Add tests asserting runner outputs are redacted/sanitized (if/when external commands are added)

---

## 9) Rescue Profile (SPEC §2.6)

- [~] Implement `rescue::verify_rescue_tools()` for minimal PATH-based checks (BusyBox or ≥6/10 GNU tools) with env override; integrate policy `require_rescue` gating in preflight/apply

Action items

- [x] Implement minimal PATH-based rescue checks and STOP when `require_rescue=true` and unmet
- [~] Unit tests: rescue preflight gating covered using PATH injection and `SWITCHYARD_FORCE_RESCUE_OK`; adapter-based mock `PathResolver` remains optional follow-up; extend facts with notes/toolset details

---

## 10) Error Model & Exit Codes (SPEC §6)

- [~] Silver mapping implemented for core failure sites; helper `exit_code_for()` and `id_str()` present
- [ ] Ensure every failure site sets `error_id/exit_code` in emitted facts (per-action and summary)

Action items

- [ ] Audit `src/api/apply.rs` branches and add missing `error_id/exit_code` insertions
- [ ] Add tests verifying presence for `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_POLICY`, `E_SMOKE` (note: `E_LOCKING` covered by `tests/locking_timeout.rs`)

---

## 11) Testing & Goldens (SPEC §8, §12)

- [~] Golden canon generation present in tests; schema validation present
- [ ] CI gate (zero-SKIP + golden diff) not wired for this crate
- [ ] Property tests (AtomicReplace, IdempotentRollback) pending

Action items

- [ ] Add property tests
- [ ] Wire CI jobs (or document upstream CI integration) for schema validate + golden diff + zero-SKIP gate
- [x] Add `SPEC/tools/traceability.py` invocation in CI to produce coverage artifact (non-blocking job added)
- [x] Add golden-style fragment for lock timeout asserting `E_LOCKING` + `exit_code=30` (see `tests/locking_timeout.rs`; `lock_wait_ms` omitted from canon due to redaction)

---

## 12) Documentation & Doc-Sync

- [x] SPEC_UPDATE: document new policy flags (`override_preflight`, `require_preservation`) and clarify degraded symlink behavior
- [x] ADR: taxonomy boundaries (Silver subset coverage; deferrals)
- [ ] Update planning docs after changes (PLAN impl notes + discrepancies)

Action items

- [ ] SPEC Updates: `SPEC/SPEC_UPDATE_0001.md` (Accepted); `SPEC/SPEC_UPDATE_0002.md` (Proposed)
- [ ] ADRs: `PLAN/adr/ADR-0013-backup-tagging.md` (Accepted); `PLAN/adr/ADR-0015-exit-codes-silver-and-ci-gates.md` (Proposed)

---

## 13) Small Cleanups / Nits

- [ ] Ensure consistent visibility (`pub(crate)`) across API submodules where appropriate
- [ ] Clear remaining lints and add `#[deny(missing_docs)]` once docs added
- [ ] Harmonize error construction in adapters to prefer typed `ErrorKind` with stable messages

---

## 14) Traceability & BDD Alignment

- [ ] Ensure each implemented REQ has at least one test and/or golden fragment; update `SPEC/traceability.md`
- [ ] Tag new/updated tests with `covers: [REQ-…]` and ensure features are in sync (see section below)

---

# Gherkin Features — Alignment & Proposed Updates

Location: `cargo/switchyard/SPEC/features/`

Summary of needed updates based on current code:

1) `atomic_swap.feature`

- EXDEV scenario currently states "copy+sync+rename into place atomically". For symlink targets we cannot `rename` across filesystems; implementation performs unlink + `symlinkat` (non-atomic) and emits `degraded=true`.
  - Proposed edit: clarify degraded semantics for symlink replacement (best-effort, non-atomic; facts include `degraded=true` and policy gates).
- Automatic rollback and smoke-triggered rollback are implemented; scenarios remain valid.

2) `observability.feature`

- Dry-run vs real-run identity: we already ensure parity for `apply.result` canon; plan/preflight parity also holds (both emitted with `TS_ZERO`), but add note to compare after redaction.
  - Proposed: keep scenario; add acceptance note: comparison occurs on redacted events.
- Provenance completeness (origin, helper, uid, gid, pkg, env_sanitized): code partially emits `uid/gid/pkg` in preflight (when oracle present) and minimal provenance in apply (`helper`/`env_sanitized`).
  - Proposed: scope scenario to "present where available" for now (Silver), or mark as `@xfail` until full provenance is implemented.
- Secret masking across all sinks: redaction masks `helper` and attestation fields; extend masking policy — keep scenario, mark partial until policy list is in place.

3) `locking_rescue.feature`

- Bounded locking timeout emits `E_LOCKING` on `apply.attempt` failure path but currently does not capture `lock_wait_ms` on error.
  - Proposed: retain scenario; add a note or `@xfail` until `lock_wait_ms` is captured on failures.
- Rescue profile verification is not yet implemented (`src/rescue.rs` placeholder).
  - Proposed: mark the rescue scenario as `@xfail` or add a "planned" note until `require_rescue` is wired.

4) `api_toctou.feature`

- Matches implementation: mutating APIs require `SafePath`, and FS ops follow the TOCTOU-safe sequence. Keep as-is.

5) `conservatism_ci.feature`

- Dry-run default and fail-closed behavior are implemented. CI golden/zero-SKIP gates are not enforced in this crate.
  - Proposed: keep scenarios; mark CI-gate scenario as non-executable/pending if not run in this repo CI.

Concrete edits to apply in `.feature` files (suggested):

- Add `@xfail` tag (or a comment) to:
  - `locking_rescue.feature` → Rescue scenario (until implemented)
  - `locking_rescue.feature` → Lock timeout scenario’s strict `lock_wait_ms` assertion (until captured on error)
  - `observability.feature` → Provenance completeness scenario (until origin/helper/pkg fully emitted)
- Update `atomic_swap.feature` EXDEV step text to:
  - "Then the operation uses a best‑effort degraded fallback for symlink replacement and facts record `degraded=true` when policy `allow_degraded_fs` is enabled"

If you’d like, I can open PR-ready patches to the `.feature` files with the above precise changes.

---

## Cross-References (where to implement)

- API orchestration: `src/api/{plan,preflight,apply,rollback}.rs`
- Preconditions/policy: `src/preflight.rs`, `src/policy/config.rs`
- Filesystem atomic ops: `src/fs/{atomic,symlink}.rs`
- Types & models: `src/types/*`
- Logging/Audit: `src/api/audit.rs`, `src/logging/*`, `SPEC/audit_event.schema.json`
- Adapters: `src/adapters/*`
- Rescue: `src/rescue.rs`
- SPEC/PLAN docs: `SPEC/*`, `PLAN/*`

---

## Delta — 2025-09-11 Evening Update

Completed in this iteration

- Gherkin alignment in `SPEC/features/` and `steps-contract.yaml`:
  - Clarified EXDEV degraded symlink behavior; added E_EXDEV failure when disallowed.
  - Locking scenario now asserts `error_id=E_LOCKING` + `exit_code=30` and makes `lock_wait_ms` conditional.
  - Added apply.result parity after redaction; marked provenance completeness and rescue as `@xfail` until implemented.
  - Noted WARN on fsync > 50ms; marked CI gate scenario `@xfail` (enforced upstream).
- Preflight/apply rescue gating hooks:
  - `src/api/preflight.rs` now checks `policy.require_rescue` and stops when `verify_rescue_tools()` reports unavailable.
  - `src/api/apply.rs` also refuses (fail-closed) when `require_rescue=true` and rescue is unavailable.
- Locking timeout metrics on failure path:
  - `src/api/apply.rs` now records `lock_wait_ms` even on the timeout failure branch and emits it in `apply.attempt`.
- Apply provenance enrichment:
  - Centralized `ensure_provenance()` now applied to all emitted facts.
  - Added unit tests for redaction masking/field removal.
  - `src/api/apply.rs` per-action results now include `provenance.{uid,gid,pkg}` when an `OwnershipOracle` is present.

Still remaining (new/updated explicit tasks)

- Implement real rescue verification in `src/rescue.rs::verify_rescue_tools()`:
  - Probe rescue symlink set and verify at least one fallback toolset (GNU or BusyBox) via a `PathResolver`.
  - Unit tests with a mock resolver; extend preflight facts with notes and summary.
- Provenance completeness to Gold tier:
  - Populate `origin` and `helper` where adapters provide them; ensure `env_sanitized=true` is set; broaden presence across relevant facts (not only apply per-action).
  - Add schema validation tests for presence/omission rules and extend redaction policy/masks as needed.
- Error/exit code coverage audit:
  - Ensure every failure site (summary and per-action) sets `error_id/exit_code`; add tests for `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_POLICY`, `E_SMOKE` where missing.
- EXDEV acceptance unit coverage:
  - Add unit tests that simulate EXDEV to assert `degraded=true` when allowed and E_EXDEV failure when disallowed.
- Deterministic smoke runner subset:
  - Implement a minimal default runner (deterministic commands/args; no env dependencies) under a feature flag; add tests and ensure redaction of outputs.
- CI traceability job:
  - Add a CI step to run `SPEC/tools/traceability.py` and publish the artifact (non-blocking initially).
- SPEC/ADR updates:
  - SPEC_UPDATE documenting `override_preflight`, `require_preservation`, degraded symlink semantics, and rescue gating; ADR addendum for Silver taxonomy boundaries.
- Property tests:
  - `AtomicReplace` (no broken/missing visibility during swap) and `IdempotentRollback` (reapplying rollback yields same state).
