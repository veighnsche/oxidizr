# German Locale Test Failures (disable-in-german)

## Summary

The `tests/disable-in-german/task.yaml` suite intermittently or consistently fails across the Arch-family matrix (Arch, Manjaro, CachyOS, EndeavourOS). The failures are typically in the final assertion that expects `sudo` to point to the `sudo-rs` alias (`/usr/bin/sudo.sudo-rs`).

## What the test currently does

- Attempts to enable the German locale `de_DE.UTF-8` on the fly.
- Runs `oxidizr-arch enable --yes` (default experiments), then disables only the `coreutils` experiment.
- Restores the English locale for the assertion phase.
- Asserts:
  - `ls` does not still point to uutils after disabling coreutils.
  - `sudo` still points to `/usr/bin/sudo.sudo-rs` (i.e., `sudo-rs` remains enabled).

See: `tests/disable-in-german/task.yaml`.

## Root causes

- __Distro compatibility for sudo-rs__: The product’s `sudo-rs` experiment (`src/experiments/sudors.rs`) explicitly gates enablement to vanilla Arch (`check_compatible()` returns true only for Arch). On Manjaro, CachyOS, and EndeavourOS, `oxidizr-arch enable --yes` does not enable `sudo-rs` by design. The test, however, expects `sudo` to point to the sudo-rs alias on all distros, which contradicts the product contract.
- __Missing strict error handling in the test__: The YAML script does not start with `set -euo pipefail`, so if `oxidizr-arch enable --yes` fails (or simply does not enable `sudo-rs` on non-Arch), the script continues. The later assertion then fails when it requires `sudo` to point to `sudo-rs` on unsupported distros.
- __Locale provisioning variability__: While the Dockerfile installs `glibc-locales`, the test dynamically enables `de_DE.UTF-8` by editing `/etc/locale.gen` and running `locale-gen`. This generally works, but if locale generation fails (mirrors, package state), the test currently falls back to `C.UTF-8` and continues. Under strict full-matrix enforcement (no skips), this fallback should be treated as an infra failure, not silently papered over.

## Why failures happen “for everybody”

- On Arch derivatives, `sudo-rs` is not enabled by the product due to compatibility gating. The test still expects it to be enabled. With no `set -euo pipefail`, enable failures are not caught, and the final assertion inevitably fails.
- On Arch itself, locale generation is typically fine and `sudo-rs` is enabled; failures are less likely. But environment flakiness (locales/mirrors) can still cause sporadic issues.

## Proposed solution

- __Make the test strict and explicit__:
  - Add `set -euo pipefail` to the YAML script so any failure aborts immediately.
  - Treat locale generation failure as a hard error in matrix mode.
- __Align assertions with product compatibility__:
  - If `sudo-rs` is installed (`pacman -Qi sudo-rs`), assert that `/usr/bin/sudo` points to `/usr/bin/sudo.sudo-rs`.
  - If `sudo-rs` is not installed (typical on derivatives), assert that `/usr/bin/sudo` does NOT point to the sudo-rs alias. This keeps the test meaningful without skipping, while honoring product behavior.
- __Keep locale provisioning in the test__:
  - Continue enabling `de_DE.UTF-8` via `/etc/locale.gen` + `locale-gen`. The Docker image already contains `glibc-locales`.

## Implementation details

- File to change: `tests/disable-in-german/task.yaml`
  - Add `set -euo pipefail` at the top of the execute block.
  - Keep the locale generation step; in full-matrix mode, any failure will abort.
  - Replace the unconditional sudo-rs assertion with a conditional branch that checks package presence before asserting the symlink target.

## Expected outcome

- On Arch: German locale is generated, default experiments enable `sudo-rs`, coreutils is disabled; assertions pass.
- On derivatives: German locale is generated, `sudo-rs` remains absent by design, `sudo` is not linked to `sudo-rs`; assertions pass without skipping the suite.

## Recent Analysis (September 2025)

### Current Test Behavior

The test has been properly implemented with the suggested improvements from the original analysis:

1. **✅ Strict error handling**: The test uses `set -euo pipefail` and treats locale generation failures as hard errors in FULL_MATRIX mode
2. **✅ Conditional sudo-rs assertions**: The test properly checks if `sudo-rs` is installed before asserting symlink targets
3. **✅ Proper locale generation**: The test correctly modifies `/etc/locale.gen` and runs `locale-gen`

### Actual Root Cause: Docker Container Locale Data Missing

From the test output analysis, the real issue is that **locale definition files are missing in the Docker containers** for some Arch derivatives, specifically CachyOS:

```
[cachyos] [error] cannot open locale definition file `de_DE': No such file or directory
[cachyos] de_DE.UTF-8 locale not available in FULL_MATRIX mode
```

This indicates that while `glibc-locales` is installed in the Dockerfile, the actual locale definition files (`/usr/share/i18n/locales/de_DE`) are either:
1. Not present in the derivative's Docker base images
2. Stripped from their glibc-locales packages  
3. Requiring additional packages/steps for full locale support

### Docker Base Image Differences

Research shows that:
- **Vanilla Arch Docker images** typically include full locale definition files with `glibc-locales`
- **Derivative Docker images** (CachyOS, Manjaro, EndeavourOS) may have more stripped-down base images
- **Container environments** often exclude locale data to reduce image size

The test is correctly detecting this infrastructure limitation and failing appropriately in FULL_MATRIX mode.

### Distribution-Specific Behavior

1. **Arch**: Full locale support, sudo-rs compatibility - tests pass
2. **CachyOS**: Missing locale definition files, no sudo-rs - locale failure causes test abort  
3. **Manjaro/EndeavourOS**: Likely similar locale issues, different package availability

### Recommended Solution

The issue is **infrastructure-level**, not code-level. To fix:

1. **Dockerfile enhancement**: Add explicit locale data installation for derivatives
2. **Alternative**: Pre-generate common locales in the Docker build process  
3. **Test improvement**: Add locale availability check before attempting generation

The current test behavior (failing fast on missing locale infrastructure) is actually **correct** and prevents silent test degradation.

## References

- Test: `tests/disable-in-german/task.yaml`
- Product implementation: `src/experiments/sudors.rs`
- Docker image provisioning: `test-orch/docker/Dockerfile`
- Related: [Arch Linux Docker locale issues](https://bbs.archlinux.org/viewtopic.php?id=253198)
- Related: [CachyOS locale forum discussions](https://discuss.cachyos.org/t/i-messed-up-my-locale/4361)
