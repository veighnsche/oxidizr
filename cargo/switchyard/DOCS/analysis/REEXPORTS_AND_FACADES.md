# Re-exports, Facades, and Migration Leftovers

This document enumerates all `pub use` re-exports in the Switchyard crate, classifies them as ergonomics facades vs. backwards-compatibility shims, and suggests actions.

**Verified Claims:**
- The crate root contains two re-exports: `api::*` and `policy::rescue`.
- The fs module re-exports atomic operations, backup helpers, metadata functions, mount inspection, path utilities, and restore functions.
- The logging module re-exports fact emission and redaction utilities.
- The types module re-exports errors, IDs, plan structures, reports, and SafePath utilities.
- The policy module re-exports the Policy configuration structure.
- The preflight module re-exports checks and YAML export functionality.
- The adapters module contains both facade re-exports and a compatibility shim for lock_file.

**Citations:**
- `src/lib.rs:L20-L21` - crate root re-exports
- `src/fs/mod.rs:L9-L15` - filesystem facade re-exports
- `src/logging/mod.rs:L5-L6` - logging facade re-exports
- `src/types/mod.rs:L7-L11` - types facade re-exports
- `src/policy/mod.rs:L5` - policy facade re-export
- `src/preflight.rs:L13-L14` - preflight facade re-exports
- `src/adapters/mod.rs:L11-L17` - adapters facade re-exports

Generated on: 2025-09-12

## Legend

- Facade: Intentional, stable public API surface aggregating internals for ergonomics.
- Shim: Backwards-compatibility layer preserving an old path; should be deprecated/removed over time.

## Inventory

1) Crate root (`src/lib.rs`)

- Re-exports:
  - `pub use api::*;` — Facade (top-level routing to API surface)
  - `pub use policy::rescue;` — Shim (compatibility re-export of `switchyard::policy::rescue`)

2) Filesystem facade (`src/fs/mod.rs`)

- Re-exports:
  - `pub use atomic::{atomic_symlink_swap, fsync_parent_dir, open_dir_nofollow};` — Facade
  - `pub use backup::{backup_path_with_tag, create_snapshot, has_backup_artifacts};` — Facade
  - `pub use meta::{detect_preservation_capabilities, kind_of, resolve_symlink_target, sha256_hex_of};` — Facade
  - `pub use mount::{ensure_rw_exec, ProcStatfsInspector};` — Facade
  - `pub use paths::is_safe_path;` — Facade
  - `pub use restore::{restore_file, restore_file_prev};` — Facade
  - `pub use swap::replace_file_with_symlink;` — Facade

3) Logging facade (`src/logging/mod.rs`)

- Re-exports:
  - `pub use facts::{AuditSink, FactsEmitter, JsonlSink};` — Facade
  - `pub use redact::{redact_event, ts_for_mode, TS_ZERO};` — Facade

4) Types facade (`src/types/mod.rs`)

- Re-exports:
  - `pub use errors::*;` — Facade
  - `pub use ids::*;` — Facade
  - `pub use plan::*;` — Facade
  - `pub use report::*;` — Facade
  - `pub use safepath::*;` — Facade

5) Policy module (`src/policy/mod.rs`)

- Re-exports:
  - `pub use config::Policy;` — Facade
- Note: `policy::checks` shim removed from module graph as part of refactor; file remains but is unused.

6) Preflight façade (`src/preflight.rs`)

- Re-exports:
  - `pub use checks::{check_immutable, check_source_trust, ensure_mount_rw_exec};` — Facade
  - `pub use yaml::to_yaml;` — Facade
- Intent: `preflight.rs` is the public entrypoint; code lives in `preflight/checks.rs` and `preflight/yaml.rs`.

7) Adapters module (`src/adapters/mod.rs`)

- Re-exports:
  - `pub use attest::*;` — Facade
  - `pub use lock::file::FileLockManager;` — Facade
  - `pub use lock::*;` — Facade
  - `pub use ownership::fs::FsOwnershipOracle;` — Facade
  - `pub use ownership::*;` — Facade
  - `pub use path::*;` — Facade
  - `pub use smoke::*;` — Facade
- Compatibility shim:
  - `pub mod lock_file { pub use super::lock::file::*; }`
  - Purpose: preserve legacy path `switchyard::adapters::lock_file::FileLockManager`

## Recommendations

- Shims to consider deprecating/removing:
  - `lib.rs`: `pub use policy::rescue;` — mark as `#[deprecated]` now; remove on next major.
  - `adapters/mod.rs`: `lock_file` namespace — migrate call sites to `switchyard::adapters::FileLockManager`; remove shim after.
- Facades: keep as-is; they define the intended public API surface.

## Searches used

- Regex: `pub use` across the crate.
- Manual inspection of modules with explicit compatibility comments.

## Notes

- If we want to avoid even façade re-exports for preflight, we can drop the `pub use` from `preflight.rs` and have direct imports from `preflight::checks`/`preflight::yaml`. Current setup keeps a single public entrypoint, which is often preferred for discoverability.
