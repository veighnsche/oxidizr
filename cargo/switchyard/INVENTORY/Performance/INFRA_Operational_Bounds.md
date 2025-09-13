# Operational bounds

- Category: Infra
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Define and observe operational bounds (e.g., fsync warn thresholds, hashing/swap/backup duration tracking) to aid SLOs and tuning.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Perf visibility in apply summary | `cargo/switchyard/src/api/apply/mod.rs` emits `perf.*` in summary |
| Fsync duration tracked against threshold | `constants.rs::FSYNC_WARN_MS`; swap path measures and records timing |

| Cons | Notes |
| --- | --- |
| No hard budgets enforced | Observability only; policy thresholds not implemented |
| Missing tests for perf fields | Marked gap; propose goldens |

## Behaviors

- Tracks per-action perf times (hash, backup, swap) and aggregates for summary.
- Emits `perf.*` fields in `apply.result` summary for observability.
- Warn-level logging may be emitted when fsync exceeds `FSYNC_WARN_MS` (implementation-specific).

## Implementation

- Constants: `cargo/switchyard/src/constants.rs::FSYNC_WARN_MS` (warn threshold for fsync).
- Apply perf aggregation: `cargo/switchyard/src/api/apply/mod.rs` attaches `perf.hash_ms`, `perf.backup_ms`, `perf.swap_ms` in `apply.result` summary.

## Wiring Assessment

- Perf reported at apply summary; fsync duration measured in atomic swap path.
- Conclusion: partially wired; bounds recorded but not enforced as hard budgets.

## Evidence and Proof

- Apply flow includes perf aggregation; logs capture durations.

## Feature Analytics

- Complexity: Low. Compute and emit durations; one config constant.
- Risk & Blast Radius: Low; observability-only; risk is missing budgets.
- Performance Budget: N/A; this is the budget telemetry, not enforcement.
- Observability: `apply.result` summary includes `perf.*` fields.
- Test Coverage: Gap — add golden asserting presence/shape of `perf.*` fields.
- Determinism & Redaction: In DryRun, durations may be zeroed/omitted to preserve determinism.
- Policy Knobs: None yet; potential future policy thresholds.
- Exit Codes & Error Mapping: N/A.
- Concurrency/Locking: Independent.
- Cross-FS/Degraded: N/A.
- Platform Notes: Timing precision may vary by platform.
- DX Ergonomics: Easy to consume in logs and analytics.

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `perf.hash_ms`, `perf.backup_ms`, `perf.swap_ms` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/api/apply/mod.rs` | perf summary tests (planned) | fields exist and are numeric |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Emit perf fields; warn threshold constant | Presence of `perf.*` in summary | Unit/integration (planned) | None | Additive |
| Silver | Policy thresholds and warn behaviors | Enforce/alert on budgets | Tests + docs | Alerts | Additive |
| Gold | CI perf regressions gates (macro) | Detect regressions in CI | CI perf jobs | CI gates | Additive |
| Platinum | SLOs with continuous monitoring | SLO compliance reporting | Monitoring tooling | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields listed and schema linkage referenced
- [ ] Tests added for `perf.*` fields
- [ ] Policy thresholds implemented and documented (if adopted)
## Gaps and Risks

- No budget enforcement; only warn-level reporting.

## Next Steps to Raise Maturity

- Add budget enforcement or policy thresholds; golden with perf fields and warn behavior.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/55-operational-bounds.md; SPEC/features/operational_bounds.feature.
