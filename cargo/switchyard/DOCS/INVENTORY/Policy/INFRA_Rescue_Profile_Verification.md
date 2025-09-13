# Rescue profile verification

- Category: Infra
- Maturity: Silver

## Summary

Verifies that a minimal rescue toolset (BusyBox or GNU subset) exists on PATH. Policy can require rescue; preflight and apply honor it.

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

## Gaps and Risks

- Container/base-image locale/tool availability can cause flakes outside of code.

## Next Steps to Raise Maturity

- Golden for missing rescue; CI environment preparation guidance.

## Related

- SPEC rescue profile section; infra test-orch project.
