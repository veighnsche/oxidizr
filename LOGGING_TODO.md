# LOGGING_TODO — tasks to comply with LOGGING_PLAN.md

This file lists concrete changes needed across components to fully align with `LOGGING_PLAN.md` and `VERBOSITY.md`.

---

## Current state snapshot (what’s already good)

- Product (Rust)
  - Uses `tracing` with `tracing-subscriber`; initialized at process start (`src/main.rs` ➜ `oxidizr_arch::logging::init_logging()`).
  - Has a dedicated audit JSONL sink targeting `target="audit"` (`src/logging/init.rs`, `src/logging/audit.rs`).
  - Emits rich progress and errors via `tracing::info!/warn!/error!/trace!` throughout experiments and symlink ops.
- Runner (Python)
  - Writes `runner.jsonl` with an envelope close to the plan via `JSONLLogger` (`test-orch/container-runner/lib/log.py`).
  - Captures per-suite stdout/stderr to `logs/<suite>/execute.*.log` and `restore.*.log` (`runner.py`).
  - Writes `summary.json` and copies `oxidizr-arch-audit.log` out of `/var/log` into `.proof/logs/` (`runner.py`).
- Host (Go)
  - Captures container stdout/stderr into host-side timestamped files, with level-filtered console streaming (`test-orch/host-orchestrator/dockerutil/run.go`).
  - Applies verbosity filtering (`-q`, `-v`, `-vv`) consistently across host output.

Gaps remain versus the plan (prefix format for human logs, canonical JSONL envelope fields everywhere, host.jsonl, artifact mirroring, guardrails).

---

## Global (all components)

- [ ] Adopt the canonical JSONL envelope fields consistently: `ts`, `component`, `level`, `run_id`, `container_id`, `distro`, `suite`, `stage`, `event`, `cmd`, `rc`, `duration_ms`, `target`, `source`, `backup_path`, `artifacts`, `message`.
  - Runner already writes most of these; add missing ones where noted below.
  - Product audit: currently emits `ts`, `component`, `level`, `run_id`, `container_id`, `distro`, `event`, `decision`, `inputs`, `outputs`, `exit_code`. Future enhancement: add optional structured fields (`target`, `source`, `backup_path`, `stage`, `suite`, `cmd`, `rc`, `duration_ms`, `artifacts`) directly as fields rather than embedding in `outputs`.
  - Host must add a `host.jsonl` writer (see Go section).
- [x] Propagate IDs and context via environment:
  - Host passes `RUN_ID` into `docker run` env; runner and product read it.
  - Runner computes `container_id` and provides it to its logger (already); product should derive `container_id` from `/etc/hostname`.
  - Distro: Host passes `ANALYTICS_DISTRO` (already) and/or Runner provides; product should read it for prefix/envelope.

---

## Product (Rust)

Files to touch: `src/logging/init.rs`, `src/logging/audit.rs`, call sites in `src/experiments/**`, `src/symlink/ops.rs`, `src/system/worker/{packages.rs,distro.rs}`.

- [x] Human log prefix format
  - Implement custom formatter to render `[<distro>][v<level>][] message` for product logs (scope blank for product/raw):
    - Read distro once from `/etc/os-release` inside `init_logging()` and cache it.
    - Map `Level` ➜ `v0..v3` for prefix; keep `VERBOSE` filtering as-is.
    - Replace the default `fmt::layer()` line/level/timestamp with a formatter that prints the prefix and message only, per `VERBOSITY.md`.
- [x] Align audit JSONL to the canonical envelope
  - Change `audit_event(...)` in `src/logging/audit.rs`:
    - Rename `timestamp` field to `ts` (RFC3339, already used).
    - Add `level` (use `info` for normal, `error` when appropriate).
    - Attach `run_id` (from `RUN_ID` env), `container_id` (from `/etc/hostname`), and `distro` (from `/etc/os-release` or `ANALYTICS_DISTRO`).
    - Accept optional structured fields: `stage`, `suite`, `cmd`, `rc`, `duration_ms`, `target`, `source`, `backup_path`, `artifacts`.
    - Keep `component` and `event` as top-level fields.
  - Update all call sites to pass structured fields instead of packing into the `inputs/outputs` strings (e.g., in `symlink/ops.rs` for `backup_created`, `link_started/done`, `restore_started/done`).
- [x] Event taxonomy and levels
  - Ensure the following are emitted at the specified levels:
    - `enabled`, `removed_and_restored` ➜ info (v1) plus run/suite end summary at v0 handled by runner/host.
    - `link_started`, `restore_started` ➜ debug (v2).
    - `link_done`, `restore_done`, `backup_created` ➜ debug (v2) with `duration_ms` where measurable.
    - `skip_applet` ➜ warn (v1) with `target`, reason.
    - `package_install`, `package_remove` ➜ info (v1); failures ➜ error (v0) with explicit exit codes.
- [x] Replace ad-hoc stdout prints with structured logs where appropriate
  - In `src/cli/handler.rs`, for `Check` and `ListTargets` commands, mirror the `println!` output with `tracing::info!(...)` so human logs remain consistent with the prefix policy (keep `println!` for tool-like output).
- [ ] Documentation
  - Add the event names and meanings to CLI help/README (short section enumerating product-side events).

---

## Container Runner (Python)

Files to touch: `test-orch/container-runner/runner.py`, `test-orch/container-runner/lib/{log.py,fs.py,suites.py}`.

