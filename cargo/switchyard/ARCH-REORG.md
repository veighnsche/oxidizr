# Switchyard module reorganization proposal (v2025-09)

Goal: maximize maintainability, testability, and determinism by grouping related concerns,
separating policy from mechanism, and eliminating duplicated logic. This proposal lays out a
layered module structure, a relocation map (old → new), and a zero-behavior-change migration plan.

## TL;DR

- Create a clear layering: Types → FS (mechanism) → Policy (rules/gates) → Adapters → API (orchestration) → Logging.
- Move stage-agnostic helpers to where they belong (e.g., `api/fs_meta.rs` → `fs/meta.rs`).
- Unify preflight/apply gating in a single `policy/gating.rs` consumed by both `api/preflight` and `api/apply`.
- Co-locate stage-specific emitters under `logging/audit.rs` rather than in `api/`.
- Split adapters into traits vs default implementations.
- Keep public API surface stable via re-exports and deprecation notes for one minor cycle.

## Current layout (excerpt)

- `src/api.rs` facade delegating to staged modules under `src/api/`
- `src/api/audit.rs` audit helpers (emits facts)
- `src/api/fs_meta.rs` filesystem metadata helpers
- `src/api/plan.rs`, `src/api/preflight.rs` (+ `preflight/report.rs`), `src/api/apply.rs` (+ `apply/{gating,handlers,audit_emit}.rs`), `src/api/rollback.rs`
- `src/preflight.rs` top-level helper module for mount/immutable/trust checks and YAML exporter
- `src/rescue.rs` PATH-based rescue-tool verification
- `src/fs/{atomic,backup,restore,swap,paths}.rs`
- `src/logging/{facts,redact}.rs`
- `src/policy/config.rs`
- `src/adapters/*` (lock, lock_file, ownership, ownership_default, attest, smoke, path)
- `src/types/*` (errors, safepath, plan, report, ids)

## Pain points observed

- Duplication/drift risk between `api/preflight.rs`, `api/apply/gating.rs`, and `preflight.rs`:
  - Policy gates are re-implemented across places. Apply invokes some functions from `crate::preflight`, but the checks and the per-action logic diverge.
- Stage-agnostic helpers sit under stage directories:
  - `api/fs_meta.rs` contains generic FS metadata/probe helpers that are also used by `apply` and `preflight`.
  - `api/audit.rs` defines stage-neutral audit emitters which conceptually belong in `logging/`.
- Mixed adapters (traits and default impls) live flatly under `adapters/` (harder to discover default vs SPI).
- Two “preflight” modules exist (`src/api/preflight.rs` stage engine and `src/preflight.rs` helpers), which blurs ownership.

## Design principles

- Strong layering:
  1) Domain: `types`, `constants`.
  2) Mechanism: `fs` (TOCTOU-safe syscalls, backup/restore), no policy here.
  3) Policy: `policy` (gating, rescue, checks), pure/predictable functions.
  4) Adapters: external integrations (locks, attestation, ownership, smoke), traits + defaults.
  5) API: orchestrates plan → preflight → apply → rollback, calls into FS/Policy/Adapters.
  6) Logging: fact emission, redaction, stage emit helpers. No policy or FS logic here.
- Single-source-of-truth for preflight/apply gating.
- Stage modules contain orchestration only; helpers live in their home layers.
- Determinism: Keep ID generation and timestamp behavior centralized (`types/ids`, `logging/redact`).

## Target layout (proposed)

```text
src/
  api/
    plan.rs
    preflight/
      mod.rs          # stage engine (was api/preflight.rs)
      rows.rs         # row builder + per-action fact emitter (was preflight/report.rs)
      yaml.rs         # YAML exporter (moved from top-level preflight.rs)
    apply/
      mod.rs          # stage engine (was api/apply.rs)
      handlers.rs
      audit_fields.rs # was apply/audit_emit.rs
    rollback.rs
    errors.rs
  adapters/
    lock/
      mod.rs          # trait (was adapters/lock.rs)
      file.rs         # FileLockManager impl (was adapters/lock_file.rs)
    ownership/
      mod.rs          # trait (was adapters/ownership.rs)
      fs.rs           # default impl (was adapters/ownership_default.rs)
    attest.rs
    smoke.rs
    path.rs
    mod.rs            # re-exports for ergonomics
  fs/
    atomic.rs
    backup.rs
    restore.rs
    swap.rs
    paths.rs
    meta.rs           # moved from api/fs_meta.rs
    mod.rs            # re-exports of public primitives
  logging/
    facts.rs
    redact.rs
    audit.rs          # moved from api/audit.rs (AuditCtx, emit_* helpers)
    mod.rs            # re-exports
  policy/
    config.rs
    gating.rs         # was api/apply/gating.rs, generalized for preflight & apply
    checks.rs         # ensure_mount_rw_exec, check_immutable, check_source_trust (from preflight.rs)
    rescue.rs         # moved from rescue.rs
    mod.rs
  types/
    errors.rs
    ids.rs
    plan.rs
    report.rs
    safepath.rs
  constants.rs
  lib.rs              # updated to re-export new modules and shims
```

## Relocation map (old → new)

- `api/fs_meta.rs` → `fs/meta.rs`
- `api/audit.rs` → `logging/audit.rs`
- `api/preflight.rs` → `api/preflight/mod.rs` (orchestrator only; use `policy/gating` and `fs/meta`)
- `api/preflight/report.rs` → `api/preflight/rows.rs`
- `preflight.rs` (helpers) → split:
  - `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust` → `policy/checks.rs`
  - `to_yaml()` → `api/preflight/yaml.rs`
