# Distro Images: Architecture, Sources, and Known Gaps

This document inventories the Docker images used by the test orchestrator, how they are built and configured, and the per-distro differences that affect tests.

## Overview

- Builder image (multi-stage): `golang:1.21` builds the in-container runner binary (`isolated-runner`).
- Final image: parameterized by `BASE_IMAGE` and provisioned with a small, consistent toolset.
- Entrypoint: `/usr/local/bin/isolated-runner` (the container-runner binary).
- Mounts at runtime: the repository, Cargo caches, pacman cache, and AUR build directory (per-distro).

References:
- Dockerfile: `test-orch/docker/Dockerfile`
- Host orchestrator: `test-orch/host-orchestrator/main.go`, `test-orch/host-orchestrator/dockerutil/dockerutil.go`
- Container runner: `test-orch/container-runner/*`
- Distro matrix notes: `test-orch/DISTRO_MATRIX.md`
- Locale issue analysis: `GERMAN_LOCALE_TEST_ISSUE.md`

## Base Images (per distro)

Host orchestrator mapping (see `test-orch/host-orchestrator/main.go`):

- arch → `archlinux:base-devel`
- manjaro → `manjarolinux/base`
- cachyos → `cachyos/cachyos:latest`
- endeavouros → `alex5402/endeavouros`

Resulting built tags: `oxidizr-<distro>:latest` (e.g., `oxidizr-arch:latest`).

## Build and Provisioning (Dockerfile)

`test-orch/docker/Dockerfile` (final stage):
- `FROM ${BASE_IMAGE}`
- `ENV LANG=C.UTF-8`
- Pacman setup:
  - `pacman -Syu --noconfirm`
  - `pacman -S --noconfirm reflector`
  - `reflector --latest 10 --sort rate --save /etc/pacman.d/mirrorlist`
  - `pacman -S --noconfirm --needed sudo git curl rustup which findutils glibc glibc-locales`
  - `pacman -Scc --noconfirm || true` (clean cache)
- DNS hardening: write Google DNS to `/etc/resolv.conf`.
- `WORKDIR /workspace`
- Copies runner binary `isolated-runner` and helper `docker/setup_shell.sh`.
- `ENTRYPOINT ["/usr/local/bin/isolated-runner"]`

Important notes:
- Even with `glibc-locales` installed, some derivative base images lack actual locale definition files (e.g., `/usr/share/i18n/locales/de_DE`). See Known Gaps below.

## Runtime Mounts and Caching

Configured in `test-orch/host-orchestrator/dockerutil/dockerutil.go`:

- Bind mounts per run:
  - Repository root → `/workspace`
  - Cargo registry → `/root/.cargo/registry`
  - Cargo git → `/root/.cargo/git`
  - Cargo target (per-distro) → `/workspace/target`
  - Pacman cache → `/var/cache/pacman`
  - AUR build workspace (per-distro) → `/home/builder/build`
- Cache root on host: `.cache/test-orch/{cargo, pacman, aur-build, cargo-target/<distro>}`
- Purpose: speed up repeated builds and avoid repeated downloads.

## Container Runner Responsibilities (inside the image)

From `test-orch/container-runner/README.md` and setup code:

- Stages the repo to `/root/project/oxidizr-arch`.
- Installs baseline system deps (`base-devel sudo git curl rustup which findutils`).
- Ensures users `builder` and `spread` exist; configures passwordless sudo for CI.
- AUR helper management:
  - Detects preinstalled helper (e.g., `paru`), common on CachyOS.
  - If missing, installs `paru-bin` from AUR. Logic is idempotent and cache-safe (clone-if-missing, `git pull --rebase` if present). See memories and `setup/rust.go`.
- Rust toolchain via rustup for root and builder; sets default to stable.
- Builds `oxidizr-arch` and installs to `/usr/local/bin/oxidizr-arch`.
- Executes YAML suites and assertions.

Distro detection utility: `test-orch/container-runner/util/distro.go` reads `/etc/os-release` (ID) and normalizes it.

## Per-Distro Differences (from `test-orch/DISTRO_MATRIX.md`)

- Locales:
  - Arch Linux: full locale definitions generally present; `de_DE.UTF-8` generation works; locale-dependent tests runnable.
  - CachyOS: missing locale definitions (e.g., `/usr/share/i18n/locales/de_DE`) in minimal images.
  - Manjaro: likely missing locale definitions in minimal images (TBD, probe).
  - EndeavourOS: likely missing locale definitions in minimal images (TBD, probe).
- AUR helpers:
  - Arch: typically none preinstalled; runner may install `paru-bin`.
  - CachyOS: `paru` commonly preinstalled; runner must detect and skip install.
  - Manjaro/EndeavourOS: may have `yay` in desktop images; minimal containers often lack helpers; probe at runtime.
- Package manager: all use `pacman`, but vendor repos differ (CachyOS, Manjaro, EndeavourOS have additional repos).

## Known Gaps and Issues

- Missing locale definitions on derivatives (infra issue):
  - Even with `glibc-locales`, the directory `/usr/share/i18n/locales/de_DE` may be missing.
  - See `GERMAN_LOCALE_TEST_ISSUE.md` for logs and analysis.
  - Effect: locale-dependent tests (e.g., `tests/disable-in-german`) cannot run on some derivatives; in FULL_MATRIX mode this should hard-fail (policy), or images must be enhanced.
- AUR helper cache conflicts with persistent mounts:
  - The AUR clone dir (`/home/builder/build/paru-bin`) persists across runs.
  - Fix implemented: clone is non-fatal, `git pull --rebase` if present, then `makepkg`. Prevents failures when cache already exists.
- Preinstalled helpers:
  - CachyOS often includes `paru` by default; runner must robustly detect via multiple methods (PATH lookup, `which`, `--version`).

## Policy Alignment (what images must guarantee vs. what runner probes)

- Images provide only a minimal base + essential tools. Runner is responsible for the rest.
- For locale-dependent tests across the matrix, either:
  - Enhance derivative images to include locale definitions (preferred if we want zero SKIPs), or
  - Fail fast with a clear infra error on derivatives until images are fixed.
- No masking in the runner or entrypoints: do not pre-create applet symlinks, do not add `hash -r`, do not install BusyBox to workaround switching.

## Commands and Usage

Host orchestrator (root required on most hosts):

```bash
# Build images and run tests across all four distros
sudo go run . --arch-build --run --distros=arch,manjaro,cachyos,endeavouros -v

# Build only (for a specific distro list)
sudo go run . --arch-build --distros=arch,manjaro

# Run using existing images
sudo go run . --run --distros=cachyos,endeavouros
```

Interactive shell helper:

```bash
# Inside a running container (shell mode), prepare binary in PATH
/usr/local/bin/setup_shell.sh
```

## Recommendations

- Pre-provision locales in derivative images (add locale definitions, pre-generate common locales), or make their absence a hard failure with remediation tips.
- Keep AUR helper logic idempotent and detection-first; avoid reinstalling when preinstalled.
- Continue using persistent caches but ensure AUR build directories are per-distro to avoid cross-distro conflicts (already implemented).

## File Pointers

- `test-orch/docker/Dockerfile`
- `test-orch/docker/setup_shell.sh`
- `test-orch/host-orchestrator/main.go`
- `test-orch/host-orchestrator/dockerutil/dockerutil.go`
- `test-orch/container-runner/util/distro.go`
- `test-orch/DISTRO_MATRIX.md`
- `GERMAN_LOCALE_TEST_ISSUE.md`
