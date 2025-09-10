# BDD Runner SPEC

Status: Draft v0.1 (incremental)  
Scope: Execute Switchyard SPEC Gherkin features and provide a thin, deterministic Rust adapter for step definitions.

## Purpose

Provide a minimal, repeatable Behavior-Driven Testing harness for the Switchyard project that can:

- Discover and execute Gherkin `.feature` files authored under `cargo/switchyard/SPEC/features/`.
- Map steps to Rust step definitions using `cucumber-rs`.
- Emit machine-readable results for CI consumption.
- Support strictness modes to gate on unimplemented steps.

This runner does NOT replace Switchyard’s end-to-end Docker/LXD test orchestrator; it complements it by validating normative behaviors expressed in the SPEC using fast, hermetic Rust tests.

## Architecture

- Test target: `tests/bdd.rs` (non-std harness; uses `cucumber-rs`).
- Library helpers: `src/world.rs`, `src/steps_common.rs`, `src/audit_schema.rs`, `src/error_codes.rs`.
- Feature source of truth: `cargo/switchyard/SPEC/features/` (see that directory’s `README.md`).
- Output: Cucumber JSON at `features_out/report.json` by default.

## Configuration

- BDD_FEATURES_DIR: absolute or relative path to the directory containing `.feature` files.  
  Default: `../switchyard/SPEC/features` relative to the bdd-runner crate.
- BDD_JSON_REPORT: path to write the JSON report.  
  Default: `features_out/report.json` under the crate root.
- BDD_FAIL_ON_STUB: if `1` or `true`, any step matched by the generic catch‑all stubs fails the run.

## Normative Requirements

- Feature Discovery
  - MUST load `.feature` files from `BDD_FEATURES_DIR` if set; otherwise from `cargo/switchyard/SPEC/features/`.
  - MUST recurse into subdirectories (cucumber default behavior).

- Execution & Isolation
  - MUST create an independent `World` instance per scenario (`cucumber-rs` default).
  - SHOULD provision a writable, scenario-local workspace (e.g., `TempDir`) exposed via `world.work` for steps that interact with the filesystem.

- Reporting & Exit Codes
  - MUST write a JSON report to `BDD_JSON_REPORT` (creating parent dirs if necessary).
  - MUST exit non-zero when any scenario fails or panics.
  - MUST exit non-zero when `BDD_FAIL_ON_STUB` is enabled and any step matches the generic stubs.

- Step Vocabulary Contract
  - SHOULD ensure every step phrase in features matches at least one regex listed in `cargo/switchyard/SPEC/features/steps-contract.yaml`.
  - MAY provide a separate “contract-lint” task in CI that validates coverage without running implementations.

- Determinism
  - The runner SHOULD avoid introducing nondeterminism (e.g., time-dependent output) beyond cucumber’s report. Any timestamps used by steps must be controlled by the step implementation.

- Traceability
  - Runner SHOULD preserve feature tags and scenario names in the JSON report to enable SPEC → CI coverage mapping.

## Out of Scope (for this crate)

- Spinning Docker/LXD containers and full-system orchestration (handled in `test-orch/`).
- Installing or invoking external package managers.
- Enforcing Switchyard business logic; that belongs to Switchyard proper. The runner only hosts test glue.

## Roadmap (Non-Normative)

- Contract linter that loads `steps-contract.yaml` and validates all steps in the feature corpus.
- Tag expression filters and per‑tag CI gating presets (e.g., `@smoke`, `@must`).
- JUnit report writer in addition to JSON.
- Minimal reference step implementations for the most common Given/When/Then in SPEC.
