# Container Runner (Python + Bash)

A crystal-clear, policy-driven in-container test runner for oxidizr-arch.

Strictly aligned with VIBE_CHECK and TESTING_POLICY:

- Bash is purely procedural (no functions, no traps); Python orchestrates and asserts.
- The runner must not mutate product-owned artifacts except via the product CLI.
- No repo/mirror normalization at runtime; DNS/locale/mirror work is owned by the Dockerfile.
- Non-zero exit codes are never swallowed.

## Stage pipeline (in order)

1. preflight — Print OS info, pacman DB timestamps, rustup/cargo versions; verify the product can be built (no repairs).
2. deps — Install only strictly required missing packages; fail if repos unavailable.
3. build — cargo build with explicit profile; cache via mounted target if present; record build metadata.
4. run_suites — For each YAML suite (deterministic order):
   - Snapshot selected files/dirs before.
   - Execute the suite's script blocks; capture structured logs and RC.
   - Snapshot after and run presence-aware assertions.
   - Any restore failure = suite FAIL.
5. collect — Package logs, snapshots, and write a run summary.

## Layout

- `test-orch/container-runner/runner.py` — single entrypoint coordinating stages.
- `test-orch/container-runner/lib/` — tiny Python helpers:
  - `proc.py` — subprocess wrapper with timeout/env/cwd/rc/stdout/stderr capture.
  - `log.py` — JSONL logging to `/var/log/runner.jsonl`.
  - `fs.py` — snapshot and assertion helpers for selected paths (uutils, sudo-rs).
  - `suites.py` — discover suites, deterministic order, per-suite parsing, distro gating.
- `test-orch/container-runner/sh/` — procedural-only Bash wrappers (no functions/traps):
  - `preflight.sh`
  - `install_deps.sh`
  - `build_product.sh`
  - `run_suites.sh`
  - `collect_artifacts.sh`

## Output locations

- JSONL log: `/var/log/runner.jsonl`
- Proofs root: `/workspace/.proof/`
  - Logs: `/workspace/.proof/logs/`
  - Snapshots: `/workspace/.proof/snapshots/<suite>/`
  - Results: `/workspace/.proof/results/<suite>/`
- Summary: `/workspace/.proof/summary.json`

The summary includes an explicit affirmation:

```json
"harness_policy": "No harness mutation of product-owned artifacts; fail-on-skip enforced"
```

## Policies enforced (hard)

- Zero masking: no repo/mirror normalization, no alternate toolchains (e.g., BusyBox).
- No symlink pre-creation/deletion by harness.
- Fail-on-restore failure: any restore error fails the suite.
- Presence-aware assertions: assert only based on actually present applets; if missing, expect WARN from product logs.
- Bash minimalism: wrappers only; all logic in Python.

## Environment variables

- `TEST_FILTER` — run only the suite whose folder name matches this value (exact match).
- `SUITE_TIMEOUT_SEC` — timeout for a single suite execution block (default: 900).
- `CARGO_PROFILE` — cargo build profile (default: `release`).
- `RUSTUP_TOOLCHAIN` — rustup toolchain to use (default: `stable`).

## Usage

Inside the container with the repository mounted at `/workspace`:

- Run the full pipeline:

```bash
python3 /workspace/test-orch/container-runner/runner.py all
```

- Or run stage-by-stage via wrappers:

```bash
bash /workspace/test-orch/container-runner/sh/preflight.sh
bash /workspace/test-orch/container-runner/sh/install_deps.sh
bash /workspace/test-orch/container-runner/sh/build_product.sh
bash /workspace/test-orch/container-runner/sh/run_suites.sh
bash /workspace/test-orch/container-runner/sh/collect_artifacts.sh
```

- Run a single suite (by name, matching its directory under `tests/`):

```bash
TEST_FILTER=40-enable-partial python3 /workspace/test-orch/container-runner/runner.py run-suites
```

## Acceptance criteria mapping

- Executes exact stage order and produces `runner.jsonl` + `summary.json` + per-suite logs.
- No direct writes to `/usr/bin/*` outside product invocations; snapshots show only product-driven changes.
- Restore failures cause suite FAIL.
- Presence-aware checks behave on Arch + Manjaro.
- Dockerfile responsibilities are reused; runner never touches DNS/locale/mirror settings at runtime.
