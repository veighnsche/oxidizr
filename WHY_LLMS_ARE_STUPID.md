# WHY_LLMS_ARE_STUPID.md

## Context

- File under discussion: `test-orch/docker/entrypoint.sh`
- Real implementation of switching coreutils: `src/experiments/uutils.rs` and `src/utils/worker.rs`
- The orchestrator invokes `oxidizr-arch --assume-yes --all ... enable` to flip applets.

## What happened

- While debugging Docker test failures, an LLM suggested installing `busybox` in the container and using it to perform file operations (`cp`, `ln`, `rm`) when flipping applet symlinks.
- It also suggested pre-creating applet symlinks (e.g., `readlink`) before the actual `oxidizr-arch enable`.

## Why this was a bad idea

- **Masked the real problem (sequencing):** The Rust implementation already performs safe, syscall-based operations to back up and atomically symlink applets (`replace_file_with_symlink()` and `restore_file()` in `src/utils/worker.rs`). The test script should not mutate `/usr/bin/*` before or around `enable`. Installing BusyBox hid the fact that the script’s sequencing was wrong.
- **Introduced non-goal dependency:** The goal is to validate `oxidizr-arch`’s switching logic, not to require an additional toolset. Adding BusyBox in the container is orthogonal to the product and drifts the test away from the intended surface area.
- **Confused the contract:** The presence of BusyBox implied that core utilities might be fundamentally unavailable, which is not the contract after `enable`. The contract is that applets are available via symlinks (GNU or uutils), and the test should assert that—after `enable`—not try to manufacture them beforehand.
- **Increased complexity and failure modes:** The test harness started managing its own applet symlinks, which can conflict with `oxidizr-arch`’s own logic, leading to churn, path/hash invalidation, and brittle state.

## The correct fix (root-cause oriented)

- **Let the product do the switching:** Do not pre-create or repair applet symlinks in `test-orch/docker/entrypoint.sh`.
- **Keep the test simple:**
  1. Build and install the `oxidizr-arch` binary into a safe path (e.g., `/usr/local/bin`).
  2. Call `oxidizr-arch --assume-yes --all --package-manager none enable`.
  3. Run assertions from `tests/lib/uutils.sh` and `tests/lib/sudo-rs.sh`.
  4. Call `oxidizr-arch ... disable`.
  5. Verify system is restored.
- **Avoid shelling out for mutation:** The product already uses Rust syscalls (`std::fs`, `unix_fs::symlink`) and avoids relying on `cp/ln/rm` being stable while they are being swapped.

## References in the codebase

- `src/experiments/uutils.rs`: Builds the list of applets (`tests/lib/rust-coreutils-bins.txt`) and delegates file ops to the worker.
- `src/utils/worker.rs`:
  - `replace_file_with_symlink()`: backs up existing targets and atomically symlinks the selected provider, without external binaries.
  - `restore_file()`: restores from backup atomically.

## Lessons learned

- **Don’t fix symptoms in tests:** If a test script mutates the same artifacts the product controls, fix the sequence so the product does the mutation. Don’t add extra tools to paper over it.
- **Minimize the test surface:** The more the test harness rewrites system state, the more it risks diverging from the product behavior under test.
- **Prefer atomic, syscall-based changes:** They are less error-prone than invoking external binaries that may be unavailable during a switch.

## Action items

- Remove BusyBox-related logic from `test-orch/docker/entrypoint.sh`.
- Keep only: build binary, run `enable`, assert, run `disable`, assert.
- If persistent caching is needed, move heavy, stable dependencies into the Dockerfile—do not mutate applet symlinks in the entrypoint.
