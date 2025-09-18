# oxidizr-cli-core — Publish Readiness

This document explains why `oxidizr-cli-core` is ready to publish on crates.io and describes the process we will follow.

## Why it is publish-ready

- Manifest completeness (crates.io metadata)
  - `name`, `version`, `edition`, `description`, `license`, `repository`, `keywords`, `categories` are set in `cargo/oxidizr-cli-core/Cargo.toml`.
  - `readme = "README.md"` so crates.io renders the crate README.
  - `documentation = "https://docs.rs/oxidizr-cli-core"` so users land on API docs.
  - `rust-version = "1.89"` (MSRV) is pinned for user expectations and CI.
  - Dependency on Switchyard uses a versioned dependency: `switchyard = { package = "switchyard-fs", version = "0.1.0" }`.
  - No path-only or git-only dependencies in the published manifest.

- Local development remains intact
  - The workspace root `Cargo.toml` configures `[patch.crates-io]` to point `switchyard-fs` and `oxidizr-cli-core` to local paths for developers, while published manifests use versions.

- Linting, tests, docs
  - `cargo clippy -p oxidizr-cli-core --all-targets --no-deps -- -D warnings` passes clean.
  - `cargo test -p oxidizr-cli-core` passes (unit tests cover coverage helpers).
  - `RUSTDOCFLAGS="--cfg docsrs" cargo doc -p oxidizr-cli-core --no-deps` builds API docs.

- Packaging check
  - `cargo package -p oxidizr-cli-core --list` shows expected files will be packaged.
  - The Cargo-generated `Cargo.toml.orig` visible during `cargo package --list` is a normal staging artifact and not a blocker.

## Publish order (dependency chain)

1) Publish `switchyard-fs` first (required by this crate).
2) Publish `oxidizr-cli-core`.
3) Then publish the CLI binaries that depend on this crate (`oxidizr-arch`, `oxidizr-deb`).

## Pre-publish checklist

- [x] Replace path deps with versioned deps in this crate.
- [x] Ensure MSRV and docs.rs metadata in `Cargo.toml`.
- [x] README contains a basic usage example.
- [x] Clippy/tests/docs build pass for this crate.
- [x] `cargo package --list` reviewed.
- [ ] Ensure `switchyard-fs` version referenced (0.1.0) is already published.
- [ ] Optionally run `cargo publish --dry-run -p oxidizr-cli-core` to validate packaging.

## How to publish

- Dry run (no network changes, safe to try):
  ```sh
  cargo publish -p oxidizr-cli-core --dry-run
  ```
- Actual publish (after `switchyard-fs` is live on crates.io):
  ```sh
  cargo publish -p oxidizr-cli-core
  ```

## Post-publish actions

- Verify the crate page on crates.io and docs.rs render correctly.
- Tag the repo (e.g., `oxidizr-cli-core-v0.1.0`) and update changelog if maintained.
- Update downstream crates (`oxidizr-arch`, `oxidizr-deb`) to ensure they pin the released version.

## Notes

- License is set via SPDX expression (`Apache-2.0 OR MIT`); shipping license files in the crate is optional but recommended for some users. We can add `LICENSE*` into the crate directory in a future revision if desired.
- As a pre-1.0 crate, breaking changes may occur in minor versions; consumers should pin explicitly.
