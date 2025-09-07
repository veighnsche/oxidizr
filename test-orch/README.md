# Isolated Test Runner (Arch Linux + Docker)

This directory contains a small Go utility and Docker assets to run the project's end-to-end assertions inside a clean Arch Linux container.

It is useful when you want strong isolation from your host OS and reproducible setup of required packages and toolchains.

- Runner: `main.go`
- Docker context: `docker/` (contains `Dockerfile` and `entrypoint.sh`)

## What it does

- Validates your Docker setup.
- Optionally runs a quick Arch smoke test (`docker pull`, `pacman`, DNS check).
- Builds an Arch Docker image with the tools needed for the tests.
- Runs that image mounting your repository at `/workspace` and executes `docker/entrypoint.sh`.
- The entrypoint will:
  - Stage the repo into `/root/project/oxidizr-arch` for paths expected by the tests.
  - Ensure base packages and Rust toolchains exist.
  - Build the project (produces `target/release/oxidizr-arch` and symlinks it to `/usr/local/bin/oxidizr-arch`).
  - Run enable/disable assertions using helper scripts in `tests/lib/`.

## Prerequisites

- Docker installed and the daemon running.
- Your user can run Docker without `sudo` (typically by being in the `docker` group).
- Go 1.21+.

## Quick start

From the repository root:

```bash
# Zero-flag: just run it (builds image if needed, then runs tests)
go run ./test-orch

# Or with the compiled runner
(cd test-orch && go build -o isolated-runner)
./test-orch/isolated-runner

```

If you prefer not to build the binary:

```bash
# Build image only
go run ./test-orch --arch-build

# Run tests (auto-builds the image if missing)
go run ./test-orch --arch-run
```

## Useful options

The runner supports several flags (see `main.go`):

- `--smoke-arch-docker` — Run a short Arch smoke test (`pacman` + DNS) with the public `archlinux:base-devel` image.
- `--arch-build` — Build the isolated Arch Docker image in `docker/`.
- `--arch-run` — Run the container and execute `docker/entrypoint.sh` to perform assertions.
- (no flags) — One-shot: build the image if needed, then run the tests.
- `--image-tag` — Image tag to build/run (default: `oxidizr-arch:latest`).
- `--docker-context` — Docker build context directory (default: `testing/isolated-runner/docker`).
- `--root-dir` — Host directory to mount at `/workspace` (defaults to the repository root; auto-detected via Git when possible).
- `--no-cache` — Build the Docker image without using cache.
- `--pull` — Always attempt to pull newer base image layers during build.
- `--keep-container` — Do not remove the container after run (omit `--rm`).
- `--timeout` — Timeout for `docker run` (default: 30m).
- `-v` — Verbose output (default: true).

Examples:

```bash
# Just verify Docker and run a quick smoke test
go run ./testing/isolated-runner --smoke-arch-docker

# Build without cache and always pull latest base
go run ./testing/isolated-runner \
  --arch-build --no-cache --pull --image-tag oxidizr-arch:latest

# Zero-flag build+run with a custom tag
go run ./test-orch \
  --image-tag oxidizr-arch:dev

# Run with an explicit repo root (if auto-detection fails)
go run ./test-orch \
  --arch-run --root-dir "$PWD" --image-tag oxidizr-arch:latest
```

## How it works

- The Go runner detects the repository root (via `git rev-parse --show-toplevel` or heuristics) and builds/runs the Docker image.
- When running, it mounts the repo root at `/workspace` and launches `docker/entrypoint.sh` inside the container.
- The entrypoint:
  - copies `/workspace` to `/root/project` (expected by the tests);
  - ensures required packages exist (via `pacman`) and installs Rust toolchains (via `rustup`);
  - installs AUR helpers/packages needed for the experiments (`paru-bin`, `uutils-*`, `sudo-rs`);
  - builds the project and exposes `oxidizr-arch` on the PATH;
  - runs enable/disable assertions from `tests/lib/*.sh`.

See:
- `docker/Dockerfile`
- `docker/entrypoint.sh`
- `main.go` (functions: `buildArchImage`, `runArchContainer`, `detectRepoRoot`)

## Troubleshooting

- Docker not found or not responding
  - Ensure Docker is installed and the daemon is running.
  - Verify your user can run `docker version` without `sudo`.

- Repo root not detected
  - Pass `--root-dir /path/to/repo/root` to the runner.

- Network/DNS issues when pulling images or running `pacman`
  - Try `--smoke-arch-docker` to verify.
  - Check your network connectivity and DNS setup.

- Build failures inside the container
  - Check the output; you can keep the container for inspection with `--keep-container` and then re-run with increased verbosity.

## Cleaning up

Images and containers are standard Docker objects. Remove them with:

```bash
# Remove container if kept
docker rm -f oxidizr-arch-test || true

# Remove the image
docker rmi oxidizr-arch:latest || true
```
