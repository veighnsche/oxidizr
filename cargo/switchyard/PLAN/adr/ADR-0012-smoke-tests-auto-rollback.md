# ADR Template

- Title: Minimal smoke tests and auto-rollback policy
- Status: Proposed
- Date: 2025-09-11

## Context

SPEC requires a minimal post-apply smoke suite to verify operational health (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date). Any mismatch or non-zero exit must trigger auto-rollback unless explicitly disabled by policy.

## Decision

- Provide a `SmokeTestRunner` adapter interface the engine calls after successful `apply` mutations.
- Treat smoke failure as `E_SMOKE`; initiate automatic rollback unless `policy.disable_auto_rollback=true`.
- Record detailed smoke results in facts and include failure diagnostics.
- Include a tiny checkfile for `sha256sum -c` as part of the test bundle.

## Consequences

+ Early detection of regressions with safe rollback by default.
+ Clear operator control via explicit policy to disable automatic rollback.
- Adds a small runtime cost to post-apply verification.

## Links

- `cargo/switchyard/SPEC/SPEC.md` §§ 2.9, 11
- `cargo/switchyard/PLAN/10-architecture-outline.md`
