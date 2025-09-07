# Plan: Making the Codebase Distribution-Aware

This document outlines the root cause of the persistent test failures and the comprehensive plan to make the `oxidizr-arch` application and its test suite fully aware of the differences between vanilla Arch Linux and its derivatives (e.g., CachyOS, EndeavourOS, Manjaro).

## 1. The Goal

The primary objective is to enable robust, reliable matrix testing across multiple Arch-based distributions. This requires the application and its tests to correctly handle variations in package names, binary paths, and feature availability between vanilla Arch and its derivatives.

## 2. The Root Cause of Failures

Our previous attempts to fix the test failures were a series of incremental patches that did not address the fundamental, interconnected issues. This created a cycle of new errors. The core problem is that the entire system was initially designed with only vanilla Arch Linux in mind.

The failures stem from four key areas:

1.  **Incorrect Distribution Detection:** The logic in `src/utils/worker.rs` was incorrectly normalizing all Arch-based distributions to a generic ID of `"arch"`. This erased the specific identity of the running OS (e.g., `endeavouros`), making it impossible for downstream code to make distribution-specific decisions.

2.  **Hardcoded Package Names and Paths:** The application code in `src/experiments/mod.rs` hardcoded the names of experimental packages (e.g., `uutils-coreutils`, `sudo-rs`). However, many Arch derivatives provide these replacements in their main repositories under standard names (`coreutils`, `sudo`). The code was therefore trying to install packages that don't exist on those platforms.

3.  **Inflexible Test Definitions:** The test suites (`*.yaml`) were written with the assumption that they would only ever run on vanilla Arch. They contain assertions that are logically incorrect on derivatives. For example, a test might assert that `sudo-rs` is installed, which will always fail on a derivative that uses the standard `sudo` package.

4.  **Resource Exhaustion:** Running the entire test matrix in parallel without constraints was causing the host system to run out of memory, killing the Docker containers. This created random-seeming failures that masked the underlying logical bugs, making them much harder to diagnose.

## 3. The Holistic Solution

To fix this properly, we need a coordinated sweep across both the application code (`src`) and the test orchestration code (`test-orch`).

### Product Code (`src`) Changes

The application logic will be made distribution-aware.

-   **`src/experiments/mod.rs`**: This file will become the **single source of truth** for experiment configurations. It will contain the logic to dynamically select the correct package names and binary installation paths based on the detected distribution ID.
-   **`src/experiments/sudors.rs`**: The compatibility check (`check_compatible`) will be simplified to restrict the `sudo-rs` experiment to **vanilla Arch only**, as derivatives use the standard `sudo` package.
-   **`src/experiments/uutils/disable.rs`**: The `disable` logic will be made smarter. It will check if a package is an experimental version (e.g., starts with `uutils-`) before attempting to uninstall it, preventing it from trying to remove essential system packages on derivatives.

### Test Orchestration (`test-orch`) Changes

The test runner will be made intelligent enough to skip tests that are not applicable to the current environment.

-   **`test-orch/host-orchestrator/main.go`**: The `--concurrency` flag will be correctly implemented and used to limit parallel test runs, preventing resource exhaustion and providing stable test results.
-   **`test-orch/container-runner/`**: We will implement a `distro-check` key in the test YAML files.
    -   The Go-based runner inside the container will read this key.
    -   It will compare the key's value against the container's actual distribution (read from `/etc/os-release`).
    -   If there is no match, the runner will **skip** the test suite and log a message, rather than running it and failing.
-   **`tests/**/*.yaml`**: We will add the `distro-check: [arch]` key to all test files that are specific to vanilla Arch Linux. This ensures tests for features like `sudo-rs` or the installation of `uutils-coreutils` only run where they are supposed to.

By implementing this comprehensive plan, we will create a robust and reliable testing matrix that correctly handles the nuances of each Arch-based distribution.
