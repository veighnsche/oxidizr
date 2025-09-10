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

- `switchyard/src/` (new)
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
  - `types.rs`: `SafePath` (validated path newtype), `ActionId`, `PlanId`, `CorrelationId`.
  - `ownership.rs`: `OwnershipOracle` trait (e.g., pacman -Qo abstraction).
  - `locking.rs`: advisory per-target/process-wide locking primitives.
  - `schema.rs`: facts/audit envelope and schema versioning.
  - `errors.rs`, `config.rs`.

---

## Public API Sketch

```rust
use std::path::PathBuf;
use uuid::Uuid;

pub struct Switchyard<E: FactsEmitter, A: AuditSink, O: OwnershipOracle> { /* facts, audit, policy, ownership */ }

pub type PlanId = Uuid;
pub type ActionId = Uuid;

#[derive(Clone)]
pub struct SafePath(PathBuf); // validated at construction; see Path Safety section

pub struct Policy {
    pub allow_roots: Vec<PathBuf>,
    pub forbid_paths: Vec<PathBuf>,
    pub strict_ownership: bool,
    pub force_untrusted_source: bool,
    pub preserve_mode: bool,
    pub preserve_owner: bool,
    pub preserve_times: bool,
    pub preserve_xattrs: bool,
}

pub enum ApplyMode {
    DryRun,
    Commit { allow_best_effort_restore: bool },
}

pub struct PlanInput { pub link: Vec<LinkRequest>, pub restore: Vec<RestoreRequest> }
pub struct LinkRequest { pub source: SafePath, pub target: SafePath }
pub struct RestoreRequest { pub target: SafePath }

pub struct Plan { pub id: PlanId, pub actions: Vec<Action> }

pub struct Action {
    pub id: ActionId,
    pub kind: ActionKind,
    pub depends_on: Vec<ActionId>,
    pub undo: InverseAction,
}

pub enum ActionKind {
    EnsureSymlink { source: SafePath, target: SafePath },
    RestoreFromBackup { target: SafePath },
}

pub enum InverseAction {
    RemoveSymlink { target: SafePath, restore_backup: bool },
    RevertRestore { target: SafePath },
}

pub struct PreflightReport {
    pub plan_id: PlanId,
    pub ok: bool,
    pub warnings: Vec<String>,
    pub stops: Vec<String>,
}

pub struct ApplyReport {
    pub plan_id: PlanId,
    pub executed: Vec<ActionId>,
    pub skipped: Vec<ActionId>,
    pub duration_ms: u64,
    pub errors: Vec<SwitchyardError>,
}

pub trait FactsEmitter {
    fn emit(&self, subsystem: &str, event: &str, decision: &str, fields: serde_json::Value);
}

pub trait AuditSink {
    fn log(&self, level: log::Level, msg: &str);
}

pub trait OwnershipOracle: Send + Sync {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo, SwitchyardError>;
}

pub struct OwnershipInfo {
    pub uid: u32,
    pub gid: u32,
    pub package: Option<String>,
}

impl<E: FactsEmitter, A: AuditSink, O: OwnershipOracle> Switchyard<E, A, O> {
    pub fn new(facts: E, audit: A, policy: Policy, ownership: std::sync::Arc<O>) -> Self;
    pub fn plan(&self, input: PlanInput) -> Plan;            // deterministic, pure (stable order)
    pub fn preflight(&self, plan: &Plan) -> PreflightReport; // read-only
    pub fn apply(&self, plan: &Plan, mode: ApplyMode) -> ApplyReport; // atomic/idempotent, crash-consistent
}

---

## Action Graph & Rollback

- Plan is a DAG: each `Action` has a stable `ActionId` and `depends_on: Vec<ActionId>`.
- Deterministic execution: topological sort with stable tie-breaker (lexicographic by `ActionId`).
- Rollback edges: each `Action` has an `undo: InverseAction`; register undo intent before any mutation.
- Failure handling: on first failure or signal, attempt inverse actions for all completed nodes in reverse execution order.
- Parallelism: DAG enables future parallel apply; initially keep sequential to reduce risk.

## Path Safety (SafePath)

- Introduce `SafePath` newtype validated at construction time:
  - canonicalize without following symlinks for the final hop (no_follow parent open).
  - confined to `Policy::allow_roots`; reject traversal (`..`, absolute escapes).
  - explicit policy for non-UTF-8 names; keep bytes internally, optional UTF-8 rendering for logs.
- Keep `PathBuf` out of public API where possible; convert to `SafePath` at boundaries.

## Filesystem Semantics (fsync/EXDEV)

- Write strategy for mutating ops:
  1) write temp in target dir → `fsync(temp)`
  2) `renameat` over target atomically
  3) `fsync(parent dir)`
- Directory creation: `mkdir` (and intermediate) then `fsync(parent of created)`.
- Cross-device rename (EXDEV): copy to temp → `fsync(temp)` → atomic swap → `fsync(parent)`; document performance and journal implications.
- Remote/NFS: disallow by default (preflight stop) or warn via policy knob; note semantics caveats.
- No assumptions about monotonic timestamps; tests must not rely on mtime ordering.

## Interruption & Locking

- Interruption: guarantee crash-consistency for each step; on SIGINT/SIGTERM/kill -9, state is either pre- or post-rename, never torn.
- Register undo before mutation; emit attempt/start facts prior to side effects.
- Locking: retain `system::lock::acquire()` process-wide lock; scope documented.
- Add advisory per-target lock in switchyard to enable safe batching; default off until validated.

## Error Taxonomy & Exit Codes

- Stable categories mapped 1:1 to process exit codes (documented table maintained):
  - `E_POLICY`, `E_PREFLIGHT`, `E_IO_ATOMICITY`, `E_PATH_SAFETY`, `E_ROLLBACK_PARTIAL`.
- Machine-parsable cause chains include source OS error codes and contextual fields.

## Facts & Audit Schema Versioning

- Event envelope fields: `schema_version`, `plan_id`, `action_id`, `correlation_id`, `mode`, `dry_run`.
- Deterministic ordering and stable serialization; golden fixtures assert full records, not substrings.

## Deterministic Dry-run

- Stable sort of actions and deterministic IDs; seeded determinism where randomness exists.
- Timing fields: use monotonic deltas or redact in DryRun; ensure byte-for-byte stability.

## Restore Semantics & Collision Policy

- Define `RestoreMode::{Strict, BestEffort}` and a collision policy for when target exists with unexpected type (file/symlink/dir).
- In `Commit { allow_best_effort_restore }`, permit best-effort when policy allows; default to Strict.

## Config Discoverability

- `Policy` originates from CLI/app; switchyard performs no global env reads. All inputs explicit.

## Observability Contract

- Every action emits: planned (preflight), attempt, success|skip|fail, plus final `plan_summary`.

## Metadata Preservation Policy

- `Policy` includes knobs: `{ preserve_mode, preserve_owner, preserve_times, preserve_xattrs }`.
- Document current gaps (xattrs/ACLs/posix caps) and behavior (warn vs stop) when not supported.

## Safety Requirements

- Preserve symlink semantics (no materialization).
- Atomicity: temp + `renameat`, parent `fsync`; EXDEV fallback defined.
- Idempotence: re-running plans converges; already-correct symlink is a no-op.
- Rollbackability: record inverse ops before mutation; partial-failure recovery.
- Least privilege; explicit policy flags for overrides.
- Path safety via `SafePath`: normalize; enforce allowed roots; reject traversal.
- Metadata handling: preserve permissions today; document gaps for xattrs/ACLs/caps.
- Observability: every decision/action emits facts + human log line.

---

## Test Plan

- Unit: symlink idempotence; backup strategies; fs_ops atomic swap; preflight rules; `SafePath` validation; EXDEV fallback.
- Property: idempotence (repeated apply is no-op); path normalization and confinement; stable action ordering.
- Integration: plan → preflight → apply for `DryRun` and `Commit { allow_best_effort_restore }`; partial failure with rollback; symlink edge cases (nested, broken, nested symlink).
- Signals: inject SIGINT mid-apply; verify rollback log and crash-consistency.
- Filesystems: run on tmpfs + ext4 + btrfs (CI containers for two of these at minimum).
- Edge cases: broken symlink; nested symlink; read-only bind mount; EXDEV; long paths; non-UTF-8 names.
- Golden fixtures: JSONL facts and human logs byte-for-byte stable across runs.
- Crash-consistency: simulate power loss at each mutation step boundary and verify post-recovery invariants.

---

## Acceptance Criteria

- All switching safety behind `switchyard` façade.
- Public API documented, minimal, stable, with `SafePath`, `ActionId`, and DAG semantics.
- Behavior parity preserved; stable error taxonomy mapped 1:1 to exit codes.
- Crash-consistency demonstrated via simulated interruptions and power-loss tests.
- Deterministic dry-run with byte-for-byte stable facts/audit outputs.
- Path confinement property tests for `SafePath` pass; no escapes beyond allowed roots.
- EXDEV and symlink-preservation tests pass; no “link becomes file” regressions.
- Locking semantics documented; optional per-target advisory locks gated behind feature.
- Legacy helpers removed/deprecated; app calls façade.

---

## Architectural Intent

- Policy vs. Mechanism separation; transaction semantics (plan → preflight → apply); deterministic dry-run; minimal API; human-auditable outputs.

## Migration Notes

- Ownership policy verification via `pacman -Qo` remains app-side or behind a trait.
- Preserve existing guardrails (e.g., checksum PRESERVE_BINS; AUR/extra gating; findutils-before-coreutils) in experiments layer.
