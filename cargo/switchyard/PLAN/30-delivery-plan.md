# Delivery Plan (v0)

## Milestones

- M1: Planning skeletons + traceability v0 + gates sketch (this week)
- M2: Core SafePath/TOCTOU primitives with tests
- M3: Determinism and audit logging coverage
- M4: Conservatism CI policy enforced in pipeline
- M5: Release candidate v0.1 with full SPEC trace for MVP

## Workstreams

- WS-A: Safety Preconditions & FS Ops (`preflight.rs`, `fs_ops.rs`)
- WS-B: API & Determinism (`api.rs`)
- WS-C: Observability/Audit & Schema validation
- WS-D: CI Gates & BDD integration

## Dependencies

- BDD runner stability and feature step coverage.

## Risks

- See `PLAN/50-risk-register.md`.

## Review Cadence

- Weekly status, mid-milestone demos, CI gate health.
