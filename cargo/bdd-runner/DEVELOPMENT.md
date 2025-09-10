# Development Guide: bdd-runner

This guide covers how to run, extend, and debug the Rust BDD runner that executes the Switchyard SPEC features.

## Prerequisites

- Rust toolchain (stable) with `cargo`
- Network access for initial dependency fetch (first build only)

## Running the Features

From the repo root, run the non-std cucumber test harness:

```bash
cargo test -p bdd-runner --test bdd -- --nocapture
```

What happens:

- Discovers features in `cargo/switchyard/SPEC/features/`
- Executes scenarios with an isolated `World` per scenario
- Writes a JSON report to `cargo/bdd-runner/features_out/report.json`

### Useful Environment Variables

- `BDD_FEATURES_DIR`: override the features directory
- `BDD_JSON_REPORT`: set a custom JSON report path
- `BDD_FAIL_ON_STUB`: if `1` or `true`, any step matched by the generic stub will panic

```bash
# Fail-fast on unimplemented steps
BDD_FAIL_ON_STUB=1 cargo test -p bdd-runner --test bdd -- --nocapture

# Custom features dir and report path
BDD_FEATURES_DIR=$(pwd)/cargo/switchyard/SPEC/features \
BDD_JSON_REPORT=$(pwd)/target/bdd/report.json \
cargo test -p bdd-runner --test bdd -- --nocapture
```

### Filtering by Name/Tags

The runner enables the cucumber default CLI parser. Common examples:

```bash
# Run only scenarios whose name matches regex
cargo test -p bdd-runner --test bdd -- --name "atomic|determinism"

# Run scenarios with a given tag (see feature files for tags)
cargo test -p bdd-runner --test bdd -- --tags "@smoke"
```

For the full list of supported CLI flags, run with `--help`:

```bash
cargo test -p bdd-runner --test bdd -- --help
```

## Project Structure

- `tests/bdd.rs` — entry point using `cucumber::World::cucumber()`
- `src/world.rs` — `TestWorld` with a per-scenario temp workspace
- `src/steps_common.rs` — catch-all step stubs (enable strictness with `BDD_FAIL_ON_STUB`)
- `src/audit_schema.rs` — placeholder helper for JSON Schema validation of audit events
- `src/error_codes.rs` — placeholder for Switchyard error code mapping

## Adding Real Step Implementations

1. Create a new module under `src/` (e.g., `steps_switchyard.rs`) and implement steps using `#[given]`, `#[when]`, and `#[then]` from `cucumber`.
2. Include the module in `tests/bdd.rs`:

```rust
#[path = "../src/steps_switchyard.rs"]
mod steps_switchyard;
```

3. Use the per-scenario workspace via `w.work` for any file-system interactions to avoid touching system paths. Temporary directories are auto-cleaned after each scenario.

### Matching the Step Vocabulary Contract

The canonical step regexes live in `cargo/switchyard/SPEC/features/steps-contract.yaml`. Aim to:

- Keep implemented step patterns aligned with these regexes.
- Prefer parameterized captures over bespoke patterns.
- Add missing patterns to the contract file in the Switchyard SPEC when new vocabulary is required.

A future task will add a contract linter to validate that all steps in the feature corpus match at least one contract regex.

## JSON Report Consumption

The test produces a Cucumber JSON report. You can:

- Upload it as a CI artifact
- Convert to JUnit with a converter if needed
- Build traceability reports by joining feature tags with requirement IDs

## Troubleshooting

- Build fails fetching crates: ensure network connectivity on first build. Subsequent builds are cached.
- Tests hang: run with `RUST_LOG=debug` and `--nocapture` to get stdout from steps.
- Unimplemented step accidentally matched the catch-all stubs: set `BDD_FAIL_ON_STUB=1` to force immediate failures.
