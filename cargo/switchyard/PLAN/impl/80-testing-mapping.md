# Testing Strategy & Fixture Layout (Planning Only)

This document maps the implementation plan to SPEC tests and fixtures. It defines what to test and where artifacts will live. No test code is implemented here.

## Coverage Sources

- SPEC Gherkin features: `cargo/switchyard/SPEC/features/*.feature`
- Requirements index: `cargo/switchyard/SPEC/requirements.yaml`
- Step contract: `cargo/switchyard/SPEC/features/steps-contract.yaml`
- Audit schema: `cargo/switchyard/SPEC/audit_event.schema.json`
- Preflight schema: `cargo/switchyard/SPEC/preflight.yaml`

## Test Taxonomy

- Unit tests
  - `fs_ops` primitives (TOCTOU pattern, EXDEV fallback selection logic)
  - `safepath` normalization and root-escape rejection
  - `determinism` ID generation and ordering
  - `logging` masking and schema fields composition

- Integration tests
  - `api` orchestration flows (plan → preflight → apply → plan_rollback)
  - Policy gates and fail-closed behavior

- Property tests
  - `AtomicReplace` — no visible broken/missing path during swap
  - `IdempotentRollback` — repeated rollback yields stable state

- BDD adapter-driven tests
  - Execute `SPEC/features/*.feature` against library APIs using:
    - Option A: Rust adapter (`cucumber-rs`)
    - Option B: Go shim (`godog`) to leverage existing `test-orch/`

## Fixtures & Evidence

- Golden fixtures (JSONL): plan, preflight, apply, rollback facts
  - Location: `tests/golden/<scenario>/facts/*.jsonl` (proposed)
- Preflight outputs (YAML):
  - Location: `tests/golden/<scenario>/preflight.yaml`
- Schema validation jobs in CI against SPEC schemas

## Matrix & Environments

- Filesystems: PR → tmpfs/ext4; Nightly → xfs/btrfs
- Modes: DryRun default, RealRun explicit
- Degraded: EXDEV toggled by policy `allow_degraded_fs`

## Traceability

- Annotate tests with `covers: [REQ-…]` where feasible
- Ensure `SPEC/tools/traceability.py` reports 100% MUST/MUST_NOT coverage

## Failure Handling

- Smoke suite failure triggers auto-rollback unless disabled
- Facts record `degraded=true` when fallback path used

## CI Hooks (planned)

- docs-lint
- spec-traceability (runs `SPEC/tools/traceability.py` and verifies `SPEC/traceability.md`)
- schema-validate (JSON for facts; YAML for preflight)
- golden-diff (zero-SKIP, stable ordering enforcement)
- bdd-smoke (post-apply health verification)
- expected-fail (xfail) accounting: suites marked as `expect: xfail` contribute ✅ on failure and ❌ on unexpected pass; must be explicitly justified in PR

## Requirement Tagging

- Each scenario and/or test should be annotated with `@REQ-…` tags to map directly to `SPEC/requirements.yaml` entries.
- Property tests name invariants exactly as in SPEC (e.g., `AtomicReplace`, `IdempotentRollback`).
- Golden fixtures directories include a `covers.yaml` referencing requirement IDs.

## Runner Integration (Planning)

- Option A: Rust adapter using `cucumber-rs` to call library APIs directly.
- Option B: Go shim using `godog` aligned with existing `test-orch/` conventions.
- The runner should emit stage logs with clear PASS/FAIL indicators and support `expect: xfail` semantics (see container-runner behavior in project memories).
