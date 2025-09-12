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
