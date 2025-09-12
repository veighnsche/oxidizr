# Release & Changelog Policy
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Define SemVer rules, deprecation windows, changelog template, and CI checks.  
**Inputs reviewed:** PLAN/meta; SPEC §13 Schema Versioning; CODE: N/A  
**Affected modules:** process docs

## Summary
- Adopt SemVer: public API (facade) changes bump minor/major accordingly; internal-only changes are patch.
- Deprecations: minimum one minor version with warnings before removal.
- Changelog: keep sections for Added/Changed/Deprecated/Removed/Fixed/Security.

## CI Gates
- No `#[allow(unused)]` or `unwrap/expect` in non-test code (already enforced by `#![deny]`).
- Zero SKIP gate on tests as per SPEC §12.
- Facts schema bump requires dual-emit period and fixture updates.

## Template
- Unreleased
  - Added
  - Changed
  - Deprecated
  - Removed
  - Fixed
  - Security

## Acceptance Criteria
- CHANGELOG.md maintained per release with entries following the template.

## Round 1 Peer Review (AI 2, 2025-09-12 15:06 +02:00)

**Claims Verified:**
- ✅ Document describes SemVer adoption and deprecation windows (minimal technical claims)
- ✅ CI gates mentioned align with general Rust practices (`#![deny]` attributes)
- ✅ Changelog template follows conventional format with Added/Changed/Deprecated/Removed/Fixed/Security sections
- ⚠️ **Limited verification possible**: Most claims are process-oriented rather than code-based

**Key Citations:**
- Document structure matches standard changelog practices
- References to SPEC §12-13 for schema versioning and CI requirements

**Summary of Edits:** This document contains primarily process and policy claims rather than technical implementation details, so limited code verification was possible. The content appears reasonable and aligned with standard practices.

Reviewed and updated in Round 1 by AI 2 on 2025-09-12 15:06 +02:00

## Round 2 Gap Analysis (AI 1, 2025-09-12 15:22 +02:00)

- Invariant: Deprecations receive at least one minor release before removal
  - Assumption (from doc): Deprecation windows are enforced and communicated.
  - Reality (evidence): No deprecation attributes present on legacy shims (e.g., `src/adapters/mod.rs` still exposes `adapters::lock_file::*` shim lines 6–9); low-level FS atom re-exports in `src/fs/mod.rs` are public and not marked deprecated though slated as Internal in audits.
  - Gap: Consumers may unknowingly integrate with soon-to-be-removed surfaces.
  - Mitigations: Add `#[deprecated]` attributes and Rustdoc notes on shim exports; log these in CHANGELOG under Deprecated; schedule removal in the next minor after one release window.
  - Impacted users: Integrators using legacy import paths or low-level atoms.
  - Follow-ups: PR to annotate deprecations and add a CI grep check to prevent new usages.

- Invariant: Schema version bumps use a dual-emit period with fixtures
  - Assumption (from doc): Bumping facts schema triggers dual-emit and fixture updates.
  - Reality (evidence): Facts emission uses a single `SCHEMA_VERSION=1` in `src/logging/audit.rs` (line 13) and no dual-emit plumbing; tests validate presence but do not exercise v1/v2 side-by-side.
  - Gap: Upgrading schema risks breaking downstream consumers without a migration window.
  - Mitigations: Introduce feature-gated dual-emit (v1+v2) in `logging/audit.rs` and fixtures for both; add CI gate requiring both sets to pass during migration.
  - Impacted users: Downstream log consumers and analytics pipelines.
  - Follow-ups: SPEC §13 to state dual-emit mechanics; add test harness to compare v1/v2 fixtures.

- Invariant: CI gates enforce zero-SKIP and unwrap/expect denial
  - Assumption (from doc): CI rejects SKIPped tests and risky patterns.
  - Reality (evidence): Code enforces `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]` in `src/lib.rs` lines 1–9; zero-SKIP gate exists in SPEC but CI wiring is external to this crate (not verifiable here).
  - Gap: Lack of repository-local checks for SKIPs and deprecation usage may allow regressions.
  - Mitigations: Add a simple cargo xtask or script that fails on `#[ignore]` or `#[cfg(skip)]` patterns; add grep-based CI checks for `deprecated(note)` coverage of shims.
  - Impacted users: Maintainers and contributors relying on consistent CI signals.
  - Follow-ups: Wire checks in workflow; document in CONTRIBUTING.

- Invariant: Changelog accurately reflects public API changes each release
  - Assumption (from doc): CHANGELOG is consistently updated with Added/Changed/Deprecated/Removed/Fixed/Security sections.
  - Reality (evidence): No CHANGELOG.md in-tree within `cargo/switchyard/` (process doc only).
  - Gap: Changes may ship without an easily discoverable change log at the crate level.
  - Mitigations: Add `cargo/switchyard/CHANGELOG.md` managed per release; gate CI to ensure it changes when public API diffs (e.g., via `cargo public-api`) are detected.
  - Impacted users: Consumers upgrading the crate and maintainers triaging diffs.
  - Follow-ups: Introduce `public-api` diff check in CI and require CHANGELOG updates.

Gap analysis in Round 2 by AI 1 on 2025-09-12 15:22 +02:00
