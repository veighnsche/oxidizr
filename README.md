# oxidizr-arch (Arch + AUR)

A Rust tool for safely switching Arch Linux core utilities (e.g., coreutils) to Rust-based replacements (e.g., uutils) using Pacman/AUR, inspired by oxidizr. Operations are performed through a `Worker` abstraction with safety guards; non-dry-run mode performs real system changes.
Modernized, distribution-aware CLI for safely switching Arch Linux core utilities to vetted Rust-based replacements, with Docker-backed, matrix-tested orchestration. This tool performs real system operations behind a well-defined `Worker` interface with safety guards (atomic backups, audit logging, lock handling, and dry-run mode).

## What this project does

The CLI manages “experiments” that switch selected tool families to Rust implementations using a safe symlink strategy and package management. Project policy: no distro gating within the supported Arch-family set (Arch, Manjaro, CachyOS, EndeavourOS). Outside this set, experiments may be treated as incompatible unless you explicitly override checks.

- coreutils → uutils-coreutils
- findutils → uutils-findutils (or `uutils-findutils-bin`)
- sudo → sudo-rs

Experiments are selected from a registry in `src/experiments/mod.rs`. We do not gate by distro ID within the supported set; the practical guardrail is whether providers can be installed and their binaries discovered. Distros outside the supported set may be incompatible unless you use `--no-compatibility-check`.

## Highlights

- Distribution-aware experiment registry (`src/experiments/mod.rs`)
- Real, safe switching with atomic backups and idempotent restores
  - `replace_file_with_symlink()` and `restore_file()` in `src/utils/worker/system.rs`
  - Backups live next to targets as `.<name>.oxidizr.bak`
- Package management via pacman; optional AUR helper fallback (paru/yay/trizen/pamac)
  - Auto-detect helper or override with `--package-manager`/`--aur-helper`
  - AUR helpers are executed as an unprivileged `builder` user inside the test container
- Safety and ops ergonomics
  - `--dry-run` prints intended actions without changing the system
  - `--wait_lock=<secs>` waits for the pacman DB lock (`/var/lib/pacman/db.lck`)
  - Audit logging to `/var/log/oxidizr-arch-audit.log` with fallback to `~/.oxidizr-arch-audit.log` (`src/utils/audit.rs`)
- End-to-end, isolated testing with Docker-backed orchestration (`test-orch/`)
  - Host orchestrator (Go) builds images and runs containers
  - In-container runner (Go) prepares environment and executes YAML suites under `tests/`

## Install

```bash
cargo install --path .
```

This builds and installs the `oxidizr-arch` binary. For non-dry-run operations you must run as root.

## CLI usage

Commands:

```bash
# Check per-experiment compatibility on this system
oxidizr-arch check

# List target paths that would be affected
oxidizr-arch list-targets

# Enable the default experiment set (coreutils + sudo-rs)
sudo oxidizr-arch enable

# Disable and restore from backups
sudo oxidizr-arch disable
```

Common flags (global):

```bash
# Select experiments
--all                                 # operate on all known experiments
--experiments coreutils,findutils      # comma-separated list
--experiment coreutils                 # legacy single selector

# Safety and execution control
--dry-run                              # print, don’t change
--no-update                            # skip pacman -Sy prior to actions
--assume-yes                           # skip confirmations (automation)
--wait_lock 30                         # wait up to 30s for pacman DB lock
--no-compatibility-check               # bypass distro gating (dangerous)

# Packaging / AUR selection
--package-manager paru                 # force helper instead of auto-detect
--aur-helper auto|paru|yay|...         # select helper; an AUR helper is assumed to exist if AUR is required
--package uutils-coreutils             # override package name per experiment
--bin-dir /usr/lib/uutils/coreutils    # override replacement bin directory
--unified-binary /usr/bin/coreutils    # override unified dispatcher path
```

Examples:

```bash
# Dry-run enabling all experiments, waiting for pacman lock
oxidizr-arch --all --dry-run --wait_lock 30 enable

# Enable only coreutils, skip pacman -Sy, and force paru AUR helper
sudo oxidizr-arch --experiments coreutils --no-update --package-manager paru enable

# Disable only coreutils (sudo remains in its current state)
sudo oxidizr-arch --experiments coreutils disable
```

Notes:

- No distro gating within supported set (Arch, Manjaro, CachyOS, EndeavourOS). If a provider can be installed via pacman or an available AUR helper, switching proceeds; otherwise the command fails with a clear error (no SKIPs). Use `--no-compatibility-check` only for debugging or for non-supported distros.
- AUR helpers: we assume an AUR helper (`paru` or `yay`) is available when AUR packages are required. In containers, the runner ensures a helper exists. On user systems, install one before running commands that require AUR. If no helper is available when needed, the command fails with guidance.
- Override flags (`--package`, `--bin-dir`, `--unified-binary`) are parsed by the CLI and are wired end-to-end to override registry defaults across distros.

## Experiments

