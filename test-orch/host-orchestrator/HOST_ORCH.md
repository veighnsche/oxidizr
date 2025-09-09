# Host Orchestrator (Go) — Design and Operator Guide

This component is a pure host-side orchestrator for the oxidizr-arch test matrix.
It builds or pulls images, launches containers in parallel, proxies logs, and
mirrors artifacts to the host. It does not mutate container filesystems or
reinterpret product/runner logs.

## Summary of responsibilities

- Spin up per-distro containers with deterministic names
- Wire bind mounts and declared env only
- Run containers with per-run deadlines and bounded retry/backoff
- Proxy container stdout/stderr to the host terminal without changing severities
- Persist container logs under stable host paths
- Emit per-container JSON summaries with stable keys
- Aggregate a run-level summary for convenience
- Always attempt best-effort cleanup

## CLI flags

- `--distros` string: comma-separated matrix (default: `arch,manjaro,cachyos,endeavouros`)
- `--docker-context` string: build context directory (default: `test-orch`)
- `--root-dir` string: host directory mounted at `/workspace` (defaults to repo root)
- `--arch-build` bool: build the per-distro image(s)
- `--run` bool: run the non-interactive test container(s)
- `--shell` bool: single interactive container (no parallelism)
- `--no-cache` bool: build without cache
- `--pull` bool: attempt to pull a newer base image at build time
- `--keep-container` bool: keep container(s) after run (omit `--rm`)
- `--timeout` duration: per-container deadline for the test run (default: 30m)
- `--concurrency` int: worker pool size (default: 4)
- `--fail-fast` bool: cancel remaining runs on first failure (default: true)
- `--retries` int: docker run retry attempts on failures (default: 2)
- `--backoff` duration: initial backoff (exponential, capped at 60s)
- `-v|-vv` flags: verbosity (see VERBOSITY.md). `-vv` additionally streams stderr live.
- `-q` flag: quiet mode (final summary only)
- `--test-filter` string: run a single YAML suite by name (passed through to container)
- `--test-ci` bool: run the GH Actions job `test-orch` locally with `act`

## Container naming and run identity

- The orchestrator computes a content hash for the build context to form tags `oxidizr-<distro>:<hash>`.
- Each run gets a `runID` (UTC timestamp `YYYYMMDD-HHMMSSZ`), included in container names.
- Deterministic name format: `oxidizr-arch-<distro>-oxidizr-<distro>-<hash>-<runID>`.
- A `--cidfile` is used to capture container ID for reliable cleanup and summaries.

## Volumes and caches

- `/workspace` ← host `--root-dir` (typically repo root)
- Per-distro, namespaced caches under `<root>/.cache/test-orch/<component>/<distro>`:
  - Cargo registry/git, cargo target, rustup, pacman cache, AUR build cache
- These mounts are read/write caches to speed up repeated runs and avoid cross-distro contention.

## Environment contract (host → container)

The orchestrator passes only declared variables:

- `VERBOSE=0|1|2|3` — controls container-runner verbosity
- `TEST_FILTER=<suite>` — optional, restricts YAML suite selection
- `ANALYTICS_DISTRO=<distro>` — used by container-runner analytics writer for file naming
- `RUST_LOG=info` — only when `-vv` is set, to surface product INFO logs

No other toggles are injected. The orchestrator never rewrites container logs.

## Logging and proxying

- Container stdout is classified by intrinsic tags and streamed according to host verbosity.
- Container stderr is always captured in full and persisted to disk.
- At `-vv` verbosity, stderr is also live-streamed to the terminal (no severity changes).
- A compact progress indicator is rendered at default verbosity via special `PB>` frames
  emitted by the runner; full build/run streams are shown at `-vv`.

## Artifacts and summaries

Per container, the orchestrator writes these artifacts under `<root>/logs/<distro>/`:

- `<container>-stdout-YYYYMMDD-HHMMSSZ.log`
- `<container>-stderr-YYYYMMDD-HHMMSSZ.log`
- `<container>.cid` (container ID)
- `<container>-summary.json` (see schema below)

An aggregated summary is also written as `<root>/logs/aggregate-<runID>.json`.

### Per-container JSON summary schema

```json
{
  "distro": "arch",
  "image_digest": "sha256:...",  
  "container_id": "<id>",
  "started_at": "2025-09-09T14:20:30Z",
  "finished_at": "2025-09-09T14:27:11Z",
  "exit_code": 0,
  "log_paths": {
    "stdout": "/abs/path/logs/arch/<container>-stdout-...log",
    "stderr": "/abs/path/logs/arch/<container>-stderr-...log"
  },
  "suites": [
    { "name": "20-enable-default", "status": "pass" },
    { "name": "30-disable-default", "status": "fail", "expect": "xfail" }
  ]
}
```

The `suites` array is parsed from the runner’s stdout using the standardized
PASS/FAIL lines and expected-fail markers.

## Exit code policy

- The host process exits 0 if all containers completed without errors and `--fail-fast`
  did not cancel any runs.
- Any container failure (non-zero exit) or orchestration error causes a non-zero exit
  for the host. Summaries and logs are still emitted.

## Timeouts, retries, cleanup

- Each `docker run` is executed under a per-container deadline (`--timeout`).
- On failure, the invocation is retried up to `--retries` times with exponential backoff
  starting at `--backoff` (capped at 60s).
- If `--fail-fast` is true, the first failure cancels the remaining runs cooperatively.
- When `--keep-container` is false (default), the orchestrator attempts:
  1. `docker stop --time 10 <id>`
  2. `docker rm -f <id>`

## No in-container mutations

The orchestrator only mounts host paths and controls process execution. It does not
create/modify files inside the container beyond those side effects performed by the
container-runner itself.

## Optional tooling

- Testcontainers-Go — concise lifecycle and wait strategies.
  - Pros: less boilerplate, resilience features
  - Cons: adds abstraction and dependency layer
- Dagger (Go SDK) — graph-based pipelines with caching.
  - Pros: reproducibility, cache primitives, potential CI simplification
  - Cons: conceptual shift, CI integration work
- Staying on Docker CLI/SDK — keeps the surface small; we’ve added resilience,
  logging and summaries to make it robust without extra layers.

## Developer tips

- Increase verbosity with `-v` or `-vv` during debugging.
- Use `--test-filter=<suite>` to iterate on a single YAML suite quickly.
- Use `RUNNER_FROM_WORKSPACE=1` to bind mount a locally built container-runner
  binary over the image’s runner for fast iteration.
