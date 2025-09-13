# JSONL file logging sink

- Category: Infra
- Maturity: Bronze

## Summary

Optional file-backed JSONL sink for facts and audit messages behind `file-logging` feature flag.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| Simple append-only file sink | `cargo/switchyard/src/logging/facts.rs::FileJsonlSink` |
| Integrates with existing emission path | Used via `Switchyard::new` sinks |
| Useful for local debugging and artifact capture | Works with Audit v2 events |

| Cons | Notes |
| --- | --- |
| No rotation/retention built-in | External log management required |
| No concurrency guarantees beyond append | May need fsync/locking for robustness |

## Behaviors

- Opens a file and appends JSON events line-by-line (append-only behavior).
- Can be injected into `Switchyard::new` when the feature is enabled.
- Does not rotate or prune logs; relies on external log management.
- Writes are best-effort; failures surface via standard I/O errors.

## Implementation

- `cargo/switchyard/src/logging/facts.rs::FileJsonlSink` (feature `file-logging`).

## Wiring Assessment

- Not enabled by default; integrators can opt-in and pass the sink to `Switchyard::new`.
- Conclusion: wired minimally.

## Evidence and Proof

- Code path writes to file with append-only behavior; no tests.

## Feature Analytics

- Complexity: Low. Thin wrapper around file append.
- Risk & Blast Radius: Low for dev; in prod, lack of rotation/retention/concurrency can be risky.
- Performance Budget: Write throughput bounded by file I/O.
- Observability: Captures Audit v2 lines for post-analysis.
- Test Coverage: Gap — add tests that write to temp file and validate lines.
- Determinism & Redaction: Same as emitted facts; sink does not alter content.
- Policy Knobs: N/A.
- Exit Codes & Error Mapping: N/A.
- Concurrency/Locking: No guarantees; consider file locking if used in multi-process.
- Cross-FS/Degraded: N/A.
- Platform Notes: POSIX file append semantics assumed.
- DX Ergonomics: Easy to use in examples and local runs.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| N/A | — | Feature is compile-time flag + constructor injection |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| N/A | — | Sink errors propagate as I/O errors; not exit taxonomy |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| All stage events | JSONL lines per Audit v2 | `SPEC/audit_event.v2.schema.json` (validation in related entry) |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/logging/facts.rs` | file sink tests (planned) | append-only write correctness |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze (current) | Append-only sink | Writes facts to file | Basic tests (planned) | Docs | Additive |
| Silver | Rotation hooks; flush/fdatasync | Resilience under load | Unit + integration | Rotation tooling | Additive |
| Gold | Retention policies; concurrency safety | Robust prod usage | System tests | Log management guidance | Additive |
| Platinum | Compliance integration | Auditable retention/rotation | Compliance tests | Continuous compliance | Additive |

## Maintenance Checklist

- [x] Code citations are accurate (paths and symbol names)
- [x] Emitted facts fields listed and schema linkage referenced
- [ ] Tests added for basic append behavior
- [ ] Rotation/retention plan documented or implemented

## Gaps and Risks

- Lacks rotation/retention; no concurrency guarantees beyond file append.

## Next Steps to Raise Maturity

- Add tests and doc examples; consider rotation hooks.

## Related

- Audit helpers in `logging/audit.rs`.
