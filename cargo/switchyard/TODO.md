# Switchyard Refactor TODO (Order of Execution)

High-level, ordered plan to land refactors with minimal risk and conflict. Each item links to the detailed playbook in `./zrefactor/`. Pre-1.0, bundle large, fast-moving changes as needed; interleave additive feature work where noted. Run acceptance greps from each linked doc as you land items, and follow the Rulebook conventions.

## 0) Baseline guardrails and checks

- [x] Add/verify CI grep guardrails and run local checks
  - Docs: `./zrefactor/responsibility_cohesion_report.md`, `./zrefactor/backwards_compat_removals.md`
  - Goal: Establish “tripwires” early (no public fs atoms, no #[path], no adapters::lock_file, no direct FactsEmitter::emit in API, etc.)
  - Rulebook applies to all refactors: use standardized markers and sweep cadence.
    - Doc: `./zrefactor/refactor_rulebook.INSTRUCTIONS.md`
  - SPEC and Clean Code adherence gates (reject changes that drift from guarantees or cleanliness):
    - Docs: `./SPEC/SPEC.md`, `../../docs/CLEAN_CODE.md`
  - Add drift-check script to CI using the acceptance greps from the linked docs.
  - Notes: Added CI checks to forbid `#[path]` under `src/api/`, any `adapters::lock_file::` usage, legacy `audit::emit_*` and direct `FactsEmitter::emit` outside `src/logging/`, and public re-exports of low-level fs atoms at `src/fs/mod.rs`. Also checked for disallowed top-level `use switchyard::rescue`.
  - Update the code review checklist with: “SPEC/SPEC.md referenced”, “CLEAN_CODE adhered”, “Rulebook markers added”, “Acceptance greps pasted as evidence”, “Execution mandate acknowledged (full-doc execution; no cherry-picking)”.
  - Rulebook quick rules (summary):
    - Use standardized markers in code: `remove this file`, `move this file`, `replace this file`, `deprecated shim`, and `BEGIN/END REMOVE BLOCK`.
    - Batch cadence (pre-1.0): A) Markers-only, B) Implement refactor, C) Sweep removals.
    - Conventional commits; mark breaking changes with a short migration note in the body.
  - Bridging tasks:
    - Add CI grep script that aggregates acceptance greps from linked docs; wire it into `ci.yml`.
    - Update the code review template with Rulebook and SPEC/CLEAN_CODE checkboxes.
    - Seed `./zrefactor/removals_registry.md` with any files that cannot carry inline markers (schemas, generated assets).
    - Replan checkpoint: confirm which optional additive proposals (schema v2, DX) will be scheduled post §12.

## 1) Execution mandate and orchestrated order (MANDATORY)

Execute every referenced document in `./zrefactor/` fully, end-to-end. No curated subsets or cherry-picking. For each document, follow all instructions in order, implement all required code/tests/docs, and paste acceptance greps/evidence per the Rulebook.

Note: This is not a separate checklist. The sections below are the single consolidated plan. Execute each linked doc end‑to‑end when you reach its section, interleaving loose TODOs, bridging tasks, and replanning checkpoints as specified.

## 2) Idiomatic module/layout cleanup
  
- [x] Make `src/api` directory module fully idiomatic; remove any lingering `#[path]`
- [x] Remove legacy adapters shim (`adapters::lock_file`), ensure imports use `adapters::lock::file::*`
- [x] Tighten fs atoms visibility; no re-exports at `src/fs/mod.rs`
- Notes: low-level FS atoms are internal-only (`pub(crate)`), satisfying trybuild compile-fail expectations.
- [ ] Docs (execute end-to-end): `./zrefactor/idiomatic_todo.INSTRUCTIONS.md`
- Bridging tasks:
  - Paste acceptance greps into the PR description; ensure CI gates for `#[path]` and `adapters::lock_file` are active.
  - Update imports across the crate after moves; add/remove `mod` declarations as needed.
  - Record file moves/removals in `./zrefactor/removals_registry.md`.
  - Replan checkpoint: confirm next steps ordering between §4 Logging facade scaffolding and §3 Policy evaluator based on compile friction.

