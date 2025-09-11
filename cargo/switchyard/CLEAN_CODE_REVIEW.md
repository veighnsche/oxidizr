# Switchyard Clean Code Review

This review evaluates the Switchyard crate against our project-wide Clean Code principles (see `docs/CLEAN_CODE.md`). It highlights strengths, specific improvement opportunities, and concrete, prioritized actions with file-level citations.

---

## Executive Summary

Switchyard demonstrates a clear, safety-first architecture with good separation of concerns:

- Atomic, TOCTOU-safe filesystem primitives under `src/fs/` using `rustix`.
- Policy-gated orchestration in `src/api/` with structured, deterministic fact emission.
- Determinism and idempotence are deliberately designed (UUIDv5 IDs, zeroed timestamps, idempotent restore).
- Production-hardening toggles are centralized in `policy::Policy`.

Key opportunities:

- Replace `unwrap()`/`expect()` in library code paths with error mapping (notably in `fs/symlink.rs`).
- Further reduce cyclomatic complexity in `api/apply.rs` by extracting intentful helpers for gating and per-action handling.
- Expand module-level documentation (public types under `src/types/`) and add `#[must_use]` to critical return types.
- Extend error taxonomy (current `ErrorKind` is coarse) and annotate errors with structured context.
- Consider narrowing public surface (`pub(crate)` for internal-only modules) and add clippy gates in CI.

All tests pass after recent updates; observability and recovery posture are strong.

---

## Strengths (by Principle)

- __Clarity Over Cleverness (1)__
  - The code favors explicit procedural steps over clever tricks. Examples: `api/preflight.rs` per-action rows with stable sort, `api/apply.rs` early-return guards.

- __Explicit > Implicit (2)__
  - Side effects (facts/audit) are injected via traits: `logging::FactsEmitter`, `logging::AuditSink`, `adapters::LockManager`, `adapters::SmokeTestRunner`, `adapters::Attestor`.
  - Public mutating APIs use `types::safepath::SafePath` enforcing rooted paths.

- __Types Encode Invariants (3)__
  - `SafePath`, `Plan`, `Action`, `ApplyMode`, `ApplyReport` encode domain flows well.
  - UUIDv5 `plan_id`/`action_id` (`types/ids.rs`) ensure deterministic identity.

- __Immutability by Default (4)__
  - Most code uses immutable bindings, with mutation localized to orchestration steps.

- __Honest Error Handling (5)__
  - Project uses `thiserror`. Fail-closed patterns and E_* error IDs are surfaced via facts.

- __Side-Effect Isolation (6)__
  - Filesystem edge is in `fs/` (TOCTOU-safe `atomic.rs`, backup/restore in `symlink.rs`). Policy and facts are in `api/`.

- __Determinism & Idempotence (7)__
  - DryRun timestamps zeroed; redaction for canon; idempotent `restore_file()` (double-invoke no-op) verified by tests.

- __Atomicity & Consistency (8)__
  - `renameat`-based atomic swap with parent `fsync` (`fs/atomic.rs`); degraded EXDEV fallback is gated and telemetered.

- __Observability Is a Feature (9)__
  - Minimal Facts v1 across stages; `plan_id` correlates events. Attestation bundle integrated when available.

- __Small, Composable Modules (10)__
  - Clear layering (`lib.rs` → `api/*` → `fs/*` → adapters). Recent constant extraction improved cohesion.

- __Safe Concurrency (11)__
  - Locking is interface-driven; `require_lock_manager` and default Commit gating ensure single mutator.

- __Dependency Discipline (12)__
  - Minimal set: `rustix`, `fs2`, `xattr`, `sha2`, serde stack; optional `file-logging` feature.

- __Document Invariants (13)__
  - New module docs in `api/*`, `fs/*`, `rescue.rs`, `adapters/smoke.rs` document purpose and failure modes.

- __Intent-Revealing Names (14)__
  - Types and functions map closely to domain (e.g., `replace_file_with_symlink`, `find_latest_backup_and_sidecar`).

- __Cohesive APIs (15)__
  - Public API surfaces minimal, opinionated operations (plan/preflight/apply/rollback).

- __Unsafe Forbidden (16)__
  - `#![forbid(unsafe_code)]` in `lib.rs`.

- __Testing for Behavior (17)__
  - Suites cover failure modes (E_LOCKING, E_EXDEV, E_POLICY, etc.), smoke-rollback, provenance presence, YAML export shape.