- `api/apply/gating.rs` → `policy/gating.rs` (generalized, no stage coupling)
- `api/apply/audit_emit.rs` → `api/apply/audit_fields.rs`
- `rescue.rs` → `policy/rescue.rs`
- `adapters/lock.rs` → `adapters/lock/mod.rs`
- `adapters/lock_file.rs` → `adapters/lock/file.rs`
- `adapters/ownership.rs` → `adapters/ownership/mod.rs`
- `adapters/ownership_default.rs` → `adapters/ownership/fs.rs`

No changes to:

- `fs/{atomic,backup,restore,swap,paths}.rs`
- `types/*`, `constants.rs`, `adapters/{attest,smoke,path}.rs`

## API and import surface after reorg

- Keep public API stable via re-exports in `lib.rs` and `logging/mod.rs`:
  - `crate::api` facade unchanged.
  - `crate::logging::{AuditSink, FactsEmitter, TS_ZERO, ts_for_mode, redact_event}` unchanged.
  - Back-compat shim: `pub mod preflight` at root can re-export `policy::checks` and `api::preflight::yaml` for one minor release:
    - `crate::preflight::to_yaml` → `crate::api::preflight::yaml::to_yaml`
    - `crate::preflight::{ensure_mount_rw_exec, check_immutable, check_source_trust}` → `crate::policy::checks::*`
- Internals in `api/*` updated to import from new locations:
  - `use crate::fs::meta::*;`
  - `use crate::logging::audit::*;`
  - `use crate::policy::{gating, checks, rescue};`

## How preflight/apply will share gates

- `policy/gating.rs` exposes pure helpers:
  - `analyze_plan(policy: &Policy, plan: &Plan) -> Vec<ActionGate>`
  - `analyze_action(policy: &Policy, action: &Action) -> ActionGate`
- `ActionGate` contains:
  - `policy_ok: bool`
  - `notes: Vec<String>` (e.g., "untrusted source allowed by policy")
  - `stops: Vec<String>` (e.g., "target not rw+exec")
  - `provenance: Option<Value>`
  - `preservation: Value`, `preservation_supported: bool`
- `api/preflight` uses `ActionGate` to build rows and emit per-action facts.
- `api/apply` uses only the `stops` portion to block execution when `override_preflight=false`.
This removes drift and makes gates unit-testable independently of stages.

## Layering rules (post-reorg)

- `api/*` may depend on `fs`, `policy`, `adapters`, `logging`, `types`, `constants`.
- `policy/*` may depend on `fs/meta`, `policy/checks`, `types`, but not on `api/*`.
- `fs/*` depends on `rustix`, standard library; must not import `policy`, `logging`, or `api`.
- `logging/*` may depend on `types`, `constants`; no `fs` or `policy` imports.
- `adapters/*` are traits and impls; no `api` imports.

## Migration plan (no behavior change)

1) Add new modules in place with `mod.rs` and copy code, keeping old modules temporarily.
2) Update `api/*` to import from new modules. Keep function names stable.
3) Add re-exports in `lib.rs` and `logging/mod.rs` to preserve public surface.
4) Deprecate old internal paths with `#[deprecated(note = "moved to …")]` for one cycle.
5) Delete old files after CI passes and downstream crates bump.

Suggested order of PRs:

- PR1: Move `api/audit.rs` → `logging/audit.rs`; add re-exports.
- PR2: Move `api/fs_meta.rs` → `fs/meta.rs`; fix imports.
- PR3: Extract `policy/checks.rs` from `preflight.rs` and wire `api/apply`/`api/preflight`.
- PR4: Generalize `api/apply/gating.rs` → `policy/gating.rs`; consume from `preflight` and `apply`.
- PR5: Move `rescue.rs` → `policy/rescue.rs`; re-export.
- PR6: Restructure `adapters/` into submodules; re-export from `adapters/mod.rs`.
- PR7: Move YAML exporter to `api/preflight/yaml.rs`; keep `crate::preflight::to_yaml` as re-export.
- PR8: Clean up deprecated modules and remove `#[path = …]` indirections in `api.rs`.

Each PR compiles independently and runs existing tests:

- No functional changes; only imports, module paths, and re-exports.
- Ensure `cargo check` and test suites pass at each step.

## Expected benefits

- Single source of truth for gating logic → fewer regressions between preflight/apply.
- Stage modules become thinner and easier to reason about.
- Logging concerns centralized; easier to evolve Minimal Facts schema.
- Adapters become discoverable (traits vs defaults) and composable.
- Clear ownership: FS mechanism is policy-free and auditable.

## Follow-ups (nice-to-haves)

- Add `policy/gating_tests.rs` with table-driven tests for `ActionGate` against crafted plans.
- Add `#[cfg(feature = "file-logging")]` examples for `logging::FileJsonlSink` in README.
- Consider `preflight` optional YAML columns toggle if specs evolve.
- Add a crate `internal::dev` module gated behind `cfg(test)` for test-only helpers.

## Appendix A — Files to touch

- `src/lib.rs`: imports/re-exports and deprecation shims.
- `src/api.rs`: stop using `#[path = …]` where possible; `mod` from new locations.
- `src/logging/mod.rs`: re-export `audit::*`.
- `src/api/preflight.rs` → `api/preflight/mod.rs`: update to use `policy/gating` and `fs/meta`.
- `src/api/apply.rs`: import `policy/gating` and `logging/audit` from new locations.

## Appendix B — Compatibility notes

- Public types (`Plan`, `PlanInput`, `ApplyMode`, `PreflightReport`, `ApplyReport`, `SafePath`) remain unchanged.
- `crate::api::errors::{ApiError, ErrorId, exit_code_for, exit_code_for_id_str}` unchanged.
- Old paths remain available for one minor version via `pub use` shims.
