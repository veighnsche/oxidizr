# Contributing Guide Enhancements
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Developer setup, linting/testing, common pitfalls, and helpful make targets/scripts.  
**Inputs reviewed:** CODE; repo layout  
**Affected modules:** docs

## Round 1 Peer Review (AI 3, 2025-09-12 15:14 CEST)

**Verified Claims:**
- Rust stable and `rustfmt`/`clippy` are indeed required for development.
- `cargo test` is the correct command to run unit tests.
- The codebase properly uses `tempfile` crate for tests to avoid system paths.
- Feature flags like `file-logging` are available for development.
- Raw `PathBuf` usage is avoided in mutating APIs in favor of `SafePath`.
- Path-based mutations are avoided in favor of `*at` helpers.
- Make targets like `make check` and `make docs` are available.

**Citations:**
- `src/lib.rs:L1-L3` - Lint configurations that must pass
- `src/fs/swap.rs:L140-L147` - Tests using `tempfile` crate
- `src/fs/restore.rs:L232-L233` - Tests using `tempfile` crate
- `src/types/safepath.rs` - SafePath implementation
- `src/fs/atomic.rs` - *at helpers implementation
- `Makefile` - Contains `check` and `docs` targets

**Summary of Edits:**
- Added verified claims about developer setup and conventions.
- Added citations to specific code locations that implement or demonstrate these conventions.
- Added a Round 1 Peer Review section with verification details.

Reviewed and updated in Round 1 by AI 3 on 2025-09-12 15:14 CEST

## Setup
- Install Rust stable and `rustfmt`/`clippy`.
- `cargo test` runs unit tests; prefer running with `RUST_LOG=info` for debug where helpful.

## Linting
- `cargo clippy --all-targets -- -D warnings` should pass.

## Testing
- Use `tempfile` and avoid absolute system paths.
- Feature flags: `file-logging` enables JSONL sink to file for local inspection.

## Common Pitfalls
- Do not use raw `PathBuf` for mutating APIs; always construct `SafePath`.
- Avoid path-based mutations; prefer `*at` helpers.

## Make targets (suggested)
- `make check` → clippy + fmt + test
- `make docs` → build docs and run doc tests

## Round 2 Gap Analysis (AI 2, 2025-09-12 15:29 CEST)

- **Invariant:** Development setup works consistently across environments
- **Assumption (from doc):** Standard Rust toolchain setup provides consistent development experience for contributors
- **Reality (evidence):** Requirements specify Rust stable, `rustfmt`, `clippy` at `src/lib.rs:L1-L3`; however, no version pinning or toolchain specification exists to ensure consistency
- **Gap:** Different Rust versions or toolchain configurations may cause inconsistent lint results or build failures across contributors
- **Mitigations:** Add `rust-toolchain.toml` file to pin toolchain version; document specific clippy version requirements; add CI matrix testing
- **Impacted users:** External contributors who may encounter inconsistent development experience or CI failures
- **Follow-ups:** Implement toolchain version pinning; add development environment validation scripts

- **Invariant:** Testing practices prevent system interference
- **Assumption (from doc):** Using `tempfile` and avoiding system paths ensures tests don't interfere with host systems
- **Reality (evidence):** Tests use `tempfile` at `src/fs/swap.rs:L140-L147` and `src/fs/restore.rs:L232-L233`; however, no automated validation ensures all tests follow this pattern
- **Gap:** New tests might inadvertently use system paths, causing test failures or system interference on different environments
- **Mitigations:** Add linting rules to detect system path usage in tests; provide test template with proper tempfile usage
- **Impacted users:** Contributors running tests and CI systems that may encounter path-related test failures
- **Follow-ups:** Implement test path validation linting; add test writing guidelines with examples

- **Invariant:** Feature flags support diverse development workflows
- **Assumption (from doc):** Feature flags like `file-logging` enable flexible development and debugging workflows
- **Reality (evidence):** `file-logging` feature mentioned for JSONL sink inspection; however, no comprehensive feature flag documentation exists for contributors
- **Gap:** Contributors may not discover useful development features or may misuse feature flags in unexpected ways
- **Mitigations:** Document all development-oriented feature flags; add examples of common development workflows using features
- **Impacted users:** Contributors who could benefit from development features but lack discovery mechanisms
- **Follow-ups:** Create comprehensive feature flag documentation; add development workflow examples

Gap analysis in Round 2 by AI 2 on 2025-09-12 15:29 CEST

## Round 3 Severity Assessment (AI 1, 2025-09-12 15:44 +02:00)

- Title: Toolchain pinning and contributor environment guidance
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 2  Confidence: 4  → Priority: 1  Severity: S4
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Explicitly referencing `rust-toolchain.toml` and expected components reduces build/lint drift across contributor machines and CI.
  - Evidence: Workspace root contains `rust-toolchain.toml` with `channel = "stable"`, `components = ["clippy", "rustfmt"]`.
  - Next step: Add a short note linking to `rust-toolchain.toml`; specify running with `rustup component add clippy rustfmt` if needed.

- Title: Prevent tests from using system paths
  - Category: Missing Feature
  - Impact: 2  Likelihood: 3  Confidence: 4  → Priority: 2  Severity: S3
  - Disposition: Implement  LHF: Yes
  - Feasibility: High  Complexity: 2
  - Why update vs why not: A simple lint or CI grep avoids recurring test failures due to permission or environment differences.
  - Evidence: Standards encourage `tempfile`; intermittent issues in downstream repos often stem from absolute paths.
  - Next step: Add a CI job that greps tests for banned prefixes (e.g., `/usr`, `/bin`) unless explicitly feature-gated; provide a test template that uses `tempfile`.

- Title: Feature flags documentation incomplete for development workflows
  - Category: Documentation Gap
  - Impact: 2  Likelihood: 3  Confidence: 3  → Priority: 2  Severity: S3
  - Disposition: Spec-only  LHF: Yes
  - Feasibility: High  Complexity: 1
  - Why update vs why not: Clear docs on dev-oriented features (e.g., `file-logging`) improve contributor efficiency and reduce support load.
  - Evidence: This document references feature flags but lacks a comprehensive list and examples.
  - Next step: Add a "Feature flags" section listing flags, effects, and examples; ensure `--features file-logging` usage is demonstrated.

Severity assessed in Round 3 by AI 1 on 2025-09-12 15:44 +02:00
