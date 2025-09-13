# Rescue profile verification

- Category: Infra
- Maturity: Silver

## Summary

Verifies that a minimal rescue toolset (BusyBox or GNU subset) exists on PATH. Policy can require rescue; preflight and apply honor it.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Ensures recovery tools are available | `cargo/switchyard/src/policy/rescue.rs::verify_rescue_tools_with_exec_min()` |
| Policy-enforceable in production | `policy::Policy::{require_rescue, rescue_exec_check, rescue_min_count}` |
| Deterministic summary in preflight | Preflight summary includes `rescue_profile` |

| Cons | Notes |
| --- | --- |
| Environment-dependent and flaky in containers | PATH and tools vary; exec checks can fail due to locales/permissions |
| Requires careful preset tuning | Presets can set min counts, but environments differ |

## Behaviors

- Scans PATH for presence of required rescue tools (BusyBox or configured subset).
- Applies `rescue_min_count` threshold and `rescue_exec_check` strategy.
- Records `rescue_profile` in preflight summary; adds STOP when `require_rescue=true` and unavailable.
- Honors environment override in tests to simulate presence/absence.

## Implementation

- `cargo/switchyard/src/policy/rescue.rs` provides `verify_rescue_tools_with_exec_min()`; environment override for tests.
- Policy: `require_rescue`, `rescue_exec_check`, `rescue_min_count`.

## Wiring Assessment

- Preflight summary includes `rescue_profile`; STOP when `require_rescue` and unavailable.
- Gating helper enforces STOP in Commit when unmet.
- Conclusion: wired correctly.

## Evidence and Proof

- Unit tests under `policy/rescue.rs` (serial env var overrides).

## Feature Analytics

- Complexity: Low. PATH scan + optional exec test.
- Risk & Blast Radius: Medium; mis-detection can block applies or allow under-provisioned environments.
- Performance Budget: Minimal; PATH scan and optional `--help` execs.
- Observability: Preflight summary records `rescue_profile` status.
- Test Coverage: Unit tests present; gaps: golden for missing rescue and CI environment prep guidance.
- Determinism & Redaction: Deterministic given PATH contents.
- Policy Knobs: `require_rescue`, `rescue_exec_check`, `rescue_min_count`.
- Exit Codes & Error Mapping: `E_POLICY` (10) when requirement unmet.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: BusyBox vs GNU variants; container images differ.
- DX Ergonomics: Presets assist, but operators must tailor to env.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `require_rescue` | `false` (true in prod preset) | STOP when rescue toolchain unavailable |
| `rescue_exec_check` | `false` (true in prod preset) | Also attempt exec checks to validate tools |
| `rescue_min_count` | `RESCUE_MIN_COUNT` | Minimum number of GNU tools when BusyBox absent |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | 10 | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.summary` | `rescue_profile`, `policy_ok` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/policy/rescue.rs` | unit tests (env overrides) | detection and thresholds |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | PATH scan and summary | Presence detected | Unit tests | Docs | Additive |
| Silver (current) | Policy-enforceable gating; exec checks | STOPs in Commit on missing rescue | Unit + integration | Inventory | Additive |
| Gold | Goldens; CI environment preparation | Deterministic reporting; operator guidance | Goldens + CI | CI docs | Additive |
| Platinum | Platform matrix validation | Robust across base images | Matrix tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Goldens added/updated and CI gates green
## Gaps and Risks

- Container/base-image locale/tool availability can cause flakes outside of code.

## Next Steps to Raise Maturity

- Golden for missing rescue; CI environment preparation guidance.

## Related

- SPEC rescue profile section; infra test-orch project.
