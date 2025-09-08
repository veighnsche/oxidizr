# German Locale Test Failures (disable-in-german)

## Summary

The `tests/disable-in-german/task.yaml` suite exhibits nondeterministic failures when the matrix runs distros in parallel across the Arch-family (Arch, Manjaro, CachyOS, EndeavourOS). Importantly, it passes reliably in isolation or when serialized. The failures often show up in the final phase asserting the `sudo` target and/or post-disable state.

Correction (September 2025): Previous documentation attributed failures on derivatives to missing `de_DE` locale definition files. That attribution was incomplete and led to an incorrect SKIP rationale. The operative reason for the single allowed SKIP is parallel-run flakiness, which can affect Arch as well when executed concurrently with other distros.

## What the test currently does

- Attempts to enable the German locale `de_DE.UTF-8` on the fly.
- Runs `oxidizr-arch enable --yes` (default experiments), then disables only the `coreutils` experiment.
- Restores the English locale for the assertion phase.
- Asserts:
  - `ls` does not still point to uutils after disabling coreutils.
  - `sudo` still points to `/usr/bin/sudo.sudo-rs` (i.e., `sudo-rs` remains enabled).

See: `tests/disable-in-german/task.yaml`.

## Root causes

- __Parallel-run nondeterminism__: When all distros are executed concurrently, cross-container timing and resource contention produce flakes that this suite is sensitive to. In isolation/serialized runs, the same steps are consistently green.
- __Distro compatibility for sudo-rs (product context)__: `sudo-rs` enablement used to be Arch-only by default. The test was updated to assert conditionally based on package presence to stay aligned with product behavior.
- __Locale provisioning variability (non-root cause of SKIP)__: Locale differences exist across images and remain probed for visibility. They are not the operative reason for the SKIP policy.

## Why failures happen “for everybody”

- On Arch derivatives, `sudo-rs` is not enabled by the product due to compatibility gating. The test still expects it to be enabled. With no `set -euo pipefail`, enable failures are not caught, and the final assertion inevitably fails.
- On Arch itself, locale generation is typically fine and `sudo-rs` is enabled; failures are less likely. But environment flakiness (locales/mirrors) can still cause sporadic issues.

## Proposed solution

- __Deflake or serialize__:
  - Short-term: run `tests/disable-in-german` serialized or in isolation when executing the full matrix in parallel.
  - Medium-term: identify and remove sources of nondeterminism in this suite (cross-container contention, ordering assumptions, environment coupling) and then remove the SKIP exception.
- __Keep the test strict and explicit__:
  - Retain `set -euo pipefail` so failures abort immediately.
- __Align assertions with actual package state__:
  - If `sudo-rs` is installed (`pacman -Qi sudo-rs`), assert `/usr/bin/sudo -> /usr/bin/sudo.sudo-rs`.
  - If not installed, assert that `/usr/bin/sudo` is not linked to the sudo-rs alias.
- __Locale handling__:
  - Keep locale checks as visibility probes; do not attribute SKIPs to locale presence/absence.

## Implementation details

- File to change: `tests/disable-in-german/task.yaml`
  - Add `set -euo pipefail` at the top of the execute block.
  - Keep the locale generation step; in full-matrix mode, any failure will abort.
  - Replace the unconditional sudo-rs assertion with a conditional branch that checks package presence before asserting the symlink target.

## Expected outcome

- On Arch and derivatives: The suite passes when run in isolation/serialized. In parallel matrix runs, a single SKIP is allowed for this suite until deflaked.

## Recent Analysis (September 2025) — Correction and historical context

### Current Test Behavior

The test has the intended strictness and conditional assertions:

1. **✅ Strict error handling**: `set -euo pipefail`
2. **✅ Conditional sudo-rs assertions**: Check package presence before asserting symlink targets
3. **✅ Locale probes**: Modify `/etc/locale.gen` and call `locale-gen` when appropriate, but locale availability is not the SKIP rationale

### Correction: Parallel-run flakiness is the operative SKIP reason

- In parallel, cross-distro runs, this suite intermittently fails across the family, including Arch.
- In isolation/serialized runs, it passes reliably.

### Historical hypothesis (kept for reference): Missing locale data in some images

Logs have shown errors like:

```
[cachyos] [error] cannot open locale definition file `de_DE': No such file or directory
[cachyos] de_DE.UTF-8 locale not available in FULL_MATRIX mode
```

Locale differences remain real across images and are still worth probing for visibility; however, they are not the reason for the SKIP policy. The allowed SKIP exists solely due to parallel-run nondeterminism and will be removed once the suite is deflaked or serialized.

## References

- Test: `tests/disable-in-german/task.yaml`
- Product implementation: `src/experiments/sudors.rs`
- Docker image provisioning: `test-orch/docker/Dockerfile`
- Related: [Arch Linux Docker locale issues](https://bbs.archlinux.org/viewtopic.php?id=253198)
- Related: [CachyOS locale forum discussions](https://discuss.cachyos.org/t/i-messed-up-my-locale/4361)
