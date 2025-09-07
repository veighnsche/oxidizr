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
3. Locales (`setup/locales.go`)
   - Ensures `/etc/locale.gen` contains `en_US.UTF-8`, `de_DE.UTF-8`, `C.UTF-8`
   - Attempts to generate locales via `locale-gen`
   - Best-effort remediation if `de_DE` definition is missing on derivative images
4. Users (`setup/users.go`)
   - Ensures `builder` and `spread` users exist
   - Writes `/etc/sudoers.d/99-builder` with passwordless sudo for CI tasks
5. Rust and AUR helper (`setup/rust.go`)
   - Sets `rustup default stable` for root and `builder`
   - Installs `paru-bin` from AUR when not present
6. Build (`setup/build.go`)
   - `cargo build --release` and installs `/usr/local/bin/oxidizr-arch`

## Usage

This program is designed to be executed inside Docker containers by the host orchestrator. It accepts commands and environment variables to control its behavior.

```bash
# Run internal test suite (called by host orchestrator)
./container-runner internal-runner

# Show help
./container-runner --help
```

## Environment Variables

- `VERBOSE`: Controls logging verbosity (0-3)
- `TEST_FILTER`: Run specific test YAML file instead of all tests

## Commands

- `internal-runner`: Execute the full test suite including YAML tests and assertions
- `--help`: Show usage information

## Locale handling

Some Arch derivative images (e.g., CachyOS, Manjaro, EndeavourOS) may ship stripped locale data. The runner:

- Populates `/etc/locale.gen` with `en_US.UTF-8`, `de_DE.UTF-8`, and `C.UTF-8`
- Runs `locale-gen`
- If German locale definition files are missing (e.g., `/usr/share/i18n/locales/de_DE`), it attempts a best-effort remediation by reinstalling `glibc-locales` and preparing `locale.gen`. If locale generation still fails, YAML tests are responsible for failing fast in FULL_MATRIX mode.

Rationale: keep the Dockerfile minimal and put environment logic in the runner, where it can be logged, retried, and tested.

## Interaction with Dockerfile

The Dockerfile provides only the minimal base packages. User management, Rust toolchain configuration, AUR helper installation, and locale generation are intentionally handled by the runner. This keeps images simple and reproducible, and centralizes logic in code instead of image layers.

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
