# German Locale Test Failures (disable-in-german)

## Summary

The `tests/disable-in-german/task.yaml` suite must run green across the Arch-family (Arch, Manjaro, CachyOS, EndeavourOS). Failures caused by missing `de_DE` locale definitions are infrastructure issues. Policy is to bake `de_DE.UTF-8` into the Docker images at build time and treat any missing-locale errors as hard failures to be fixed in the image.

## What the test currently does

- Attempts to enable the German locale `de_DE.UTF-8` on the fly.
- Runs `oxidizr-arch enable --yes` (default experiments), then disables only the `coreutils` experiment.
- Restores the English locale for the assertion phase.
- Asserts:
  - `ls` does not still point to uutils after disabling coreutils.
  - `sudo` still points to `/usr/bin/sudo.sudo-rs` (i.e., `sudo-rs` remains enabled).

See: `tests/disable-in-german/task.yaml`.

## Root causes (historical)

- Locale provisioning variability in minimal images caused `locale-gen` to fail when `de_DE` definitions were absent.
- Some parallel-run nondeterminism made the suite sensitive to timing in cross-distro runs.
- `sudo-rs` enablement policy used to vary; assertions have since been aligned with package presence.

## Why failures happen “for everybody”

- On Arch derivatives, `sudo-rs` is not enabled by the product due to compatibility gating. The test still expects it to be enabled. With no `set -euo pipefail`, enable failures are not caught, and the final assertion inevitably fails.
- On Arch itself, locale generation is typically fine and `sudo-rs` is enabled; failures are less likely. But environment flakiness (locales/mirrors) can still cause sporadic issues.

## Solution (current policy)

- Bake `de_DE.UTF-8` into Docker images at build time (see `test-orch/docker/Dockerfile`).
- Retain `set -euo pipefail` so failures abort immediately.
- Align assertions with actual package state (e.g., `sudo-rs` present -> `sudo` points to `sudo-rs`).
- No SKIPs are permitted for this suite. Missing locales must be fixed in image builds.

## Implementation details

- File to change: `tests/disable-in-german/task.yaml`
  - Add `set -euo pipefail` at the top of the execute block.
  - Keep the locale generation step; in full-matrix mode, any failure will abort.
  - Replace the unconditional sudo-rs assertion with a conditional branch that checks package presence before asserting the symlink target.

## Expected outcome

- On Arch and derivatives: The suite passes in isolation and in parallel matrix runs. Locale errors are not tolerated and must be fixed by image provisioning.

## Recent Analysis (September 2025) — Correction and historical context

### Current Test Behavior

The test has the intended strictness and conditional assertions:

1. **✅ Strict error handling**: `set -euo pipefail`
2. **✅ Conditional sudo-rs assertions**: Check package presence before asserting symlink targets
3. **✅ Locale probes**: Modify `/etc/locale.gen` and call `locale-gen` when appropriate, but locale availability is not the SKIP rationale

### Note on parallel runs

- Historical flakes under parallel execution should be deflaked. Do not use SKIPs to mask them.

### Historical failure mode: Missing locale data

Logs have shown errors like:

```
[cachyos] [error] cannot open locale definition file `de_DE': No such file or directory
[cachyos] de_DE.UTF-8 locale not available in FULL_MATRIX mode
```

These are image-provisioning bugs. Our Docker builds now generate the locale during image creation to prevent this.

## References

- Test: `tests/disable-in-german/task.yaml`
- Product implementation: `src/experiments/sudors.rs`
- Docker image provisioning: `test-orch/docker/Dockerfile`
- Related: [Arch Linux Docker locale issues](https://bbs.archlinux.org/viewtopic.php?id=253198)
- Related: [CachyOS locale forum discussions](https://discuss.cachyos.org/t/i-messed-up-my-locale/4361)
