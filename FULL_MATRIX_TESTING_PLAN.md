# Full Matrix Testing Plan (Strict, with a Single Allowed SKIP)

Goal

- Ensure every target distribution runs the complete test suite (YAML suites under `tests/` and the heavy Go assertions in `test-orch/container-runner/assertions/`) with zero silent skips.
- Target matrix (initial): arch, manjaro, cachyos, endeavouros. Extendable via a single flag.
- Any SKIP in matrix mode fails the run, except the single allowed SKIP: `tests/disable-in-german` may skip when the matrix runs distros in parallel due to test flakiness under parallel execution (affects `arch, manjaro, cachyos, endeavouros`). See `TESTING_POLICY.md`.

Current gaps (sources of false positives)

- YAML suites report SKIP as PASS
  - In `test-orch/container-runner/yamlrunner/yamlrunner.go`, `runSingleSuite()` returns `nil` for distro mismatch or detection error and the caller logs a PASS.
- Heavy assertions gated to Arch only
  - In `test-orch/container-runner/assertions/assertions.go`, `Run()` skips on non-Arch and returns `nil`.
- Coreutils leniency hides failures
  - In `ensureCoreutilsInstalled()`, missing applet symlinks are only warnings, so enable can “pass” with near-zero coverage.
- Parallel-run nondeterminism
  - `tests/disable-in-german/task.yaml` exhibits nondeterministic failures only when executed in parallel across distros; it passes reliably when run in isolation/serialized.

Planned changes (implementation checklist)

1) Container Runner (in-container)

- yamlrunner: explicit PASS/FAIL/SKIP (with one exception)
  - Introduce a `SuiteResult { Name, Status, Reason }` and aggregate results in `yamlrunner.Run()`.
  - In `runSingleSuite()`, return `Status = SKIP` when distro check fails or is indeterminate; return `FAIL` on execution error; return `PASS` only on success.
  - Add env toggle `FULL_MATRIX=1` (or `NO_SKIP=1`). When set, treat any SKIP as a hard error (convert to FAIL) and set a distinct exit code, except the single allowed SKIP (`disable-in-german` when executed in parallel across distros due to known flakiness).
  - Emit a JSON summary to `projectDir/artifacts/yaml-results.json` and a human summary.

- assertions: run everywhere and tighten checks
  - Distro gating: remove Arch-only gating or expand to `arch,manjaro,cachyos,endeavouros`. If `FULL_MATRIX=1`, any unsupported distro -> FAIL (infra/config issue) rather than silent skip.
  - Coreutils coverage: in `ensureCoreutilsInstalled()`:
    - Fail if zero applets are symlinked from `tests/lib/rust-coreutils-bins.txt`.
    - Add env `COREUTILS_MIN_SYMLINKS` (default e.g. 10) OR check a fixed critical subset: `date, ls, readlink, stat, rm, cp, ln, mv, cat, echo`.
    - Replace “not a symlink” warnings with failures when below threshold.
  - Sudo-rs checks: keep `sudo -> sudo-rs` target check; continue to accept alias `/usr/bin/sudo.sudo-rs`.
  - Disable path: keep strong assertions (no symlink, no backups, GNU returns), fail on any deviation.
  - Emit `projectDir/artifacts/assertions.json` with per-check results and totals.

- setup: pre-provision consistent environment
  - In `test-orch/container-runner/setup/setup.go`:
    - Mirrors: keep `pacman -Syy` and reflector (Arch). For derivatives, validate pacman is usable (Manjaro/CachyOS/EndeavourOS use pacman).
    - Locales: install `glibc-locales` if not present; ensure `de_DE.UTF-8 UTF-8` and `C.UTF-8` in `/etc/locale.gen`, then run `locale-gen`.
    - AUR helper: keep `paru-bin` approach. Verify builder user and permissions across derivatives.
    - Rust: rustup default stable for root and builder.

2) Host Orchestrator (outside container)

