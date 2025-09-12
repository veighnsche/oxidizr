# Backwards Compatibility Shims and Re-exports: Inventory and Plan

This document lists all compatibility shims and re-exports found in the Switchyard crate, what they do, and whether they can be removed. It also records the changes just made to remove the `policy/checks` shim per the refactor plan.

**Verified Claims:**
- The `policy::checks` shim has been successfully removed from the module graph as verified by `cargo check` and `cargo test --no-run`.
- The `adapters::lock_file` shim is still active and used by tests in `tests/lock_wait_fact.rs`.
- The top-level `policy::rescue` re-export is still present in `src/lib.rs`.

**Citations:**
- `src/lib.rs:L21` - `pub use policy::rescue; // compatibility re-export`
- `src/adapters/mod.rs:L6-L9` - lock_file compatibility shim module
- `tests/lock_wait_fact.rs` - usage of `switchyard::adapters::lock_file::FileLockManager`

## Changes just implemented

- Moved preflight checks into a dedicated module and updated imports:
  - New files: `src/preflight/checks.rs`, `src/preflight/yaml.rs`.
  - `src/preflight.rs` now delegates to `preflight::checks` and `preflight::yaml` and re-exports `to_yaml` and the three checks.
  - Updated call sites to import from `crate::preflight::checks::*` instead of `crate::policy::checks::*` or `crate::preflight::*`.
  - Removed `policy::checks` from the module tree by editing `src/policy/mod.rs` (the old `src/policy/checks.rs` file remains in the workspace but is no longer compiled or referenced).
  - Verified with `cargo check` and `cargo test --no-run` (no errors).

**Verified Implementation:**
- The refactor was successfully implemented with no compilation errors.
- All preflight checks now reside in `src/preflight/checks.rs` and are re-exported via `src/preflight.rs`.
- The `policy::gating.rs` module correctly imports checks from `crate::preflight::checks::*`.

## Current compatibility shims and re-exports

1) Adapters: lock_file (explicit compatibility shim)

- Path: `src/adapters/mod.rs`
- Snippet:
  - Comment: `// Compatibility shim for old path switchyard::adapters::lock_file::FileLockManager`
  - Module:

    ```rust
    pub mod lock_file {
        pub use super::lock::file::*;
    }
    ```

- Purpose: Preserve the legacy import path `switchyard::adapters::lock_file::FileLockManager`.
- Call sites: e.g., `tests/lock_wait_fact.rs` imports `switchyard::adapters::lock_file::FileLockManager`.
- Recommendation: Keep for now or migrate tests and external users to `switchyard::adapters::FileLockManager` (already re-exported at the adapters crate level). After migration, delete the `lock_file` shim. This would be a breaking change for any code using the old path.

2) Crate root re-export of policy::rescue (compatibility re-export)

- Path: `src/lib.rs`
- Snippet:

  ```rust
  pub use policy::rescue; // compatibility re-export
  ```

- Purpose: Allow `switchyard::rescue` as a path instead of `switchyard::policy::rescue`.
- Call sites: none found within this repo (`grep`), but external users may rely on it.
- Recommendation: Mark as deprecated in a future release and remove in the next major version. Provide a changelog note to switch to `switchyard::policy::rescue`.

3) Module facades (not compatibility shims)

- Paths:
  - `src/fs/mod.rs` re-exports a curated facade for FS helpers.
  - `src/logging/mod.rs` re-exports common traits and helpers.
  - `src/types/mod.rs` re-exports `errors`, `ids`, `plan`, `report`, `safepath`.
  - `src/policy/mod.rs` re-exports `Policy`.
  - `src/preflight.rs` re-exports `preflight::checks::*` and `preflight::yaml::to_yaml` (after the split).
- Purpose: These are ergonomic facades, not backwards-compatibility shims. They define the public API surface.
- Recommendation: Keep as-is.

4) Removed shim: `policy/checks`

- Path: `src/policy/checks.rs` (file still present), `src/policy/mod.rs` (no longer includes it)
- Prior behavior: Re-exported `crate::preflight::{ensure_mount_rw_exec, check_immutable, check_source_trust}` to support `crate::policy::checks::*` imports during migration.
- Current status: Removed from the module graph. All imports updated to `crate::preflight::checks::*`.
- Recommendation: Delete the file in a follow-up PR to avoid confusion once all branches are aligned. No public API impact because it was crate-internal and is now unused.

## Proposed removal plan

- Phase 1 (done):
  - Create `preflight::{checks,yaml}` and update imports.
  - Remove `policy::checks` from module tree.

- Phase 2 (compat cleanup):
  - Migrate all internal call sites (including tests) away from `switchyard::adapters::lock_file::FileLockManager` to `switchyard::adapters::FileLockManager`.
  - Provide a PR that removes `adapters::lock_file` shim; note breaking change in the crate’s changelog.

- Phase 3 (public API clean):
  - Mark `pub use policy::rescue` as deprecated with `#[deprecated]` (soft deprecation) and document the new path in README.
  - Remove the re-export in the next major version bump.

## Searches performed (for traceability)

- `crate::policy::checks::` — updated occurrences in:
  - `src/api/preflight.rs`
  - `src/policy/gating.rs`
- `pub use` scans across `src/` to identify re-exports and shims.
- `compatibility shim` markers found in:
  - `src/adapters/mod.rs`
  - `src/policy/checks.rs` (now unused)
- `switchyard::rescue` usage scan (none found in-tree).

## Summary

- The only active shims now are:
  - `adapters::lock_file` (legacy path to FileLockManager)
  - `lib.rs` top-level re-export of `policy::rescue`
- `policy::checks` shim has been removed from the module tree and all call sites now use `preflight::checks`.
- Next steps: decide policy for the remaining two shims (deprecate now, remove later vs. keep indefinitely for ergonomics).

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:23 CEST)

- **Invariant:** Import path stability across package upgrades
- **Assumption (from doc):** Consumers rely on stable import paths like `switchyard::adapters::lock_file::FileLockManager` and `switchyard::rescue` to remain available indefinitely
- **Reality (evidence):** Active compatibility shims at `src/adapters/mod.rs:L6-L9` and `src/lib.rs:L21` maintain these paths; however, no deprecation timeline or major version policy is documented; external consumers may build dependencies on these legacy paths without awareness of planned removal
- **Gap:** No consumer notification mechanism for pending breaking changes; removal could break downstream integrations silently
- **Mitigations:** Implement deprecation attributes (`#[deprecated]`) with migration guidance; document breaking change policy in CHANGELOG template; add CI lint to detect usage of deprecated paths in tests
- **Impacted users:** External library consumers and CLI integrations that import legacy paths
- **Follow-ups:** Add deprecation policy to RELEASE_AND_CHANGELOG_POLICY.md; implement staged deprecation warnings

- **Invariant:** Module path consistency during refactoring
- **Assumption (from doc):** Internal refactoring (like preflight checks migration) should not affect external consumers
- **Reality (evidence):** The `policy::checks` to `preflight::checks` migration successfully avoided external breakage by updating internal imports at `src/policy/gating.rs` and `src/api/preflight.rs`, but no automated testing validates external API stability
- **Gap:** No integration tests verify that public API surface remains stable during internal refactoring
- **Mitigations:** Add API surface stability tests that import public paths and verify availability; document public vs internal API boundaries clearly
- **Impacted users:** Library integrators who depend on stable public API
- **Follow-ups:** Implement API stability CI checks; clarify public API surface documentation

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:23 CEST
