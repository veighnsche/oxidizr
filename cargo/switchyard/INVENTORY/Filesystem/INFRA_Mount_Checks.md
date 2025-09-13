# Mount checks (rw+exec)

- Category: Infra
- Maturity: Silver

## Summary

Ensures targets reside on mounts that are writable and executable before mutations proceed.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Prevents mutations on read-only or no-exec mounts | `cargo/switchyard/src/fs/mount.rs::{ProcStatfsInspector, ensure_rw_exec}`; preflight checks |
| Fail-closed on ambiguity | Implementation treats parse/resolve issues as not rw+exec |
| Policy-driven: add extra roots to verify | `policy::Policy::extra_mount_checks` |

| Cons | Notes |
| --- | --- |
| Parser depends on `/proc/self/mounts` | Lacks statfs fallback; noted in Gaps |
| Canonicalization can be best-effort | May proceed with raw path on canonicalization failure (advisory) |

## Behaviors

- Parses `/proc/self/mounts` via `ProcStatfsInspector` to collect mount flags.
- Verifies the target and any policy-provided extra roots are on mounts with `rw` and `exec`.
- On ambiguity or parsing errors, fails closed (treats as not rw+exec) to avoid unsafe mutations.
- Used by preflight to produce STOP/notes and by gating to enforce in Commit.

## Implementation

- Mount inspector and helper: `cargo/switchyard/src/fs/mount.rs::{MountInspector, ProcStatfsInspector, ensure_rw_exec}`.
- Preflight wrapper: `cargo/switchyard/src/preflight/checks.rs::ensure_mount_rw_exec()`.

## Wiring Assessment

- Preflight and gating iterate additional mount roots from policy and verify target.
- Conclusion: wired correctly; fail-closed on ambiguity.

## Evidence and Proof

- Unit tests for `ensure_rw_exec` with mocked inspector.

## Feature Analytics

- Complexity: Low-Medium. Parser + flag checks + policy integration.
- Risk & Blast Radius: High safety value; wrong positive could block safe ops; defaults prefer safety.
- Performance Budget: Minimal; parse once per run or per check.
- Observability: Preflight rows include mount status; STOPs recorded; YAML export available.
- Test Coverage: Unit tests present; gaps: add goldens for common mount setups.
- Determinism & Redaction: Deterministic given mount file; redaction not applicable.
- Policy Knobs: `extra_mount_checks` to enforce additional roots.
- Exit Codes & Error Mapping: Violations result in `E_POLICY` at apply.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: Linux `/proc/self/mounts`; consider `statfs` fallback.
- DX Ergonomics: Simple helper; policy list ergonomics adequate.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `extra_mount_checks` | `[]` | Additional roots to enforce rw+exec requirement |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.row` | `mount_ok`, `notes/stops` | Audit v2 |
| `preflight.summary` | `policy_ok`, `error_ids?` | Audit v2 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/mount.rs` | inspector tests | flag parsing and rw+exec check |
| `src/preflight/checks.rs` | ensure_mount_rw_exec tests | STOP/notes behavior |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Parser + rw+exec enforcement | STOP on not-rw+exec; fail-closed on ambiguity | Unit tests | None | Additive |
| Silver | statfs fallback; goldens | Consistent detection across envs | Additional tests/goldens | CI validation | Additive |
| Gold | Policy presets and runbooks | Clear operator guidance | Docs + CI | CI docs checks | Additive |
| Platinum | Platform matrix validation | Robust across kernels/containers | Matrix CI | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Goldens added/updated and CI gates green
- [ ] Platform fallback (statfs) implemented and tested

## Gaps and Risks

- Parser relies on `/proc/self/mounts`; no statfs-based fallback yet.

## Next Steps to Raise Maturity

- Add rustix::statfs-based implementation; golden tests for common mount configs.

## Related

- Policy `extra_mount_checks`.
