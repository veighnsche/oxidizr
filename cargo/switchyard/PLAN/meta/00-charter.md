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

### Toolchain & Licensing

- Rust channel: `stable` (per `rust-toolchain.toml`).
- Minimum supported Rust version (MSRV): pinned to current `stable` in CI during implementation (document exact version when code lands).
- License: GPL-3.0-only (per `LICENSE`). All dependencies must be license-compatible.

## Success Metrics (planning-level targets)

- SPEC conformance: 100% coverage of MUST/MUST_NOT in `SPEC/requirements.yaml` (validated by `SPEC/tools/traceability.py`).
- Determinism: dry-run vs real-run facts are byte-identical after redaction; golden diffs are stable across two consecutive CI runs.
- Safety: TOCTOU-safe syscall pattern verified by unit/integration/BDD tests; SafePath enforced on all mutating APIs.
- Observability: 100% of apply steps emit facts that validate against `SPEC/audit_event.schema.json`.
- Delivery: Milestones M1–M5 achieved within ±1 week of plan; zero-SKIP CI gate enforced on planning artifacts.

## MVP Exit Criteria

- Green CI with quality gates.
- SPEC traceability first pass complete.
- Core scenarios in `SPEC/features` implemented and tested.

## Living Docs Ownership

- `PLAN/00-charter.md`: Maintainers (owner), Security reviewer (co-owner for NFRs).
- `PLAN/10-architecture-outline.md`: Tech lead (owner), Maintainers (reviewers).
- `PLAN/20-spec-traceability.md`: QA/Requirements (owner), Maintainers (reviewers).
- `PLAN/30-delivery-plan.md`: PM/Tech lead (owner), All (reviewers).
- `PLAN/40-quality-gates.md`: SRE/CI (owner), Maintainers (reviewers).
- `PLAN/50-risk-register.md`: PM (owner), All (contributors).
- `PLAN/adr/*`: Tech lead (owner), reviewers per decision area.
