# Distro Environment Matrix

This document tracks known environment differences across the four Arch-based distributions used in the test matrix:

- Arch Linux (official)
- CachyOS
- Manjaro
- EndeavourOS

Use this as the source of truth to adapt test preparation and expectations per distro. Where items are TBD, the container-runner will probe at runtime and log results.

## Summary

- Locales: Only Arch reliably contains full locale definitions. Derivatives often strip locale data.
- AUR helpers: CachyOS commonly ships with `paru` preinstalled. Arch usually has none. Manjaro/EndeavourOS status TBD.
- Package manager: All use `pacman`, but repo sets differ (vendor repos).
- Base packages: We install a consistent baseline via `setup/deps.go` inside the container.

## Detailed Differences

### Locales

- Arch Linux
  - Locale definitions present by default (can generate `en_US.UTF-8`, `de_DE.UTF-8`, `C.UTF-8`).
  - `disable-in-german` test: runnable.
- CachyOS
  - Known issue: missing locale definition files (e.g., `/usr/share/i18n/locales/de_DE`) in stripped Docker images.
  - `disable-in-german` test: not runnable; should fail fast in FULL_MATRIX mode.
- Manjaro
  - Likely similar to CachyOS with stripped locale data in minimal images. TBD: verify presence of locale definitions.
  - `disable-in-german` test: currently assumed not runnable; confirm with probe.
- EndeavourOS
  - Likely similar to CachyOS/Manjaro for minimal images. TBD: verify presence of locale definitions.
  - `disable-in-german` test: currently assumed not runnable; confirm with probe.

Runtime behavior:
- See `container-runner/setup/locales.go` for logic that:
  - Ensures `/etc/locale.gen` contains `en_US.UTF-8`, `de_DE.UTF-8`, `C.UTF-8`
  - Attempts `locale-gen`
  - If definitions are missing, attempts best-effort remediation by reinstalling `glibc-locales` and retrying; if still missing, YAML tests must fail fast when `FULL_MATRIX=1`.

### AUR helper preinstallation

- Arch Linux
  - Typically none preinstalled.
  - Runner installs `paru-bin` from AUR if needed.
- CachyOS
  - Commonly ships with `paru` preinstalled.
  - Runner detects and skips installation when found.
- Manjaro
  - TBD: `yay` or other helpers may be present in some images; verify.
- EndeavourOS
  - TBD: `yay` is often present on desktop images; minimal containers may lack it; verify.

Detection details (implemented): see `container-runner/setup/rust.go` and related utilities:
- Detect via `exec.LookPath("paru")`, `which paru`, `paru --version`.
- Clone/update logic for `paru-bin` is idempotent and resilient to persistent cache mounts.

### Package manager and repos

- All distros use `pacman`.
- Vendor repos differ:
  - CachyOS: additional CachyOS repos; cache folder may persist and affect behavior.
  - Manjaro: Manjaro-specific repos.
  - EndeavourOS: Arch repos plus EndeavourOS repo.
- Runner normalizes system deps via `container-runner/setup/deps.go` (installs `base-devel sudo git curl rustup which findutils`).

### Users and sudo

- Not assumed to exist consistently. Runner ensures:
  - Users `builder` and `spread` are present.
  - Passwordless sudo for CI in `/etc/sudoers.d/99-builder`.
  - See `container-runner/setup/users.go`.

## Test Impact and Expectations

- `disable-in-german` YAML suite
  - Run on Arch Linux where `de_DE.UTF-8` can be generated.
  - On derivatives (CachyOS/Manjaro/EndeavourOS), expect skip/fail-fast under FULL_MATRIX due to missing locale definitions.
- AUR-dependent steps
  - Should work across all distros; installation is skipped when a helper (e.g., `paru`) is preinstalled.
- Build and assertions
  - Unified via container-runner steps; not distro-specific except for locale handling and AUR helper presence.

## Probing Plan (automated)

The container-runner will probe and log these per-distro at startup:

- Locale definitions presence
  - Check: `/usr/share/i18n/locales/de_DE` and `/usr/share/i18n/locales/en_US`
  - Attempt `locale-gen` for `de_DE.UTF-8`
- AUR helper presence
  - Check: `paru` via PATH lookup and `paru --version`
  - If not present, install `paru-bin` (idempotent handling of existing clone directory)
- Distro detection
  - Parse `/etc/os-release` and fall back to `uname` if necessary
  - See `container-runner/util/distro.go`

All probe results are logged; add additional probes as needed.

## Action Items / TBD

- Verify AUR helper presence on Manjaro and EndeavourOS minimal images.
- Confirm locale definition availability on Manjaro and EndeavourOS base images used by CI.
- Decide test matrix expectations: mark locale-dependent tests to run only on distros with locales present, or keep FULL_MATRIX fail-fast semantics.
- Optionally enhance Docker build for derivatives to include locale data (trade-off: bigger images vs. faithful minimal environments).
