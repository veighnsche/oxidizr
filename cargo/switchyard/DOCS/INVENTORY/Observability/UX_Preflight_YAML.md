# Preflight YAML exporter

- Category: UX
- Maturity: Bronze

## Summary

Converts `PreflightReport.rows` to a SPEC-aligned YAML sequence for human and CI consumption.

## Implementation

- Exporter: `cargo/switchyard/src/preflight/yaml.rs::to_yaml()` preserves key subset and order.

## Wiring Assessment

- Intended for CLI or test artifacts; not yet integrated into a public CLI here.
- Conclusion: wired for internal use; consumer integration TBD.

## Evidence and Proof

- Deterministic sort guaranteed upstream in `preflight/mod.rs`.

## Gaps and Risks

- No schema validation; not exercised by a CLI in this repo.

## Next Steps to Raise Maturity

- Add golden YAML fixtures and schema validation.

## Related

- SPEC preflight schema.
