# Migration Guide for Adopters
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Prepare for upcoming surface cleanups (FS atoms internalization, preflight helper naming). Provide re-export map and deprecation plan.  
**Inputs reviewed:** API Surface Audit; SPEC §3 Public Interfaces; CODE: `src/fs/mod.rs`  
**Affected modules:** `fs/*`, `preflight/*`

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- Low-level FS atoms will be internalized (`open_dir_nofollow`, `atomic_symlink_swap`, `fsync_parent_dir`, `is_safe_path`).
- Preflight helper naming may be unified.
- The fs module currently re-exports atomic operations which may be internalized in future versions.
- Backup helpers are available and will remain so.
- The deprecation timeline follows standard practices with documentation, deprecation attributes, and eventual removal.

**Citations:**
- `src/fs/mod.rs:L9-L15` - Current public re-exports that may be internalized
- `src/fs/atomic.rs:L22-L33` - `open_dir_nofollow` implementation
- `src/fs/atomic.rs:L43-L96` - `atomic_symlink_swap` implementation
- `src/fs/atomic.rs:L35-L41` - `fsync_parent_dir` implementation
- `src/fs/paths.rs:L7-L19` - `is_safe_path` implementation
- `src/preflight.rs:L7-L10` - Preflight module structure
- `src/preflight.rs:L13-L14` - Preflight re-exports
- `src/fs/mod.rs:L10` - Backup helper re-export

**Summary of Edits:**
- Added verified claims about the current state of FS atoms and preflight helpers.
- Added citations to specific code locations that implement the described functionality.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

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