- Strict matrix flag and summary
  - Add `--require-full-matrix` flag. When set:
    - Propagate `FULL_MATRIX=1` to the container via `dockerutil.RunArchContainer()`.
    - Treat any container exit due to skip-as-error as FAIL for that distro.
    - Aggregate per-distro results (read artifacts if present) and print a colored summary table.
    - Exit nonzero if any distro had SKIP or FAIL.
  - Allow `--distros=arch,manjaro,cachyos,endeavouros` (already supported) and `--test-filter` pass-through (already supported).
  - Write `artifacts/host-summary.json` summarizing all distro outcomes.

3) YAML suites under `tests/`

- Coverage and gating
  - Replace `distro-check: [arch]` with `distro-check: [arch, manjaro, cachyos, endeavouros]` OR remove `distro-check` entirely and rely on product `--no-compatibility-check` path when needed.
  - Where distro-specific deltas exist, detect at runtime and assert equivalently, not by skipping.

- Parallel-sensitive suite: `tests/disable-in-german/task.yaml`
  - Not fundamentally blocked by locale provisioning; the dominant issue is nondeterminism under parallel, cross-distro runs.
  - Mitigation: run this suite serialized or in isolation when deflaking is not yet complete. In `FULL_MATRIX=1`, a SKIP is permitted only for this suite when running the full matrix in parallel.
  - Optionally keep locale assertions for clarity, but do not attribute SKIP to locale availability.

- Use `set -euo pipefail` (already present). Keep `|| true` only in restore/cleanup blocks.

4) Unit tests (Rust) hardening

- Add tests to exercise `which() -> None` paths
  - In `test-units/experiments/uutils/mock_worker.rs`, the current `which()` returns a synthetic `bin/<name>` under `cfg!(test)`. Add a second mock or parameter to return `None` so we cover negative paths in enable/disable code.
- Ensure all test file ops live under `tempfile::TempDir` (already aligned with memories), no writes to real `/usr/bin`.

5) CI integration

- Add a GitHub Actions job (and `act` compatibility) that:
  - Builds the host orchestrator and runs: `go run . --arch-build --run --distros=arch,manjaro,cachyos,endeavouros --require-full-matrix -v`
  - Uploads `artifacts/*` from each distro run (yaml-results.json, assertions.json, host-summary.json).
  - Fails the job if the orchestrator exits nonzero.

Acceptance criteria

- All four distros complete YAML suites and heavy assertions with no SKIPs, except that `disable-in-german` is allowed to SKIP when executed in parallel across distros due to known flakiness (affects `arch, manjaro, cachyos, endeavouros`), as documented in `TESTING_POLICY.md`.
- `host-orchestrator` exits nonzero if any suite or assertion is SKIP/FAIL in `--require-full-matrix` mode.
- Missing coreutils symlinks cause failures (not warnings) when below the threshold.
- German-locale suite reliably runs (locale pre-provisioned) or fails loudly in full-matrix mode.
- JSON artifacts are generated for postmortem debugging.

Operational notes

- Commands to run locally
  - Build+run with full matrix enforcement:
    - `cd test-orch/host-orchestrator`
    - `sudo go run . --arch-build --run --distros=arch,manjaro,cachyos,endeavouros --require-full-matrix -v`
  - Run a single suite across the matrix:
    - `sudo go run . --arch-build --run --distros=arch,manjaro,cachyos,endeavouros --require-full-matrix -v --test-filter=disable-in-german`

- Extending the matrix
  - Add distro to `--distros` and ensure Docker base image mapping exists in `test-orch/host-orchestrator/main.go`.
  - Verify pacman compatibility or introduce a package-manager abstraction if needed.

Risks & mitigations

- Derivative mirror instability: keep reflector or pinned mirrors; retry logic on pacman operations.
- Parallel execution variability: either serialize the `disable-in-german` suite or track and deflake; remove the SKIP once stable.
- Threshold tuning for coreutils: start with a small set of critical applets; evolve to percentage-based coverage once stable.

File references (where to implement)

- YAML runner: `test-orch/container-runner/yamlrunner/yamlrunner.go`
- Assertions: `test-orch/container-runner/assertions/assertions.go`
- Container setup: `test-orch/container-runner/setup/setup.go`
- Host orchestrator: `test-orch/host-orchestrator/main.go`, `test-orch/host-orchestrator/dockerutil/dockerutil.go`
- YAML suites: `tests/*/task.yaml`
- Unit test mocks: `test-units/experiments/uutils/mock_worker.rs`
