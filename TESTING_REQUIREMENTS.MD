# Technical Test Requirements

This document defines the mandatory requirements, conventions, tooling, and quality gates for all automated tests in this repository. It applies to all contributors, CI pipelines, and release processes.

## 1. Purpose and Scope

- Ensure reliability, determinism, and maintainability of the codebase through comprehensive automated tests.
- Provide a single source of truth for how tests must be written, organized, executed, and evaluated.
- Cover all layers: unit, integration, end-to-end/CLI, property-based, fuzz, performance/benchmarks, and security-related tests.

## 2. Terminology

- Unit Test: Verifies a single function/module in isolation with mocks/stubs.
- Integration Test: Verifies multiple components interacting through public interfaces.
- End-to-End (E2E) / CLI Test: Exercises the built artifact (binary/CLI or service) as a user would.
- Property-Based Test: Generates many inputs to validate invariant properties hold (e.g., round-trip, idempotency).
- Fuzz Test: Feeds semi-random inputs to find crashes, panics, or UB.
- Snapshot Test: Captures and approves human-readable outputs that should remain stable.
- Benchmark: Measures performance, memory, and regressions against baselines.

## 3. Supported Platforms and Toolchain

- Operating Systems: Linux (x86_64), macOS (arm64 and x86_64), Windows (x86_64).
- Architectures: x86_64 and arm64 where feasible.
- Toolchain:
  - The minimum supported language/toolchain version (MSRV) equals the `rust-version` (or equivalent) declared in the project manifest.
  - Tests must pass on stable toolchains and the MSRV.
- Optional Tooling (enable if present in the repo):
  - Parallel test runner.
  - Property-based testing library.
  - Snapshot testing library.
  - Coverage tooling with LLVM-based or platform-native backends.
  - Benchmarking framework.
  - Fuzzing framework (nightly toolchain may be required).

## 4. Directory Layout and Naming

- Unit tests colocated with source files where supported (e.g., `src/**` with inline test modules).
- Integration tests placed under `tests/` as multiple files; each file compiles to a separate test binary.
- Benchmarks under `benches/`.
- Fuzz targets under `fuzz/` (if used).
- Test data and fixtures under `tests/fixtures/`:
  - Large assets (>100 KB) should be compressed or generated on the fly if possible.
  - Prefer minimal, representative samples over large datasets.

Naming conventions:
- Test files: `*_test.rs` or standard integration test naming (e.g., `tests/http.rs`).
- Test functions: `test_<behavior>_<condition>_<expected>()`.
- Property tests: `prop_<property>()`.
- Benchmarks: `bench_<subject>()`.
- Fuzz targets: `fuzz_<target>()`.

## 5. Test Writing Conventions

- Follow the Arrange–Act–Assert pattern.
- One assertion of behavior per test where practical; multiple related assertions permitted if they validate a single behavior.
- Prefer pure, side-effect-free helpers. Avoid hidden global state.
- Tests must be deterministic:
  - Seed random generators explicitly (configurable via env var: `TEST_SEED`).
  - Use fixed timestamps/clock abstractions and temporary directories.
  - Isolate filesystem/network state; don’t rely on user environment.
- Parallel-safety:
  - Tests must be safe to run in parallel.
  - Use per-test temporary directories (e.g., OS temp dir with unique prefix).
  - Avoid shared mutable global state; if unavoidable, guard with synchronization and document.

## 6. Mocking, Stubbing, and Test Doubles

- Mock only at module boundaries; prefer real integrations for integration tests.
- Avoid overspecifying mocks (assert behavior not call order unless necessary).
- When mocking FS/network/time:
  - Provide clear fixtures and teardown.
  - Ensure predictable paths and deterministic responses.

## 7. Test Data and Fixtures

- Place reusable fixtures in `tests/fixtures/` or a dedicated module.
- Document the meaning of each fixture and its expected format.
- Keep fixtures small; prune redundancy.
- For generated fixtures, include a generator script with deterministic seeds.

## 8. Property-Based Testing

- For critical parsers/encoders/transformations, add property tests:
  - Round-trip: `decode(encode(x)) == x` for valid inputs.
  - Idempotency: `f(f(x)) == f(x)` where applicable.
  - Monotonicity/order-preserving properties where applicable.
- Minimum generated cases per property: 256 in CI, 4096 in nightly/extended jobs.
- Shrinking must be enabled to minimize failing cases.

## 9. Fuzz Testing

- Add fuzz targets for public parsers, deserializers, and boundary handlers.
- Run fuzzing continuously on a CI schedule for at least 15 minutes per target.
- Crashes, panics, and OOM are failures and must be triaged within 24 hours.
- Store minimized corpus and regression seeds under `fuzz/corpus/` and `fuzz/artifacts/` with descriptions.

## 10. Snapshot Testing

- Snapshots must be human-readable (e.g., JSON/TOML/Text with stable formatting).
- Review snapshots into VCS explicitly; do not auto-accept in CI.
- Normalize nondeterministic fields (timestamps, paths, hashes) before snapshotting.
- Breaking snapshot changes require an entry in the changelog and reviewer approval.

## 11. Performance and Benchmarks

