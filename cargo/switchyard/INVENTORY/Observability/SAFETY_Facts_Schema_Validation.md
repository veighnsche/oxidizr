# Facts schema validation

- Category: Safety
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Validate emitted facts against a JSON Schema to guarantee shape consistency and enable stable goldens.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Prevents schema drift and breaking analytics | `SPEC/audit_event.schema.json` exists; planned CI validation |
| Enables stable goldens | Deterministic envelope maps cleanly to schema |

| Cons | Notes |
| --- | --- |
| Not yet enforced in CI | Marked as gap; add test/CI helper |
| Schema evolution needs versioning discipline | Anchor on `schema_version` and migration docs |

## Behaviors

- Loads `SPEC/audit_event.schema.json` and validates emitted JSONL facts against it (planned).
- Fails tests/CI on schema mismatches to prevent drift (planned).
- Anchors facts to a schema version field for forward migration.

## Implementation

- Schema: `cargo/switchyard/SPEC/audit_event.schema.json` defines the Minimal Facts v1 envelope and fields.
- Current state: schema exists; no automated validation wired into tests yet.

## Wiring Assessment

- Emission via `logging/audit.rs` is centralized, but schema validation step is missing.
- Conclusion: planned; partial artifacts exist.

## Evidence and Proof

- Presence of schema file; structured helpers used for emission.

## Feature Analytics

- Complexity: Low. Add validation helper and wire into tests/CI.
- Risk & Blast Radius: High benefit; prevents drift across all facts.
- Performance Budget: Minimal; JSON Schema validation in tests only.
- Observability: Validates Minimal Facts v1 envelope/fields.
- Test Coverage: Gap — add validation tests; add goldens tied to schema.
- Determinism & Redaction: Orthogonal; ensures shape, not value variability.
- Policy Knobs: N/A.
- Exit Codes & Error Mapping: N/A.
- Concurrency/Locking: N/A.
- Cross-FS/Degraded: N/A.
- Platform Notes: None.
- DX Ergonomics: Developer helper simplifies validation.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| N/A | — | Schema validation is a test/CI concern |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| N/A | — | Not applicable |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| All stage events | Envelope fields per Minimal Facts v1 | `SPEC/audit_event.schema.json` |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `tests/schema_validation.rs` (planned) | validates JSONL facts | conformance to schema |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Schema present | Reference only | None | Docs | Additive |
| Silver | Test-time validation | Facts conform in tests | Unit/integration | Test helper | Additive |
| Gold | CI gate on schema; golden facts | Prevent drift at PR time | CI + goldens | CI gates | Additive |
| Platinum | Versioned schema + migrations | Safe evolution; dual-emit if needed | Migration tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields listed and schema version up to date (via related entries)
- [ ] Validation helper implemented and tested
- [ ] CI gate added and green

## Gaps and Risks

- Schema drift risk without validation; missing test-time enforcement.

## Next Steps to Raise Maturity

- Add test helper to validate JSONL facts against the schema; add CI check.

## Observations log

- <YYYY-MM-DD> — <author> — <note>

## Change history

- <YYYY-MM-DD> — <author> — Initial entry.

## Related

- SPEC/audit_event.schema.json; PLAN/40-facts-logging.md.
