# Switchyard Refactor TODO (Order of Execution)

High-level, ordered plan to land refactors with minimal risk and conflict. Each item links to the detailed playbook in `./zrefactor/`. Pre-1.0, bundle large, fast-moving changes as needed; interleave additive feature work where noted. Run acceptance greps from each linked doc as you land items, and follow the Rulebook conventions.

## 0) Baseline guardrails and checks

- [ ] Add/verify CI grep guardrails and run local checks
  - Docs: `./zrefactor/responsibility_cohesion_report.md`, `./zrefactor/backwards_compat_removals.md`
  - Goal: Establish “tripwires” early (no public fs atoms, no #[path], no adapters::lock_file, no direct FactsEmitter::emit in API, etc.)
  - Rulebook applies to all refactors: use standardized markers and sweep cadence.
    - Doc: `./zrefactor/refactor_rulebook.INSTRUCTIONS.md`
  - SPEC and Clean Code adherence gates (reject changes that drift from guarantees or cleanliness):
    - Docs: `./SPEC/SPEC.md`, `../../docs/CLEAN_CODE.md`
  - Add drift-check script to CI using the acceptance greps from the linked docs.
  - Update the code review checklist with: “SPEC/SPEC.md referenced”, “CLEAN_CODE adhered”, “Rulebook markers added”, “Acceptance greps pasted as evidence”.
  - Rulebook quick rules (summary):
    - Use standardized markers in code: `remove this file`, `move this file`, `replace this file`, `deprecated shim`, and `BEGIN/END REMOVE BLOCK`.
    - Batch cadence (pre-1.0): A) Markers-only, B) Implement refactor, C) Sweep removals.
    - Conventional commits; mark breaking changes with a short migration note in the body.

## 1) Idiomatic module/layout cleanup

- [ ] Make `src/api` directory module fully idiomatic; remove any lingering `#[path]`
- [ ] Remove legacy adapters shim (`adapters::lock_file`), ensure imports use `adapters::lock::file::*`
- [ ] Tighten fs atoms visibility; no re-exports at `src/fs/mod.rs`
  - Docs: `./zrefactor/idiomatic_todo.INSTRUCTIONS.md`

## 2) Policy-owned gating (single evaluator)

- [ ] Implement typed `policy::gating::evaluate_action(..)` and shared helpers
- [ ] Preflight: call evaluator per action; delete inlined checks and hard-coded mount paths (use `policy.extra_mount_checks`)
- [ ] Apply: call evaluator before mutation; enforce `override_preflight`
- [ ] Define grouped policy types and profiles
  - Introduce enums/groups (e.g., `RiskLevel`, `ExdevPolicy`, `LockingPolicy`, `SmokePolicy`, `PreservationPolicy`) under `policy::types`.
  - Add curated `profiles` for common presets (e.g., production) and a `Policy::builder()` if needed.
- Docs: `./zrefactor/policy_refactor.INSTRUCTIONS.md`, `./zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`

## 3) Logging/Audit facade migration

- [ ] Introduce `StageLogger`/`EventBuilder` facade under `src/logging/`
- [ ] Migrate API call sites (plan, preflight, apply, rollback, prune) to the facade
- [ ] Keep field parity with current emissions (lock_backend/attempts, perf, attestation, error_id/exit_code, restore_ready, prune fields)
- Sub-migrations:
  - Migrate `plan` to builder (replace `emit_plan_fact` usage with builder calls).
  - Extract attestation bundle construction from `api/apply/mod.rs` into a helper under `adapters::attest` (typed struct), then attach via builder.
  - Add CI guardrails: forbid `audit::emit_` and direct `FactsEmitter::emit` outside `src/logging/`.
- Docs: `./zrefactor/logging_audit_refactor.INSTRUCTIONS.md`

## 4) API DX/DW alignment

- [ ] Keep `Switchyard::new(facts, audit, policy)` and current fluent `.with_*` methods
- [ ] Add `ApiBuilder` that mirrors `.with_*` and delegates to avoid duplication
- [ ] Ensure API uses policy evaluator + logging facade consistently
  - Docs: `./zrefactor/api_refactor.INSTRUCTIONS.md`

## Feature work interleaving (allowed, scoped)

- You may interleave additive feature work alongside refactors when noted below. Keep feature batches focused and avoid mixing with breaking refactors in the same changeset.
- Recommended insertion points:
  - After §3 (Logging/Audit facade): small, user-facing extras that only use the facade.
  - After §4 (API DX/DW): additive ergonomics (e.g., consumer docs, examples) and `ApiBuilder`-adjacent helpers.
  - After §7 (Tests reorg): developer-facing improvements that rely on the new test layout.
- Candidate feature docs to pull from (can be shipped in separate phases):
  - `./zrefactor/library_consumer_dx.INSTRUCTIONS.md` (additive ergonomics)
  - `./zrefactor/features_ux_refactor.PROPOSAL.md` (additive UX/features)
  - `./zrefactor/audit_event_schema_overhaul.PROPOSAL.md` (schema v2; consider batching near a minor bump)

## 5) Types and invariants consolidation (low-risk enablers)

- [ ] Centralize data-only types under `src/types/` where beneficial
  - Move `OwnershipInfo` to `src/types/ownership.rs` and re-export from `types/mod.rs`.
  - Consider `RescueStatus`/`RescueError` → `src/types/rescue.rs`; `MountFlags`/`MountError` → `src/types/mount.rs` (keep traits/impls in their modules).
