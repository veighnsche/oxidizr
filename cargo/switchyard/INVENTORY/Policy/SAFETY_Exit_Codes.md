# Exit codes taxonomy and mapping

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

Stable `error_id` → `exit_code` mapping for core failure classes, aligned with SPEC. Emitted in facts where applicable.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Stable, SPEC-aligned mapping | `cargo/switchyard/src/api/errors.rs::{ErrorId, exit_code_for, exit_code_for_id_str}` |
| Facts carry both error_id and exit_code | Emission in preflight/apply failures |
| Aids routing/analytics and operator scripts | Consistent numeric codes across scenarios |

| Cons | Notes |
| --- | --- |
| Curated subset; gaps possible | Not all failure paths may attach `exit_code` yet |
| Best-effort summary error chain | `infer_summary_error_ids()` can miss specifics; always includes `E_POLICY` |

## Behaviors

- Maps `ErrorId` to a stable numeric `exit_code` for process-level failures.
- Attaches `error_id` and `exit_code` to facts on failures in preflight/apply.
- Provides a best-effort summary error chain for apply-stage failures.

## Implementation

- `cargo/switchyard/src/api/errors.rs` defines `ErrorId`, `id_str()`, `exit_code_for()`, and `infer_summary_error_ids()`.
- Apply and Preflight include `error_id`/`exit_code` in emitted facts on failure.

## Wiring Assessment

- `apply/handlers.rs` maps swap failures and restore failures to IDs and attaches `exit_code`.
- `preflight/mod.rs` summary includes `E_POLICY` mapping on STOP.
- Conclusion: wired correctly for covered sites.

## Evidence and Proof

- Unit tests in `api.rs::tests` assert exit code presence and consistency indirectly.

## Feature Analytics

- Complexity: Low. Enum + mapping functions.
- Risk & Blast Radius: Low; improves clarity and automation for failures.
- Performance Budget: Negligible.
- Observability: `error_id`/`exit_code` included in facts; summary co-ids added.
- Test Coverage: Some coverage; gaps for full-path mapping and negative scenarios.
- Determinism & Redaction: Deterministic mapping.
- Policy Knobs: N/A.
- Exit Codes & Error Mapping: Table below.
- Concurrency/Locking: N/A.
- Cross-FS/Degraded: Includes `E_EXDEV` mapping.
- Platform Notes: None.
- DX Ergonomics: Easy to consume by scripts/CI.

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where |
| --- | --- | --- |
| `E_POLICY` | 10 | `api/errors.rs::exit_code_for` |
| `E_OWNERSHIP` | 20 | `api/errors.rs::exit_code_for` |
| `E_LOCKING` | 30 | `api/errors.rs::exit_code_for` |
| `E_ATOMIC_SWAP` | 40 | `api/errors.rs::exit_code_for` |
| `E_EXDEV` | 50 | `api/errors.rs::exit_code_for` |
| `E_BACKUP_MISSING` | 60 | `api/errors.rs::exit_code_for` |
| `E_RESTORE_FAILED` | 70 | `api/errors.rs::exit_code_for` |
| `E_SMOKE` | 80 | `api/errors.rs::exit_code_for` |
| `E_GENERIC` | 1 | `api/errors.rs::exit_code_for` |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `error_id`, `exit_code`, `summary_error_ids` | Minimal Facts v1 |
| `preflight.summary` | `error_ids?` includes `E_POLICY` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/api.rs` | exit mapping tests (planned) | `error_id`/`exit_code` presence and values |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Define ErrorId + mapping | Stable list + codes | Basic tests | Docs | Additive |
| Silver (current) | Facts carry ids/codes | Emitted fields in failure paths | Integration tests | Inventory | Additive |
| Gold | Broader coverage; goldens | All failure paths mapped with tests | Goldens + CI | CI gates | Additive |
| Platinum | Compliance and reporting | Error taxonomy audited; dashboards | System/CI tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields listed and schema linkage referenced
- [ ] Tests added/expanded for full-path coverage
## Gaps and Risks

- Mapping is a curated subset; not all paths may attach `exit_code` yet.

## Next Steps to Raise Maturity

- Expand coverage and add goldens for failure scenarios asserting `error_id`/`exit_code`.

## Related

- SPEC v1.1 error codes; PLAN/30-errors-and-exit-codes.md.