## 3) Policy-owned gating (single evaluator)
  
- [x] Implement typed `policy::gating::evaluate_action(..)` and shared helpers
- [x] Preflight: call evaluator per action; delete inlined checks and hard-coded mount paths (use `policy.extra_mount_checks`)
- [x] Apply: call evaluator before mutation; enforce `override_preflight`
- [ ] Define grouped policy types and profiles
- Introduce enums/groups (e.g., `RiskLevel`, `ExdevPolicy`, `LockingPolicy`, `SmokePolicy`, `PreservationPolicy`) under `policy::types`.
- Add curated `profiles` for common presets (e.g., production) and a `Policy::builder()` if needed.
- [ ] Docs (execute end-to-end): `./zrefactor/policy_refactor.INSTRUCTIONS.md`, `./zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`
- Bridging tasks:
  - Land grouped types and evaluator with unit tests; keep legacy flat fields compiling until API is migrated.
  - Add CI grep to forbid duplicate gating logic outside `src/policy/gating.rs` once preflight/apply are migrated.
  - Replan checkpoint: choose whether to migrate Preflight (§3→§4) or scaffold Logging facade (§4) first depending on call-site impact.

## 4) Logging/Audit facade migration
  
- [x] Introduce `StageLogger`/`EventBuilder` facade under `src/logging/`
- [x] Migrate API call sites (plan, preflight, apply, rollback, prune) to the facade
- [x] Keep field parity with current emissions (lock_backend/attempts, perf, attestation, error_id/exit_code, restore_ready, prune fields)
- Sub-migrations:
- [x] Migrate `plan` to builder (replace `emit_plan_fact` usage with builder calls).
- [x] Extract attestation bundle construction from `api/apply/mod.rs` into a helper under `adapters::attest` (typed struct), then attach via builder.
- [x] Add CI guardrails: forbid `audit::emit_` and direct `FactsEmitter::emit` outside `src/logging/`.
- [ ] Docs (execute end-to-end): `./zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Bridging tasks:
  - Replace any remaining direct emissions in API with facade calls; update or add golden tests for emitted events.
  - Ensure only `src/logging/` touches `FactsEmitter::emit`; add/verify CI greps.
  - Replan checkpoint: confirm API DX/DW migration scope (§5) based on facade availability.

## 5) API DX/DW alignment
  
- [x] Keep `Switchyard::new(facts, audit, policy)` and current fluent `.with_*` methods
- [x] Add `ApiBuilder` that mirrors `.with_*` and delegates to avoid duplication
- [x] Ensure API uses policy evaluator + logging facade consistently
- [ ] Docs (execute end-to-end): `./zrefactor/api_refactor.INSTRUCTIONS.md`
- Bridging tasks:
  - Introduce/validate `ApiBuilder` examples in rustdoc; map error taxonomy to `ApiError` and ensure public signatures are stable.
  - Verify evaluator is called in preflight/apply orchestrators; remove any ad‑hoc gating helpers.
  - Replan checkpoint: decide whether to proceed to Types consolidation (§6) or FS split (§7) based on dependency surface.
- Candidate feature docs to pull from (can be shipped in separate phases):
  - `./zrefactor/library_consumer_dx.INSTRUCTIONS.md` (additive ergonomics)
  - `./zrefactor/features_ux_refactor.PROPOSAL.md` (additive UX/features)
  - `./zrefactor/audit_event_schema_overhaul.PROPOSAL.md` (schema v2; consider batching near a minor bump)

## 6) Types and invariants consolidation (low-risk enablers)
  
- [ ] Docs (execute end-to-end): `./zrefactor/TYPES_AUDIT.md`
- [x] Centralize data-only types under `src/types/` where beneficial
- [ ] Move `OwnershipInfo` to `src/types/ownership.rs` and re-export from `types/mod.rs`.
- [ ] Consider `RescueStatus`/`RescueError` → `src/types/rescue.rs`; `MountFlags`/`MountError` → `src/types/mount.rs` (keep traits/impls in their modules).
- [x] Introduce a typed `PreflightRow` under `src/types/preflight.rs`
  - [ ] Refactor `api/preflight/rows.rs` to build `PreflightRow` and serialize for emission (after logging facade lands).
- Acceptance
  - `cargo check && cargo test` pass; imports updated across adapters/policy/preflight.
  - `rg -n "struct OwnershipInfo" cargo/switchyard/src/adapters/ownership -S` returns 0; new `types/ownership.rs` exists.
  - Preflight builds rows via the typed struct (no ad-hoc serde_json object assembly once facade is in place).
  - Bridging tasks:
    - Update imports across adapters/policy/preflight to new `types::*` paths; use `cargo fix` if helpful.
    - Replace ad‑hoc `serde_json` row assembly with `PreflightRow` serialization; add unit tests around `Serialize` shape.
    - Add brief module docs to new `types/*` files describing invariants and cross-layer usage.
    - Replan checkpoint: choose §7 FS split next if compile surface is stable; otherwise finish any lingering type migrations first.

## 7) FS backup/restore split (internal reorg)
  
- [x] Split `src/fs/backup.rs` into `backup/{mod,snapshot,sidecar,index}.rs`
- [ ] Split `src/fs/restore.rs` into `restore/{mod,types,selector,idempotence,integrity,steps,engine}.rs`
- [ ] Remove any internal re-exports of atoms at `fs/mod.rs`; prefer direct module use
- Notes: backup split completed with module re-exports; restore split in progress.
- [ ] Docs (execute end-to-end): `./zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`
- Bridging tasks:
  - Extract restore code into `restore/*` modules; wire `engine::restore_impl` behind public fns; update `fs/mod.rs` re‑exports.
  - Update API/handlers call sites to new module paths; move and add unit tests for selector/idempotence/integrity/steps.
  - Remove public re‑exports of low‑level atoms from `fs/mod.rs`; ensure internal callers use `fs::atomic` directly.
  - Replan checkpoint: if large diffs, consider splitting into PR B (implementation) and PR C (sweep removals) per Rulebook.

## 8) Tests reorganization (crate + repo e2e)
  
- [ ] Group crate integration tests under `tests/{locking,preflight,apply,fs,audit}/`; keep `tests/common.rs`
- [ ] Ensure all test files `mod common;` and update `tests/README.md`
- [ ] Maintain golden fixtures and trybuild tests
- [ ] Docs (execute end-to-end): `./zrefactor/tests_refactor.INSTRUCTIONS.md`
- Acceptance greps (mirrored from doc):
  - `rg -n "^mod common;" cargo/switchyard/tests/*.rs | wc -l` matches count of non-helper test files.
  - No top-level test files remain outside `cargo/switchyard/tests/{locking,preflight,apply,fs,audit}/` (except `common.rs`, `trybuild.rs`, `README.md`, golden fixtures).
- Bridging tasks:
  - Create domain subdirectories and move files per `removals_registry.md`; ensure each imports `mod common;` at top.
  - Update `tests/README.md` to document orchestration and new layout; adjust CI to collect domain paths.
  - Replan checkpoint: consider scheduling a small E2E expansion after reorg to validate matrix resilience.

## 9) Clean Code and Code Smell sweep (non-functional)
  
- [ ] Run the combined smell/cleanliness audit; fix top offenders without changing behavior
- [ ] Docs (execute end-to-end): `./zrefactor/CODE_SMELL_AND_CLEAN_CODE_AUDIT.md`, `../../docs/CLEAN_CODE.md`
- Grep aids (examples in the audit doc): no `unsafe`, production code avoids `.unwrap()`/`.expect()`, no stray `println!/dbg!/todo!`, no `tracing` in `src/`, etc.
- Cohesion guardrails: avoid god functions; extract helpers where duplication exists; aim for submodules < ~800 LOC as noted in cohesion report.
- Bridging tasks:
  - Run the provided grep suite and open small follow‑ups for any non‑blocking findings; tackle top offenders in this sweep.
  - Add optional lightweight `tracing` spans at API boundaries behind a feature if decided; keep emitted facts unchanged.
  - Replan checkpoint: verify readiness for removals sweep (§10).

## 10) Backwards-compat removals sweep
  
- [ ] Remove deprecated shims and re-exports once their replacements are in place
- [ ] Verify acceptance greps (no adapters::lock_file, no top-level `pub use policy::rescue`, no stray fs atoms)
- [ ] Docs (execute end-to-end): `./zrefactor/backwards_compat_removals.md`, `./zrefactor/removals_registry.md`
- Bridging tasks:
  - Generate removal list from markers and registry; delete files per Rulebook PR C; ensure CI guards remain.
  - Re-run `cargo test -p switchyard`; grep tree to confirm deprecated surfaces are gone.
  - Replan checkpoint: confirm docs sync (§11) is the final blocking step before release prep.

## 11) Documentation alignment (public docs + SPEC references)
  
- [ ] Follow the documentation plan to sync crate docs and guides to the code
- [ ] Docs (execute end-to-end): `./zrefactor/documantation/documentation_plan.md`
- Include a pass to verify and refresh `./zrefactor/FEATURES_CATALOG.md` against sources (paths cited remain accurate; emitted fields match code).
- Bridging tasks:
  - Add `#![deny(missing_docs)]` selectively and fix high‑value public items; ensure examples compile.
  - Add or refresh `examples/` programs referenced by the plan (dry-run, commit with lock, rollback, audit/redaction, exdev).
  - Replan checkpoint: decide which optional proposals to schedule for the next minor.

## 12) Optional proposals (post‑refactor, later phases/releases)
  
- [ ] Audit Event Schema v2 (stage‑specific constraints, formats)
- [ ] Doc (execute end-to-end when scheduled): `./zrefactor/audit_event_schema_overhaul.PROPOSAL.md`
- [ ] Features UX refactor (additive, user‑facing docs/organization)
- [ ] Doc (execute end-to-end when scheduled): `./zrefactor/features_ux_refactor.PROPOSAL.md`
- [ ] Library consumer DX (additive ergonomics beyond `ApiBuilder` basics)
- [ ] Doc (execute end-to-end when scheduled): `./zrefactor/library_consumer_dx.INSTRUCTIONS.md`
- Bridging tasks:
  - Convert proposals into backlog tickets with scope/acceptance; sequence behind a minor release.
  - Add feature flags where appropriate (`serde-reports`, `jsonl-file-sink`, `tracing`, `test-utils`, etc.) and set up a feature matrix job in CI.
  - Replan checkpoint: revisit SPEC and guardrails to ensure optional additions don’t regress safety posture.

---

Notes

- Prefer “one theme per change batch” with clear acceptance greps from the linked docs.
- Run `cargo check && cargo test` for each step; keep changes behavior‑neutral except where the doc explicitly calls out new behavior.
- Record any file moves/removals in `./zrefactor/removals_registry.md` as you go.

## Reassessment checkpoints (drift control)

- After §0 Baseline guardrails:
  - Ensure CI grep gates are active; SPEC and Clean Code docs linked in the review checklist.
- After §3 Policy-owned gating:
  - Grep for duplicate gating helpers and hard-coded mount paths; confirm `policy.extra_mount_checks` usage in preflight.
  - Reconcile `./zrefactor/policy_refactor.INSTRUCTIONS.md` and `./zrefactor/preflight_gating_refactor.INSTRUCTIONS.md` with code.
- After §4 Logging/Audit facade:
  - Verify field parity for `apply.attempt`, `apply.result` (per-action and summary), `preflight` rows, and `prune.result`.
  - Confirm no direct `FactsEmitter::emit` in API; update schema docs as needed.
- After §5 API DX/DW:
  - Check rustdoc public API map; error taxonomy alignment with SPEC; keep `Switchyard::new` + fluent `.with_*` working.
- After §7 FS split:
  - Confirm fs atoms are internal-only and no re-exports at `src/fs/mod.rs`; acceptance greps pass.
- After §8 Tests reorg:
  - Ensure all integration tests import `mod common;`; `tests/README.md` documents orchestration.
- Before §10 Removals sweep:
  - Generate removal list (markers + registry); ensure all targets are marked; then sweep.
- Before release:
  - Full SPEC and Clean Code pass; update CHANGELOG with breaking changes labeled per Rulebook.
