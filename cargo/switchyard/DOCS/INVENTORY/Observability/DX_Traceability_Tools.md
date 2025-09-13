# Traceability tools

- Category: DX
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Documentation tooling to map requirements → implementations → tests for better reviewability and compliance.

## Behaviors

- Parses SPEC requirements and links them to code references and tests.
- Produces a human-readable traceability report.
- Highlights gaps where requirements lack code or tests.
- Intended for maintainers; not enforced at runtime.

## Implementation

- Tooling: `cargo/switchyard/SPEC/tools/traceability.py` and `SPEC/traceability.md`.

## Wiring Assessment

- Not part of runtime; used in documentation workflows and reviews.
- Conclusion: available for maintainers; not enforced.

## Evidence and Proof

- Scripts and docs present under `SPEC/`.

## Gaps and Risks

- No CI job enforcing traceability completion.

## Next Steps to Raise Maturity

- Add a CI job to generate a traceability report and fail on missing links for curated requirements.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- SPEC/requirements.yaml; PLAN/meta/20-spec-traceability.md.
