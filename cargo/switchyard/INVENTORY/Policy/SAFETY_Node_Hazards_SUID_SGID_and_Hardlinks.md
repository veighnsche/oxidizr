# Node hazards: SUID/SGID and hardlinks

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Detect and gate on node hazards including SUID/SGID permission bits and multi-link (hardlink) targets. Policy toggles allow operators to permit certain hazards with explicit acknowledgment.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Surfaces critical file hazards pre-mutation | `cargo/switchyard/src/preflight/checks.rs::{check_suid_sgid_risk, check_hardlink_hazard}` |
| Policy-configurable strictness | `policy/config.rs::{allow_suid_sgid_mutation, allow_hardlink_breakage}` |
| Fail-closed by default | Defaults are false; Commit STOPs on hazards unless overridden |

| Cons | Notes |
| --- | --- |
| Best-effort detection in some environments | Symlink resolution and metadata access can fail; advisory behavior in those cases |
| Policy overrides may allow risky operations | Operators must be explicit when downgrading hazards |

## Behaviors

- Detects SUID/SGID bits on symlink-resolved targets best-effort.
- Detects multi-link (hardlink) hazards via `nlink > 1` for regular files.
- Adds preflight STOPs when hazards are found and the policy forbids them.
- Adds notes when hazards are allowed by policy; advisory when detection inconclusive.

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

## Feature Analytics

- Complexity: Low. Metadata checks + policy integration.
- Risk & Blast Radius: High safety value; overrides increase risk; defaults prefer STOP.
- Performance Budget: Minimal; metadata reads.
- Observability: Preflight rows include notes/stops; summary may co-emit ownership/error IDs.
- Test Coverage: Gap — explicit tests for SUID/SGID and hardlink STOP vs allowed-by-policy.
- Determinism & Redaction: Deterministic given FS state; no redaction needed for flags.
- Policy Knobs: `allow_suid_sgid_mutation`, `allow_hardlink_breakage`.
- Exit Codes & Error Mapping: `E_POLICY` (10) enforced in Commit on STOP.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: Depends on Unix metadata semantics; symlink resolution may differ across FS.
- DX Ergonomics: Clear flags; behaviors documented in inventory.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `allow_suid_sgid_mutation` | `false` | If false, STOP on SUID/SGID hazard; if true, downgrade to note |
| `allow_hardlink_breakage` | `false` | If false, STOP on hardlink hazard; if true, downgrade to note |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.row` | hazard notes/stops | Minimal Facts v1 |
| `preflight.summary` | `policy_ok`, `error_ids?` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/preflight/checks.rs` | suid/sgid + hardlink tests (planned) | STOP/notes behavior per policy |
| `src/policy/gating.rs` | gating tests (planned) | enforcement in Commit |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic hazard detection + STOP/notes | Hazards surfaced; defaults STOP | Unit tests (planned) | None | Additive |
| Silver (current) | Policy-integrated gating; observability | STOPs enforced in Commit; notes recorded | Integration tests | Inventory docs | Additive |
| Gold | Goldens for hazard scenarios | Deterministic reporting; coverage | Goldens + CI | CI validation | Additive |
| Platinum | Platform matrix and edge cases | Robust across FS/containers | Matrix tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Goldens added/updated and CI gates green
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
