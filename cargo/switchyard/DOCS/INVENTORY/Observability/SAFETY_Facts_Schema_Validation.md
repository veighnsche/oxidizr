# Facts schema validation

- Category: Safety
- Maturity: Bronze
- Owner(s): <owner>
- Last reviewed: <YYYY-MM-DD>
- Next review due: <YYYY-MM-DD>
- Related PR(s): <#NNNN>

## Summary

Validate emitted facts against a JSON Schema to guarantee shape consistency and enable stable goldens.

## Implementation

- Schema: `cargo/switchyard/SPEC/audit_event.schema.json` defines the Minimal Facts v1 envelope and fields.
- Current state: schema exists; no automated validation wired into tests yet.

## Wiring Assessment

- Emission via `logging/audit.rs` is centralized, but schema validation step is missing.
- Conclusion: planned; partial artifacts exist.

## Evidence and Proof

- Presence of schema file; structured helpers used for emission.

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
