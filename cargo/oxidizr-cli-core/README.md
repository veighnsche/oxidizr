# oxidizr-cli-core

Common helpers for oxidizr-* CLIs.

This crate provides:

- API builder helpers for constructing `switchyard-fs` APIs in a consistent way
- Prompt and UX utilities shared by `oxidizr-arch` and `oxidizr-deb`

## Usage

```rust
use oxidizr_cli_core::api::build_api;
use switchyard::policy::Policy;

fn main() {
    // Choose or construct a Policy appropriate for your CLI.
    let policy = Policy::default();

    // Provide a lock file path for process-wide coordination.
    let lock_file = std::path::PathBuf::from("/tmp/oxidizr.lock");

    // Build a Switchyard instance with file-backed JSONL audit/facts sinks and defaults.
    let api = build_api(policy, lock_file);

    // Now you can plan/apply swaps, e.g. api.plan(...), api.apply(...).
}
```

Status: pre-1.0, unstable. See the main project repository for usage examples.

Links:

- Repository: https://github.com/veighnsche/oxidizr-arch
- License: Apache-2.0 OR MIT
