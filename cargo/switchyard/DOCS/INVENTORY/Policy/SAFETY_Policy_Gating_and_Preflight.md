# Policy gating and preflight

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

Preflight computes per-action policy status and emits facts. Apply refuses Commit when policy gates fail unless `override_preflight=true`.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Fail-closed default for production safety | `cargo/switchyard/src/policy/config.rs::Policy { override_preflight: false }` default; gating in `api/apply/mod.rs` |
| Comprehensive checks (ownership, mounts, hazards, preservation) | `cargo/switchyard/src/api/preflight/mod.rs`; `preflight/checks.rs` |
| Deterministic rows and summary for CI | Sort in preflight; YAML exporter preserves order `preflight/yaml.rs::to_yaml()` |
| Clear STOP vs note semantics | `policy/gating.rs::gating_errors()` mirrors checks; STOPs in Commit |

| Cons | Notes |
| --- | --- |
| Some checks are heuristic/best-effort (immutability) | `preflight/checks.rs::check_immutable()` uses `lsattr` best-effort |
| Requires policy tuning per environment | Numerous flags; presets help but still need allow_roots precise scoping |

## Behaviors

- Builds per-action preflight rows (preservation, ownership, suid/sgid, hardlinks, mounts, immutability).
- Computes STOPs vs notes based on policy and observed state.
- Emits a preflight summary and per-action entries for observability.
- Blocks `apply` in Commit on STOPs unless `override_preflight=true`.

## Implementation

- Orchestrator: `cargo/switchyard/src/api/preflight/mod.rs` builds rows, detects preservation, ownership, suid/sgid, hardlink hazards, mount rw+exec, immutability.
- Policy evaluation helper: `cargo/switchyard/src/policy/gating.rs::gating_errors()` mirrors checks for Commit enforcement.
- Policy model: `cargo/switchyard/src/policy/config.rs` (allow_roots, forbid_paths, strict_ownership, require_preservation, allow_hardlink_breakage, allow_suid_sgid_mutation, rescue knobs, overrides).

## Wiring Assessment

- `Switchyard::preflight()` emits per-action facts + summary.
- `Switchyard::apply()` fails closed on gating errors (E_POLICY) unless overridden.
- OwnershipOracle path honored under `strict_ownership`.
- Conclusion: wired correctly; gating exercised in both stages.

## Evidence and Proof

- Preflight and Apply tests in `cargo/switchyard/src/api.rs::tests` assert facts and rollback semantics.
- Preflight YAML stable sort via `rows.sort_by` for determinism.

## Feature Analytics

- Complexity: Medium. Central orchestrator + multiple checks; touches `preflight`, `policy`, and `apply`.
- Risk & Blast Radius: High safety impact; incorrect overrides can broaden blast radius; defaults are fail-closed.
- Performance Budget: Light; primarily metadata checks and filesystem queries.
- Observability: Emits per-row and summary facts; YAML exporter provides human-readable output; schema alignment pending.
- Test Coverage: Integration tests in `api.rs::tests` (behavior asserted). Gaps: golden YAML fixtures and JSON Schema validation.
- Determinism & Redaction: Rows sorted; emitted facts go through redaction in DryRun.
- Policy Knobs: See matrix below; presets in `Policy::production_preset()` and `Policy::coreutils_switch_preset()`.
- Exit Codes & Error Mapping: STOPs aggregate under `E_POLICY` (10) with best-effort co-ids via `infer_summary_error_ids()`.
- Concurrency/Locking: Independent of locking; enforced before mutations.
- Cross-FS/Degraded: Coordinates with `allow_degraded_fs` indirectly via atomic swap policy.
- Platform Notes: Immutability detection may rely on `lsattr` availability.
- DX Ergonomics: Clear policy surface with presets; requires careful allow_roots scoping.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `override_preflight` | `false` | When true, allows Commit despite preflight STOPs |
| `strict_ownership` | `false` | Requires `OwnershipOracle`; STOP on mismatches |
| `require_preservation` | `false` | STOP when preservation not supported |
| `allow_hardlink_breakage` | `false` | Downgrades hardlink hazard from STOP to note |
| `allow_suid_sgid_mutation` | `false` | Downgrades suid/sgid hazard from STOP to note |
| `require_rescue` | `false` | STOP when rescue toolchain unavailable |
| `allow_roots` | `[]` | Limits scope of mutations |
| `forbid_paths` | `[]` | Blocks sensitive path prefixes |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.row` | `path`, `policy_ok`, `notes`, `stops` | Minimal Facts v1 (planned validation) |
| `preflight.summary` | `rows`, `policy_ok`, `error_ids?` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/api.rs` | preflight/apply gating tests (see module tests) | STOPs block Commit; override behavior |
| `src/preflight/yaml.rs` | to_yaml roundtrip tests (planned) | deterministic field order |

## Gaps and Risks

- Some checks are best-effort (immutability via `lsattr`), may be inconclusive.
- Missing explicit schema validation for preflight rows.

## Next Steps to Raise Maturity

- Add golden fixtures for positive/negative gates; CI gate.
- Schema validation for preflight rows.

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic gating with STOPs/notes; manual review | Deterministic rows; fail-closed by default | Integration tests | None | Additive |
| Silver (current) | YAML export; broader checks wired; presets available | Same as Bronze + consistent summary | Integration + inventory docs | Inventory/index | Additive |
| Gold | Schema validation + goldens; policy audit | Validated facts; CI gates; runbooks | Goldens + CI | CI gates & dashboards | Additive |
| Platinum | Formalized invariants; multi-env validation | Strong guarantees; continuous checks | Property/generative tests | Continuous compliance | Additive |
## Related

- SPEC v1.1 (policy defaults, preflight summary, fail-closed).
- `cargo/switchyard/src/preflight/checks.rs`.
