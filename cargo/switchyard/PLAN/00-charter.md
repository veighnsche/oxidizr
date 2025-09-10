# Switchyard Charter

## Purpose

Define the scope, constraints, and MVP for `cargo/switchyard`, enabling safe, deterministic, and auditable filesystem and process operations per SPEC.

## Scope (MVP)

- Implement core API surfaces described in `SPEC/features/*.feature` for safe filesystem operations and symlink management.
- Provide preflight checks and conservative defaults suitable for CI usage.
- Emit structured audit events matching `SPEC/audit_event.schema.json`.

Out of scope (MVP):

- Full recovery tooling and initramfs integrations.
- Cross-host orchestration.

## Non-Functional Requirements

- Determinism and reproducibility (IDs, timestamps in dry-run, stable logs).
- Safety preconditions (SafePath, TOCTOU-safe sequences, explicit consent).
- Conservatism in CI (fail-closed on ambiguity, zero-SKIP policy).
- Observability and auditability (JSONL logs, schema-validated, append-only).

## Stakeholders

- Engineering (authors/maintainers)
- CI/Release engineers
- Security/Platform reviewers

## Constraints

- Rust stable toolchain; compatible with existing repo CI.
- Minimal external dependencies; alignment with SPEC and BDD runner.

## MVP Exit Criteria

- Green CI with quality gates.
- SPEC traceability first pass complete.
- Core scenarios in `SPEC/features` implemented and tested.
