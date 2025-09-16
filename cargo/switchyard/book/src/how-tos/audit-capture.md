# Capture and Verify Audit

- Provide a `FactsEmitter` implementation (e.g., `FileJsonlSink` via `--features file-logging`).
- For canon comparisons, run in DryRun then Commit and compare redacted events.

Citations:
- `src/logging/facts.rs`
- `src/logging/redact.rs`
- `src/logging/audit.rs`
