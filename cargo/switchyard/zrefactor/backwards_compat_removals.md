# Backwards Compatibility Shims and Deprecated Items — Removal Plan (breaking)

This document catalogs all backward-compatibility shims and deprecated or legacy surfaces discovered in the codebase, with concrete removal actions and acceptance criteria. The goal is to remove all shims and land a clean, typed, and centralized API.

## Summary of removals

- __Adapters shim__: `switchyard::adapters::lock_file::*` (deprecated)
- __Top-level rescue re-export__: `pub use policy::rescue` in `src/lib.rs` (deprecated)
- __FS low-level atoms re-export__: `open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir` (deprecated re-exports)
- __Legacy logging helpers__: `logging::audit.rs::emit_*` family (not marked deprecated, but slated for removal)
- __Non-idiomatic API module shim__: `src/api.rs` using `#[path]` includes
- __Gating duplication__: ad-hoc gating in API/preflight; duplicate logic outside policy evaluator
- __Policy legacy flat fields__: boolean flags slated for replacement by grouped types/enums
- __Legacy presets names__: `Policy::production_preset()`, `coreutils_switch_preset()` and `apply_*_preset()` variants
- __Obsolete file__: `src/api/telemetry.rs` (if present)
- __Monolithic FS files__: `src/fs/backup.rs`, `src/fs/restore.rs` to be replaced by submodules

---

## 1) Adapters shim: `switchyard::adapters::lock_file::*`

- File: `src/adapters/mod.rs`
- Code:

  ```rust
  // Compatibility shim for old path switchyard::adapters::lock_file::FileLockManager
  #[deprecated(note = "Deprecated shim: use `switchyard::adapters::lock::file::*` instead. This `lock_file` module will be removed in 0.2.")]
  pub mod lock_file { pub use super::lock::file::*; }
  ```

- Action:
  - Update all imports to `switchyard::adapters::FileLockManager` (or `switchyard::adapters::lock::file::*`).
  - Remove the `lock_file` module.
- Acceptance:
  - `grep -R "adapters::lock_file::" src/ tests/` returns 0.
  - `cargo test` passes.

/// remove this shim: `src/adapters/mod.rs::lock_file`

## 2) Top-level rescue re-export

- File: `src/lib.rs`
- Code:

  ```rust
  #[deprecated(note = "Deprecated facade re-export: use `switchyard::policy::rescue` instead. This top-level alias will be removed in 0.2.")]
  pub use policy::rescue; // compatibility re-export
  ```

- Action:
  - Update in-tree imports to `switchyard::policy::rescue`.
  - Remove the top-level `pub use`.
- Acceptance:
  - `grep -R "use switchyard::rescue" src/ tests/` returns 0.
  - `cargo check` passes.

/// remove this re-export: `src/lib.rs` top-level `pub use policy::rescue`

## 3) FS low-level atoms re-export

- File: `src/fs/mod.rs`
- Code:

  ```rust
  #[deprecated(note = "Low-level FS atoms are internal: prefer high-level API. This re-export will be removed in 0.2.")]
  pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};
  ```

- Action:
  - Stop re-exporting the atoms publicly; restrict visibility to `pub(crate)` inside `fs::atomic`.
  - Update consumers to use high-level safe APIs: `replace_file_with_symlink`, `restore_file`, etc.
- Acceptance:
  - Low-level atoms are referenced only within the filesystem module tree: `rg -n "open_dir_nofollow|atomic_symlink_swap|fsync_parent_dir" cargo/switchyard/src | rg -v "cargo/switchyard/src/fs/"` returns 0.
  - No public re-exports at the fs module root: `rg -n "^\s*pub\s+use\s+atomic::|^\s*pub\(crate\)\s+use\s+atomic::" cargo/switchyard/src/fs/mod.rs -S` returns 0 after cleanup.
  - `cargo test` passes.

/// remove public re-exports: `src/fs/mod.rs` for low-level atoms

## 4) Legacy logging helpers (`emit_*` family)

- File: `src/logging/audit.rs`
- Items: `emit_plan_fact`, `emit_preflight_fact(_ext)`, `emit_apply_attempt`, `emit_apply_result`, `emit_prune_result`, `emit_summary(_extra)`, `emit_rollback_step`.
- Action:
  - Replace with `StageLogger`/`EventBuilder` facade (see `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`).
  - Make legacy helpers private or delete them after all call sites are migrated.
- Acceptance:
  - `grep -R "audit::emit_" src/` returns 0 outside `src/logging/`.
  - Golden tests on emitted JSON pass under the facade.

/// remove legacy helpers after migration: `src/logging/audit.rs::emit_*`

## 5) Non-idiomatic API module shim (`#[path]` includes)

- File: `src/api.rs`
- Issue: uses `#[path = "api/..."]` to include modules.
- Action:
  - Move to directory module: `src/api.rs` → `src/api/mod.rs`, declare submodules via `mod ...;`.
  - Update imports accordingly.
- Acceptance:
  - `grep -R "#\[path\]" src/api` returns 0.
  - `cargo check && cargo test` pass.

/// remove this file after migration: `src/api.rs`