- __Config Is Data (18)__
  - `policy::Policy` aggregates toggles; default production preset provided; now includes `rescue_min_count`.

- __Fail Closed, Loudly (19)__
  - Commit w/o lock fails (policy-gated), missing backups emit `E_BACKUP_MISSING`, smoke failures emit `E_SMOKE` and trigger rollback unless disabled.

- __Formatting/Lints/CI (20)__
  - Style is consistent; feature flags are declared; tests pass locally.

- __Happy Path Obvious (21)__
  - Guard clauses clear; success path reads linearly in most modules.

- __Comments Explain Why (22)__
  - Module docs cover intent; inline comments in `symlink.rs` explain backup/sidecar schema.

- __Time/Locale/FS Are Adversaries (23)__
  - DryRun zeroes timestamps; mount/immutability checks; degraded EXDEV semantics are documented and gated.

- __Minimal Public Surface (24)__
  - Adapters namespace avoids leaking internals; `PathResolver` export removed to reduce drift.

- __Design for Recovery (25)__
  - Sidecar metadata captures `prior_kind`, `prior_dest`, `mode`; restore is idempotent and verified by tests.

---

## Issues and Recommendations

- __[Error Handling]__ Replace `unwrap()` in library code paths with fallible conversions
  - Files: `src/fs/symlink.rs` contains multiple `unwrap()` calls on `CString::new` and `File::create` contexts:
    - e.g., `let fname_c = std::ffi::CString::new(fname).unwrap();`
    - Risk: C-string creation may fail on embedded NUL bytes. While target file names are OS-provided, mapping errors preserves safety-in-depth.
  - Recommendation: Replace with `map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cstring"))?` and propagate.
  - Benefit: Aligns with Principle 5 (Honest Error Handling). Improves robustness for corner-case filenames.

- __[Complexity]__ Extract intentful helpers from `api/apply.rs`
  - File: `src/api/apply.rs` has sizeable inner match handling for `Action::EnsureSymlink` and `Action::RestoreFromBackup`, as well as inline policy gating per action.
  - Recommendation: Factor into helpers such as `apply_symlink_action(...)`, `apply_restore_action(...)`, and `enforce_policy_gates(...)` to reduce nesting and improve readability/testability.
  - Benefit: Principle 10 (Small, Composable Modules), 21 (Happy Path Obvious).

- __[Error Taxonomy]__ Enrich `types/errors.rs` categories and attach structured context
  - Current `ErrorKind`: `InvalidPath`, `Io`, `Policy`.
  - Recommendation: Introduce granular variants (e.g., `AtomicSwap`, `Fsync`, `Cstring`, `BackupSidecar`, `Xdev`, `OwnershipCheck`) and include structured fields (path, syscall, errno).
  - Benefit: Better telemetry and remediation messages (Principles 5, 19).

- __[Public Surface Narrowing]__ Consider `pub(crate)` for internal-only modules
  - Modules like `preflight` and `fs` might remain public for integrators, but if not explicitly supported as public API, mark as `pub(crate)` to reserve evolution space.
  - Benefit: Principle 24 (Minimal Public Surface).

- __[Must-Use Annotations]__ Mark critical types as `#[must_use]`
  - Types: `ApplyReport`, `PreflightReport`. Functions: `Switchyard::preflight`, `Switchyard::apply` already return `Result<...>` (which is must-use). Adding `#[must_use]` to structs helps catch accidental discards if they are constructed or returned by helpers.
  - Benefit: Principle 2 (Explicit > Implicit).

- __[Observability]__ Optional `tracing` integration
  - Consider adding `tracing` spans (e.g., `apply_plan`, `preflight_plan`, `rollback`) with `plan_id` as a root span field.
  - Benefit: Principle 9 (Observability); complementary to Facts.

- __[Documentation Coverage]__ Add module/top-level docs for types
  - Files: `src/types/plan.rs`, `src/types/report.rs`, `src/types/errors.rs`, `src/types/safepath.rs` could include brief docs on invariants and usage.
  - Benefit: Principle 13 (Document Invariants, Not Trivia).

- __[Builder/Validation]__ Consider builders for `Policy`
  - Replace numerous `bool` toggles passed around via ad-hoc mutation with a builder that validates consistency (e.g., `require_lock_manager` implies `allow_unlocked_commit == false`).
  - Benefit: Principles 2, 3, 4 (explicit config objects, invariants, immutability by default).

