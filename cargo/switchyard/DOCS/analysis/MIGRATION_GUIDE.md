# Migration Guide for Adopters
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Prepare for upcoming surface cleanups (FS atoms internalization, preflight helper naming). Provide re-export map and deprecation plan.  
**Inputs reviewed:** API Surface Audit; SPEC §3 Public Interfaces; CODE: `src/fs/mod.rs`  
**Affected modules:** `fs/*`, `preflight/*`

## Summary
- Low-level FS atoms will be internalized (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`, `is_safe_path`). Use high-level helpers instead.
- Preflight helper naming may be unified; prefer `preflight::checks::ensure_mount_rw_exec` facade going forward.

## Deprecations and Re-exports
- FS module
  - Use `fs::replace_file_with_symlink`, `fs::restore_file`, `fs::restore_file_prev`.
  - Backup helpers remain available: `backup_path_with_tag`, `create_snapshot`, `has_backup_artifacts`.

## Timeline
- v0.x: mark Internal in docs; keep re-exports with `#[doc(hidden)]`.
- v0.(x+1): add `#[deprecated]` attributes to internalized re-exports.
- v1.0: remove deprecated re-exports; stable facade only.

## Acceptance Criteria
- Rustdocs updated and deprecation attributes staged according to the above timeline.

## References
- API_SURFACE_AUDIT.md; CODE: `src/fs/mod.rs`
