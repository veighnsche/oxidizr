# oxidizr-cli-core — Release Readiness

This document captures why `oxidizr-cli-core` is ready for a tagged release and what quality bars it meets.

## Executive summary

`oxidizr-cli-core` is a small, well-scoped library that provides shared helpers for the oxidizr CLIs. It is:

- Lint-clean (Clippy) with warnings as errors for this crate
- Covered by unit tests for its non-trivial logic
- Documented (crate docs + docs.rs metadata) and ships a README with usage
- Manifest-complete with MSRV pinned
- Packaging-inspected via `cargo package`

The crate is suitable for a 0.1.0 release and consumption by the oxidizr CLI binaries.

## Scope & API surfaces

- `src/lib.rs` exports:
  - `api::build_api(policy, lock_path) -> Switchyard<JsonlSink, JsonlSink>`
  - `prompts::should_proceed(assume_yes, root)`
  - Re-exports from `coverage2` and `packages` for applet discovery/coverage
- `src/coverage2.rs` contains the tested logic for applet discovery/intersection/coverage preflight
- `src/adapter.rs` defines the `DistroAdapter` trait used by coverage logic

The crate does not perform privileged operations. It composes the Switchyard API and provides safe coverage helpers.

## Quality gates

- Clippy (crate-only):
  - Command: `cargo clippy -p oxidizr-cli-core --all-targets --no-deps -- -D warnings`
  - Status: PASS
- Tests:
  - Command: `cargo test -p oxidizr-cli-core`
  - Status: PASS (unit tests in `coverage2.rs` cover happy/edge cases)
- Docs build:
  - Command: `RUSTDOCFLAGS="--cfg docsrs" cargo doc -p oxidizr-cli-core --no-deps`
  - Status: PASS
- Packaging sanity:
  - Command: `cargo package -p oxidizr-cli-core --list --allow-dirty`
  - Status: PASS (expected files present)

## Manifest completeness

- `description`, `license`, `repository`, `keywords`, `categories`, `readme`
- `documentation = "https://docs.rs/oxidizr-cli-core"`
- `rust-version = "1.89"`
- Versioned dependency on `switchyard-fs` (no path deps in the published manifest)

## Versioning & compatibility

- Pre-1.0 SemVer: minor releases may contain breaking changes. Consumers should pin explicitly.
- MSRV: 1.89 (tested; pinned in `Cargo.toml`)
- No `unsafe` usage in this crate.

## Risks & mitigations

- Discovery heuristics depend on vendor `--list/--help` output formats → mitigated by allow-list fallback and intersection logic.
- Upstream Switchyard API evolution → mitigated by version pinning and re-export minimalism.

## Release process checklist

1) Ensure Switchyard and CLIs build against this commit
2) Run gates:
   - `cargo clippy -p oxidizr-cli-core --all-targets --no-deps -- -D warnings`
   - `cargo test -p oxidizr-cli-core`
   - `RUSTDOCFLAGS="--cfg docsrs" cargo doc -p oxidizr-cli-core --no-deps`
   - `cargo package -p oxidizr-cli-core --list`
3) Tag the release (e.g., `v0.1.0`) and push
4) Proceed to publish once dependency order allows (see PUBLISH_READY.md)

## Documentation status

- Crate docs: present
- README: includes a usage example
- docs.rs metadata: configured

This crate meets our internal release-readiness bar for 0.1.0.
