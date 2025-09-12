# Switchyard Refactor TODO (Order of Execution)

High-level, ordered plan to land refactors with minimal risk and conflict. Each item links to the detailed playbook in `./zrefactor/`. Keep PRs small and refactor-only; do not mix in new features. Run acceptance greps from each linked doc as you land items.

## 0) Baseline guardrails and checks

- [ ] Add/verify CI grep guardrails and run local checks
  - Docs: `./zrefactor/responsibility_cohesion_report.md`, `./zrefactor/backwards_compat_removals.md`
  - Goal: Establish “tripwires” early (no public fs atoms, no #[path], no adapters::lock_file, no direct FactsEmitter::emit in API, etc.)

## 1) Idiomatic module/layout cleanup

- [ ] Make `src/api` directory module fully idiomatic; remove any lingering `#[path]`
- [ ] Remove legacy adapters shim (`adapters::lock_file`), ensure imports use `adapters::lock::file::*`
- [ ] Tighten fs atoms visibility; no re-exports at `src/fs/mod.rs`
  - Docs: `./zrefactor/idiomatic_todo.INSTRUCTIONS.md`

## 2) Policy-owned gating (single evaluator)

- [ ] Implement typed `policy::gating::evaluate_action(..)` and shared helpers
- [ ] Preflight: call evaluator per action; delete inlined checks and hard-coded mount paths (use `policy.extra_mount_checks`)
- [ ] Apply: call evaluator before mutation; enforce `override_preflight`
  - Docs: `./zrefactor/policy_refactor.INSTRUCTIONS.md`, `./zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`

## 3) Logging/Audit facade migration

- [ ] Introduce `StageLogger`/`EventBuilder` facade under `src/logging/`
- [ ] Migrate API call sites (plan, preflight, apply, rollback, prune) to the facade
- [ ] Keep field parity with current emissions (lock_backend/attempts, perf, attestation, error_id/exit_code, restore_ready, prune fields)
  - Docs: `./zrefactor/logging_audit_refactor.INSTRUCTIONS.md`

## 4) API DX/DW alignment

- [ ] Keep `Switchyard::new(facts, audit, policy)` and current fluent `.with_*` methods
- [ ] Add `ApiBuilder` that mirrors `.with_*` and delegates to avoid duplication
- [ ] Ensure API uses policy evaluator + logging facade consistently
  - Docs: `./zrefactor/api_refactor.INSTRUCTIONS.md`

## 5) FS backup/restore split (internal reorg)

- [ ] Split `src/fs/backup.rs` into `backup/{mod,snapshot,sidecar,index}.rs`
- [ ] Split `src/fs/restore.rs` into `restore/{mod,types,selector,idempotence,integrity,steps,engine}.rs`
- [ ] Remove any internal re-exports of atoms at `fs/mod.rs`; prefer direct module use
  - Docs: `./zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md`

## 6) Tests reorganization (crate + repo e2e)

- [ ] Group crate integration tests under `tests/{locking,preflight,apply,fs,audit}/`; keep `tests/common.rs`
- [ ] Ensure all test files `mod common;` and update `tests/README.md`
- [ ] Maintain golden fixtures and trybuild tests
  - Docs: `./zrefactor/tests_refactor.INSTRUCTIONS.md`

## 7) Backwards-compat removals sweep

- [ ] Remove deprecated shims and re-exports once their replacements are in place
- [ ] Verify acceptance greps (no adapters::lock_file, no top-level `pub use policy::rescue`, no stray fs atoms)
  - Docs: `./zrefactor/backwards_compat_removals.md`, `./zrefactor/removals_registry.md`

## 8) Documentation alignment (public docs + SPEC references)

- [ ] Follow the documentation plan to sync crate docs and guides to the code
  - Docs: `./zrefactor/documantation/documentation_plan.md`

## 9) Optional proposals (post‑refactor, separate PRs/releases)

- [ ] Audit Event Schema v2 (stage‑specific constraints, formats)
  - Doc: `./zrefactor/audit_event_schema_overhaul.PROPOSAL.md`
- [ ] Features UX refactor (additive, user‑facing docs/organization)
  - Doc: `./zrefactor/features_ux_refactor.PROPOSAL.md`
- [ ] Library consumer DX (additive ergonomics beyond `ApiBuilder` basics)
  - Doc: `./zrefactor/library_consumer_dx.INSTRUCTIONS.md`

---

Notes

- Prefer “one theme per PR” with clear acceptance greps from the linked docs.
- Run `cargo check && cargo test` for each step; keep changes behavior‑neutral except where the doc explicitly calls out new behavior.
- Record any file moves/removals in `./zrefactor/removals_registry.md` as you go.