- __[Env Isolation]__ Wrap env access behind interface in `rescue.rs`
  - Reading `PATH` and `SWITCHYARD_FORCE_RESCUE_OK` is appropriate for now; providing an injectable env reader would improve testability and isolation if rescue logic grows.
  - Benefit: Principle 6 (Side-Effect Isolation).

- __[CI/Lints]__ Strengthen CI gates
  - Add `clippy --workspace --all-features -D warnings` and `cargo fmt -- --check` to CI for this crate.
  - Add `cargo deny` or `cargo audit` when applicable.
  - Benefit: Principle 20.

- __[Locale/Test Stability]__ Stabilize environment for tests
  - In test harness or test-orch, set `LC_ALL=C`, fixed `TZ=UTC`, and `UMASK=022` for tighter determinism (Principle 23). Some of this exists in outer infra; recommend documenting in the crate README.

---

## Quick Wins (Low Effort / High Impact)

- __Replace `unwrap()` in `fs/symlink.rs` with error mapping__
- __Factor `api/apply.rs` per-action handlers into helpers__
- __Add module docs to `types/*`__
- __Add `#[must_use]` to `ApplyReport` and `PreflightReport`__
- __Add clippy & fmt checks in CI for `switchyard`__

---

## Medium / Strategic Improvements

- __Error taxonomy expansion__ with structured context (paths, errno, syscall labels).
- __Policy builder__ with validation (e.g., production presets) and eliminate ambiguous states.
- __Optional `tracing` spans__ with `plan_id` and action IDs for local debugging in addition to Facts.
- __Consider reducing public surface__ if `fs`/`preflight` modules aren’t intended as public API for integrators.

---

## Notable Citations

- `src/fs/symlink.rs`: several `unwrap()` calls on `CString::new` (`fname_c`, `bname_c`), on `File::create` (sidecar, tombstone), and `read_link` in restore. Replace with `?` + mapped errors.
- `src/api/apply.rs`: inline gating and per-action handling (recommend helper extraction).
- `src/types/errors.rs`: coarse `ErrorKind` (recommend richer variants and context fields).
- `src/api/fs_meta.rs`: improved owner/xattr detection (good), now documented.
- `src/adapters/smoke.rs`: minimal expectations documented.

---

## Appendix: Example Pattern for C-String Error Mapping

```rust
let fname_c = std::ffi::CString::new(fname)
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
```

## Appendix: Example Helper Extraction (sketch)

```rust
fn apply_symlink_action<E: FactsEmitter, A: AuditSink>(
    ctx: &AuditCtx,
    api: &Switchyard<E, A>,
    act: &Action,
    dry: bool,
) -> (Option<ErrorId>, Option<Action>) {
    // ...
}
```

## 2025-09-12 Update: Codebase scan findings and decisions

This addendum reflects a fresh scan of `cargo/switchyard/src/` against `docs/CLEAN_CODE.md` and SPEC v1.1 (`cargo/switchyard/SPEC/SPEC.md`). It records measured file sizes, warning/smell inventory, SPEC alignment observations, and concrete actions.

### Large files and measured LOC (targets for split)

- `src/fs/symlink.rs` — 607 LOC
- `src/api/apply.rs` — 529 LOC
- `src/api/preflight.rs` — 286 LOC
- `src/api/audit.rs` — 193 LOC
- Reference: `src/fs/atomic.rs` — 94 LOC; `src/rescue.rs` — 74 LOC; `src/api/fs_meta.rs` — 130 LOC; `src/types/safepath.rs` — 105 LOC; `src/api.rs` — 257 LOC

Decisions:

- __[Split symlink.rs]__ Create focused submodules under `src/fs/` to isolate concerns and reduce risk surface:
  - `fs/backup.rs` (backup payload + sidecar schema read/write; restore helpers)
  - `fs/atomic.rs` (already present; keep TOCTOU-safe primitives here)
  - `fs/cross_fs.rs` (EXDEV handling and degraded mode helpers)
  - `fs/restore.rs` (idempotent restore orchestrator)
  - Keep `fs/symlink.rs` as a thin façade wiring these pieces.
- __[Extract in api/apply.rs]__ Factor per-action handlers and gating into helpers:
  - `apply_symlink_action(...)`, `apply_restore_action(...)`, and `enforce_policy_gates(...)`.
