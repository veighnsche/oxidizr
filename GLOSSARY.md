# Project Glossary (oxidizr-arch)

Authoritative dictionary of terminology used across code (`src/`), orchestration (`test-orch/`), tests (`tests/`, `test-units/`), CI, and docs. Use this as the single reference when reading code, writing tests, or making policy decisions.

## Core Product Concepts

- __Experiment__
  - A switchable unit of functionality the CLI can enable/disable (e.g., `coreutils`, `findutils`, `sudo-rs`).
  - Registry: `src/experiments/mod.rs` decides defaults and compatibility per distro.

- __Enable / Disable__
  - `enable`: Install/choose provider package and atomically swap applet symlinks to the selected provider.
  - `disable`: Restore original files from backups and remove/undo symlinks.
  - Core FS ops implemented in `src/utils/worker/system.rs` via `replace_file_with_symlink()` and `restore_file()`.

- __Applet Symlink__
  - The symlink under `/usr/bin` (or target dir) that points to the chosen provider’s binary (GNU or uutils). E.g., `/usr/bin/ls -> /usr/lib/uutils/coreutils/ls` or unified dispatcher.

- __Backup Artifact__
  - When replacing a target, the original is copied next to it as `.<name>.oxidizr.bak` with permissions preserved.

- __Unified Binary__
  - A single coreutils dispatcher (e.g., `/usr/lib/uutils/coreutils/coreutils`) that serves multiple applets via argv[0]. Overridable via `--unified-binary`.

- __Bin Dir__
  - Directory where the replacement provider’s binaries reside (e.g., `/usr/lib/uutils/coreutils`). Overridable via `--bin-dir`.

- __Worker__
  - Abstraction implementing filesystem and package operations (prod: `System` in `src/utils/worker/system.rs`; tests: mocks in `test-units/`).

## Packaging & AUR

- __AUR Helper__
  - Userland helper such as `paru`/`yay` invoked to build/install AUR packages when `pacman` doesn’t provide them.

- __Helper Assumption__
  - We assume an AUR helper (`paru` or `yay`) is available when AUR packages are needed. In containers, the runner ensures a helper exists. On user systems, users must install one. If no helper is present when required, commands fail loudly with guidance. There is no "no AUR" mode.

- __Overrides (CLI)__
  - `--package`, `--bin-dir`, `--unified-binary` force provider selection and paths regardless of registry defaults.
  - Status: Parsing present; wire-through to experiments may be incomplete; policy requires full end-to-end effect.

- __Probe-Based Registry__
  - For Arch-family distros, always attempt to install via pacman; when only available via AUR, use the available helper. Probing is used to improve messages and selection decisions, not to gate or skip tests. If installation fails, commands fail loudly (no SKIPs).

## Experiments (Current)

- __UutilsExperiment (coreutils/findutils)__
  - Arch: aims to switch `/usr/bin/*` applets to uutils provider; uses unified binary when available.
  - Derivatives: defaults to stock providers unless overridden or probed available.

- __SudoRsExperiment__
  - Arch-only by default. Can be relaxed: if `sudo-rs` is provably installed, allow on derivatives; else error (no SKIP).

## Orchestration & Runners

- __Host Orchestrator__ (`test-orch/host-orchestrator/`)
  - Go program that builds images, runs containers, mounts caches, propagates env (`FULL_MATRIX`, `VERBOSE`), and aggregates results.
  - Key functions: `dockerutil.BuildArchImage()`, `dockerutil.RunArchContainer()`.

- __Container Runner__ (`test-orch/container-runner/`)
  - Go program executed as container entrypoint (`/usr/local/bin/isolated-runner`). Handles setup, YAML execution, and assertions.
  - Stages repo, installs deps, manages users, detects/installs AUR helper, sets `rustup`, builds binary, runs tests.

- __Dockerfile__ (`test-orch/docker/Dockerfile`)
  - Multi-stage: `golang:1.21` builder for runner, final stage `FROM ${BASE_IMAGE}` (per distro). Installs minimal tools and `glibc-locales`; sets `LANG=C.UTF-8`.

- __Base Images__ (mapped in `host-orchestrator/main.go`)
  - `archlinux:base-devel`, `manjarolinux/base`, `cachyos/cachyos:latest`, `alex5402/endeavouros`.
  - Built tags: `oxidizr-<distro>:latest`.

## Caching & Mounts

- __Persistent Caches__ (created per distro in host orchestrator)
  - Cargo registry/git → `/root/.cargo/{registry,git}`
  - Cargo target per-distro → `/workspace/target`
  - Pacman cache → `/var/cache/pacman`
  - AUR build dir per-distro → `/home/builder/build`
  - Rationale: reduce rebuild time; AUR handling is idempotent (clone || true; `git pull --rebase`).

## Test Suites & Assertions

- __YAML Suites__ (`tests/*/task.yaml`)
  - Executed inside the container by the runner (`yamlrunner`). Support `distro-check` semantics via runner logic.

- __Assertions__ (`test-orch/container-runner/assertions/`)
  - Go checks for state after enable/disable (e.g., applet symlinks count/targets, sudo-rs target).
  - Artifacts (planned): `artifacts/{yaml-results.json, assertions.json, host-summary.json}`.