## 6) Gating duplication outside policy evaluator

- Files: `src/api/preflight/mod.rs`, any apply-time gating code; `src/policy/gating.rs` (ensure one canonical evaluator only).
- Action:
  - Centralize gating under `policy::gating::evaluate_action`.
  - Preflight/apply must call the evaluator; delete any duplicate/ad-hoc gating logic.
- Acceptance:
  - `grep -R "evaluate_action\(" src/api` shows calls only; no direct low-level checks in API.
  - Unit/integration tests confirm parity of decisions between preflight and apply.

/// remove duplicate gating logic: in `src/api/preflight/mod.rs` and apply

## 7) Policy legacy flat fields (to be replaced)

- File: `src/policy/config.rs`
- Legacy fields include (non-exhaustive): `allow_roots`, `forbid_paths`, `strict_ownership`, `force_untrusted_source`, `force_restore_best_effort`, `allow_degraded_fs`, `disable_auto_rollback`, `backup_tag`, `override_preflight`, `require_preservation`, `require_rescue`, `rescue_exec_check`, `rescue_min_count`, `allow_suid_sgid_mutation`, `allow_hardlink_breakage`, `require_backup_durability`, `require_sidecar_integrity`, `require_lock_manager`, `allow_unlocked_commit`, `require_smoke_in_commit`, `extra_mount_checks`, `capture_restore_snapshot`.
- Action:
  - Replace with grouped types/enums per `zrefactor/policy_refactor.INSTRUCTIONS.md` (Scope, Rescue, Risks, Durability, ApplyFlow, Governance, Backup).
  - Remove legacy booleans and migrate call sites.
- Acceptance:
  - `grep -R "allow_suid_sgid_mutation\|allow_hardlink_breakage\|force_untrusted_source\|force_restore_best_effort\|allow_degraded_fs\|override_preflight\|require_preservation\|require_rescue\|require_backup_durability\|require_sidecar_integrity\|require_lock_manager\|allow_unlocked_commit\|require_smoke_in_commit\|rescue_exec_check\|rescue_min_count\|capture_restore_snapshot" src/` returns 0 (outside the migration commit replacing with grouped fields).

/// remove legacy flat fields after migrating to grouped Policy

## 8) Legacy preset names (`*_preset`)

- File: `src/policy/config.rs`
- Items: `Policy::production_preset()`, `Policy::coreutils_switch_preset()`, and `apply_*_preset()` variants.
- Action:
  - Introduce `Policy::production()`, `Policy::coreutils_switch()`, `Policy::permissive_dev()` in `profiles.rs`.
  - Migrate callers; remove old `*_preset` methods.
- Acceptance:
  - `grep -R "_preset\(" src/` returns 0.

/// remove legacy preset methods after adding `profiles.rs`

## 9) Obsolete telemetry file (if present)

- File: `src/api/telemetry.rs`
- Action:
  - Delete after migrating to logging facade under `src/logging/`.
- Acceptance:
  - File removed; no imports.

/// remove this file: `src/api/telemetry.rs` (if present)

## 10) Monolithic FS files replaced by submodules

- Files: `src/fs/backup.rs`, `src/fs/restore.rs`
- Action:
  - Extract into `src/fs/backup/{mod.rs,snapshot.rs,sidecar.rs,index.rs}` and `src/fs/restore/{mod.rs,types.rs,selector.rs,idempotence.rs,integrity.rs,steps.rs,engine.rs}`.
  - Update re-exports; migrate tests; delete monolith files.
- Acceptance:
  - `grep -R "src/fs/backup.rs\|src/fs/restore.rs"` returns 0 in code references.
  - `cargo test` passes.

/// remove these files after extraction: `src/fs/backup.rs`, `src/fs/restore.rs`

---

## CI guardrails to prevent regressions

- __No shims in adapters__: forbid `adapters::lock_file::` in tree.
- __No top-level rescue alias__: forbid `use switchyard::rescue`.
- __No public FS atoms__: forbid public re-exports of `open_dir_nofollow|atomic_symlink_swap|fsync_parent_dir`.
- __No `#[path]` under API__: forbid `#[path]` usage in `src/api/**`.
- __No legacy logging calls__: forbid `audit::emit_` outside `src/logging/`.
- __No ad-hoc gating in API__: require use of `policy::gating::evaluate_action`.

## Removal PR plan (coordinated)

- PR1: Remove adapters shim; update imports; add CI check. (Low risk)
- PR2: Remove top-level rescue re-export; update imports; add CI check. (Low risk)
- PR3: Replace legacy logging helpers at call sites with StageLogger; make helpers private; add CI guard. (Medium)
- PR4: Reshape API modules (`src/api.rs` → `src/api/mod.rs`), drop `#[path]`. (Low)
- PR5: Centralize gating in policy; delete duplicate logic in API; parity tests. (Medium)
- PR6: Migrate Policy to grouped types; remove legacy booleans and `*_preset` methods. (High, breaking)
- PR7: Split FS backup/restore; remove monolithic files; adjust visibility of low-level atoms. (Medium)
- PR8: Final CI gates to forbid deprecated surfaces; docs and changelog updates. (Low)
