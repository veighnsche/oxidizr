# JSONL file logging sink

- Category: Infra
- Maturity: Bronze

## Summary

Optional file-backed JSONL sink for facts and audit messages behind `file-logging` feature flag.

## Implementation

- `cargo/switchyard/src/logging/facts.rs::FileJsonlSink` (feature `file-logging`).

## Wiring Assessment

- Not enabled by default; integrators can opt-in and pass the sink to `Switchyard::new`.
- Conclusion: wired minimally.

## Evidence and Proof

- Code path writes to file with append-only behavior; no tests.

## Gaps and Risks

- Lacks rotation/retention; no concurrency guarantees beyond file append.

## Next Steps to Raise Maturity

- Add tests and doc examples; consider rotation hooks.

## Related

- Audit helpers in `logging/audit.rs`.
