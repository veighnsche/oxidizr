# Golden fixtures and CI gates

- Category: Infra
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Curate deterministic golden artifacts for key scenarios and gate CI on a selected set, uploading diffs on failure.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Enforces determinism and prevents drift | Deterministic redaction (`TS_ZERO`) and UUIDv5 IDs; acceptance tests produce stable outputs |
| Developer feedback via diff artifacts | Planned CI upload of diffs on mismatch |
| Encourages coverage of critical scenarios | Curated scenario set documented in `DOCS/GOLDEN_FIXTURES.md` |

| Cons | Notes |
| --- | --- |
| Maintenance overhead for goldens | Requires periodic updates when intentional behavior changes |
| CI runtime cost | Comparing artifacts and uploading diffs adds CI time |

## Behaviors

- Generates deterministic artifacts in DryRun with redaction and stable IDs.
- Stores curated golden outputs for representative scenarios.
- Compares current outputs against goldens in CI; fails gate on mismatch (planned).
- Uploads diffs for developer triage (planned).

## Implementation

- Process: `cargo/switchyard/DOCS/GOLDEN_FIXTURES.md` documents golden generation using `GOLDEN_OUT_DIR` and redaction policy.
- Tests: acceptance tests (e.g., `tests/sprint_acceptance-0001.rs`) produce goldens in dry-run.

## Wiring Assessment

- Goldens supported by tests; CI blocking gate not yet enabled in this repo.
- Conclusion: partially wired; needs CI integration.

## Evidence and Proof

- Deterministic redaction (`TS_ZERO`) and UUIDv5 IDs ensure stable outputs.

## Feature Analytics

- Complexity: Low-Medium. Requires scripting in CI and curated artifacts.
- Risk & Blast Radius: Medium; improper redaction leads to flaky goldens.
- Performance Budget: CI time for artifact generation/compare.
- Observability: Diff uploads aid diagnosis; facts under Audit v2.
- Test Coverage: Acceptance/golden tests; gaps: CI gate implementation.
- Determinism & Redaction: Depends on `logging/redact.rs` and stage helpers.
- Policy Knobs: N/A.
- Exit Codes & Error Mapping: N/A (CI exit status governs).
- Concurrency/Locking: N/A.
- Cross-FS/Degraded: Goldens should cover EXDEV degraded paths.
- Platform Notes: Prefer containerized/generic env for golden generation.
- DX Ergonomics: Clear regeneration instructions.

Observability Map

| Artifact/Fact | Fields | Schema |
| --- | --- | --- |
| Golden artifacts | Scenario-specific fields (preflight/apply outputs) | Project-defined; redaction aligns with Audit v2 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `tests/sprint_acceptance-0001.rs` | acceptance scenario | Stable dry-run outputs align with goldens |
| SPEC features | determinism features | Deterministic behavior under scenarios |

## Gaps and Risks

- Missing zero-SKIP blocking gate; artifacts upload not automated.

## Next Steps to Raise Maturity

- Add CI job to verify curated golden set and upload diffs.

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Manual golden generation and compare | Deterministic outputs documented | Acceptance tests | Docs only | Additive |
| Silver | Scripted generation; local check | Reproducible local comparisons | Scripts + docs | Helper scripts | Additive |
| Gold | CI gate with diff uploads | Blocking gate prevents drift | CI job + curated goldens | CI diff artifacts | Additive |
| Platinum | Multi-platform golden matrix; auto-redaction audit | Stability across environments | Matrix CI + audits | Continuous reporting | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields redaction aligns with Minimal Facts v1
- [ ] CI job implemented and green
- [ ] Golden set curated and documented
- [ ] Diff artifact upload wired in CI
- [x] Maturity rating reassessed and justified if changed

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- PLAN/90-implementation-tiers.md (Determinism/Goldens); DOCS/EXIT_CODES_TIERS.md.
