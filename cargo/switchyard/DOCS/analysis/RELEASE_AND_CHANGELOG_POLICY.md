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
