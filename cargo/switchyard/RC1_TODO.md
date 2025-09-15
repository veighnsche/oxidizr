# Switchyard RC1 Readiness Plan

Last updated: 2025-09-15

This document is the single source of truth for tasks required to reach RC1 for the `switchyard` crate. It aggregates open items from SPEC/BDD, BUGS, code audit, and docs.

---

## RC1 Exit Criteria

- [ ] All tests green:
  - [ ] `cargo test -p switchyard` (unit + integration + trybuild)
  - [ ] `cargo test -p switchyard --features bdd --test bdd` (0 undefined/ambiguous, all scenarios pass)
- [ ] No panics/unwrap/expect in library code paths (tests allowed)
  - Enforced via `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented)]` in `src/lib.rs`
- [ ] Clippy clean with `-D warnings` and `cargo fmt` clean
- [ ] JSON Schema v2 compatibility: every emitted fact validates; SPEC and schemas updated if new fields were added
- [ ] README, SPEC, DOCS, INVENTORY up to date and traceability passes

---

## P0 — Release Blockers

- [ ] Rollback: per-action events + reverse order verification
  - Files: `src/api/apply/rollback.rs`, `src/api/apply/executors/ensure_symlink.rs`, `tests/steps/rollback_steps.rs`
  - Ensure that on failure after some actions executed, the engine emits per-action `rollback` events for each reversed target, not just `rollback.summary`.
  - Add/assert reverse ordering using action IDs or `rolled_back_paths` ordering.
  - Acceptance: BDD `Feature: Rollback guarantees and idempotence` scenarios fully green.

- [ ] Idempotent topology restoration via captured snapshots
  - Files: `src/api/rollback.rs`, `src/fs/backup/*`, `src/fs/restore/*`, `tests/steps/rollback_steps.rs`
  - Guarantee `policy.apply.capture_restore_snapshot=true` produces an invertible rollback plan that restores pre-state exactly after two successive rollbacks.
  - Acceptance: “apply, then apply a rollback plan twice” restores original link/file topology.

- [ ] Schema v2 alignment for new summary fields
  - Files: `src/api/apply/summary.rs`, `SPEC/audit_event.v2.schema.json` (if present), `SPEC/SPEC.md`
  - New fields introduced: `rolled_back` (bool), `rolled_back_paths` (string[]). Make optional and document.
  - Acceptance: Schema validation BDD remains green; SPEC updated with field descriptions and traceability.

- [ ] Preflight override semantics are consistent and documented
  - Files: `src/api/preflight/mod.rs`, `src/policy/gating.rs`, `tests/steps/safety_preconditions_steps.rs`
  - Ensure `override_preflight=true` clears summary STOPs but still emits accurate per-row data. Remove duplicated gating logic or document it clearly (source world-writable STOP used for specific scenarios).
  - Acceptance: Safety preconditions feature stays green; no ambiguous steps; behavior matches SPEC.

- [ ] Trybuild expected outputs updated
  - Files: `tests/trybuild.rs`, `tests/trybuild/*.rs`, `BUGS.md`
  - Align expected compiler messages with current toolchain (tracked in `BUGS.md`).
  - Acceptance: trybuild tests reliable and green locally and in CI.

- [ ] Environment-driven EXDEV injection is opt-in and tested
  - Files: `src/fs/atomic.rs`, `src/api/overrides.rs`, docs
  - We disabled default env overrides in tests; require `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1` to inject EXDEV.
  - Add tests that explicitly set/unset the env to verify behavior and avoid test cross-talk.
  - Acceptance: No unintended EXDEV failures in unrelated scenarios; dedicated EXDEV tests pass.

- [ ] BDD step glue completeness & de-duplication
  - Files: `tests/steps/*`
  - Ensure zero ambiguous or overlapping regexes. Steps centralized per feature, anchored regexes used.
  - Acceptance: cucumber matches uniquely for each step across the suite.

---

## P1 — High Priority

- [ ] Reverse-order rollback assertion using action IDs
  - Files: `src/types/plan.rs` (action_id), `tests/steps/rollback_steps.rs`
  - Add Then-step asserting rollback order by `action_id` decreasing for executed set.