- [ ] Introduce a typed `PreflightRow` under `src/types/preflight.rs`
  - Refactor `api/preflight/rows.rs` to build `PreflightRow` and serialize for emission (after logging facade lands).
- Acceptance
  - `cargo check && cargo test` pass; imports updated across adapters/policy/preflight.
  - `rg -n "struct OwnershipInfo" cargo/switchyard/src/adapters/ownership -S` returns 0; new `types/ownership.rs` exists.
  - Preflight builds rows via the typed struct (no ad-hoc serde_json object assembly once facade is in place).

## 6) FS backup/restore split (internal reorg)

- [ ] Split `src/fs/backup.rs` into `backup/{mod,snapshot,sidecar,index}.rs`
- [ ] Split `src/fs/restore.rs` into `restore/{mod,types,selector,idempotence,integrity,steps,engine}.rs`
- [ ] Remove any internal re-exports of atoms at `fs/mod.rs`; prefer direct module use
  - Docs: `./zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`

## 7) Tests reorganization (crate + repo e2e)

- [ ] Group crate integration tests under `tests/{locking,preflight,apply,fs,audit}/`; keep `tests/common.rs`
- [ ] Ensure all test files `mod common;` and update `tests/README.md`
- [ ] Maintain golden fixtures and trybuild tests
  - Docs: `./zrefactor/tests_refactor.INSTRUCTIONS.md`
  - Acceptance greps (mirrored from doc):
    - `rg -n "^mod common;" cargo/switchyard/tests/*.rs | wc -l` matches count of non-helper test files.
    - No top-level test files remain outside `cargo/switchyard/tests/{locking,preflight,apply,fs,audit}/` (except `common.rs`, `trybuild.rs`, `README.md`, golden fixtures).

## 8) Clean Code and Code Smell sweep (non-functional)

- [ ] Run the combined smell/cleanliness audit; fix top offenders without changing behavior
  - Docs: `./zrefactor/CODE_SMELL_AND_CLEAN_CODE_AUDIT.md`, `../../docs/CLEAN_CODE.md`
  - Grep aids (examples in the audit doc): no `unsafe`, production code avoids `.unwrap()`/`.expect()`, no stray `println!/dbg!/todo!`, no `tracing` in `src/`, etc.
  - Cohesion guardrails: avoid god functions; extract helpers where duplication exists; aim for submodules < ~800 LOC as noted in cohesion report.

## 9) Backwards-compat removals sweep

- [ ] Remove deprecated shims and re-exports once their replacements are in place
- [ ] Verify acceptance greps (no adapters::lock_file, no top-level `pub use policy::rescue`, no stray fs atoms)
  - Docs: `./zrefactor/backwards_compat_removals.md`, `./zrefactor/removals_registry.md`

## 10) Documentation alignment (public docs + SPEC references)

- [ ] Follow the documentation plan to sync crate docs and guides to the code
  - Docs: `./zrefactor/documantation/documentation_plan.md`
  - Include a pass to verify and refresh `./zrefactor/FEATURES_CATALOG.md` against sources (paths cited remain accurate; emitted fields match code).

## 11) Optional proposals (post‑refactor, later phases/releases)

- [ ] Audit Event Schema v2 (stage‑specific constraints, formats)
  - Doc: `./zrefactor/audit_event_schema_overhaul.PROPOSAL.md`
- [ ] Features UX refactor (additive, user‑facing docs/organization)
  - Doc: `./zrefactor/features_ux_refactor.PROPOSAL.md`
- [ ] Library consumer DX (additive ergonomics beyond `ApiBuilder` basics)
  - Doc: `./zrefactor/library_consumer_dx.INSTRUCTIONS.md`

---

Notes

- Prefer “one theme per change batch” with clear acceptance greps from the linked docs.
- Run `cargo check && cargo test` for each step; keep changes behavior‑neutral except where the doc explicitly calls out new behavior.
- Record any file moves/removals in `./zrefactor/removals_registry.md` as you go.

## Reassessment checkpoints (drift control)

- After §0 Baseline guardrails:
  - Ensure CI grep gates are active; SPEC and Clean Code docs linked in the review checklist.
- After §2 Policy-owned gating:
  - Grep for duplicate gating helpers and hard-coded mount paths; confirm `policy.extra_mount_checks` usage in preflight.
  - Reconcile `./zrefactor/policy_refactor.INSTRUCTIONS.md` and `./zrefactor/preflight_gating_refactor.INSTRUCTIONS.md` with code.
- After §3 Logging/Audit facade:
  - Verify field parity for `apply.attempt`, `apply.result` (per-action and summary), `preflight` rows, and `prune.result`.
  - Confirm no direct `FactsEmitter::emit` in API; update schema docs as needed.
- After §4 API DX/DW:
  - Check rustdoc public API map; error taxonomy alignment with SPEC; keep `Switchyard::new` + fluent `.with_*` working.
- After §5 FS split:
  - Confirm fs atoms are internal-only and no re-exports at `src/fs/mod.rs`; acceptance greps pass.
- After §6 Tests reorg:
  - Ensure all integration tests import `mod common;`; `tests/README.md` documents orchestration.
- Before §7 Removals sweep:
  - Generate removal list (markers + registry); ensure all targets are marked; then sweep.
- Before release:
  - Full SPEC and Clean Code pass; update CHANGELOG with breaking changes labeled per Rulebook.
