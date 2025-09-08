# Isolated Test Runner (Arch Linux + Docker)

This directory contains small Go utilities and Docker assets to run the project's end-to-end assertions inside clean Arch Linux-based containers.

It is useful when you want strong isolation from your host OS and reproducible setup of required packages and toolchains.

The test orchestration system has been separated into two independent Go programs:

1. **Host Orchestrator** (`host-orchestrator/`): Manages Docker operations from the host system
2. **Container Runner** (`container-runner/`): Executes tests inside Docker containers

This separation provides better modularity, clearer responsibilities, and easier maintenance.

## Quick Start

### Running Tests

```bash
# Navigate to host orchestrator directory
cd host-orchestrator/

# Build image and run tests across default distributions (requires sudo for Docker access)
sudo go run .

# Run tests on a single distribution (e.g., arch)
sudo go run . --distros=arch

# Build Docker image only
sudo go run . --arch-build

# Run tests with verbose output
sudo go run . -v

# Quick smoke test to validate Docker + Arch base image
sudo go run . --smoke-arch-docker

# Run GitHub Actions 'test-orch' job locally with act
sudo go run . --test-ci

# Run specific test filter
sudo go run . --test-filter="disable-all"

# Increase parallelism of distro runs
sudo go run . --concurrency=6

# Open interactive shell in test container
sudo go run . --shell
```

### Prerequisites

- Docker installed and running
- Root privileges (sudo) for Docker access
- Go 1.21 or later

## Architecture

The test orchestration system consists of two separate Go programs:

### Host Orchestrator (`host-orchestrator/`)

Responsible for:
- Docker image building and management
- Container lifecycle operations
- Environment variable propagation
- Host-side logging and error handling
- Interactive shell access

### Container Runner (`container-runner/`)

Responsible for:
- Environment setup inside containers
- YAML test suite execution
- Test assertions and validation
- In-container logging and reporting

## Directory Structure

```
test-orch/
├── host-orchestrator/           # Host-side Docker orchestration
│   ├── main.go                  # Host orchestrator entry point
│   ├── dockerutil/              # Docker operations and utilities
│   ├── helpers.go               # Shared utility functions
│   ├── docker_checks.go         # Docker validation and troubleshooting
│   ├── go.mod                   # Host orchestrator dependencies
│   └── README.md                # Host orchestrator documentation
├── container-runner/            # In-container test execution
│   ├── main.go                  # Container entry point
│   ├── runner.go                # Test execution logic
│   ├── setup/                   # Environment setup
│   ├── yamlrunner/              # YAML test suite execution
│   ├── assertions/              # Test assertions
│   ├── util/                    # Shared utilities
│   ├── go.mod                   # Container runner dependencies
│   └── README.md                # Container runner documentation
├── docker/
│   └── Dockerfile               # Container image definition
└── README.md                   # This file
```

## Command Line Options

The host orchestrator supports these options:

- `--smoke-arch-docker` (bool): Run a short Arch Docker smoke test (pacman + DNS). Default: `false`
- `--arch-build` (bool): Build the Docker image used for isolated tests. Default: `false`
- `--run` (bool): Run the Docker container to execute tests via the Go runner. Default: `false`
- `--shell` (bool): Open an interactive shell inside the Docker container. Default: `false`
- `--distros` (string): Comma-separated list of distributions to test. Default: `arch,manjaro,cachyos,endeavouros`
- `--docker-context` (string): Docker build context directory. Default: `test-orch`
- `--root-dir` (string): Host directory to mount at `/workspace`. Defaults to repo root when omitted
- `--no-cache` (bool): Build without using cache. Default: `false`
- `--pull` (bool): Always attempt to pull a newer base image during build. Default: `false`
- `--keep-container` (bool): Do not remove container after run (omit `--rm`). Default: `false`
- `--timeout` (duration): Timeout for `docker run`. Default: `30m`
- `--test-filter` (string): Run a single test YAML suite (e.g., `disable-all`). Default: empty (run all)
- `--test-ci` (bool): Run the CI `test-orch` job locally using `act`. Default: `false`
- `--concurrency` (int): Number of distributions to test in parallel. Default: `4`
- `-v` (bool): Verbose output (level 2)
- `-vv` (bool): Very verbose/trace output (level 3)
- `-q` (bool): Quiet output (level 0)

