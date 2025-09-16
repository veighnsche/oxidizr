# Contributing to Switchyard

Thanks for your interest in contributing!

- Code style: `cargo fmt` and `cargo clippy -- -D warnings` must pass.
- MSRV: 1.75 (see `Cargo.toml`). Try to avoid new language features beyond MSRV.
- Tests: `cargo test` should pass locally. For BDD, run `cargo test --features bdd --test bdd`.
- Docs: update the mdBook under `book/` when you change behavior.
- Changelog: add notable user-visible changes to `CHANGELOG.md`.

## Running checks

```
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps
```

## Pull requests

- Include a clear description, motivation, and any risk/mitigation notes.
- Link to SPEC sections or tests if behavior is normative.
- Small, focused PRs are easier to review.
