# Coding Standards & Module Layout

**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Establish conventions for modules, naming, errors, logging, and re-exports.  
**Inputs reviewed:** CODE structure; PLAN/00-structure.md  
**Affected modules:** all

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- Directory modules prefer `mod.rs` with submodules per domain (e.g., `fs/{atomic,swap,backup,restore,meta}`).
- Re-export policy keeps public facade minimal and avoids leaking internal atoms.
- Error patterns use domain enums with `thiserror` crate.
- Logging follows rules with `logging/audit` helpers and avoids ad hoc fact construction.
- Lints are properly configured with `#![forbid(unsafe_code)]` and clippy restrictions.
- Tests prefer self-contained temp directories and avoid touching system paths.

**Citations:**
- `src/fs/mod.rs` - Filesystem module with submodules
- `src/preflight.rs` - Preflight module with checks and yaml submodules
- `src/types/errors.rs` - Error definitions with thiserror
- `src/logging/audit.rs` - Audit logging helpers
- `src/lib.rs:L1-L3` - Lint configurations
- `src/fs/swap.rs:L140-L147` - Test using tempfile
- `src/fs/restore.rs:L232-L233` - Test using tempfile

**Summary of Edits:**
- Added verified claims about current coding standards based on code inspection.
- Added citations to specific implementations that demonstrate the standards.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

## Standards

- Directory modules: prefer `mod.rs` with submodules per domain (`fs/{atomic,swap,backup,restore,meta}` etc.).
- Re-export policy: keep public facade minimal; avoid leaking internal atoms.
- Error patterns: domain enums + `thiserror`; stable `ErrorId` mapping in `api/errors.rs`.
- Logging rules: use `logging/audit` helpers; never construct facts ad hoc.
- Lints: `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used)]`, `#![warn(clippy::all, clippy::cargo, clippy::pedantic)]` must pass.
- Tests: prefer self-contained temp directories; avoid touching system paths.

## Acceptance Criteria

- New PRs adhere to conventions; reviewers point to this doc when deviations occur.