- [x] Stage boundaries as events
  - Emit `stage_start` and `stage_end` events at `v1` for `preflight`, `deps`, `build`, `run_suites`, `restore` (per-suite where applicable), and `collect`.
  - Include timings as `duration_ms` in the `stage_end` event.
- [x] Assertions as events
  - On presence-checks and suite expectation evaluation, emit:
    - `assert_pass` (v1 info) with `suite` and relevant `artifacts`.
    - `assert_fail` (v0 error) with `suite`, `artifacts`, and short `message`.
  - Extend `lib/fs.py::assert_presence()` to send `event="assert_fail"` or `assert_pass` via the injected `logger`.
- [x] Raw product stdout/stderr capture
  - Introduce a minimal wrapper to run the product and tee raw output to:
    - `.proof/logs/product.stdout.log`
    - `.proof/logs/product.stderr.log`
  - Options to implement:
    1) Add a small shell helper (`oxidizr` function) that wraps `oxidizr-arch` calls; update YAMLs to use it.
    2) Provide a `proc.run_product(argv)` path and document its usage for YAMLs that call directly.
  - Until wrappers are in place, keep per-suite `execute.*.log`/`restore.*.log` (already captured). Mark the TODO to migrate YAMLs.
- [x] Guardrails (collect stage)
  - [x] Validate `runner.jsonl` lines: fail the run if any line is missing `ts`, `component`, or `run_id` (per plan).
  - [x] If the product was invoked during any suite but `.proof/logs/product.stdout.log`/`.stderr.log` are missing, mark the run INCONCLUSIVE and fail it.
- [x] Envelope completeness
  - Ensure `JSONLLogger.event()` also includes optional fields when provided (`artifacts`, `target`, `source`, etc.).
  - Add `stage="suite_start"/"suite_end"` events with `suite` and summary outcome.

---

## Host Orchestrator (Go)

Files to touch: `test-orch/host-orchestrator/{main.go, dockerutil/*.go}` (likely add a `hostlog` helper).

- [x] Pass `RUN_ID` to containers
  - In `main.go` where `envVars` is built, append `RUN_ID=<runID>` (runID is already computed and used in container names).
- [x] Host JSONL (host.jsonl)
  - Add a simple JSONL writer similar to the Python `JSONLLogger` with the same envelope.
  - Emit events at appropriate levels:
    - `container_start` (v1) when `docker run` is invoked.
    - `container_ready` (v1) after start succeeds and cid is available.
    - `stderr_tail` (v2/v3) lines appended at high verbosity without altering content.
    - `container_exit` (v1; v0 if non-zero) with rc and timings.
    - `artifact_mirror` (v1) with destination paths.
  - Write `host.jsonl` alongside mirrored artifacts under `.artifacts/<run_id>/<distro>_<container_id>/`.
- [x] Artifact mirroring
  - Mirror the container’s `/workspace/.proof/` directory to the host after the run but before container removal:
    - Target: `.artifacts/<run_id>/<distro>_<container_id>/`
    - Contents: `product.stdout.log`, `product.stderr.log`, `runner.jsonl`, `host.jsonl`, `summary.json`, plus `snapshots/` and `results/`.
  - Implementation options:
    1) Add a bind mount for `/workspace/.proof` to a run-scoped host path derived from `runID` and `distro` (preferred for simplicity and speed).
    2) Or perform `docker cp <cid>:/workspace/.proof <dest>` before container removal.
- [x] Verbosity behavior (live tail)
  - Continue to print container stdout filtered by `classifyLine()`; at `-vv` print container stderr lines live as-is (already in place) and also write them to `host.jsonl` as `stderr_tail` events.
- [x] Non-blocking lifecycle
  - Keep the current channel handling; ensure `host.jsonl` always receives a final `container_exit` event even on early failures.

---

## CI Guardrails

- [ ] Add a small verifier (Python or Go) invoked in CI to assert:
  - Every line in `runner.jsonl` and `host.jsonl` has `ts`, `component`, `run_id`.
  - For any suite that invoked the product, `product.stdout.log` and `product.stderr.log` exist.
  - Any `restore` failure results in a suite FAIL, not a WARN.
  - Presence-aware assertions only (no hardcoded counts); fail lint otherwise.

---

## Documentation

- [ ] Update `README.md` with a brief "Logging and Artifacts" section:
  - Show the artifact tree under `.artifacts/<run_id>/<distro>_<container_id>/`.
  - Document event names used by the product and their levels.

---

## Pointers to key code paths (for implementers)

- Rust initialization and audit:
  - `src/logging/init.rs` — subscriber setup, human JSON/txt formatting.
  - `src/logging/audit.rs` — envelope fields and audit emit.
- Rust emit sites:
  - `src/experiments/util.rs` — `create_symlinks()`, `restore_targets()` progress and audits.
  - `src/symlink/ops.rs` — `replace_file_with_symlink()`, `restore_file()` (backup, atomics, audit emits).
  - `src/system/worker/{packages.rs,distro.rs}` — pacman operations, repo probes, AUR helper calls.
  - `src/experiments/{coreutils.rs,findutils.rs,checksums.rs}` — install/enable/disable flows.
- Runner:
  - `test-orch/container-runner/runner.py` — stages, collection, JSONL write and proof packaging.
  - `test-orch/container-runner/lib/{log.py,fs.py,suites.py,proc.py}` — logging, assertions, exec wrapper.
- Host:
  - `test-orch/host-orchestrator/{main.go}` — flags, concurrency, run orchestration, summary aggregation.
  - `test-orch/host-orchestrator/dockerutil/{run.go,run_args.go,classify.go}` — docker run, log capture, prefixing & levels.
