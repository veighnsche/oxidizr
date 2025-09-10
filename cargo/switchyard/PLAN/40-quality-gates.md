# Quality Gates (Aligned to SPEC)

## Gates

- Determinism Gate
  - Checks: deterministic IDs/logs in deterministic mode; golden snapshot compare.
- Safety Preconditions Gate
  - Checks: SafePath enforcement, TOCTOU pattern tests, negative paths.
- Conservatism CI Gate
  - Checks: zero-SKIP policy, expected-fail accounting, fail-closed on ambiguity.
- Audit Schema Gate
  - Checks: JSONL events validated against `SPEC/audit_event.schema.json`.

## Pipeline Integration

- Unit + integration + BDD suites.
- Schema validation step for audit logs.
- Policy lints for conservative defaults.

## Exception Policy

- Temporary waivers require ADR and explicit owner/date; timeboxed.
