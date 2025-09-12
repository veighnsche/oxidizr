# Exit codes taxonomy and mapping

- Category: Safety
- Maturity: Silver

## Summary

Stable `error_id` → `exit_code` mapping for core failure classes, aligned with SPEC. Emitted in facts where applicable.

## Implementation

- `cargo/switchyard/src/api/errors.rs` defines `ErrorId`, `id_str()`, `exit_code_for()`, and `infer_summary_error_ids()`.
- Apply and Preflight include `error_id`/`exit_code` in emitted facts on failure.

## Wiring Assessment

- `apply/handlers.rs` maps swap failures and restore failures to IDs and attaches `exit_code`.
- `preflight/mod.rs` summary includes `E_POLICY` mapping on STOP.
- Conclusion: wired correctly for covered sites.

## Evidence and Proof

- Unit tests in `api.rs::tests` assert exit code presence and consistency indirectly.

## Gaps and Risks

- Mapping is a curated subset; not all paths may attach `exit_code` yet.

## Next Steps to Raise Maturity

- Expand coverage and add goldens for failure scenarios asserting `error_id`/`exit_code`.

## Related

- SPEC v1.1 error codes; PLAN/30-errors-and-exit-codes.md.
