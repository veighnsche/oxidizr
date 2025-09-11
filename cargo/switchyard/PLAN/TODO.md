# Switchyard Cargo Library – Planning TODO

This TODO list tracks planning-only activities for the Switchyard Cargo library package. Do not implement the Rust code yet. Work through these items to establish a clear, reviewable plan and delivery path.

References:

- `cargo/switchyard/PLAN/README.md` (meta-plan and stack overview)
- `cargo/switchyard/PLAN/00-charter.md`
- `cargo/switchyard/PLAN/10-architecture-outline.md`
- `cargo/switchyard/PLAN/20-spec-traceability.md`
- `cargo/switchyard/PLAN/30-delivery-plan.md`
- `cargo/switchyard/PLAN/40-quality-gates.md`
- `cargo/switchyard/PLAN/50-risk-register.md`
- `cargo/switchyard/PLAN/60-adr-template.md` (use under `cargo/switchyard/PLAN/adr/`)
- `cargo/switchyard/PLAN/70-status-cadence.md`

## Planning Status Summary

- ✅ All planning tasks (sections 0–15) are complete.
- ⏳ Formal sign-offs are pending (see "Sign-offs" below).
- Exit criterion to begin implementation: all sign-offs recorded with owners and dates.

## 0) Planning Setup

- [x] Confirm scope: this TODO is planning-only; no Rust implementation yet
- [x] Identify stakeholders and decision makers (PM, tech lead, reviewers)
- [x] Define planning timeline and checkpoints per `70-status-cadence.md`
- [x] Establish the living docs locations and owners (who updates which doc)

## 1) Product Charter (What/Why)

- [x] Draft `00-charter.md`: problem statement, goals, non-goals
- [x] Define success metrics (functional, reliability, performance, maintainability)
- [x] Define constraints and assumptions (platforms, Rust MSRV, licensing)

## 2) Architecture Outline (How at a high level)

- [x] Draft `10-architecture-outline.md`: high-level modules, data flow, boundaries
- [x] Define public API surface vs. internal modules (stability, semver promises)
- [x] Establish error model and result types (this is planning, not implementation)
- [x] Define configuration approach (files, env, builder patterns)
- [x] Note cross-cutting concerns: logging, audit trail, determinism, SafePath/TOCTOU, locking

## 3) SPEC Traceability (Requirements)

- [x] Enumerate requirements in `20-spec-traceability.md`
- [x] Map requirements to architectural elements and planned tests
- [x] Identify compliance to existing program-level constraints (e.g., SafePath, TOCTOU-safe ops, LockManager)
- [x] Define acceptance criteria per requirement

## 4) Delivery Plan (Milestones & Work Breakdown)

- [x] Draft `30-delivery-plan.md`: phases, milestones, and target dates
- [x] Decompose into epics/stories (planning tasks only; no code tasks yet)
- [x] Identify external dependencies and lead times (crates, CI secrets, infra)
- [x] Define entry/exit criteria for each milestone (Definition of Ready/Done)

## 5) Quality Gates (Definition of Done per stage)

- [x] Update `40-quality-gates.md` with stage gates for planning outputs
- [x] Specify review sign-offs (architecture, API, security, reproducibility)
- [x] Define plan validation checks (lint docs, link checks, traceability completeness)

## 6) Risk Register (Unknowns & Mitigations)

- [x] Populate `50-risk-register.md` with technical and delivery risks
- [x] Add mitigations, owners, and trigger conditions
- [x] Track assumptions to validate and fallback plans

## 7) ADRs (Key Decisions)

- [x] Create ADRs under `adr/` using `60-adr-template.md`
- [x] Seed initial ADRs (crate layout, error strategy, logging/audit approach, feature flags)
- [x] Define decision cadence and review process

## 8) Status Cadence (Operating Rhythm)

- [x] Align standups/checkpoints with `70-status-cadence.md`
- [x] Define artifact updates expected per cadence event
- [x] Establish reporting format (brief status, risks, decisions, changes)

## 9) Public API Planning

- [x] Draft initial API sketch (module tree, traits, types) in architecture outline
- [x] Define stability policy and semver commitments
- [x] Plan deprecation policy and change management

## 10) Dependency & Supply Chain Policy

- [x] Draft policy for third-party crates (audit, minimal set, MSRV, license)
- [x] Plan for SBOM generation and provenance logging in build pipeline
- [x] Define review criteria for introducing/replacing dependencies

## 11) Determinism, Safety, and Concurrency Planning

- [x] Plan reproducibility and determinism rules (UUIDv5 for ids, timestamp control)
- [x] Specify SafePath and TOCTOU-safe sequences for any mutating I/O (if applicable)
- [x] Define locking strategy and bounded wait semantics (LockManager requirement)
- [x] Capture these as requirements in `20-spec-traceability.md`

## 12) Testing Strategy (Plan Only)

- [x] Define test taxonomy (unit, integration, property, golden, smoke)
- [x] Plan fixtures and golden outputs (locations, update policy)
- [x] Plan minimal smoke test suite and CI hooks (no CI config yet)
- [x] Plan cross-platform matrix (if applicable) and degraded modes

## 13) CI/CD & Release Planning (Plan Only)

- [x] Define planned CI stages and gates aligned with `40-quality-gates.md`
- [x] Plan versioning, tags, and release notes process
- [x] Plan pre-release checks (docs completeness, API audit, SBOM)

## 14) Documentation & Developer Experience

- [x] Plan `README` structure for the crate and module-level docs strategy
- [x] Plan contribution guidelines and code of conduct references
- [x] Plan examples and usage guides (scope only)

## 15) Backlog Grooming

- [x] Translate plan into a backlog (planning tasks only)
- [x] Prioritize by risk and dependency order
- [x] Identify fast-follower items to revisit after MVP planning

## Sign-offs

- [ ] Product/Stakeholder review of `00-charter.md`
  - Owner: Product (TBD)
  - Due: TBD
  - Notes: Confirm scope, success metrics, and constraints.

- [ ] Architecture review of `10-architecture-outline.md`
  - Owner: Tech Lead / Architecture (TBD)
  - Due: TBD
  - Notes: Validate module boundaries, public API, error model, and cross-cutting concerns.

- [ ] Requirements/QA review of `20-spec-traceability.md`
  - Owner: QA/Requirements (TBD)
  - Due: TBD
  - Notes: Verify requirement mappings to code/test evidence; ensure coverage strategy.

- [ ] Delivery/PMO review of `30-delivery-plan.md`
  - Owner: PM/Delivery (TBD)
  - Due: TBD
  - Notes: Confirm milestones, dependencies, sequencing, and exit criteria.

- [ ] Quality/SRE/Sec review of `40-quality-gates.md`
  - Owner: SRE/Security (TBD)
  - Due: TBD
  - Notes: Validate CI gates (zero-SKIP, golden diffs, schema validation) and operational checks.

- [ ] Risk review of `50-risk-register.md`
  - Owner: Risk Owner (TBD)
  - Due: TBD
  - Notes: Validate mitigations, triggers, and owners; align with delivery plan.

- [ ] Final planning package approval to begin implementation phase
  - Owner: Steering/Leads (TBD)
  - Due: TBD
  - Notes: Go/No-Go decision; after approval, transition to implementation backlog.

### Next Steps to Finish Planning

- Circulate artifacts to the listed owners and schedule brief reviews.
- Capture decisions in ADRs as needed and update PLAN docs per feedback.
- Record sign-off dates and move this section’s checkboxes to [x] upon approval.
