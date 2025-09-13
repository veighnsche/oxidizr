# Golden fixtures and CI gates

- Category: Infra
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Curate deterministic golden artifacts for key scenarios and gate CI on a selected set, uploading diffs on failure.

## Implementation

- Process: `cargo/switchyard/DOCS/GOLDEN_FIXTURES.md` documents golden generation using `GOLDEN_OUT_DIR` and redaction policy.
- Tests: acceptance tests (e.g., `tests/sprint_acceptance-0001.rs`) produce goldens in dry-run.

## Wiring Assessment

- Goldens supported by tests; CI blocking gate not yet enabled in this repo.
- Conclusion: partially wired; needs CI integration.

## Evidence and Proof

- Deterministic redaction (`TS_ZERO`) and UUIDv5 IDs ensure stable outputs.

## Gaps and Risks

- Missing zero-SKIP blocking gate; artifacts upload not automated.

## Next Steps to Raise Maturity

- Add CI job to verify curated golden set and upload diffs.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/90-implementation-tiers.md (Determinism/Goldens); DOCS/EXIT_CODES_TIERS.md.
