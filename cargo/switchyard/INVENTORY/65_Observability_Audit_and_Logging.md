# Audit and logging (StageLogger) — Concept

**Owner:** <owner>
**Last Reviewed:** 2025-09-14 · **Next Review:** 2025-10-14

---

## 1) Contract (non-negotiables)

- Centralized emission with stable envelope and redaction in DryRun.
- Stage-specific events: plan, preflight(.summary), apply.attempt/result, rollback(.summary), prune.result.
- Facts emit to pluggable sinks.

Policy Impact:

- None directly; envelope contains policy outcomes (error_id/exit_code etc.).

---

## 2) Wiring (code-referential only)

- `cargo/switchyard/src/logging/audit.rs:107-144` (StageLogger)
- `cargo/switchyard/src/logging/audit.rs:146-254` (EventBuilder)
- `cargo/switchyard/src/logging/audit.rs:256-335` (redact_and_emit envelope)
- Sinks: `cargo/switchyard/src/logging/facts.rs:12-21` (JsonlSink), `:23-97` (FileJsonlSink feature)

---

## 3) Risk & Impact (scored)

- Impact: 3 — Uniform observability for analysis and goldens.
- Likelihood: 2 — Centralized helper.
- Risk: 6 → Tier: Silver

---

## 4) Maturity / Robustness

- [x] StageLogger in use across stages
- [ ] JSON Schema validation in CI
- [ ] Golden facts

**Current Tier:** Silver
**Target Tier:** Gold

---

## 5) Failure Modes

- Missing schema validation → drift undetected; mitigated by planned tests.

---

## 6) Evidence

- Apply/preflight/rollback codepaths call StageLogger; sinks compile and can be injected.

---

## 7) Gaps → Actions

| ID | Action | Evidence target | Expected effect | Owner | Due |
|----|--------|-----------------|-----------------|-------|-----|
| G-1 | Add schema validation and golden facts | tests + CI | L: 2→1 | <owner> | 2025-10-14 |

---

## 8) Notes for IDE-AI

- Ensure all stage emissions go through StageLogger; verify dry-run redaction.
