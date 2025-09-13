# Ownership and provenance

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Enforce strict ownership policy and record provenance (uid/gid/pkg where available) for targets. Used to reduce risk of hijacking and ensure controlled sources.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Surfaces uid/gid provenance for auditability | `cargo/switchyard/src/adapters/ownership/fs.rs::{FsOwnershipOracle, OwnershipOracle}`; attached in `api/preflight/mod.rs` |
| Enforceable via `strict_ownership` | `policy/config.rs::strict_ownership`; STOPs mapped to `E_POLICY` |
| Optional: can run best-effort without oracle | When oracle missing, enrichment is advisory; no false STOPs |

| Cons | Notes |
| --- | --- |
| No package (pkg) provenance by default | Requires environment-specific oracle implementation |
| Non-Unix platforms not supported | Current oracle relies on Unix metadata |

## Behaviors

- Queries filesystem metadata for uid/gid and (optionally) package provenance via an injected oracle.
- Attaches provenance fields to preflight rows for operator visibility.
- Enforces `strict_ownership` by adding STOPs when provenance does not meet policy.
- Leaves behavior advisory when no oracle is configured (best-effort enrichment only).

## Implementation

- Adapter: `cargo/switchyard/src/adapters/ownership/fs.rs::{FsOwnershipOracle, OwnershipOracle}` provides uid/gid via filesystem metadata.
- Policy: `cargo/switchyard/src/policy/config.rs` — `strict_ownership` toggles enforcement; provenance fields included in facts when available.
- Preflight: `cargo/switchyard/src/api/preflight/mod.rs` consults oracle to emit provenance (uid/gid) and compute `policy_ok` under strict ownership.

## Wiring Assessment

- `Switchyard` can be constructed with an `OwnershipOracle`. Preflight checks policy and attaches provenance.
- Apply respects preflight STOP decisions (E_POLICY) unless overridden.
- Conclusion: wired correctly; provenance is surfaced where supported.

## Evidence and Proof

- Unit: `adapters/ownership/fs.rs` basic behavior.
- Integration: preflight rows include ownership provenance fields; apply enforces E_POLICY for violations.

## Feature Analytics

- Complexity: Low. Adapter trait + fs-backed oracle; preflight integration.
- Risk & Blast Radius: Moderately scoped; affects policy gating only; avoids false STOPs without oracle.
- Performance Budget: Negligible; metadata lookups per target.
- Observability: Emits provenance fields in preflight rows; co-emits `E_OWNERSHIP` in summary when applicable.
- Test Coverage: Adapter unit tests; integration via preflight facts. Gaps: explicit enforcement tests for strict_ownership.
- Determinism & Redaction: Deterministic fields; sensitive IDs are not redacted; envelope timestamps follow stage policy.
- Policy Knobs: `strict_ownership`, `force_untrusted_source`.
- Exit Codes & Error Mapping: Violations contribute to `E_POLICY` (10); ownership classification co-emitted via `E_OWNERSHIP` in best-effort summary.
- Concurrency/Locking: None.
- Cross-FS/Degraded: N/A.
- Platform Notes: Unix-only metadata.
- DX Ergonomics: Adapter injection via `with_ownership_oracle`.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `strict_ownership` | `false` | Enforce ownership constraints during preflight/apply; STOP on mismatch |
| `force_untrusted_source` | `false` | Downgrades untrusted source checks to warnings |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_POLICY` | `10` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |
| `E_OWNERSHIP` (co-id) | `20` | Classification only; included in summary chain |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `preflight.row` | `uid`, `gid`, `policy_ok` | Minimal Facts v1 |
| `preflight.summary` | `error_ids?` includes `E_OWNERSHIP` when found | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/adapters/ownership/fs.rs` | unit tests (if present) | uid/gid extraction |
| `src/api.rs` | strict_ownership gating tests (planned) | STOP mapping and summary chain |

## Gaps and Risks

- Package ownership (`pkg`) not populated by default oracle; requires environment-specific oracle.
- Non-Unix platforms not supported by default oracle.

## Next Steps to Raise Maturity

- Provide package DB oracle example; expand tests for ownership edge cases (symlink, broken links).

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic uid/gid enrichment | Advisory provenance visible | Unit tests | None | Additive |
| Silver (current) | Strict enforcement via policy; summary co-ids | STOPs on mismatch; co-emit `E_OWNERSHIP` | Integration tests | Inventory entry | Additive |
| Gold | Package provenance; schema-validated facts | Extended provenance with validation | Goldens + schema checks | CI gates | Additive |
| Platinum | Multi-platform or hardened provenance oracles | Strong guarantees across platforms | Property tests; platform matrix | Continuous compliance | Additive |

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/15-policy-and-adapters.md; ADR-0002 error strategy; ADR-0008 safepath-toctou.