Notes:
- When no action flags are provided, the default behavior is to perform `--arch-build` and `--run`.
- Root privileges are required on hosts not configured with a `docker` group for reliable Docker access.

## Container Runner Options and Environment

The container-runner (executed inside the container) supports:

Command line options:
- `--test-filter` (string): Run only the named YAML suite directory (e.g., `disable-in-german`). Default: empty (run all)
- `--full-matrix` (bool): Fail on skipped suites (equivalent to setting `FULL_MATRIX=1`). Default: `false`

Environment variables:
- `VERBOSE`: Controls logging verbosity (0-3). Propagated by the host orchestrator.
- `TEST_FILTER`: Run specific test YAML file. Set automatically when `--test-filter` is used.
- `FULL_MATRIX`: When `1`, fail fast on missing prerequisites or skipped suites.

Commands:
- `internal-runner`: Execute the full test suite including YAML tests and assertions
- `--help`: Show usage information

## Locale and parallel-run handling

The runner avoids modifying locales and only probes/logs their presence during preflight for visibility. Most suites are independent of locale status and run across all distros.

- `disable-in-german` has a known flakiness when the matrix runs distros in parallel. This suite may SKIP in parallel runs across the Arch-family (including Arch) but passes when run in isolation/serialized.
- All other suites run across all distros.

Rationale: keep images minimal and avoid mutating system locales during tests; address nondeterminism by deflaking or serializing the affected suite rather than masking with harness logic.

## Interactive shell helper

When launching an interactive container shell, `docker/setup_shell.sh` can be used to compile a release build and symlink it into `/usr/local/bin/oxidizr-arch` for convenience:

```bash
/usr/local/bin/setup_shell.sh
```

This script runs `cargo build --release` in `/workspace` and symlinks the resulting binary. It is not used during CI runs handled by the container runner.

## Test Flow

1. **Host Orchestrator**: Builds Docker image and starts container
2. **Container Runner**: Takes over inside the container
3. **Environment Setup**: Install system packages, Rust toolchain, and build tools
4. **Project Build**: Compile oxidizr-arch from source
5. **YAML Test Suites**: Execute declarative test scenarios
6. **Assertions**: Run custom validation logic
7. **Cleanup**: Restore system state and report results

## Development

### Working with Separated Programs

Each program has its own `go.mod` file and can be developed independently:

```bash
# Work on host orchestrator
cd host-orchestrator/
go build .
go test ./...

# Work on container runner
cd container-runner/
go build .
go test ./...
```

### Adding New Features

- **Host-side features**: Modify files in `host-orchestrator/`
- **Container-side features**: Modify files in `container-runner/`
- **Docker image changes**: Update `docker/Dockerfile`

### Building and Testing

```bash
# Build host orchestrator
cd host-orchestrator/
go build .

# Build container runner
cd container-runner/
go build .

# Test the complete system
cd host-orchestrator/
sudo ./host-orchestrator
```

## Benefits of Separation

1. **Modularity**: Each program has a single, clear responsibility
2. **Independent Development**: Programs can be developed and tested separately
3. **Cleaner Dependencies**: Each program only includes necessary dependencies
4. **Better Documentation**: Each program has its own focused README
5. **Easier Maintenance**: Smaller, more focused codebases
6. **Deployment Flexibility**: Programs can be distributed and versioned independently
- `main.go` (functions: `buildArchImage`, `runArchContainer`, `detectRepoRoot`)

## Distro Environment Matrix

For known differences between the Arch-based distros in the test matrix (locales, AUR helpers, repos), see:

- [test-orch/DISTRO_MATRIX.md](./DISTRO_MATRIX.md)

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
