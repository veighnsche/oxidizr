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

# Open interactive shell in container
sudo go run . --shell

# Run with verbose output
sudo go run . -v

# Run specific test filter
sudo go run . --test-filter="disable-all"
```

## Command Line Options

- `--distros`: Comma-separated list of distributions to test. Defaults to all. E.g., `--distros=arch`
- `--concurrency`: Number of distributions to test in parallel (default: 4)
- `--arch-build`: Build the Docker image for the specified distributions
- `--run`: Run tests in Docker container
- `--shell`: Open interactive shell in container
- `--image-tag`: Docker image tag (default: oxidizr-arch:latest)
- `--docker-context`: Docker build context directory
- `--root-dir`: Host directory to mount at /workspace
- `--no-cache`: Build without using cache
- `--pull`: Always pull newer base image during build
- `--keep-container`: Don't remove container after run
- `--timeout`: Timeout for docker run (default: 30m)
- `--test-filter`: Run specific test YAML file
- `-v`: Verbose output
- `-vv`: Very verbose (trace) output
- `-q`: Quiet output

## Requirements

- Docker installed and running
- Root privileges (sudo) for Docker access
- Go 1.21 or later

## Architecture

The host orchestrator communicates with a separate container-runner program that executes inside the Docker container. The container-runner handles the actual test execution, environment setup, and assertions.
