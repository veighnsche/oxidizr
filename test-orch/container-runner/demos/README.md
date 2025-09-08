# Demos for Container Runner

This directory contains interactive/demo scripts that showcase oxidizr-arch features. These are not part of the automated YAML test suite and are excluded by the YAML runner.

## Demo: demo-utilities.sh

Exercises core utilities (uutils), findutils and sudo inside the prepared container environment.

Usage (inside the container image):

```bash
# Optional: build and link the binary into the PATH for interactive shells
/usr/local/bin/setup_shell.sh

# Run the demo
chmod +x /workspace/test-orch/container-runner/demos/demo-utilities.sh
/workspace/test-orch/container-runner/demos/demo-utilities.sh --cleanup
```

Notes:

- Requires the standard container-runner setup (users, sudoers, rust toolchain).
- The `--cleanup` flag cleans temporary files and disables experiments at the end (no-op in CI).
- The YAML runner will skip `tests/demo-utilities/` so this demo does not affect CI results.
