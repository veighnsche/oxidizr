# Preservation capabilities probe

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Probe filesystem support for preservation dimensions (owner, mode, timestamps, xattrs, ACLs, capabilities) and record both the desired preservation and what is supported.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Surfaces platform capabilities upfront | `cargo/switchyard/src/fs/meta.rs::detect_preservation_capabilities()` populates support flags |
| Feeds policy gating to prevent unsafe mutations | `policy::Policy::require_preservation`; enforced in preflight/apply |
| Exposed via YAML for operator visibility | `preflight/yaml.rs::to_yaml()` includes fields |

| Cons | Notes |
| --- | --- |
| Best-effort detection can be inconclusive | Environment-sensitive; falls back to advisory |
| No schema validation of these fields yet | Marked as gap; propose tests and schema checks |

## Behaviors

- Probes platform/filesystem to detect if preservation dimensions are supported.
- Populates `preservation` (desired) and `preservation_supported` (detected) in preflight rows.
- Feeds policy gating when strict preservation is required.
- Exposed via YAML exporter for operator visibility.

## Implementation

- Probe: `cargo/switchyard/src/fs/meta.rs::detect_preservation_capabilities()` — best-effort detection of supported preservation features.
- Preflight rows: exporter preserves `preservation` and `preservation_supported` fields
  - `cargo/switchyard/src/preflight/yaml.rs::to_yaml()` includes these keys when present.
- Policy influence: `policy::Policy` may require certain preservation guarantees (e.g., timestamps, ownership), used in preflight gating.

## Wiring Assessment

- Preflight populates capability information into rows; YAML exporter includes fields; apply/restore attempt to preserve when supported.
- Conclusion: wired correctly for advisory capability reporting.

## Evidence and Proof

- Presence of fields in YAML output; apply/restore code paths preserve owner/mode and attempt timestamps/xattrs where available.

## Feature Analytics

- Complexity: Low. Probe + row population + YAML export.
- Risk & Blast Radius: Medium; false negatives could block operations if `require_preservation=true`.
- Performance Budget: Minimal overhead; metadata/syscalls as needed.
- Observability: Preflight rows carry both desired and supported fields; YAML makes it reviewable.
- Test Coverage: Gap — add unit/integration tests for detection; add schema validation.
- Determinism & Redaction: Deterministic given environment; no redaction needed.
- Policy Knobs: `require_preservation`, `preservation_tier`.
- Exit Codes & Error Mapping: Violations map to `E_POLICY` (10) via apply gating.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: Behavior varies by fs and kernel; container permissions can influence capabilities.
- DX Ergonomics: Clear YAML representation.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `require_preservation` | `false` | STOP when required preservation not supported |
| `preservation_tier` | `Basic` | Advisory preference; influences capture/restore fidelity |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.row` | `preservation`, `preservation_supported` | Minimal Facts v1 |
| `preflight.summary` | `policy_ok`, `error_ids?` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/meta.rs` | detection tests (planned) | capability detection per fs |
| `src/preflight/yaml.rs` | YAML field presence tests (planned) | stable YAML representation |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Probe + YAML exposure | Fields populated when possible | Manual checks/tests | None | Additive |
| Silver | Schema validation + goldens | Validated fields in CI | Goldens + CI | CI gate | Additive |
| Gold | Expanded probe coverage; platform notes | Consistent behavior across common fs | Matrix tests | Docs + CI | Additive |
| Platinum | Formalized capability model | Strong guarantees across platforms | Property/model tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Policy knobs documented reflect current `policy::Policy`
- [x] Error mapping and `exit_code` coverage verified
- [x] Emitted facts fields listed and schema version up to date
- [ ] Goldens added/updated and CI gates green
## Gaps and Risks

- Probes are best-effort and environment-sensitive; lack of schema validation for these fields.

## Next Steps to Raise Maturity

- Add explicit tests for capability detection on supported platforms; add schema validation for preflight rows.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- `cargo/switchyard/src/fs/meta.rs`
- `cargo/switchyard/src/preflight/yaml.rs`
