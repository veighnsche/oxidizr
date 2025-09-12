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

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:29 CEST)

- **Invariant:** Consistent coding standards across external contributors
- **Assumption (from doc):** Documented standards ensure consistent code quality from all contributors including external integrators
- **Reality (evidence):** Standards exist for modules, errors, logging at `src/lib.rs:L1-L3`, `src/types/errors.rs`, `src/logging/audit.rs`; however, no automated enforcement beyond clippy lints exists to validate compliance
- **Gap:** External contributors may not follow standards without automated checking; review burden increases without enforcement tooling
- **Mitigations:** Implement pre-commit hooks or CI checks for coding standards; add rustfmt configuration to enforce formatting consistency
- **Impacted users:** External contributors and library maintainers dealing with inconsistent code quality
- **Follow-ups:** Add automated code style enforcement; implement CI gates for coding standards compliance

- **Invariant:** Error handling patterns provide consistent consumer experience
- **Assumption (from doc):** Domain enums with `thiserror` provide stable error handling for consumers
- **Reality (evidence):** Error patterns implemented at `src/types/errors.rs` using `thiserror`; stable `ErrorId` mapping exists; however, no guidelines prevent ad-hoc error handling in new modules
- **Gap:** Without enforcement, new modules might introduce inconsistent error patterns that complicate consumer error handling
- **Mitigations:** Add linting rules to detect non-standard error patterns; provide error handling templates for new modules
- **Impacted users:** Library consumers who need predictable error handling across all Switchyard operations
- **Follow-ups:** Implement error pattern linting; add error handling guidelines to contributor documentation

- **Invariant:** Module organization supports consumer understanding and navigation
- **Assumption (from doc):** Directory modules with domain-specific organization help consumers find and use appropriate functionality
- **Reality (evidence):** Current organization follows `mod.rs` pattern at `src/fs/mod.rs`, `src/preflight.rs`; however, no architectural decision records document the reasoning or guidelines for future module organization
- **Gap:** Without documented module organization principles, future additions might not follow consistent patterns, confusing consumers
- **Mitigations:** Document module organization principles; add architectural decision records for significant structural changes
- **Impacted users:** New contributors and library consumers who need to understand code organization for effective usage
- **Follow-ups:** Create architectural documentation; establish module organization guidelines for future development

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:29 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: No automated enforcement of standards beyond clippy
  - Category: Missing Feature
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: Pre-commit hooks/CI checks reduce reviewer burden and drift from documented practices.
  - Evidence: Standards documented here; enforced only via crate lints in `src/lib.rs:1-3`.
  - Next step: Add pre-commit hooks (fmt, clippy, tests) and a CI workflow job that fails on deviations (format, lints, doc links).

- Title: Error-handling pattern consistency not enforced
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Document concrete templates and examples to avoid ad-hoc error patterns.
  - Evidence: Guidance references `thiserror` patterns (`src/types/errors.rs`), but no prescriptive template.
  - Next step: Add an “Error patterns” section with examples and a checklist; consider custom lint or review checklist.

- Title: Module organization principles not captured in ADRs
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 4  → Priority: 1  Severity: S4
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: ADRs make rationale explicit and guide future structure changes.
  - Evidence: This doc mentions `mod.rs` conventions; no ADR documenting layering principles.
  - Next step: Write a short ADR on module layout and re-export policy; link from this doc.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
