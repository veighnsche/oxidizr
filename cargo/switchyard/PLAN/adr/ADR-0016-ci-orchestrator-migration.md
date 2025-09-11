# ADR-0016: Migrate from test-orch (Docker-based) to GitHub Actions + test-ci-runner

- Status: Accepted
- Date: 2025-09-11
- Authors: Switchyard team
- Supersedes: n/a
- Superseded by: n/a

## Context

The `test-orch/` subsystem was created to orchestrate acceptance tests using Docker containers and a host orchestrator written in Go. Over time, it has become brittle and is now broken. The majority of our current acceptance needs (unit and light integration at the crate level) can be served directly by GitHub Actions runners without custom container orchestration.

Additionally, our Sprint 01 theme prioritizes determinism gates, redaction policy, and CI policy enforcement. Rewriting `test-orch` would slow these deliverables. We need a simpler, native CI path that:

- Runs on GitHub Actions without Docker.
- Invokes crate tests deterministically with stable outputs.
- Is easy to extend with golden fixtures and schema validation in future sprints.

## Decision

Deprecate `test-orch/` and remove it from CI. Adopt GitHub Actions as the only orchestrator for now, and introduce a lightweight, repository-root `test-ci-runner` (Python) that runs our Rust test suites and emits CI-friendly status.

## Rationale

- GitHub Actions already supports our Rust matrix and caching.
- A small, auditable runner script is easier to evolve than a multi-language, Docker-heavy orchestrator.
- Aligns with our Testing Policy (zero-SKIP; fix product/tests instead of masking).

## Scope

- In scope: CI migration, new runner script, workflow changes, deprecation note.
- Out of scope: Cross-filesystem EXDEV matrix and containerized acceptance. We will re-introduce as separate, incremental work when needed.

## Consequences

- Faster iteration on CI policy (goldens, schema gates) because the runner is simple and lives in-repo.
- Loss of immediate Docker-based acceptance runs; acceptable given current priorities and broken state.

## Options Considered

1. Rewrite `test-orch` (Go + Docker) now
   - Pros: parity with previous design; container fidelity
   - Cons: large time investment; distracts from Sprint 01 goals
2. Use GitHub Actions + Python test-ci-runner (Chosen)
   - Pros: simple, maintainable, integrates with existing jobs
   - Cons: no Docker parity; limited to Actions environment
3. Use Makefile-only orchestration
   - Pros: minimal tooling
   - Cons: less structure for later golden/schema expansion

## Implementation Plan

1. Create `test_ci_runner.py` at repo root to:
   - Run `cargo test` (configurable via args).
   - Stream output and return non-zero exit codes on failures.
   - Provide a stable place to add golden/schema checks later.
2. Update `.github/workflows/ci.yml`:
   - Remove the `test-orch` job.
   - Add a new `test-ci-runner` job that calls the Python runner after unit tests.
3. Leave the `test-orch/` directory in the tree for now (archived) or delete in a separate change after stakeholders confirm.
4. Document the migration in PLAN and TODO.

## Acceptance Criteria

- CI green with the new `test-ci-runner` job replacing `test-orch`.
- No references to `test-orch` remain in CI.
- Runner supports basic flags to target packages/tests as needed.

## Risks & Mitigations

- Risk: Loss of container fidelity for EXDEV matrix.
  - Mitigation: Plan a separate, explicit acceptance track once golden/schema gates are in place.

## Future Work

- Add golden fixtures generation and schema validation into `test_ci_runner.py`.
- Add optional matrix execution (Rust toolchains, feature sets) through runner flags.
- Re-introduce containerized acceptance as a dedicated workflow, not as a hard dependency for core CI.
