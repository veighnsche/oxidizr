## Sprint 03 — Draft (Proposed)

### Theme

- CI Traceability + Smoke Suite Expansion + Policy/Docs Sync

### Objectives

- Produce CI traceability artifacts and integrate with PRs/releases.
- Expand smoke runner toward SPEC §11 command set (deterministic subset).
- Complete SPEC_UPDATE/ADR notes for policy flags and error taxonomy boundaries.

### Candidate Stories

1) Traceability CI Integration (Owner: Dev-D)
   - Add CI job to run `python SPEC/tools/traceability.py` and upload artifact.
   - Gate (non-blocking) pass/fail on script health; report linked in PR summary.

2) Smoke Runner Command Coverage (Owner: Dev-E)
   - Implement deterministic subset of commands (e.g., `stat`, `readlink`, `sha256sum`) with fixed args and environment.
   - Add tests for command invocation and failure classification; ensure auto-rollback path preserves invariants.

3) Policy & SPEC Doc-Sync (Owner: Dev-C)
   - SPEC_UPDATE for `override_preflight`, `require_preservation` and clarifications in preflight rows.
   - ADR addendum on Silver-tier scope and deferrals.

4) Exit Code Coverage Report (Owner: Dev-B)
   - Generate a simple summary (by test) of covered `ErrorId`→`exit_code` in CI; attach as artifact.

### Risks / Constraints

- Keep smoke deterministic (no time or locale dependence); avoid fragile file system expectations.
- CI environment limitations; ensure jobs run without privileged operations.

### References

- `SPEC/tools/traceability.py`, `.github/workflows/ci.yml`
- `src/adapters/smoke.rs`, `src/api/apply.rs`
