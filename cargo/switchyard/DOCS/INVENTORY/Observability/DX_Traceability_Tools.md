# Traceability tools

- Category: DX
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Documentation tooling to map requirements → implementations → tests for better reviewability and compliance.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Improves reviewability and compliance | `SPEC/tools/traceability.py`; `SPEC/traceability.md` |
| Highlights missing links early | Report identifies gaps in req→code→test mapping |
| Low operational coupling | Docs-only; no runtime impact |

| Cons | Notes |
| --- | --- |
| Not enforced in CI | Marked gap; propose CI job |
| Requires disciplined annotations | Requires maintainers to keep links updated |

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

## Feature Analytics

- Complexity: Low. Script + markdown/docs.
- Risk & Blast Radius: Low; mislinks reduce usefulness but not runtime safety.
- Performance Budget: N/A; doc generation only.
- Observability: Report shows mapping and gaps.
- Test Coverage: Gap — add a simple CI job to run the tool and assert no missing links for a curated subset.
- Determinism & Redaction: N/A.
- Policy Knobs: N/A.
- Exit Codes & Error Mapping: N/A.
- Concurrency/Locking: N/A.
- Cross-FS/Degraded: N/A.
- Platform Notes: Runs in typical Python env used by docs.
- DX Ergonomics: Helps reviewers and new contributors.

Observability Map

| Artifact | Fields | Schema |
| --- | --- | --- |
| Traceability report | requirement id → impl/test links | Markdown/HTML (docs) |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `SPEC/tools/traceability.py` | CI run (planned) | report runs without missing curated links |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Script and doc | Report can be generated | Manual run | Docs | Additive |
| Silver | CI job runs report | No missing links for curated set | CI job | CI gate | Additive |
| Gold | Broader coverage and auto-linking | Extensive link coverage | Advanced scripts | CI dashboards | Additive |
| Platinum | Continuous compliance and notices | Enforced link discipline | Compliance tooling | Continuous checks | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [ ] CI job implemented and green
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
