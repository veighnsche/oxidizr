# Node hazards: SUID/SGID and hardlinks

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Detect and gate on node hazards including SUID/SGID permission bits and multi-link (hardlink) targets. Policy toggles allow operators to permit certain hazards with explicit acknowledgment.

## Implementation

- Checks: `cargo/switchyard/src/preflight/checks.rs::{check_suid_sgid_risk, check_hardlink_hazard}`
  - `check_suid_sgid_risk(path) -> io::Result<bool>` resolves symlinks and inspects mode bits (04000/02000) best-effort.
  - `check_hardlink_hazard(path) -> io::Result<bool>` uses `symlink_metadata` and `nlink > 1` for regular files.
- Policy: `cargo/switchyard/src/policy/config.rs`
  - `allow_suid_sgid_mutation: bool`
  - `allow_hardlink_breakage: bool`
- Enforcement wiring: `cargo/switchyard/src/policy/gating.rs::evaluate_action()`
  - Adds STOPs or notes based on check results and policy flags.

## Wiring Assessment

- Preflight populates STOPs/notes; Apply enforces via E_POLICY unless `override_preflight=true`.
- Conclusion: wired correctly; hazards surfaced deterministically when detectable, advisory otherwise.

## Evidence and Proof

- Preflight rows include notes when hazards are present; policy STOP path tested in preflight/apply tests.

## Gaps and Risks

- SUID/SGID check is best-effort for symlinked targets; failures do not STOP by themselves (advisory) unless deterministically detected.

## Next Steps to Raise Maturity

- Add explicit tests for SUID/SGID and hardlink STOP vs allowed-by-policy scenarios; add goldens for both.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- `cargo/switchyard/src/preflight/checks.rs`
- `cargo/switchyard/src/policy/gating.rs`
