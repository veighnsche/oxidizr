# Coding Standards & Module Layout
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Establish conventions for modules, naming, errors, logging, and re-exports.  
**Inputs reviewed:** CODE structure; PLAN/00-structure.md  
**Affected modules:** all

## Standards
- Directory modules: prefer `mod.rs` with submodules per domain (`fs/{atomic,swap,backup,restore,meta}` etc.).
- Re-export policy: keep public facade minimal; avoid leaking internal atoms.
- Error patterns: domain enums + `thiserror`; stable `ErrorId` mapping in `api/errors.rs`.
- Logging rules: use `logging/audit` helpers; never construct facts ad hoc.
- Lints: `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used)]`, `#![warn(clippy::all, clippy::cargo, clippy::pedantic)]` must pass.
- Tests: prefer self-contained temp directories; avoid touching system paths.

## Acceptance Criteria
- New PRs adhere to conventions; reviewers point to this doc when deviations occur.
