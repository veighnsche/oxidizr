# Contributing Guide Enhancements
**Status:** Draft  
**Generated:** 2025-09-12  
**Scope:** Developer setup, linting/testing, common pitfalls, and helpful make targets/scripts.  
**Inputs reviewed:** CODE; repo layout  
**Affected modules:** docs

## Setup
- Install Rust stable and `rustfmt`/`clippy`.
- `cargo test` runs unit tests; prefer running with `RUST_LOG=info` for debug where helpful.

## Linting
- `cargo clippy --all-targets -- -D warnings` should pass.

## Testing
- Use `tempfile` and avoid absolute system paths.
- Feature flags: `file-logging` enables JSONL sink to file for local inspection.

## Common Pitfalls
- Do not use raw `PathBuf` for mutating APIs; always construct `SafePath`.
- Avoid path-based mutations; prefer `*at` helpers.

## Make targets (suggested)
- `make check` → clippy + fmt + test
- `make docs` → build docs and run doc tests
