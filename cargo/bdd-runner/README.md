# bdd-runner

A minimal Rust-based Behavior-Driven Development (BDD) runner for Switchyard SPEC features.

- Executes Gherkin `.feature` files from `cargo/switchyard/SPEC/features/`
- Uses `cucumber-rs` for step definitions and reporting
- Emits a machine-readable JSON report
- Provides a strict mode to fail on unimplemented steps

See `SPEC.md` for goals and constraints, and `DEVELOPMENT.md` for contributor instructions.

## Quickstart

From the repository root:

```bash
# Build only
cargo check -p bdd-runner

# Run all features (default paths)
cargo test -p bdd-runner --test bdd -- --nocapture
```

This will:

- Load features from `cargo/switchyard/SPEC/features/`
- Write a Cucumber JSON report to `cargo/bdd-runner/features_out/report.json`

## Configuration

Environment variables:

- `BDD_FEATURES_DIR`: override features directory
- `BDD_JSON_REPORT`: override JSON report path
- `BDD_FAIL_ON_STUB`: if `1` or `true`, generic stub steps will panic (useful to ensure steps are implemented)

Examples:

```bash
# Run against the default features dir but force failure on unimplemented steps
BDD_FAIL_ON_STUB=1 cargo test -p bdd-runner --test bdd -- --nocapture

# Run against a custom directory and custom report path
BDD_FEATURES_DIR=$(pwd)/cargo/switchyard/SPEC/features \
BDD_JSON_REPORT=$(pwd)/target/bdd/report.json \
cargo test -p bdd-runner --test bdd -- --nocapture
```

## Project Layout

- `tests/bdd.rs`: entry point (non-std test harness)
- `src/world.rs`: per-scenario state and temp workspace
- `src/steps_common.rs`: generic catch-all steps (stubs)
- `src/audit_schema.rs`: placeholder for JSON schema validation helpers
- `src/error_codes.rs`: placeholder for Switchyard error code mapping

## Implementing Real Steps

Add new step modules under `src/` and include them in `tests/bdd.rs` with a `mod` declaration, e.g.:

```rust
#[path = "../src/steps_switchyard.rs"]
mod steps_switchyard;
```

Then implement steps using `#[given]`, `#[when]`, and `#[then]` macros from `cucumber`.

For detailed guidance, see `DEVELOPMENT.md`.
