# Container Runner

The container runner executes inside Docker containers to perform the actual test execution for the oxidizr-arch test suite. It handles environment setup, YAML test suite execution, and test assertions in an isolated Arch Linux environment.

## Features

- Environment setup and configuration
- YAML test suite parsing and execution
- Test assertions and validation
- Logging and error reporting
- Integration with host orchestrator

## Setup phases (performed automatically)

The runner performs a well-defined sequence of setup phases inside the container (see `setup/`):

1. Workspace staging (`setup/workspace.go`)
   - Copies the mounted repository into `/root/project/oxidizr-arch`
2. System dependencies (`setup/deps.go`)
   - Installs: `base-devel sudo git curl rustup which findutils`
   - Removes CachyOS-specific repo cache if present to ensure standard behavior
3. Users (`setup/users.go`)
   - Ensures `builder` and `spread` users exist
   - Writes `/etc/sudoers.d/99-builder` with passwordless sudo for CI tasks
4. AUR helper (`setup/users.go#installAurHelper`)
   - Installs `paru-bin` from AUR when not present (skips if preinstalled)
5. Rust toolchain (`setup/rust.go`)
   - Sets `rustup default stable` for root and `builder`
6. Build (`setup/build.go`)
   - `cargo build --release` and installs `/usr/local/bin/oxidizr-arch`

## Usage

This program is designed to be executed inside Docker containers by the host orchestrator. It accepts commands and environment variables to control its behavior.

```bash
# Run internal test suite (called by host orchestrator)
./container-runner internal-runner

# Show help
./container-runner --help

# Run only a specific YAML suite
./container-runner --test-filter="disable-in-german"

# Enforce full-matrix semantics (fail fast on skipped suites)
./container-runner --full-matrix
```

## Command Line Options

- `--test-filter` (string): Run only the named YAML suite directory (e.g., `disable-in-german`). Default: empty (run all)
- `--full-matrix` (bool): Fail on skipped suites (equivalent to setting `FULL_MATRIX=1`). Default: `false`

## Environment Variables

- `VERBOSE`: Controls logging verbosity (0-3). Propagated by the host orchestrator.
- `TEST_FILTER`: Run specific test YAML file instead of all tests. Set automatically when `--test-filter` is used.
- `FULL_MATRIX`: When `1`, fail fast on missing prerequisites or skipped suites. Set automatically when `--full-matrix` is used.

## Commands

- `internal-runner`: Execute the full test suite including YAML tests and assertions
- `--help`: Show usage information

## Locale and parallel-run handling

Locales are baked into the Docker image at build time (see `test-orch/docker/Dockerfile`), including `de_DE.UTF-8`. The runner may probe/log locale status for visibility, but tests must not SKIP due to locale availability.

- `disable-in-german` must run across the Arch-family and must not SKIP. Any failure is treated as a hard error to be fixed (infra or product), not skipped.
- All other suites run across all distros.

Rationale: bake deterministic prerequisites into images for reproducible CI. Address any parallel-run nondeterminism by deflaking tests or serializing in CI configuration, not by skipping.

## Interaction with Dockerfile

The Dockerfile pre-provisions prerequisites for deterministic execution, including baking `de_DE.UTF-8` into the image. User management, Rust toolchain configuration, and AUR helper installation remain the runner's responsibility.

## Interactive shell helper

When launching an interactive container shell, `docker/setup_shell.sh` can be used to compile a release build and symlink it into `/usr/local/bin/oxidizr-arch` for convenience:

```bash
/usr/local/bin/setup_shell.sh
```

This script simply runs `cargo build --release` in `/workspace` and symlinks the resulting binary. It is not used during CI runs handled by the container runner.

## Architecture

The container runner is organized into several packages:

- `setup/`: Environment setup and configuration
- `yamlrunner/`: YAML test suite execution
- `assertions/`: Test assertions and validation
- `util/`: Shared utility functions

## Test Flow

1. Environment setup (Rust toolchain, system packages)
2. YAML test suite execution
3. Custom assertions and validations
4. Result reporting

## Requirements

- Go 1.21 or later
- Arch Linux environment (provided by Docker container)
- Access to oxidizr-arch source code (mounted at /workspace)

## Integration

This program works in conjunction with the host orchestrator, which:
- Builds the Docker image containing this runner
- Starts containers with appropriate environment variables
- Mounts the source code and manages container lifecycle
