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

## Current Status (2025-09-10 19:30:05+02:00)

- Implemented a reusable workspace crate `switchyard/` and wired the app to depend on it (`Cargo.toml`: `switchyard = { path = "switchyard" }`).
- Migrated safety mechanisms into `switchyard`:
  - `switchyard/src/symlink.rs`: pure mechanism for `backup_path`, `is_safe_path`, atomic idempotent `replace_file_with_symlink`, and `restore_file` (no product logging/UI).
  - `switchyard/src/fs_ops.rs`: `open_dir_nofollow`, `atomic_symlink_swap` using `renameat`, and `fsync_parent_dir`.
  - `switchyard/src/preflight.rs`: read-only checks `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust`.
  - `switchyard/src/api.rs`: minimal façade (`Policy`, `Plan`/`Action`, `ApplyMode`, `FactsEmitter`, `AuditSink`, and `Switchyard::{plan, preflight, apply}`).
- Product now delegates to `switchyard`:
  - `src/system/fs_checks.rs` calls `switchyard::preflight::*` and maps messages to product `Error`.
  - `src/symlink/ops.rs` calls `switchyard::symlink::*` for mechanisms while preserving product audit and progress UI logs.
  - `src/experiments/util.rs` can run through the façade (`Switchyard`) under feature `switchyard` (now enabled by default).
- Build and tests pass (`cargo check`, `cargo test`).

Behavior/compatibility:
- Dry-run semantics preserved (façade maps to dry-run; audit sink still controlled by app env `OXIDIZR_DRY_RUN`).
- Audit/facts continue via product logging; `switchyard` itself contains no global logging.

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
  - `types.rs`: `SafePath` (validated path newtype), `ActionId`, `PlanId`, `CorrelationId`.
  - `ownership.rs`: `OwnershipOracle` trait (e.g., pacman -Qo abstraction).
  - `locking.rs`: advisory per-target/process-wide locking primitives.
  - `schema.rs`: facts/audit envelope and schema versioning.
  - `errors.rs`, `config.rs`.
- Phase 1 (Option A): implement as internal module under `src/switchyard/` and later promote to workspace crate.

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

## Refactor Plan (stepwise) — Checklist

- [ ] Freeze behavior with golden fixtures for JSONL audit and human logs across representative flows.
- [x] Introduce façade (adapted): Implemented in external crate `switchyard` as a reusable façade instead of an internal module; app wires to it.
- [x] Feature-flag pilot: `experiments/util::{create_symlinks, restore_targets}` can execute via façade under feature `switchyard` (enabled by default).
- [x] Move internals incrementally:
  - [x] `symlink/ops` → `switchyard/src/symlink.rs` (pure mechanisms; no product logging/UI).
  - [x] `system/fs_checks` → `switchyard/src/preflight.rs` (read-only checks).
  - [x] Added `switchyard/src/fs_ops.rs` (O_NOFOLLOW, renameat, fsync).
  - [ ] Add `rollback` module and wire inverse operations (TBD).
- [ ] Stabilize errors and mapping to app `Error::exit_code()`:
  - [x] Preflight messages mapped back to product `Error` in `src/system/fs_checks.rs`.
  - [ ] Introduce `SwitchyardError` taxonomy in crate and centralize mappings (TBD).
- [x] Flip default to façade; keep shims for safety. (`switchyard` feature enabled by default.)
- [x] Promote to `switchyard/` workspace crate (Option B) with minimal public API.
- [ ] Documentation: Add `docs/switchyard-architecture.md` with diagrams/examples.

Rollback strategy: retain feature flag and shims; revert to legacy path if regressions are detected.

---

## Module TODOs

- [x] /src/switchyard/mod.rs — public facade root module (or lib.rs in external crate)
- [x] /src/switchyard/api.rs — public facade: Plan, Action, Executor
- [ ] /src/switchyard/plan.rs — build action graph from inputs/policy
- [x] /src/switchyard/preflight.rs — read-only validations (permissions, collisions, cycles)
- [x] /src/switchyard/fs_ops.rs — atomic file/dir/link ops; temp/rename patterns
- [x] /src/switchyard/symlink.rs — safe (re)pointing, semantics preserved
- [ ] /src/switchyard/backup.rs — backup/restore strategies; symlink-aware
- [ ] /src/switchyard/policy.rs — allow/deny rules; constraints
- [ ] /src/switchyard/rollback.rs — record & perform undo steps
- [ ] /src/switchyard/facts.rs — structured events (JSON/serde), stable schema
- [ ] /src/switchyard/audit.rs — human-readable audit lines
- [ ] /src/switchyard/errors.rs — error taxonomy + exit code mapping
- [ ] /src/switchyard/config.rs — typed config for policies/paths/modes
- [ ] /src/switchyard/types.rs — common types; path wrappers; invariants

---

## Cross-Document TODOs

