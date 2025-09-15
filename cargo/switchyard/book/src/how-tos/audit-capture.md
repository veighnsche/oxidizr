# Capture and Verify Audit

- Provide a `FactsEmitter` implementation (e.g., `FileJsonlSink` via `--features file-logging`).
- For canon comparisons, run in DryRun then Commit and compare redacted events.

Citations:
- `cargo/switchyard/src/logging/facts.rs`
- `cargo/switchyard/src/logging/redact.rs`
- `cargo/switchyard/src/logging/audit.rs`
