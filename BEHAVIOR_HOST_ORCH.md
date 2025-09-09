# Host Orchestrator (Go)

Catalog of observable behavior for the host-side Go orchestrator that builds images and runs tests inside Docker containers. This documents lifecycle, concurrency, logging proxying, environment/volume wiring, and artifact handling as implemented today.

References:

- `test-orch/host-orchestrator/` (e.g., `main.go`, `helpers.go`, `docker_checks.go`)
- `test-orch/host-orchestrator/dockerutil/` (`build.go`, `run.go`, `run_args.go`, `shell.go`, `classify.go`, `util.go`)
- Docker context: `test-orch/docker/` (`Dockerfile`, `setup_shell.sh`)
- Verbosity policy: `VERBOSITY.md`

## Responsibilities

- Build per-distro Docker images that contain the in-container test runner.
- Launch non-interactive test executions and interactive shells.
- Coordinate parallel runs across multiple distributions with bounded concurrency and shared cancellation on failure.
- Proxy container stdout/stderr to the host with unified verbosity and prefix format.
- Wire the workspace and persistent caches via bind mounts; manage env propagation into the container.
- Capture, rotate, and persist container logs on failure.

## CLI surface and defaults (`test-orch/host-orchestrator/main.go`)

- Flags (subset):
  - `--arch-build`: build images; `--pull` refreshes base; `--no-cache` disables cache.
  - `--run`: run tests using the in-container runner.
  - `--shell`: open interactive shell (single distro only; defaults to `arch` if `--distros` left at the default multi-value).
  - `--distros`: comma-separated list; default `arch,manjaro,cachyos,endeavouros`.
  - `--docker-context`: build context dir; default `test-orch` (resolved relative to repo root when possible).
  - `--root-dir`: host path to mount at `/workspace`; defaults to repo root.
  - `--concurrency`: max concurrent distros (default 4).
  - `-q`, `-v`, `-vv`: select verbosity (maps to v0..v3; see VERBOSITY.md).
  - `--test-filter`: passes `TEST_FILTER=<name>` to container runner.
  - `--test-ci`: runs the GitHub Actions `test-orch` job locally via `act`.
- Defaults: with no action flags, performs build + run across selected distros.
- Requires root (UID 0) for reliable Docker access; exits with guidance otherwise.

## Distro mapping, context, and image tags

- Base images:
  - `arch → archlinux:base-devel`
  - `manjaro → manjarolinux/base`
  - `cachyos → cachyos/cachyos:latest`
  - `endeavouros → alex5402/endeavouros` (aliases `endeavoros`/`endeavouros` normalized).
- Docker context directory: if relative, resolved against repo root if detected; else CWD.
- Image tag format: `oxidizr-<distro>:<hash>` where `<hash>` is a short SHA-256 over selected inputs (`docker/Dockerfile`, `docker/setup_shell.sh`, container-runner sources and demos) computed by `computeBuildHash()`.
  - Hash length: 12 hex characters; changes whenever any input file content changes.
  - Inputs include the `container-runner` source tree, the `container-runner/demos/` directory, and also the `container-runner` binary if present (used when iterating locally).

## Container lifecycle

### Build path (`dockerutil.BuildArchImage`)

- Uses Docker SDK to run `docker build` with:
  - `Dockerfile`: `docker/Dockerfile` under the provided context.
  - `--pull` (`PullParent`) and `--no-cache` supported via flags.
  - Tag includes the computed content hash to enable reuse across unchanged inputs.
- Logging:
  - v1: compact progress bar derived from "Step X/Y" lines; errors summarized.
  - v3: full JSON stream/trace with `[<distro>][v3][HOST]` prefixes.
  - v2: command echo `RUN> docker build ...` and selected status lines.

### Non-interactive run (`dockerutil.RunArchContainer`)

- Prepares `docker run` args via `BuildDockerRunArgs` and executes with a timeout using `exec.CommandContext`.
- Container name: `oxidizr-arch-test-<tag-with-colon-replaced-by-dash>`; any existing container with this name is force-removed before start.
- `--rm` is used by default; omitted when `--keep-container` is set.
- Timeout: the run is bounded by `--timeout` (default 30m). When one distro fails, a shared context cancels sibling runs.
- On failure: temp stdout/stderr files are renamed to timestamped artifacts under `logs/<distro>/` and chmod `0644`; error includes the command and log paths.
- On success: temp logs are deleted.

### Interactive shell (`dockerutil.RunArchInteractiveShell`)

- Ensures image exists (auto-build handled by caller when missing); starts container with TTY, stdin open, and entrypoint `setup_shell.sh && bash -l`.
- Container name: `oxidizr-arch-shell`; `AutoRemove` is enabled.
- Replicates cache bind mounts used by non-interactive runs.
- Distro selection: when `--shell` is used with the default multi-value `--distros` setting, the orchestrator defaults to `arch`. Otherwise a single explicit distro is required. Aliases such as `endeavoros`/`endeavouros` are normalized to `endeavouros`.
- Does not treat non-zero exit as an error; logs exit code at v2.

