# Public API Surface Audit
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Inventory all publicly exposed items across `fs`, `types`, `logging`, `policy`, `preflight`, `api`, and `adapters`; classify stability and propose cleanup/re-exports.  
**Inputs reviewed:** SPEC §3 (Public Interfaces), SPEC §2 (Requirements), PLAN/10-types-traits.md, PLAN/12-api-module.md, PLAN/40-facts-logging.md, CODE under `src/**`  
**Affected modules:** `src/lib.rs`, `src/api.rs`, `src/api/errors.rs`, `src/types/**`, `src/fs/**`, `src/logging/**`, `src/policy/**`, `src/preflight.rs`, `src/adapters/**`, `src/constants.rs`

## Summary
- Switchyard exposes a clean facade via `src/lib.rs` (`pub mod …; pub use api::*;`) with focused core types (`SafePath`, `Plan`, `ApplyMode`, reports) and the `Switchyard` orchestrator.
- Low-level FS atoms (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`) are publicly re-exported from `fs/` and should be considered Internal; they are footguns for integrators and duplicate internal invariants.
- Adapters’ trait surfaces (LockManager, OwnershipOracle, SmokeTestRunner, Attestor, PathResolver) are stable; default impls (`FileLockManager`, `FsOwnershipOracle`, `DefaultSmokeRunner`) are provisional.
- Logging sinks (`FactsEmitter`, `AuditSink`, `JsonlSink`) and redaction helpers are public but expected to evolve; mark as Provisional and document backward-compat policy.

## Inventory / Findings

Legend: Stability = Stable | Provisional | Internal (consider private)

- Core facade and orchestrator
  - Item: `Switchyard<E,A>` struct — Module: `api` — Stability: Stable — Ref: `src/api.rs`
  - Item: `plan`, `preflight`, `apply`, `plan_rollback_of` — Module: `api` — Stability: Stable — Ref: `src/api.rs`
  - Item: `errors::ApiError`, `errors::ErrorId`, `errors::exit_code_for*` — Module: `api/errors.rs` — Stability: Stable — Ref: `src/api/errors.rs`

- Types (re-exported at crate root via `pub use types::*;`)
  - `ApplyMode`, `PlanInput`, `Action`, `Plan` — Stable — `src/types/plan.rs`
  - `PreflightReport`, `ApplyReport` — Stable — `src/types/report.rs`
  - `SafePath` — Stable — `src/types/safepath.rs`
  - `types::errors::{ErrorKind, Error, Result}` — Provisional (internal to lib users) — `src/types/errors.rs`
  - `types::ids::{plan_id, action_id}` — Provisional — `src/types/ids.rs`

- Policy
  - `Policy` — Stable — `src/policy/config.rs`
  - Presets: `Policy::production_preset*`, `Policy::coreutils_switch_preset*` — Stable — `src/policy/config.rs`
  - `policy::rescue::{verify_rescue*, RescueStatus}` — Provisional — `src/policy/rescue.rs`

- Adapters (re-exported at crate root via `pub use adapters::*;`)
  - Traits: `LockManager`, `LockGuard`, `OwnershipOracle`, `OwnershipInfo`, `Attestor`, `Signature`, `PathResolver`, `SmokeTestRunner`, `SmokeFailure` — Stable — `src/adapters/**`
  - Default impls: `FileLockManager`, `FsOwnershipOracle`, `DefaultSmokeRunner` — Provisional — `src/adapters/**`
  - Shim: `adapters::lock_file::*` (compat alias) — Internal — `src/adapters/mod.rs`

- Filesystem helpers (re-exported via `pub use` in `fs/mod.rs`)
  - High-level: `replace_file_with_symlink`, `restore_file`, `restore_file_prev` — Stable — `src/fs/swap.rs`, `src/fs/restore.rs`
  - Backup helpers: `backup_path_with_tag`, `create_snapshot`, `has_backup_artifacts` — Provisional — `src/fs/backup.rs`
  - Metadata: `detect_preservation_capabilities`, `kind_of`, `resolve_symlink_target`, `sha256_hex_of` — Provisional — `src/fs/meta.rs`
  - Mount inspection: `ProcStatfsInspector`, `ensure_rw_exec` — Provisional — `src/fs/mount.rs`
  - Low-level atoms: `open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir` — Internal — `src/fs/atomic.rs`
  - Path check: `is_safe_path` — Internal — `src/fs/paths.rs`

- Preflight module
  - `preflight::checks::{ensure_mount_rw_exec, check_immutable, check_source_trust}` — Provisional — `src/preflight/checks.rs`
  - `preflight::to_yaml(report)` — Stable — `src/preflight/yaml.rs`

- Logging
  - `logging::{FactsEmitter, AuditSink, JsonlSink}` — Provisional — `src/logging/facts.rs`
  - `logging::{redact_event, TS_ZERO, ts_for_mode}` — Provisional — `src/logging/redact.rs`
  - Audit emitters in `logging/audit.rs` are crate-internal (not re-exported) — Internal

- Constants
  - `constants::{DEFAULT_BACKUP_TAG, TMP_SUFFIX, FSYNC_WARN_MS, LOCK_POLL_MS, DEFAULT_LOCK_TIMEOUT_MS, NS_TAG, RESCUE_MUST_HAVE, RESCUE_MIN_COUNT}` — Provisional — `src/constants.rs`

## Recommendations
- Tighten FS surface
  1. Mark `fs::open_dir_nofollow`, `fs::atomic_symlink_swap`, `fs::fsync_parent_dir`, and `fs::paths::is_safe_path` as `pub(crate)` and stop re-exporting them from `fs::mod.rs`. Keep only `replace_file_with_symlink`, `restore_file`, `restore_file_prev`, and metadata/backup helpers public.
  2. Document `replace_file_with_symlink` invariants (preconditions and guarantees) in Rustdoc, pointing to SPEC §2.1 and §2.10.

- Logging facade
  3. Keep `FactsEmitter`/`AuditSink` public but mark as Provisional; add a stability note in `logging/mod.rs` Rustdoc referencing SPEC §5 for schema versioning.

- Preflight naming
  4. Avoid duplicate naming with `fs::mount::ensure_rw_exec` vs `preflight::checks::ensure_mount_rw_exec`. Either:
     - Re-export the `fs::mount` helper into preflight and remove the wrapper, or
     - Rename the preflight wrapper to `ensure_mount_is_rw_exec_preflight`.

- Shims and aliases
  5. Mark `adapters::lock_file::*` as deprecated in Rustdoc with a pointer to `adapters::lock::file::*` and a removal timeline.

## Risks & Trade-offs
- Reducing public FS atoms may break power users. Mitigate with a deprecation window and release notes.
- Stabilizing logging may constrain schema evolution; mitigate via versioned schema (already present) and additive-only changes.

## Spec/Docs deltas
- Add a note to SPEC §3.1/§3.2 stating that low-level FS atoms are intentionally not part of the stable public API; only high-level operations are.
- Update module-level Rustdocs to reflect stability classification.

## Acceptance Criteria
- Public docs reflect which items are Stable/Provisional/Internal.
- Low-level FS atoms are no longer publicly re-exported or are clearly marked Internal.
- Preflight helper naming conflict resolved or documented.

## References
- SPEC: §3 Public Interfaces; §2.1 Atomicity; §2.10 Filesystems & Degraded; §5 Audit Facts
- PLAN: 10-types-traits.md; 12-api-module.md; 40-facts-logging.md; 45-preflight.md; 50-locking-concurrency.md
- CODE: `src/lib.rs`, `src/api.rs`, `src/api/errors.rs`, `src/types/**`, `src/fs/**`, `src/logging/**`, `src/policy/**`, `src/preflight.rs`, `src/adapters/**`
