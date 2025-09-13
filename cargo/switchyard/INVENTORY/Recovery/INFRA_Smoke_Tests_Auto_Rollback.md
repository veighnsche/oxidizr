# Smoke tests and auto-rollback

- Category: Infra
- Maturity: Silver

## Summary

Post-apply health verification via `SmokeTestRunner`. Failures map to `E_SMOKE` and trigger auto-rollback unless disabled by policy.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Early detection of broken state | `cargo/switchyard/src/api/apply/mod.rs` runs smoke post-apply |
| Auto-rollback reduces MTTR | Apply attempts rollback on smoke failure |
| Pluggable runner per environment | `adapters/smoke.rs::{SmokeTestRunner, DefaultSmokeRunner}` |

| Cons | Notes |
| --- | --- |
| Default runner is minimal | Must supply environment-specific checks |
| False positives risk rollback | Runner quality critical; policy can tune behavior |

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

## Feature Analytics

- Complexity: Low-Medium. Hook + adapter + rollback orchestration.
- Risk & Blast Radius: Medium-High; false positives/negatives can lead to rollbacks or missed issues.
- Performance Budget: Runner-dependent; should be fast sanity checks.
- Observability: `apply.result` includes failure classification; rollback facts emitted.
- Test Coverage: Basic coverage; gaps: golden for smoke failure → rollback path.
- Determinism & Redaction: Facts redacted in DryRun; smoke only in Commit.
- Policy Knobs: `require_smoke_in_commit`, `disable_auto_rollback`.
- Exit Codes & Error Mapping: `E_SMOKE` (80) on failure.
- Concurrency/Locking: Follows apply lock; runs post-mutation.
- Cross-FS/Degraded: N/A.
- Platform Notes: Runner must be tailored to platform/app domain.
- DX Ergonomics: Simple adapter trait; integrators can plug custom checks.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `require_smoke_in_commit` | `false` (true in prod preset) | Require a runner and passing result in Commit |
| `disable_auto_rollback` | `false` | When true, do not auto-rollback on failure |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_SMOKE` | 80 | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `error_id=E_SMOKE` on failure; `rolled_back` | Minimal Facts v1 |
| `rollback.step`/`rollback.summary` | rollback details | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `tests/smoke_rollback.rs` | smoke failure triggers rollback | classification + rollback execution |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Hook for post-apply checks | Smoke can run; classification present | Unit/integration | None | Additive |
| Silver (current) | Auto-rollback on failure; policy knobs | Rollback executed; codes mapped | Integration tests | Inventory docs | Additive |
| Gold | Golden smoke scenarios; richer runners | Deterministic reporting; robust checks | Goldens + CI | CI validation | Additive |
| Platinum | Health SLOs and rollback policies | Continuous health validation and auto-remediation | Monitoring + CI | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Goldens added/updated and CI gates green
## Gaps and Risks

- Default runner is minimal; environment-specific suites required.

## Next Steps to Raise Maturity

- Provide reference suites and goldens for smoke failure/rollback.

## Related

- SPEC v1.1 smoke requirements in production preset.
