# Preflight YAML exporter

- Category: UX
- Maturity: Bronze

## Summary

Converts `PreflightReport.rows` to a SPEC-aligned YAML sequence for human and CI consumption.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Human-readable snapshot for operators/CI | `cargo/switchyard/src/preflight/yaml.rs::to_yaml()` |
| Stable ordering supports goldens | Upstream sort in `preflight/mod.rs` before export |

| Cons | Notes |
| --- | --- |
| No schema validation yet | Planned in Facts Schema Validation entry |
| Not a public CLI in this crate | Intended for integrator CLIs/tests |

## Behaviors

- Serializes a subset of preflight row fields to YAML in a stable order.
- Relies on upstream deterministic sorting to ensure golden-friendly output.
- Intended to be consumed by CLIs and CI jobs; not a public API within this crate.

## Implementation

- Exporter: `cargo/switchyard/src/preflight/yaml.rs::to_yaml()` preserves key subset and order.

## Wiring Assessment

- Intended for CLI or test artifacts; not yet integrated into a public CLI here.
- Conclusion: wired for internal use; consumer integration TBD.

## Evidence and Proof

- Deterministic sort guaranteed upstream in `preflight/mod.rs`.

## Feature Analytics

- Complexity: Low. Serialization with fixed field set and order.
- Risk & Blast Radius: Low; advisory output; mis-ordering would affect goldens.
- Performance Budget: Minimal.
- Observability: Exposes preflight fields in YAML for human/CI.
- Test Coverage: Gap — add YAML golden fixtures; field presence assertions.
- Determinism & Redaction: Deterministic ordering; values derived from preflight (which are deterministic for the same env).
- Policy Knobs: N/A (renders outputs of policy-based checks).
- Exit Codes & Error Mapping: N/A.
- Concurrency/Locking: N/A.
- Cross-FS/Degraded: N/A.
- Platform Notes: YAML content will reflect environment-specific paths.
- DX Ergonomics: Straightforward consumption by tooling.

Observability Map

| Artifact | Fields (subset) | Schema |
| --- | --- | --- |
| Preflight YAML | selected fields from rows | SPEC-aligned (no machine schema enforced) |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/preflight/yaml.rs` | YAML golden tests (planned) | stable ordering and subset |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | YAML exporter function | Stable order | Unit tests (planned) | None | Additive |
| Silver | Golden YAML fixtures | Deterministic YAML under scenarios | Goldens + CI | CI diff on change | Additive |
| Gold | Schema tie-in and doc | Mapped to facts schema where relevant | CI schema checks | CI gates | Additive |
| Platinum | Integrated CLI output | Operator-friendly CLI and runbooks | System/CLI tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [ ] Goldens added/updated and CI gates green

## Gaps and Risks

- No schema validation; not exercised by a CLI in this repo.

## Next Steps to Raise Maturity

- Add golden YAML fixtures and schema validation.

## Related

- SPEC preflight schema.
