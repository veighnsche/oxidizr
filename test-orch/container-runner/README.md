# Container Runner

The container runner executes inside Docker containers to perform the actual test execution for the oxidizr-arch test suite. It handles environment setup, YAML test suite execution, and test assertions in an isolated Arch Linux environment.

## Features

- Environment setup and configuration
- YAML test suite parsing and execution
- Test assertions and validation
- Logging and error reporting
- Integration with host orchestrator

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
