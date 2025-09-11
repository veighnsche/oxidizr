# Switchyard: Plan-to-Plan (Meta-Plan)

This document defines the structure, artifacts, and cadence for building the actual delivery plan for `cargo/switchyard`. It clarifies what gets planned, how we verify it, and which concrete documents we will produce next inside `cargo/switchyard/PLAN/` and related folders.

Relevant code and specs:

- Source code: `cargo/switchyard/src/`
- Normative spec: `cargo/switchyard/SPEC/` (including `.md`, `.json`, and Gherkin features)
- BDD runner: `cargo/bdd-runner/`

Goals of the meta-plan:

- Establish a layered planning stack (from strategy to tasks).
- Define the minimal set of artifacts and acceptance criteria for each layer.
- Map SPEC features (e.g., determinism, safety, conservatism) to CI gates and test evidence.
- Produce a short, time-bounded process to converge on the first executable delivery plan.

---

## Multi‑Layered Planning Stack

1) Vision & Charter

- Problem statement, constraints, scope boundaries (MVP vs later).
- Non-functional imperatives: determinism, safety preconditions, conservatism in CI, audit trail.

2) Architecture & Interfaces

- Component map and responsibilities inside `switchyard`.
- Public surface (traits, modules, CLI) and data contracts (e.g., `audit_event.schema.json`).
- Trust and safety design patterns required by SPEC (SafePath, TOCTOU-safe sequences).

3) SPEC Conformance Mapping

- Traceability from `SPEC/*.md` and `SPEC/features/*.feature` to code modules and tests.
- Evidence types per feature: unit, integration, BDD, property tests, and logs.

4) Delivery Plan & Workstreams

- Milestones with exit criteria; dependency graph.
- Workstreams (e.g., Audit, Determinism, Safety Preconditions, Conservatism CI, Core Ops).
- Rough capacity planning and sequencing, including risks and fallbacks.

5) Quality Gates & CI Policy

- Gate definitions aligned to SPEC features and `bdd-runner`.
- Reproducibility checks, safety preflight, and conservative defaults in CI.
- Audit logging coverage and schema validation within pipelines.

6) Execution Backlog

- Epics → stories → tasks with DOR/DoD.
- ADRs for non-obvious technical decisions.

---

## Artifacts to Produce (the plan to make the plan)

- PLAN/meta/00-charter.md
  - Purpose, scope, MVP, stakeholders, constraints, glossary.

- PLAN/meta/10-architecture-outline.md
  - Module boundaries, data flow, interfaces/traits, failure domains, lock/IO patterns.

- PLAN/meta/20-spec-traceability.md
  - Table mapping SPEC requirements and `.feature` scenarios to code and tests.

- PLAN/meta/30-delivery-plan.md
  - Milestones, workstreams, sequencing, estimates, and dependencies.

- PLAN/meta/40-quality-gates.md
  - CI gate definitions, required evidence, pass/fail policy and exceptions.

- PLAN/meta/50-risk-register.md
  - Risks, assumptions, mitigations, owners, and review cadence.

- PLAN/meta/60-adr-template.md & PLAN/meta/adr/ (folder)
  - Lightweight ADR template and a directory to store decisions.

- PLAN/meta/70-status-cadence.md
  - Weekly status template, demo criteria, review checklist.

Note: This README is the meta-plan. The above artifacts constitute the “real plan.”

---

## SPEC → CI Gate Alignment (seed)

- Determinism attestation: `SPEC/features/determinism_attestation.feature`
  - Evidence: deterministic IDs (UUIDv5), dry-run timestamp zeroing, reproducible logs.
  - CI: deterministic mode check on PR; snapshot/golden comparisons.

- Safety preconditions: `SPEC/features/safety_preconditions.feature`
  - Evidence: SafePath enforcement, TOCTOU-safe syscall patterns, early-fail checks.
  - CI: static checks, tests exercising failure paths, negative BDDs.

- Conservatism in CI: `SPEC/features/conservatism_ci.feature`
  - Evidence: fails closed on ambiguity, explicit override flags, zero-SKIP gate.
  - CI: policy lints, suite completeness check, expected-fail (xfail) accounting.

- Audit events: `SPEC/audit_event.schema.json`
  - Evidence: JSONL log validation, field completeness, append-only guarantees.
  - CI: schema validation step, coverage on critical operations.

---

## Process & Cadence

- Day 0–1: Discovery
  - Inventory `src/` modules and current capabilities.
  - Extract current implicit decisions into draft ADRs.

- Day 2–3: Draft
  - Create all PLAN/* skeletons above with initial content and owners.
  - Build first pass of SPEC→Code→Test traceability and quality gates.

- Day 4: Review & Ratify
  - Stakeholder review, resolve open questions, lock v0 of the plan.
  - Publish execution backlog (initial epics/stories) and CI gates.

- Ongoing: Weekly
  - Status updates, risk review, ADR updates, plan refinement.

---

## Working Agreements

- Definition of Ready: story has acceptance criteria, SPEC links, and test strategy.
- Definition of Done: code + tests + docs + audit logs + CI green, SPEC trace updated.
- ADRs for decisions that impact correctness, determinism, or safety.

---

## Immediate Next Actions

1) Create PLAN/* skeleton files listed above.
2) Inventory `cargo/switchyard/src/` and list modules/functions with owners.
3) Draft `20-spec-traceability.md` with initial links to `.feature` files and `audit_event.schema.json`.

Exit criteria for this meta‑phase: all skeleton artifacts exist with owners and initial content, CI gate list is sketched, and SPEC traceability has a first pass.
