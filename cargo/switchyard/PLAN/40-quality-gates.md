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

---

## Planning‑Stage Gates (this phase)

- Planning Completeness Gate
  - Checks: All `PLAN/*` skeletons exist with owners; sections populated per templates.
  - Evidence: Git diff + presence checks; reviewers verify substance.

- Traceability Gate (planning)
  - Checks: `SPEC/tools/traceability.py` runs successfully; `SPEC/traceability.md` generated; MUST/MUST_NOT coverage reported.
  - Evidence: CI job artifact and non-zero exit on uncovered MUSTs.

- Link & Schema Lint Gate (planning)
  - Checks: Markdown links resolve; front-matter and anchors valid; YAML/JSON schemas load.
  - Evidence: docs-lint job success; pre-commit hooks optional.

## Sign‑offs (planning)

- Charter (`00-charter.md`): Maintainers + Security reviewer
- Architecture (`10-architecture-outline.md`): Tech lead + Maintainers
- Traceability (`20-spec-traceability.md`): QA/Requirements + Maintainers
- Delivery (`30-delivery-plan.md`): PM/Tech lead
- Gates (`40-quality-gates.md`): SRE/CI lead
- Risks (`50-risk-register.md`): PM
- ADR Set (`PLAN/adr/*`): Owners per ADR topic

## Automated Checks (planned CI)

- `python3 SPEC/tools/traceability.py` → fail on uncovered MUST/MUST_NOT or unmatched steps
- JSON schema validation of emitted facts against `SPEC/audit_event.schema.json`
- YAML schema validation of preflight diffs against `SPEC/preflight.yaml`
- Golden fixture diff job with zero‑SKIP enforcement
