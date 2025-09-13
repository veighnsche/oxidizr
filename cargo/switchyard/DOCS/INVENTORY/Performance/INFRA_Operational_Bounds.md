# Operational bounds

- Category: Infra
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Define and observe operational bounds (e.g., fsync warn thresholds, hashing/swap/backup duration tracking) to aid SLOs and tuning.

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
