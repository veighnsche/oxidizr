# Delivery Plan (v0)

## Milestones

- M1 (2025-09-12): Planning skeletons complete + traceability v0 + gates sketch
  - Entry: Meta-plan (`PLAN/README.md`) acknowledged; stakeholders identified
  - Exit: All PLAN/* skeletons created and owned; `SPEC/tools/traceability.py` runs; `40-quality-gates.md` drafted

- M2 (2025-09-19): Architecture/API freeze (planning) + ADR set v1
  - Entry: `10-architecture-outline.md` drafted; open questions listed
  - Exit: Public API sketch stabilized; ADRs created and accepted for crate layout, error model, SafePath/TOCTOU, locking, determinism, attestation, dependency policy

- M3 (2025-09-26): SPEC mapping and evidence plan complete
  - Entry: `20-spec-traceability.md` seed exists
  - Exit: Full requirement→module→test mapping; acceptance criteria conventions captured; golden fixture plan defined

- M4 (2025-10-03): CI/CD plan & gates ratified (planning)
  - Entry: Quality gates drafted
  - Exit: CI stages defined (lint-docs, traceability, schema-validate, golden-diff, BDD smoke); exception policy documented; sign-offs assigned

- M5 (2025-10-10): Ready-to-implement package (v0 planning complete)
  - Entry: Risks documented; cadence in place; backlog groomed
  - Exit: Green planning checks; sign-off across stakeholders; handoff to implementation

## Workstreams

- WS-A: Safety Preconditions & FS Ops (`preflight.rs`, `fs/*`)
- WS-B: API & Determinism (`api.rs`)
- WS-C: Observability/Audit & Schema validation
- WS-D: CI Gates & BDD integration

## Test Matrix (planning)

- Filesystems: ext4, xfs, btrfs, tmpfs
  - PR CI: tmpfs, ext4
  - Nightly/Matrix: xfs, btrfs
- Modes: Dry-run (default), RealRun
- Scenarios: BDD features under `cargo/switchyard/SPEC/features/*.feature`
- Evidence: Golden fixtures for plan, preflight, apply, rollback; `SPEC/traceability.md` coverage
- Environments: containerized (Docker/LXD) via existing test orchestrator adapters; minimal Rust or Go adapter to drive library APIs

## Planning Backlog (prioritized)

1) Finalize UUIDv5 namespace selection and normalization rules (Determinism)
2) Define SafePath root selection strategy and error messages (Safety)
3) LockManager timeout defaults and telemetry field semantics (Concurrency)
4) Attestation dev/test key provisioning approach in CI (Audit)
5) Preflight capability detection scope and overrides (Safety Preconditions)
6) EXDEV fallback acceptance criteria details per FS (Filesystems)
7) Golden fixture governance policy draft (CI)
8) Dependency license audit checklist and tooling (Supply chain)
9) API stability notes and deprecation template (Semver)
10) Minimal adapter selection (cucumber-rs vs godog) and scaffolding plan (BDD)

## Epics → Stories (planning-only)

- Epic A: Safety & TOCTOU (planning)
  - Story A1: Define SafePath type semantics and constructor rules (SPEC §3.3)
  - Story A2: Document TOCTOU-safe syscall sequence and coverage plan (SPEC §2.3, §3.3)

- Epic B: Determinism & IDs (planning)
  - Story B1: Define UUIDv5 namespace strategy and normalization rules
  - Story B2: Redaction policy for timestamps; dry-run equivalence plan

- Epic C: Audit & Attestation (planning)
  - Story C1: Map fact emission points; schema v1 adoption
  - Story C2: Plan ed25519 signing and dev/test key handling; SBOM-lite inclusion

- Epic D: Locking & Concurrency (planning)
  - Story D1: Define `LockManager` semantics and timeout policy; telemetry fields
  - Story D2: Dev/test mode WARN behavior without lock manager

- Epic E: CI Gates & Evidence (planning)
  - Story E1: Integrate `SPEC/tools/traceability.py` in CI
  - Story E2: Golden fixture governance and update policy

## Dependencies & Lead Times

- BDD runner availability and step alignment to `SPEC/features/steps-contract.yaml`
- Key management for attestation (dev/test keys initially)
- Schema validation tooling (JSON/YAML)

## Entry/Exit Criteria per Milestone

- Entry criteria defined at each milestone above
- Exit criteria include:
  - Docs updated and owners assigned
  - Traceability and lint checks passing
  - Sign-offs recorded per `40-quality-gates.md`

## CI/CD & Release Planning (planning-only)

- CI Stages (planned):
  - docs-lint (links, anchors)
  - spec-traceability (py tool)
  - schema-validate (audit events, preflight)
  - golden-diff (fixtures)
  - bdd-smoke (adapters TBD)
- Versioning & tags: plan v0.1.0-alpha for first implementation drop; semver policy via ADR
- Pre-release checks: docs completeness, API audit, SBOM/provenance presence

## Documentation & DevEx (planning-only)

- Crate README: purpose, API quickstart (post-implementation), safety model, determinism, CI gates
- Module-level docs: responsibilities and invariants
- Contribution guidelines: align with repo-level, add crate-specific notes
- Examples & guides: planned scenarios mapped from BDD features (post-implementation)

## Dependencies

- BDD runner stability and feature step coverage.

## Risks

- See `PLAN/50-risk-register.md`.

## Review Cadence

- Weekly status, mid-milestone demos, CI gate health.
