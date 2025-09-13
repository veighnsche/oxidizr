# Smoke tests and auto-rollback

- Category: Infra
- Maturity: Silver

## Summary

Post-apply health verification via `SmokeTestRunner`. Failures map to `E_SMOKE` and trigger auto-rollback unless disabled by policy.

## Behaviors

- Executes configured `SmokeTestRunner` after successful apply in Commit mode.
- On smoke failure, maps to `E_SMOKE` and attempts rollback of executed actions unless disabled by policy.
- Emits failure classification in summary facts; includes rollback step events.
- When no runner present but required by policy, records an error and attempts rollback accordingly.

## Implementation

- Adapter: `cargo/switchyard/src/adapters/smoke.rs::{SmokeTestRunner, DefaultSmokeRunner}`.
- Apply integration: `cargo/switchyard/src/api/apply/mod.rs` runs smoke, maps failure to `E_SMOKE`, and attempts rollback.
- Policy: `require_smoke_in_commit`, `disable_auto_rollback` in `policy::Policy`.

## Wiring Assessment

- Only in Commit mode; DryRun unaffected. Summary facts include failure classification.
- Conclusion: wired correctly; effectiveness depends on runner quality.

## Evidence and Proof

- Apply code paths and tests exercise rollback; default runner validates symlink resolution.

## Gaps and Risks

- Default runner is minimal; environment-specific suites required.

## Next Steps to Raise Maturity

- Provide reference suites and goldens for smoke failure/rollback.

## Related

- SPEC v1.1 smoke requirements in production preset.
