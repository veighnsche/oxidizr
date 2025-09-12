# Determinism and redaction
- Category: Safety
- Maturity: Silver

## Summary
Deterministic IDs and ordering with stable timestamps in DryRun, plus redaction of volatile fields to enable golden tests and reproducible logs.

## Implementation
- IDs: `cargo/switchyard/src/types/ids.rs::{plan_id, action_id}` with UUIDv5 under namespace `constants::NS_TAG`.
- Redaction: `cargo/switchyard/src/logging/redact.rs::{TS_ZERO, ts_for_mode, redact_event}`.
- Sorting: preflight rows sorted by `(path, action_id)`.

## Wiring Assessment
- All stage facts go through `logging/audit.rs` helpers which apply redaction in dry-run.
- Apply selects timestamp via `ts_for_mode()`.
- Conclusion: wired correctly.

## Evidence and Proof
- Tests for `redact_event` and facts presence in `api.rs::tests`.

## Gaps and Risks
- No schema validation on facts; redaction policy could broaden.

## Next Steps to Raise Maturity
- Add JSON Schema for Minimal Facts v1 and validate in tests.
- Add property tests for UUIDv5 stability.

## Related
- SPEC v1.1 determinism requirements.
- `cargo/switchyard/src/logging/audit.rs`.