## Concurrency and cancellation (parallel multi-distro runs)

- One goroutine per distro; start gated by a semaphore of size `--concurrency`.
- A shared `context.Context` is used; the first failing run cancels the parent context to nudge other runs to terminate.
- Errors are collected; a final v0 summary prints pass/fail and the process exits with code 0/1.

## Logging proxying and verbosity

- Unified prefix format for host-emitted lines: `[<distro>][vN][HOST] ...` (see `dockerutil.Prefix`).
- Container stdout is scanned line-by-line, classified, and either:
  - printed according to selected verbosity, prefixed `[<distro>][vN]` (scope omitted for product/raw lines), or
  - suppressed into a bounded ring buffer (last 200 lines) when below the threshold.
- Container stderr is always captured to temp files and not printed to console; tail retained for postmortem.
- Classification (`dockerutil.classifyLine`):
  - Explicit tags like `[v2][RUNNER]` or `[vN]` are honored.
  - Lines containing ` audit ` or `component=... event=` are treated as v3 (tracey product audit).
  - Rust-style `ERROR/WARN/INFO/DEBUG/TRACE` substrings map to v0..v3.
- Progress bar protocol for runs (v1 only): lines like `PB> <x>/<y> <label>` update an in-place progress bar on stderr; frames are still written to stdout temp logs.
- Color: each distro run is assigned a distinct ANSI color for readability.

## Known issues and quirks

- Error aggregation channel not closed: in `main.go`, errors from parallel runs are sent on a buffered `errs` channel, but the channel is never closed before iterating with `for err := range errs`. This can hang the final summary phase. Mitigation: close the channel after all goroutines complete, or collect errors in a slice guarded by a mutex.
- Ownership of host-side cache directories: when running as root without `sudo`, `SUDO_UID/SUDO_GID` are not set, so cache dirs remain owned by root. Running via `sudo` ensures directories are chowned back to the invoking user for easier cleanup.

## Environment propagation and bind mounts (`dockerutil.BuildDockerRunArgs`)

- Workspace mount: `-v <rootDir>:/workspace` with `--workdir /workspace`.
- Environment variables set:
  - `ANALYTICS_DISTRO=<distro>` derived from the image tag; used by the container runner for report naming.
  - `VERBOSE=0..3` from host CLI flags.
  - `RUST_LOG=info` when host verbosity is v3 to surface product logs.
  - `TEST_FILTER=<name>` when `--test-filter` is provided.
- Persistent cache mounts (namespaced per-distro under `<rootDir>/.cache/test-orch/`):
  - Cargo registry → `/root/.cargo/registry`
  - Cargo git → `/root/.cargo/git`
  - Cargo target → `/workspace/target`
  - Pacman cache → `/var/cache/pacman`
  - AUR build workspace → `/home/builder/build`
  - Rustup root → `/root/.rustup`
- Host-side directory preparation:
  - All cache dirs and `logs/<distro>/` are created (`0755`).
  - When running as root via `sudo`, ownership is chowned to `SUDO_UID:SUDO_GID` to keep artifacts removable by the invoking user; perms reset to `0755`.
- Optional runner override for fast iteration:
  - When `RUNNER_FROM_WORKSPACE=1` and `test-orch/container-runner/isolated-runner` exists in the host workspace, it is bind-mounted read-only to `/usr/local/bin/isolated-runner` in the container, replacing the baked binary. A v2 host log notes the override; a v2 WARN prints when env is set but the file is absent.

## Artifacts and outputs

- Logs on failure:
  - Saved under `<rootDir>/logs/<distro>/` as `oxidizr-arch-test-<tag>-stdout-<ts>.log` and `...-stderr-<ts>.log` (`0644`).
- Persistent caches under `<rootDir>/.cache/test-orch/<component>/<distro>/` speed up repeated runs and avoid cross-distro contention.
- Interactive sessions do not produce rotated logs by design; console I/O is pass-through.

## Host readiness checks and utilities

- Docker checks (`docker_checks.go`): verifies `docker` presence and daemon responsiveness; prints troubleshooting tips (v2) or concise hints (v1).
- Smoke test (`--smoke-arch-docker`): `docker run --rm archlinux:base-devel pacman --version` to validate basic networking and container execution.
- Local CI (`--test-ci`): runs `act -j test-orch` from the repo root with a fixed runner image.

## Exit semantics and summaries

- On overall success, prints a v0 summary line and exits 0.
- On any error, prints a v0 failure summary, ensures logs are persisted (for non-interactive runs), and exits 1.
