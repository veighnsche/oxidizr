# Switchyard Planning Review Instructions

This document guides all reviewer parties through the review of the Switchyard planning package. It references the normative SPEC and the planning artifacts under `cargo/switchyard/`.

Use this as your single source of truth for what to read, what to verify, and what to sign off.

---

## Quick Links

- SPEC (normative):
  - `cargo/switchyard/SPEC/SPEC.md`
  - `cargo/switchyard/SPEC/requirements.yaml`
  - `cargo/switchyard/SPEC/audit_event.schema.json`
  - `cargo/switchyard/SPEC/preflight.yaml`
  - `cargo/switchyard/SPEC/error_codes.toml`
  - `cargo/switchyard/SPEC/features/` (Gherkin)
  - `cargo/switchyard/SPEC/features/steps-contract.yaml`
  - `cargo/switchyard/SPEC/tools/traceability.py`
  - `cargo/switchyard/SPEC/traceability.md` (generated report)

- Planning (this phase):
  - `cargo/switchyard/PLAN/README.md` (meta-plan)
  - `cargo/switchyard/PLAN/TODO.md` (planning checklist)
  - `cargo/switchyard/PLAN/00-charter.md`
  - `cargo/switchyard/PLAN/10-architecture-outline.md`
  - `cargo/switchyard/PLAN/20-spec-traceability.md`
  - `cargo/switchyard/PLAN/30-delivery-plan.md`
  - `cargo/switchyard/PLAN/40-quality-gates.md`
  - `cargo/switchyard/PLAN/50-risk-register.md`
  - `cargo/switchyard/PLAN/70-status-cadence.md`
  - `cargo/switchyard/PLAN/adr/` (ADR set)

---

## Review Flow (All Parties)

1. Read `PLAN/README.md` to understand the planning scope and expected outputs.
2. Skim `SPEC/SPEC.md` sections 1–3 (Main Guarantees, Normative Requirements, Public Interfaces) to calibrate expectations.
3. Follow your role-specific checklist below.
4. Run evidence checks (where applicable) and record results.
5. Provide sign-off per `PLAN/40-quality-gates.md`.

---

## Maintainers / Tech Lead

Focus: correctness, feasibility, and internal consistency of the design.

- Architecture & API
  - Verify `PLAN/10-architecture-outline.md` planned API matches `SPEC/SPEC.md §3.1`:
    - `plan(PlanInput) -> Plan`
    - `preflight(&Plan) -> PreflightReport`
    - `apply(&Plan, ApplyMode, &Adapters) -> ApplyReport`
    - `plan_rollback_of(&ApplyReport) -> Plan`
  - Confirm SafePath-only mutations (REQ-API1) and TOCTOU-safe sequence (REQ-TOCTOU1) are captured in modules (`fs_ops.rs`, `api.rs`).
  - Check concurrency model and `LockManager` boundaries (REQ-L1..L4; SPEC §14 thread-safety).

- Error Model & Taxonomy
  - Cross-check `PLAN/10-architecture-outline.md` and `PLAN/adr/ADR-0002-error-strategy.md` with `SPEC/error_codes.toml`.
  - Ensure stable identifiers (e.g., `E_LOCKING`) and exit codes appear in facts per SPEC §6.

- Determinism & Facts
  - Review `ADR-0006-determinism-ids.md` for UUIDv5 namespace and normalization strategy (REQ-D1, REQ-D2).
  - Ensure golden fixture policies are feasible (`PLAN/30-delivery-plan.md`, `PLAN/40-quality-gates.md`).

- Filesystems & EXDEV
  - Validate `ADR-0011-exdev-degraded-mode.md` against REQ-F1..F3 and acceptance tests plan.

- Outcome
  - Provide Architecture sign-off in `PLAN/40-quality-gates.md`.

---

## Security / Platform Reviewers

Focus: safety preconditions, provenance, masking, attestation, and policy.

- Safety Preconditions
  - Verify `PLAN/10-architecture-outline.md` and `ADR-0008-safepath-toctou.md` enforce SafePath and the normative TOCTOU sequence (REQ-S1..S5, REQ-TOCTOU1).
  - Confirm preflight gating for preservation capabilities; see `ADR-0010-preflight-diff-preservation.md` (REQ-S5).