- [ ] Align with `AUDIT_CHECKLIST.md` gaps (map to concrete modules/tests)
  - [ ] Transactionality: implement graph-based rollback that is automatic and complete (register inverse ops; test partial failure) in `switchyard/rollback.rs`; add integration tests in `src/switchyard/`.
  - [ ] Auditability: add before/after cryptographic hashes via op buffering and selective hashing (tie-in to Stream C) in `src/logging/audit.rs` and optional `src/logging/attest.rs`.
  - [ ] Audit completeness: ensure logs record actor, versions, provenance, and exit codes centrally via `AuditFields` in `src/logging/audit.rs`; verify call sites in `experiments//*` and `system/worker/*` attach these.
  - [ ] Least Intrusion: introduce preflight diff rendering (no mutation) and document metadata preservation policy (mode/owner/times; xattrs/ACLs/caps policy) — `src/experiments/util.rs` renderer + policy knobs in `switchyard/policy.rs`.
  - [ ] Determinism: stabilize preflight/apply outputs; seed IDs and redact non-deterministic fields in DryRun — assertions in tests under `src/switchyard/`.
  - [ ] Conservatism: make DryRun the default unless `--assume-yes` (Stream B) — `src/cli/{parser.rs,handler.rs}` + docs.
  - [ ] Recovery First: document and test one-step rollback (profile pointer re-flip or `restore_file`) — E2E in `tests/` via `test-orch/`.
  - [ ] Health Verification: add post-commit smoke tests and rollback trigger (Stream A) — `src/experiments/util.rs::run_smoke_tests` and wiring in experiments.
  - [ ] Supply Chain Integrity: provenance enrichment (owner, repo presence) and per-op signature/SBOM-lite (Stream C/D) — `src/system/worker/packages.rs` + `src/logging/audit.rs` + `src/logging/attest.rs`.

- [ ] Integrate `PROJECT_PLANS/` Streams A–E
  - [ ] Stream A — Profiles & Atomic Flip + Canary + Smokes + Backup
    - [ ] Add `rename_active_pointer(active, new_target)` helper (O_NOFOLLOW + `renameat` + fsync) in `src/symlink/ops.rs`.
    - [ ] Profile scaffolding/helpers and tree population via `src/experiments/util.rs` (reuse `create_symlinks`, target profile dirs).
    - [ ] Post-flip `run_smoke_tests()` with auto-rollback on failure; wire into `src/experiments/{coreutils.rs,findutils.rs}`.
    - [ ] CLI: `canary --shell` and `profile --set {gnu|uutils}` in `src/cli/{parser.rs,handler.rs}`.
    - [ ] Security: detect/preserve `security.capability`; optional label relabel on active tree; ACL detect/warn — `src/system/security.rs` (new) + audits.
  - [ ] Stream B — Preflight Plan, Compat Detectors, UX
    - [ ] `--preflight` flag and default-dry-run posture in `src/cli/{parser.rs,handler.rs}`; render plan in human logs via `src/experiments/util.rs`.
    - [ ] Add `assets/compat_matrix.json` and `src/compat/mod.rs` scanners; gate risky flags/semantics.
    - [ ] Adjust human log verbosity per `VERBOSITY.md`; attach plan rows to audit (`AuditFields.artifacts`).
  - [ ] Stream C — Audit Attestation + Docs
    - [ ] Introduce op buffer/finalizer in `src/logging/audit.rs` to emit per-op `audit-<op_id>.jsonl`.
    - [ ] Optional signing in `src/logging/attest.rs` (Ed25519), CLI `audit verify` in `src/cli//*`.
    - [ ] Selective hashing (changed/untrusted targets) with caching keyed by `(dev,inode,mtime,size)`; SBOM-lite JSON.
    - [ ] Operator docs/playbooks: recovery, exit codes, audit verification.
  - [ ] Stream D — Supply Chain Policy + Lock Wait UX
    - [ ] Enforce repo-first/AUR opt-in in `src/system/worker/packages.rs::install_package` with flags `--allow-aur`, `--aur-user`.
    - [ ] Bounded pacman lock wait with periodic progress in `Worker::wait_for_pacman_lock_clear`.
    - [ ] Provenance fields (helper, command, repo presence) attached via `audit_event_fields`.
  - [ ] Stream E — Dependency Footprint Trim
    - [ ] Implement internal `path_search` behind `Worker.which()` in `src/system/worker/fs_ops.rs`; feature-gate external `which` crate in `Cargo.toml`.

- [ ] Cross-cutting — Tests & CI Pipeline
  - [ ] Add YAML suites under `tests/` for: preflight-only, profile flip + rollback + smokes, AUR gating behaviors, capability/labels preservation, audit verify/SBOM.
  - [ ] Expand Rust tests: `src/switchyard/` (rollback, determinism, SafePath), `src/symlink/ops.rs` (atomicity invariants), `src/compat/` (matrix matching).
  - [ ] Ensure `test-orch/` orchestrators remain the single runner; namespace caches per distro (already implemented) and add xfail markers where infra-limited.

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
