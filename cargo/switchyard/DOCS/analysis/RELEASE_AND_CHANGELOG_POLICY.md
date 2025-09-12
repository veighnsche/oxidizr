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

## Round 3 Severity Assessment (AI 4, 2025-09-12 15:52 CET)

- **Title:** Lack of Deprecation Warnings for Legacy Shims and API Changes
  - **Category:** Documentation Gap (DX/Usability)
  - **Impact:** 3  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 3  **Severity:** S2
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** Without deprecation warnings on legacy shims and low-level API exports, consumers may integrate with components slated for removal, risking breakage in future updates. Adding `#[deprecated]` attributes and changelog entries is a simple, high-value step to ensure smooth transitions. The cost of inaction is potential integration issues during upgrades.
  - **Evidence:** No deprecation attributes on shims like `adapters::lock_file::*` in `src/adapters/mod.rs` (lines 6–9) or low-level FS atoms in `src/fs/mod.rs` (lines 9–15).
  - **Next step:** Add `#[deprecated]` attributes with notes to legacy shims and low-level exports in relevant files. Document these in CHANGELOG under 'Deprecated' for the next release. Add CI check to prevent new usages. Implement in Round 4.

- **Title:** Missing Dual-Emit Period for Schema Version Bumps
  - **Category:** Missing Feature (Reliability)
  - **Impact:** 3  **Likelihood:** 2  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
  - **Disposition:** Implement  **LHF:** No
  - **Feasibility:** Medium  **Complexity:** 3
  - **Why update vs why not:** Without a dual-emit period for facts schema changes, downstream consumers may face immediate breakage when upgrading. Implementing dual-emit (v1+v2) during migration ensures compatibility, reducing disruption. The cost of inaction is potential analytics pipeline failures during schema updates.
  - **Evidence:** `src/logging/audit.rs` uses a single `SCHEMA_VERSION=1` with no dual-emit mechanism (line 13).
  - **Next step:** Introduce feature-gated dual-emit for v1 and v2 schemas in `src/logging/audit.rs`. Update test fixtures for both versions and add CI gate to ensure compatibility during migration. Update SPEC §13 for dual-emit mechanics. Plan for Round 4.

- **Title:** Absence of Repository-Local CI Gates for SKIP and Unwrap/Expect
  - **Category:** Test & Validation (Reliability)
  - **Impact:** 2  **Likelihood:** 2  **Confidence:** 5  → **Priority:** 1  **Severity:** S4
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** Without local checks for skipped tests or unwrap/expect patterns, there's a risk of regressions slipping through before CI. Adding simple scripts or xtasks to catch these issues locally enhances code quality with minimal effort. The cost of inaction is occasional CI failures, which are minor but avoidable.
  - **Evidence:** `src/lib.rs` enforces `#![deny]` for unwrap/expect in non-test code (lines 1–9), but no local checks for SKIPs or deprecation usage are in place.
  - **Next step:** Add a cargo xtask or script to fail on `#[ignore]` or similar patterns in test code. Add grep-based CI checks for deprecation coverage. Document in CONTRIBUTING. Implement in Round 4.

- **Title:** Missing Crate-Level CHANGELOG for Public API Changes
  - **Category:** Documentation Gap (DX/Usability)
  - **Impact:** 2  **Likelihood:** 4  **Confidence:** 5  → **Priority:** 2  **Severity:** S3
  - **Disposition:** Implement  **LHF:** Yes
  - **Feasibility:** High  **Complexity:** 1
  - **Why update vs why not:** Without a crate-level CHANGELOG, consumers and maintainers lack an easily accessible record of API changes, complicating upgrade decisions. Adding `CHANGELOG.md` and gating CI on updates for public API diffs is a straightforward way to improve transparency. The cost of inaction is minor user inconvenience during upgrades.
  - **Evidence:** No `CHANGELOG.md` exists in `cargo/switchyard/` directory (process doc only).
  - **Next step:** Create `cargo/switchyard/CHANGELOG.md` following the template. Add CI gate using `cargo public-api` to require changelog updates on API diffs. Implement in Round 4.

Severity assessed in Round 3 by AI 4 on 2025-09-12 15:52 CET