- Observability & Attestation
  - Review `ADR-0003-audit-attestation-logging.md` for facts schema v1, secret masking, provenance completeness (REQ-O1..O7, REQ-VERS1).
  - Check signing details (ed25519) and dev/test key provisioning approach.

- Conservatism & Modes
  - Confirm defaults: dry-run by default; fail-closed without override (REQ-C1, REQ-C2). See `PLAN/10-architecture-outline.md` and `ADR-0004-feature-flags-policy.md`.

- Outcome
  - Provide Security sign-off in `PLAN/40-quality-gates.md`.

---

## SRE / CI

Focus: CI gates and automated evidence.

- Quality Gates & Zero-SKIP
  - Validate `PLAN/40-quality-gates.md` gates and exception policy.
  - Ensure `SPEC/tools/traceability.py` is planned in CI to produce `SPEC/traceability.md` and fail on uncovered MUST/MUST_NOT or unmatched steps.

- Schema Validation & Golden Fixtures
  - Confirm planned jobs for JSON schema validation (`SPEC/audit_event.schema.json`) and YAML preflight validation (`SPEC/preflight.yaml`).
  - Review golden fixture diff policy and stability measures (deterministic ordering, redactions).

- Test Matrix & Environments
  - See `PLAN/30-delivery-plan.md` Test Matrix and Planning Backlog for PR vs nightly coverage.
  - Note adapters: minimal Rust `cucumber-rs` or Go `godog` shim integration with existing `test-orch/`.

- Outcome
  - Provide Quality/SRE sign-off in `PLAN/40-quality-gates.md`.

---

## QA / Requirements

Focus: traceability, coverage, and acceptance criteria.

- Traceability & Coverage
  - Review `PLAN/20-spec-traceability.md` mapping and acceptance criteria conventions.
  - Run the traceability tool:
    - `python3 cargo/switchyard/SPEC/tools/traceability.py`
    - Confirm `cargo/switchyard/SPEC/traceability.md` shows 100% coverage of MUST/MUST_NOT.

- Gherkin Features & Contract
  - Ensure `.feature` files tag scenarios with requirement IDs and match `features/steps-contract.yaml`.
  - Check health verification scenarios (smoke suite) and rollback behavior (REQ-H1..H3).

- Outcome
  - Provide Requirements/QA sign-off in `PLAN/40-quality-gates.md`.

---

## PM / Stakeholders

Focus: scope, milestones, risks, and cadence.

- Charter & Scope
  - Verify `PLAN/00-charter.md` goals/non-goals and success metrics.

- Delivery Plan
  - Review `PLAN/30-delivery-plan.md` milestones (M1–M5), entry/exit criteria, dependencies, and Planning Backlog priorities.

- Risks & Cadence
  - Review `PLAN/50-risk-register.md` risks, triggers, owners; assumptions & fallback plans.
  - Check `PLAN/70-status-cadence.md` weekly rhythm and artifact update expectations.

- Outcome
  - Provide Delivery/PMO sign-off in `PLAN/40-quality-gates.md`.

---

## Auditors (Optional, if applicable)

Focus: evidence sufficiency and reproducibility posture.

- Evidence Plan
  - Validate that required artifacts will be reproducible and independently verifiable: facts JSONL, preflight diffs, golden fixtures, attestation bundles.
  - Ensure schema versioning and migration plan in `SPEC/SPEC.md §13` is acknowledged.

- Determinism Checks
  - Confirm UUIDv5 policy and redaction rules support audit repeatability.

---

## Commands & Evidence (One-Click Checks)

- Generate the traceability report and exit on issues:
```
python3 cargo/switchyard/SPEC/tools/traceability.py
```
- Validate JSON schema of facts (placeholder example; actual wiring in CI):
```
# Example: use `ajv` or `jsonschema` tool of choice in CI
```
- Validate YAML preflight schema (placeholder example; actual wiring in CI):
```
# Example: use `pykwalify` or `yamale` in CI
```

---

## Sign-offs & Recording

Record sign-offs in `cargo/switchyard/PLAN/40-quality-gates.md` under the Sign-offs section. Update `cargo/switchyard/PLAN/TODO.md` once all planning-phase sign-offs are complete. Promote relevant ADRs from Proposed to Accepted after sign-off.