- Use a benchmarking framework with statistical noise reduction (warm-up, multiple samples).
- Define performance budgets:
  - Unit hot path: no regression >10% in p95 latency without explicit approval.
  - Memory: no regression >10% in peak usage on representative workloads.
- Bench results are compared to the last main-branch baseline.
- For flaky perf environments (e.g., CI), use relative comparisons with confidence intervals.

## 12. Security, Robustness, and Error Handling

- Validate robust handling of:
  - Zero-length inputs
  - Extremely large inputs (boundary limits)
  - Invalid encodings / malformed data
  - Permission errors (EACCES), missing files (ENOENT), disk full (ENOSPC)
  - Network timeouts/resets (if networking is present)
  - Concurrency hazards (races, deadlocks)
- Panics must not be user-triggerable in release builds; tests must verify error paths return structured errors.
- Sensitive data must never be present in logs or test artifacts.

## 13. Compatibility and Feature Flags

- Tests must cover:
  - Default configuration
  - Each feature flag enabled/disabled (matrix)
  - MSRV and latest stable toolchain
- Backwards/forwards-compatibility tests where serialization or on-disk formats are involved:
  - Golden files with version annotations
  - Migration tests for format changes

## 14. Code Coverage and Quality Gates

- Line coverage threshold: ≥85% overall, ≥90% for critical modules.
- Branch coverage threshold: ≥75% overall.
- Coverage excludes generated code and test-only helpers, but includes error handling branches where feasible.
- Lints:
  - Formatting check must pass.
  - Static analysis warnings are denied (treated as errors).
- No allowed flaky tests; any flakes must be quarantined and fixed before merging.

## 15. Test Execution

Local developer commands (examples; adapt to project tooling):
- Format and lint:
  - Format check: `cargo fmt -- --check`
  - Lints: `cargo clippy --all-targets --all-features -- -D warnings`
- Unit/integration:
  - `cargo test --all-features`
  - With parallel runner (if configured): `cargo nextest run --all-features`
- Coverage (preferred):
  - `cargo llvm-cov nextest --all-features --lcov --output-path lcov.info`
- Snapshots:
  - Run: `cargo test -p <pkg>`
  - Review new/changed snapshots: `cargo insta review`
- Benchmarks:
  - `cargo bench`
- Fuzzing (if enabled; nightly required):
  - `cargo fuzz run <target> -- -runs=0 -max_total_time=900`

CI requirements:
- Matrix:
  - OS: ubuntu-latest, macos-latest, windows-latest
  - Toolchain: MSRV (from manifest), stable
  - Features: default, all-features
- Required steps (must pass):
  - Format check
  - Lints with warnings denied
  - Unit/integration tests
  - Coverage thresholds enforced
  - Snapshot review not auto-accepted
  - Optional: property tests with elevated case counts
  - Optional: scheduled fuzz jobs

Artifacts:
- Upload coverage reports (HTML/LCOV).
- Upload failing seeds and minimized corpora for fuzz failures.
- Persist benchmark comparisons for trend analysis.

## 16. Logging and Observability in Tests

- Tests should configure logging to capture debug information on failure.
- Do not assert on log contents unless part of the contract; prefer structured assertions on returned values/events.

## 17. Time, Randomness, and External Dependencies

- Abstract time via a clock interface; avoid reading system time directly in code paths under test.
- Fix random seeds via `TEST_SEED` env var; tests must print/record the seed on failure.
- Network calls must be mocked or recorded; no live external calls in CI.
- Filesystem:
  - Use temp dirs per test.
  - Clean up resources on success; keep on failure with clear path outputs for debugging.

## 18. Error Messages and UX (for CLI/E2E)

- Validate exit codes:
  - `0` for success; non-zero for error categories.
- Validate stderr/stdout formatting and helpful error messages.
- Ensure help/usage output is accurate and includes examples.

## 19. Documentation and Test Discoverability

- Each non-trivial test must have a short doc comment describing:
  - Scenario being tested
  - Preconditions
  - Expected outcome
- Complex test helpers must be documented with examples.

## 20. Adding New Tests: Checklist

- Choose the appropriate layer (unit/integration/e2e/property/fuzz/bench).
- Create/locate minimal fixtures; avoid duplication.
- Ensure deterministic behavior (seed, time, tempdir).
- Follow naming conventions and AAA structure.
- Run locally with all features and on MSRV if possible.
- Update snapshots and review diffs if applicable.
- Verify coverage remains above thresholds.
- Add CI configuration if introducing a new test target (e.g., fuzz).

## 21. Non-Goals and Out of Scope

- Manual QA processes are not covered here.
- UI accessibility testing is out of scope unless the project provides a UI.
- Long-running stress/load tests belong to a separate performance test suite and infrastructure.

## 22. Governance and Exceptions

- Changes to these requirements require approval from maintainers.
- Temporary exceptions (e.g., coverage dips) must:
  - Be time-boxed with an issue reference,
  - Include a remediation plan,
  - Be tracked in CI as a known debt item.

---

By adhering to this specification, contributors ensure the test suite remains fast, reliable, and informative, enabling safe refactoring and confident releases.