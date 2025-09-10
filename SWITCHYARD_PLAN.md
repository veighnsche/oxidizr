# Switchyard Extraction Plan

Planning only — no behavior changes implied by this document. This plan extracts the safety “switchyard” into a reusable, minimal API while preserving current behavior.

## Goal

- Extract and modularize safety features into a reusable package named `switchyard` with a minimal, stable API.
- Target only the current repo’s `src/` (entrypoint `src/main.rs`) initially; design for later promotion to a separate crate.

## Scope (included)

- Symlink creation/repointing/replacement (atomic + idempotent)
- Backups & restores (symlink-aware vs regular file)
- Preflight checks (permissions, path safety, existence, collisions, dry-run feasibility)
- Policy-driven allow/deny rules and guardrails
- Rollback strategy & transaction boundaries
- Fact/telemetry emission (structured, machine-readable)
- Audit logging (human-readable; who/what/when; before/after; dry-run vs apply)
- Error taxonomy & exit codes
- Dry-run mode with deterministic output
- Safety interlocks (“stop” conditions)
- Filesystem metadata handling where currently implemented (permissions; hooks for xattrs/ACLs/caps)
- Centralize scattered safety code from `system/*`, `symlink/*`, `logging/*`, `checks/*`, and call sites in `experiments/*`.

Out of scope (for now): new safety features, packaging/SBOM/signatures/initramfs, or user-visible behavioral changes beyond existing audit-noted bug fixes.

---

## Inventory Map (current code)

- `src/symlink/ops.rs`
  - `replace_file_with_symlink(source, target, dry_run)`: O_NOFOLLOW parent open, TOCTOU-aware checks, symlink-aware backup, temp symlink + `renameat`, parent `fsync`, idempotent, dry-run path, structured audit.
  - `restore_file(target, dry_run, force_best_effort)`: backup rename via `renameat` under O_NOFOLLOW, parent `fsync`, dry-run path, best-effort.
  - `backup_path()`, `is_safe_path()`, helpers `open_dir_nofollow`, `fsync_parent_dir`, `atomic_symlink_swap`.
- `src/system/fs_checks.rs`
  - `ensure_mount_rw_exec()`, `check_immutable()`, `check_source_trust()` (world-writable, non-root owner, HOME, exec mount); used before symlink ops.
- `src/system/worker/fs_ops.rs`
  - `Worker::{replace_file_with_symlink, restore_file}` compose preflights and policy, then delegate to `symlink::ops`.
- `src/system/lock.rs`
  - `acquire()` process lock to serialize mutating commands.
- `src/logging/audit.rs`, `src/logging/init.rs`
  - Structured JSONL audit (target="audit"), human logs, dry-run gates file sink via `OXIDIZR_DRY_RUN`.
- `src/experiments/util.rs`
  - `create_symlinks()`, `restore_targets()` loops with audit + progress UI; call Worker methods.
- `src/experiments/{coreutils,findutils,checksums}.rs`
  - Orchestration around linking/restoring; policies like `PRESERVE_BINS` for checksum applets; state persistence.
- `src/experiments/mod.rs`
  - Repo gating (`extra_repo_available`, `repo_has_package`), `relink_managed()`.
- `src/main.rs`, `src/cli/handler.rs`
  - Entrypoint, dry-run env, process lock, root enforcement, CLI orchestration.
- `src/state/mod.rs`, `src/error.rs`
  - State persistence; stable error → exit code mapping.

### Known cross-layer couplings to decouple

- `symlink::ops` directly logs and touches UI progress decisions; should emit via injected interfaces.
- Ownership verification uses `pacman -Qo` (in `packages.rs`); keep as app policy or abstract behind a trait.

---

## Dependency Graph (high-level)

- `main.rs` → `cli::handle_cli()` → `system::lock::acquire()` → constructs `Worker` (carries dry-run/policy flags).
- `experiments/*` → `experiments::util::{create_symlinks, restore_targets}` → `Worker::{replace_file_with_symlink, restore_file}`.
- `Worker::…` → `system::fs_checks::{ensure_mount_rw_exec, check_immutable, check_source_trust}` + ownership policy → `symlink::ops::{replace_file_with_symlink, restore_file}`.
- `symlink::ops` ↔ `logging::audit` (direct today) and uses `ui::progress` for INFO suppression.
- `experiments/mod.rs` gating → `Worker::{extra_repo_available, repo_has_package, check_installed}`.
- Cross-cutting: `error::Error` + `exit_code()`; `state::*`.

---

## Proposed Module Layout (Option B preferred; start with internal Option A)