- __Coreutils Coverage Threshold__
  - Policy to fail when too few applets are symlinked (e.g., minimum critical set: `date, ls, readlink, stat, rm, cp, ln, mv, cat, echo`).

## CI, Matrix, and Policy

- __FULL_MATRIX__
  - Environment/flag causing the runner/host orchestrator to treat SKIPs as failures and enforce strict coverage.
  - Default: propagated by host orchestrator into containers.

- __Allowed SKIPs Table__ (`TESTING_POLICY.md`)
  - Single permitted SKIP: `tests/disable-in-german` on `cachyos, manjaro, endeavouros` due to missing locale definition files.
  - All other suites must not skip. In FULL_MATRIX CI, SKIPs fail the run, except the single permitted one.

- __Masking__
  - Any harness change that hides product bugs (e.g., BusyBox file ops, pre-creating applets, `hash - r`). Forbidden by policy.

- __Distro-Gating__
  - Restricting tests or features to specific OS IDs. Policy: do not gate across the Arch-family to make tests pass. Gate only OSes we do not support or have never tested. Within the Arch-family (`arch, manjaro, cachyos, endeavouros`), run the same assertions and fail loudly when unavailable.

- __Sequencing__
  - The order of operations (build → enable → assert → disable → assert). Mutations must be performed by product code, not the harness.

## Distro Awareness & Locales

- __CurrentDistroID()__
  - Runner utility reading `/etc/os-release` `ID` lowercased (e.g., `arch`, `manjaro`, `cachyos`, `endeavouros`). Used for gating/probing.

- __Locale Definitions__
  - Files under `/usr/share/i18n/locales/` (e.g., `de_DE`), required to generate `de_DE.UTF-8`. Often missing in derivative base images.

- __disable-in-german Suite__
  - Locale-dependent scenario. Only allowed SKIP on derivatives while images are missing `de_DE` definitions. See `GERMAN_LOCALE_TEST_ISSUE.md`.

## CLI Flags (Selected)

- __--all / --experiments / --experiment__
  - Select experiments to operate on.

- __--dry-run__
  - Print intended actions without making changes.

- __--no-update__
  - Skip `pacman -Sy` refresh.

- __--assume-yes__
  - Non-interactive mode.

- __--wait_lock <secs>__
  - Wait for pacman DB lock (`/var/lib/pacman/db.lck`).

- __--no-compatibility-check__
  - Bypass distro gating (dangerous). For test/debug only.

- __--package-manager / --aur-helper__
  - Choose helper or disable all helpers (`none`) for “no AUR mode”.

- __--package / --bin-dir / --unified-binary__
  - Overrides for provider package name and paths. Must override registry behavior end-to-end.

## Users & Permissions (Runner)

- __builder / spread Users__
  - Created by runner for AUR builds and Spread-style tests. Passwordless sudo configured via `/etc/sudoers.d/99-builder`.

## Known Decisions Needed (Owner: you)

- __Enforce “No AUR Mode”__
  - Implement strict enforcement so helpers are never called when `--package-manager none` or `--aur-helper none` is set.

- __Wire CLI Overrides__
  - Ensure `--package`, `--bin-dir`, `--unified-binary` override experiment defaults across distros.

- __Implement Probe-Based Registry__
  - On derivatives, select provider when installed/installable; otherwise error (not SKIP).

- __Relax `sudo-rs` Compatibility When Provable__
  - Permit on derivatives when `sudo-rs` is installed; otherwise error with clear message.

- __Infra: Locales on Derivatives__
  - Decide whether to pre-provision locale definitions (remove last SKIP) or keep the temporary SKIP until images are fixed.

## Anti-Patterns (Forbidden)

- __BusyBox Workarounds__
  - Installing BusyBox to perform `cp/ln/rm` for switching.

- __Applet Pre-Creation / Repair in Harness__
  - Creating `readlink` / other applets before `enable`.

- __Shell Cache Flush Masking__
  - Using `hash -r` to hide sequencing issues.

## File Pointers (Jump Table)

- `src/experiments/mod.rs` — Registry, default behaviors per distro
- `src/experiments/uutils/*` — Uutils experiment constants/targets/enable/disable
- `src/experiments/sudors.rs` — sudo-rs experiment
- `src/utils/worker/system.rs` — FS and package operations
- `src/utils/worker/helpers.rs` — path safety (`is_safe_path`), helpers
- `TESTING_POLICY.md` — SKIP policy and harness rules
- `FULL_MATRIX_TESTING_PLAN.md` — enforcement details and acceptance criteria
- `DISTRO_IMAGES.md` — base images, mounts, known gaps
- `test-orch/host-orchestrator/*` — Docker build/run, caching mounts, FULL_MATRIX propagation
- `test-orch/container-runner/*` — setup, YAML runner, assertions, distro detection
- `test-orch/DISTRO_MATRIX.md` — per-distro environment differences
- `GERMAN_LOCALE_TEST_ISSUE.md` — analysis of missing `de_DE` on derivatives
- `WHY_LLMS_ARE_STUPID.md` — cautionary notes, masking anti-patterns