- coreutils (`UutilsExperiment`)
  - Package `uutils-coreutils`; bin dir `/usr/lib/uutils/coreutils`; unified binary candidates include `/usr/lib/uutils/coreutils/coreutils`, `/usr/lib/cargo/bin/coreutils`, `/usr/bin/coreutils.uutils` (`src/experiments/uutils/constants.rs`).

- findutils (`UutilsExperiment`)
  - Package `uutils-findutils` (or `uutils-findutils-bin`); paths wired in registry (`src/experiments/mod.rs`).

- sudo-rs (`SudoRsExperiment`)
  - Installs `sudo-rs` and wires stable aliases like `/usr/bin/sudo.sudo-rs`, then links `/usr/bin/sudo -> /usr/bin/sudo.sudo-rs` and similarly for `su`. `visudo` target lives in `/usr/sbin/visudo` (`src/experiments/sudors.rs`).

## Enable/Disable behavior matrix (uutils-*)

This table documents what happens to `uutils-*` packages and applet symlinks depending on whether the packages were already present on the system.

| Initial state of `uutils-*` | On `enable` | On `disable` |
|-----------------------------|-------------|--------------|
| Already installed by user   | - Package installation is skipped (detected via `pacman -Qi`)<br>- Backups created next to targets as `.<name>.oxidizr.bak`<br>- Applets under `/usr/bin/<applet>` are switched to the installed provider (split or unified binary) | - Applet targets are restored from backups<br>- `uutils-*` packages are uninstalled (current policy) |
| Not installed               | - Package is installed via pacman; if not in pacman, the available AUR helper is used<br>- Backups created next to targets as `.<name>.oxidizr.bak`<br>- Applets under `/usr/bin/<applet>` are switched to the installed provider | - Applet targets are restored from backups<br>- `uutils-*` packages are uninstalled |

Notes:
- Applies to `uutils-coreutils` and `uutils-findutils` experiments.
- `sudo-rs` follows the same disable removal policy: on `disable`, the `sudo-rs` package is removed after restoring targets.
- Path selection during `enable` can be adjusted with overrides like `--bin-dir` and `--unified-binary`; uninstall behavior is controlled by the experiment and currently removes the provider package on `disable`.

## Safety model

- Atomic backups: when replacing a target, the original is copied to `.<name>.oxidizr.bak` next to it. Permissions are preserved.
- Idempotent switching: re-running enable updates incorrect symlinks in place; disable restores originals where backups exist.
- Path safety: basic traversal checks guard against `..` in paths (`src/utils/worker/helpers.rs::is_safe_path`).
- Audit trail: every symlink/restore is logged (`src/utils/audit.rs`).

## Architecture overview

- CLI and entrypoints
  - `src/main.rs` wires logging and delegates to `cli::handle_cli()`
  - `src/cli.rs` defines flags/subcommands and drives experiments via a `Worker` implementation
- Experiments
  - Registry and routing: `src/experiments/mod.rs`
  - Uutils experiment: `src/experiments/uutils/*` (constants, targets, enable/disable logic)
  - sudo-rs experiment: `src/experiments/sudors.rs`
- Worker abstraction and system implementation
  - Trait: `src/utils/worker/traits.rs`
  - System (Arch-centric) with pacman/AUR + symlink ops: `src/utils/worker/system.rs`
  - Helpers and constants: `src/utils/worker/helpers.rs`, `src/config.rs`
- Common utilities
  - Audit logging: `src/utils/audit.rs`
  - Commands/helpers: `src/utils/command.rs`

## Testing

1) Rust unit tests

```bash
cargo test
```

2) Orchestrated, isolated tests (Docker)

The `test-orch/` directory contains two Go programs:

- Host orchestrator: `test-orch/host-orchestrator/` builds the Docker image and runs containers
- Container runner: `test-orch/container-runner/` executes setup, YAML suites under `tests/`, and assertions

Quick start:

```bash
# From repo root
make test-orch

# Or directly
cd test-orch/host-orchestrator
sudo go run . --arch-build --run                 # build image + run tests
sudo go run . --distros=arch                     # restrict to a single distro
sudo go run . --test-filter=disable-all          # run one YAML suite
sudo go run . -v                                 # verbose; -vv for trace
```

Environment toggles (propagated into the container):

- `VERBOSE` controls runner verbosity (0–3)

Fail-on-skip is the default behavior of the test runner; there is no special flag or environment variable to enable it.

See `test-orch/host-orchestrator/README.md` and `test-orch/container-runner/README.md` for full options and details.

## Known notes

- Root is required for non-dry-run `enable`/`disable` since targets live under `/usr/bin` and `/usr/sbin`.
- No exceptions: all test suites run across all supported Arch-family distributions (Arch, Manjaro, CachyOS, EndeavourOS). Any SKIP indicates an issue to fix in infra or product.

## License

This project is licensed under the GNU General Public License, version 3 or (at your option) any later version.
 
Copyright (C) 2025 veighnsche
 
See the `LICENSE` file for the full text of the GPL-3.0-or-later license.