- __[preflight.rs]__ Extract `detectors` and `report` builders to keep the module <200 LOC.

Rationale: Aligns with Clean Code principles (Small Modules, Happy Path obvious) and SPEC’s separation between mechanism (fs) and policy/telemetry (api).

### Warning/smell inventory and actions

- __[unwrap/expect in library code]__
  - Found in production modules dealing with C-string creation:
    - `src/fs/atomic.rs`: `CString::new(...).unwrap()` at symlink tmp name and final name creation.
    - `src/fs/symlink.rs`: multiple `CString::new` uses for filenames during `openat/renameat/unlinkat` sequences.
  - Action: Replace with error-mapped variants and `?`, e.g.:
    - `CString::new(x).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cstring"))?`.
  - Scope: Tests may continue using `unwrap`.

- __[allow attributes]__
  - Only localized `#[allow(non_camel_case_types)]` in `src/api/errors.rs` for `ErrorId`. Decision: keep localized with comment; acceptable to match emitted IDs.

- __[TODO/FIXME/panic/todo/unimplemented/dbg/println]__
  - None found under `src/` at scan time. Decision: keep CI guardrails to prevent regressions.

- __[Must-use annotations]__
  - `types/report.rs`: add `#[must_use]` to `PreflightReport` and `ApplyReport` to catch accidental discard.

- __[Clippy and deny policy]__
  - Adopt crate-level lints in `src/lib.rs` (non-test):
    - `#![cfg_attr(not(test), deny(warnings))]`
    - `#![deny(clippy::unwrap_used, clippy::expect_used)]`
    - `#![warn(clippy::all, clippy::cargo, clippy::pedantic)]`
  - CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `RUSTFLAGS="-Dwarnings" cargo check`.

### SPEC alignment observations (v1.1)

- __[Atomicity]__ `fs/atomic.rs` upholds the required sequence (open_dir_nofollow → symlinkat tmp → renameat → fsync parent). `FSYNC_WARN_MS` in `constants.rs` is used to annotate slow fsyncs in `api/apply.rs`.
- __[Rescue requirements]__ `rescue::verify_rescue_tools_with_exec_min(exec_check, min_count)` is called from both `preflight.rs` (summary field) and `apply.rs` (gating). Decision: keep boolean surface for now but consider an internal `Result<RescueOk, RescueError>` to enrich audit fields later without API churn.
- __[Determinism]__ `types/ids.rs` uses UUIDv5 with `NS_TAG` and `SafePath::rel()`; ordering is deterministic. Decision: keep.
- __[Cross-FS degraded mode]__ `fs/atomic.rs` handles EXDEV with best-effort fallback when allowed; `api/apply.rs` emits `degraded=true` on action results. Next: ensure EXDEV mapping to `E_EXDEV` and policy denial paths remain test-covered.
- __[Locking and facts]__ `apply.rs` records `lock_wait_ms` and maps failures to `E_LOCKING`. Decision: add unit/integration tests for contention and bounded wait behavior.

### Prioritized actions

- __P0 (non-breaking)__
  - Replace `CString::new(...).unwrap()` in `fs/atomic.rs` and `fs/symlink.rs` with error-mapped `?` returns.
  - Add `#[must_use]` on `ApplyReport` and `PreflightReport`.
  - Introduce helper extraction in `api/apply.rs` to reduce size/complexity (no behavior changes).
  - Add crate lint gates (deny unwrap/expect in non-test) and wire CI.

- __P1 (refactor + tests)__
  - Split `fs/symlink.rs` along the boundaries listed above; add targeted unit tests per new module.
  - Extract detectors/report builders from `api/preflight.rs`.
  - Add rescue probe timeouts and structured `RescueError` internally; keep the bool façade.

- __P2 (polish)__
  - Expand error taxonomy beyond coarse `ErrorKind` and plumb structured context (path, syscall, errno).
  - Consider tightening `pub(crate)` on internal modules if not intended as public surface for integrators.

All actions above preserve current external behavior and align with `docs/CLEAN_CODE.md` and SPEC v1.1.

---

## Conclusion

Switchyard’s core is clean, robust, and safety-oriented. The listed quick wins will further harden correctness and maintainability without changing behavior. I can implement the quick wins now (unwrap removal, helper extraction, docs, must_use) as a follow-up PR if desired.