- [ ] Strengthen restore failure observability
  - Files: `src/fs/restore/*`, `src/api/apply/rollback.rs`
  - On any restore failure, ensure `rollback` per-action facts include `error_id` (e.g., `E_RESTORE_FAILED`) and `error_detail`.

- [ ] Expand preflight coverage for preservation capabilities matrix
  - Files: `src/api/preflight/mod.rs`, `src/preflight/checks.rs`
  - Add tests that toggle each preservation dimension to confirm STOP vs WARN per policy setting.

- [ ] CI enhancements
  - Files: `.github/workflows/ci.yml`
  - Add a separate job for BDD with caching, and a trybuild lane. Enforce `-D warnings`.

- [ ] Docs alignment
  - Files: `SPEC/SPEC.md`, `SPEC/traceability.md`, `INVENTORY/*`, `DOCS/*`
  - Update to reflect: attestation in summary, preflight override semantics, rollback fields, and rescue profile verification.

---

## P2 — Medium Priority

- [ ] Ownership oracle integration (pkg provenance)
  - Files: `src/adapters/ownership/fs.rs`, `src/types/ownership.rs`
  - Provide real package ownership data (pacman/apt backends, or pluggable adapter). Update BDD to assert presence where available.

- [ ] Typed API errors for public surfaces
  - Files: `src/api/*`
  - Introduce a crate-specific error type for public APIs, with stable `ErrorId` mapping and source chaining.

- [ ] ENOSPC and long-path test harnesses
  - Files: `BUGS.md`, `tests/*`
  - Provide mock adapters or feature-gated tests to simulate ENOSPC and to avoid `ENAMETOOLONG` while still asserting behavior via facts.

- [ ] Determinism deepening
  - Files: `src/logging/redact.rs`, tests
  - Additional property tests for redaction stability and timestamp/UUID reproducibility.

---

## P3 — Nice-to-Have / Post-RC

- [ ] Performance budgets and telemetry expansion (fsync latency distribution, per-action timings)
- [ ] More granular policy knobs for SUID/SGID and hardlink hazards with per-target overrides
- [ ] Golden fixtures coverage expansion and auto-regeneration tooling

---

## Hidden Bug Candidates (Audit Findings)

- __Rollback summary fields vs schema__
  - We added `rolled_back` and `rolled_back_paths` to `apply.result` summary in `src/api/apply/summary.rs`. Ensure the JSON schema/validators tolerate these as optional.

- __EXDEV injection cross-talk__
  - Previously test builds could inherit EXDEV injection implicitly. We changed `src/fs/atomic.rs` to require explicit `SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES=1`. Add tests asserting the default is off.

- **Duplicate EnsureSymlink gating**
  - Both `src/policy/gating.rs` and `src/api/preflight/mod.rs` may introduce similar checks (source world-writable). Consolidate to one place or document layered behavior to avoid drift.

- **Dead code: rolled_back_paths return value**
  - `rollback::do_rollback()` returning a `Vec<String>` was introduced earlier; current summary uses reconstruction from `executed`. Either wire this value through to summary or remove to avoid confusion.

- **Top-level perf fields**
  - `fsync_ms` currently set from aggregated values; review correctness and ensure consistency across per-action vs summary metrics.

- **Clippy/test hygiene**
  - Many `unwrap/expect/panic` occurrences are confined to unit tests within `src/` modules (guarded by `#[test]`). Confirm none leak into non-test code paths.

---

## Test Coverage Map (What to run)

- Unit/integration:
  - `cargo test -p switchyard --all-features`
- BDD:
  - `cargo test -p switchyard --features bdd --test bdd`
- Focused checks:
  - Rollback scenarios in `SPEC/features/rollback.feature`
  - Safety preconditions in `SPEC/features/safety_preconditions.feature`
  - Trybuild in `tests/trybuild.rs`

---

## Ownership / Contacts

- Code owners: see `CODEOWNERS` (if present). Otherwise, Switchyard maintainers in this repo.
- For RC1 callouts, link PRs to this document and update checkboxes.