- `switchyard/` (new)
  - `api.rs`: façade (Plan, Action, ApplyMode, PreflightReport, ApplyReport; `Switchyard`, `Policy`, traits `FactsEmitter`, `AuditSink`).
  - `plan.rs`: translate inputs + policy → deterministic actions; pure.
  - `preflight.rs`: read-only validations (rw/exec, immutable, trust, collisions, path safety).
  - `fs_ops.rs`: O_NOFOLLOW dir open, atomic `renameat`, parent `fsync`, mkdirs.
  - `symlink.rs`: idempotent symlink semantics; link-aware backup/restore; uses `fs_ops`.
  - `backup.rs`: backup layout & metadata preservation.
  - `policy.rs`: allow/deny rules; ownership checks (abstracted); allowed roots.
  - `rollback.rs`: undo steps & recovery.
  - `facts.rs`: structured event schema; emits via `FactsEmitter`.
  - `audit.rs`: human-readable lines via `AuditSink`.
  - `errors.rs`, `config.rs`, `types.rs`.
- Phase 1 (Option A): implement as internal module under `src/switchyard/` and later promote to workspace crate.

---

## Public API Sketch

```rust
pub struct Switchyard<E: FactsEmitter, A: AuditSink> { /* facts, audit, policy */ }

pub struct Policy {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
    pub strict_ownership: bool,
    pub force_untrusted_source: bool,
    pub force_restore_best_effort: bool,
}

pub enum ApplyMode { DryRun, Commit }

pub struct PlanInput { pub link: Vec<LinkRequest>, pub restore: Vec<RestoreRequest> }
pub struct LinkRequest { pub source: PathBuf, pub target: PathBuf }
pub struct RestoreRequest { pub target: PathBuf }

pub struct Plan { pub actions: Vec<Action> }

pub enum Action {
    EnsureSymlink { source: PathBuf, target: PathBuf },
    RestoreFromBackup { target: PathBuf },
}

pub struct PreflightReport { pub ok: bool, pub warnings: Vec<String>, pub stops: Vec<String> }
pub struct ApplyReport { pub executed: Vec<Action>, pub skipped: Vec<Action>, pub duration_ms: u64, pub errors: Vec<SwitchyardError> }

pub trait FactsEmitter { fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: serde_json::Value); }
pub trait AuditSink { fn log(&self, level: log::Level, msg: &str); }

impl<E: FactsEmitter, A: AuditSink> Switchyard<E, A> {
    pub fn new(facts: E, audit: A, policy: Policy) -> Self;
    pub fn plan(&self, input: PlanInput) -> Plan;           // deterministic, pure
    pub fn preflight(&self, plan: &Plan) -> PreflightReport; // read-only
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport; // atomic/idempotent
}
```

---

## Safety Requirements

- Preserve symlink semantics (no materialization).
- Atomicity: temp + `renameat`, parent `fsync`.
- Idempotence: re-running plans converges; already-correct symlink is a no-op.
- Rollbackability: record inverse ops before mutation; partial-failure recovery.
- Least privilege; explicit policy flags for overrides.
- Path safety: normalize; enforce allowed roots; reject traversal.
- Metadata handling: preserve permissions today; document gaps for xattrs/ACLs/caps.
- Observability: every decision/action emits facts + human log line.

---

## Refactor Plan (stepwise)

1) Freeze behavior with golden fixtures for JSONL audit and human logs across representative flows.
2) Introduce façade internally (Option A) with pass-through to existing Worker/symlink paths; inject `FactsEmitter`/`AuditSink` adapters to current logging.
3) Feature-flag pilot: route `experiments/util::{create_symlinks, restore_targets}` through façade under a feature flag; parity check against goldens.
4) Move internals incrementally: `symlink/ops` → `switchyard/symlink`; `system/fs_checks` → `switchyard/preflight`; add `fs_ops`, `rollback`.
5) Stabilize errors and mapping to app `Error::exit_code()`.
6) Flip default to façade; deprecate Worker shims; tighten visibility.
7) Promote to `switchyard/` workspace crate (Option B) without API changes.
8) Documentation: `docs/switchyard-architecture.md` with diagrams/examples.

Rollback: retain feature flag and shims each step; revert to legacy wiring if regressions.

---

## Test Plan

- Unit: symlink idempotence; backup strategies; fs_ops atomic swap; preflight rules; error mapping.
- Property: idempotence (repeated apply is no-op); path normalization determinism.
- Integration: plan → preflight → apply for DryRun/Commit; partial failure with rollback; symlink edge cases (nested, broken).
- Golden fixtures: JSONL facts and human logs stable/deterministic.

---

## Acceptance Criteria

- All switching safety behind `switchyard` façade.
- Public API documented, minimal, stable.
- Behavior parity preserved (errors and exit codes) with passing tests.
- Deterministic facts/audit outputs, documented.
- Legacy helpers removed/deprecated; app calls façade.

---

## Architectural Intent

- Policy vs. Mechanism separation; transaction semantics (plan → preflight → apply); deterministic dry-run; minimal API; human-auditable outputs.

## Migration Notes

- Ownership policy verification via `pacman -Qo` remains app-side or behind a trait.
- Preserve existing guardrails (e.g., checksum PRESERVE_BINS; AUR/extra gating; findutils-before-coreutils) in experiments layer.
