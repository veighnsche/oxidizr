# Host Orchestrator

The host orchestrator is responsible for managing Docker operations for the oxidizr-arch test suite. It builds Docker images, starts containers, and coordinates the execution of tests in isolated environments.

## Features

- Docker image building with caching support
- Container lifecycle management
- Interactive shell access to test containers
- Environment variable propagation to containers
- Verbose logging and error handling

## Usage

```bash
# Build and run tests across all supported distributions (default behavior)
sudo go run .

# Run tests on a single distribution (e.g., arch)
sudo go run . --distros=arch

# Build Docker image only
sudo go run . --arch-build

# Run tests in existing image
sudo go run . --run

# Open interactive shell in container (single distro only)
# Defaults to arch when not specified
sudo go run . --shell

# Or specify one distro explicitly
sudo go run . --shell --distros=manjaro

# Quick smoke test to validate Docker + Arch base image
sudo go run . --smoke-arch-docker

# Run the GitHub Actions 'test-orch' job locally with act
sudo go run . --test-ci

# Run with verbose or very verbose output
sudo go run . -v
sudo go run . -vv

# Quiet mode (only critical errors and final summary)
sudo go run . -q

# Run a single YAML test suite by name (example)
sudo go run . --test-filter="disable-all"

# Increase parallelism of distro runs
sudo go run . --concurrency=6

# Fail-fast cancel and retries with backoff
sudo go run . --fail-fast=true --retries=2 --backoff=8s
```

## Command Line Options

- `--smoke-arch-docker` (bool): Run a short Arch Docker smoke test (pacman + DNS). Default: `false`
- `--arch-build` (bool): Build the Docker image used for isolated tests. Default: `false`
- `--run` (bool): Run the Docker container to execute tests via the Go runner. Default: `false`
- `--shell` (bool): Open an interactive shell inside the Docker container. Default: `false`
- Notes for `--shell`:
  - Defaults to `arch` when `--distros` is left at the default multi-value.
  - Only a single distro is allowed (e.g., `--distros=cachyos`). Multiple values will exit with an error.
  - Automatically runs `setup_shell.sh` and drops you into `bash -l`, printing a hint to run the demo.
- `--distros` (string): Comma-separated list of distributions to test. Default: `arch,manjaro,cachyos,endeavouros`
- `--docker-context` (string): Docker build context directory. Default: `test-orch`
- `--root-dir` (string): Host directory to mount at `/workspace`. Defaults to repo root when omitted
- `--no-cache` (bool): Build without using cache. Default: `false`
- `--pull` (bool): Always attempt to pull a newer base image during build. Default: `false`
- `--keep-container` (bool): Do not remove container after run (omit `--rm`). Default: `false`
- `--timeout` (duration): Timeout for `docker run`. Default: `30m`
- `--test-filter` (string): Run only the named YAML suite directory (e.g., `disable-all`). Default: empty (run all)
- `--test-ci` (bool): Run the CI `test-orch` job locally using `act`. Default: `false`
- `--concurrency` (int): Number of distributions to test in parallel. Default: `4`
- `--fail-fast` (bool): Cancel remaining runs on first failure. Default: `true`
- `--retries` (int): Retry attempts for docker run failures. Default: `2`
- `--backoff` (duration): Initial backoff between retries (exponential). Default: `8s`
- `-v` (bool): Verbose output (level 2)
- `-vv` (bool): Very verbose/trace output (level 3)
- `-q` (bool): Quiet output (level 0)

Notes:

- When no action flags are provided, the default behavior is to perform `--arch-build` and `--run`.
- Root privileges are required on hosts not configured with a `docker` group for reliable Docker access.

## Requirements

- Docker installed and running
- Root privileges (sudo) for Docker access
- Go 1.21 or later

## Architecture

The host orchestrator communicates with a separate container-runner program that executes inside the Docker container. The container-runner handles the actual test execution, environment setup, and assertions.

For detailed design, logging, artifact layout, and JSON summary schema, see `HOST_ORCH.md` in this directory.
