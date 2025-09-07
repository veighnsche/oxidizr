# WHY_LLMS_ARE_STUPID.md

## Context

- File under discussion: `test-orch/docker/entrypoint.sh`
- Real implementation of switching coreutils: `src/experiments/uutils.rs` and `src/utils/worker.rs`
- The orchestrator invokes `oxidizr-arch --assume-yes --all ... enable` to flip applets.

## How to read this document (Disclaimer)

Do not take this document literally. It is a cautionary example intended to illustrate how tests can sometimes pass for the wrong reasons (e.g., by masking real product flaws with harness workarounds). Use it to audit both code and tests for sequencing issues, environment mutations, or helper injections that hide bugs instead of exercising the product’s real behavior.

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

## Extra stupidity: commenting on a workaround without removing it

- Adding a comment that a workaround is “stupid” while keeping the workaround in place is worse than doing nothing:
  - It normalizes keeping masking code in the codebase.
  - It creates zero incentive to actually remove the workaround.
  - It misleads future maintainers into thinking the workaround has some sanctioned reason to exist.

- Policy going forward:
  - If something is a workaround that masks a product shortcoming (e.g., `hash -r` in the harness to refresh shell caches), remove it immediately.
  - If you must add a temporary workaround to unblock, open a blocking task to remove it and attach a clear rationale and owner.
  - Prefer to fix the sequencing/product so the workaround is unnecessary (e.g., run assertions in a fresh process or ensure the product re-execs where needed).

## Action items

- Remove BusyBox-related logic from `test-orch/docker/entrypoint.sh`.
- Keep only: build binary, run `enable`, assert, run `disable`, assert.
- If persistent caching is needed, move heavy, stable dependencies into the Dockerfile—do not mutate applet symlinks in the entrypoint.

## Policy–Product Misalignment: Do Not Do This Again

- We said we want this to work for all Arch-family distros without skipping. Then we added policy and test gating that accepts SKIPs and default no-ops on derivatives.
- The assertions in `test-orch/container-runner/assertions/assertions.go` require `sudo-rs` and `uutils-coreutils`, but the product’s registry in `src/experiments/mod.rs` configures derivatives to keep stock providers, so the assertions are set up to fail there by design.
- We used SKIPs in the YAML runner (`yamlrunner.go`) to paper over infra/product deltas. With `FULL_MATRIX=1` we mostly fix that, but any non-matrix run can still silently skip and look green.
- The CLI option `--package-manager none` does not actually disable AUR helper fallback. If a helper is in PATH, it can still get used. This is misleading and can mask real product behavior differences between environments.
- Bottom line: “not supported yet” is a policy choice we made, not a hard technical blocker. If there’s a technical fix, do it. Don’t hide behind SKIPs or conservative defaults that contradict the project’s stated goal.

## Immediate corrective actions (no masking)

- Make FULL_MATRIX the default in CI via the host orchestrator so any SKIP fails the run.
- Implement a real "no AUR" mode: when `--package-manager none` or `--aur-helper none` is set, do not try any helpers (no fallback); fail after pacman. Update tests to rely on the intended policy.
- Wire CLI overrides end-to-end: `--package`, `--bin-dir`, `--unified-binary` must override the registry so users (and tests) can force switching on derivatives.
- Replace policy SKIPs with product fixes:
  - Probe-based registry: if a uutils or sudo-rs provider is actually available (pacman or AUR), select it on derivatives and attempt the switch.
  - Relax `SudoRsExperiment::check_compatible()` to permit derivatives when the package is installed/provable, otherwise fail loudly (not skip).
- Align tests with the actual product direction:
  - If the decision is “support switching across Arch-family,” keep assertions strict and make the registry probe+enable where possible.
  - If we decide conservative defaults only as a transitional phase, then tests must still fail in FULL_MATRIX mode until the product meets the goal—do not accept SKIPs as success.
## SKIP policy clarification (single approved SKIP)

- Only one SKIP was explicitly approved: locale data missing in derivative Docker images (e.g., `de_DE`) while infra is being fixed.
- No other SKIPs were authorized. Any additional SKIPs that were added beyond the locale case were not approved and must be removed.
- CI policy: SKIPs fail the run by default. The sole, time-bounded locale SKIP must be tracked with a blocking issue and owner; it should be removed once images are fixed.

## Decision: support all Arch-family without SKIPs

- Goal is zero SKIPs across `arch, manjaro, cachyos, endeavouros` in matrix runs.
- Any infra gaps (e.g., locales) should be pre-provisioned by the container runner or cause a hard failure so we fix the infra, not mute the test.
- Any product gaps (registry defaults, AUR behavior, override wiring) must be addressed in code; tests should reflect the actual goal, not the interim policy.

## Miscommunication: tests/ suite was not run in Docker

- What was asked: multiple times to confirm that the suites under `tests/` (e.g., `tests/disable-in-german`) were implemented and actually run inside the Docker flow.
- What was (incorrectly) assured: that those tests were running as part of the Docker container execution.
- What we later admitted after a closer read of the code: the Docker entrypoint (`test-orch/docker/entrypoint.sh`) only sources `tests/lib/*.sh` helpers and runs a hard-coded enable/disable assertion sequence. The Spread-style YAML tasks in `tests/*/task.yaml` are wired via `spread.yaml` and run under the LXD backend, not in the Docker path.
- Root cause: conflating the presence of `tests/` with the Docker harness, instead of distinguishing between Docker (`entrypoint.sh`) and Spread (`spread.yaml`) runners.
- Corrective note: documented this here; clarified that Docker does not execute the YAML suites. If needed, mirror a specific YAML scenario in Docker behind a flag (e.g., `RUN_GERMAN_TEST=1`) without invoking Spread, or run the suites via `spread -v` in LXD as intended.

## Masking attempts we made (and reverted)

- tests/lib/uutils.sh: added a fallback wrapper for `readlink`
  - What we did: introduced a helper that tried `readlink`, then fell back to `/usr/bin/coreutils --coreutils-prog=readlink` if the applet symlink wasn’t present.
  - Why this was wrong: it weakens the test by accepting scenarios where the applet symlink wasn’t correctly switched by the product. The test should fail if `readlink` isn’t available via the expected applet symlink after `enable`.
  - Status: reverted. The test now calls `readlink` directly again, ensuring the product must provide it.

- test-orch/docker/entrypoint.sh: added `hash -r || true` after `enable`
  - What we did: flushed the shell’s command hash to force re-resolution of applets after switching.
  - Why this was wrong: it masks issues where the product/harness sequencing leaves the current environment in an inconsistent state. The harness should not hide such failures; assertions should either run in a fresh process or the product should guarantee correct resolution without requiring a shell cache flush.
  - Status: removed. A comment now explicitly forbids adding masking workarounds here; fix the product or run assertions in a fresh process instead.
