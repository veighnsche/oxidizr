# oxidizr-arch (Arch + AUR)

A Rust tool for safely switching Arch Linux core utilities (e.g., coreutils) to Rust-based replacements (e.g., uutils) using Pacman/AUR, inspired by oxidizr. Operations are performed through a `Worker` abstraction with safety guards; non-dry-run mode performs real system changes.
Modernized, distribution-aware CLI for safely switching Arch Linux core utilities to vetted Rust-based replacements, with Docker-backed, matrix-tested orchestration. This tool performs real system operations behind a well-defined `Worker` interface with safety guards (atomic backups, audit logging, lock handling, and dry-run mode).

## What this project does

The CLI manages “experiments” that switch selected tool families to Rust implementations using a safe symlink strategy and package management on Arch and Arch derivatives:

- coreutils → uutils-coreutils (on vanilla Arch) or keep distro defaults (on derivatives)
- findutils → uutils-findutils (when available on Arch) or distro defaults
- sudo → sudo-rs (on vanilla Arch only)

Experiments are distribution-aware and selected from a registry in `src/experiments/mod.rs`. Compatibility is checked per experiment; incompatible ones are skipped unless you explicitly bypass checks.

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

# Enable the default experiment set (coreutils + sudo-rs on Arch)
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
--aur-helper auto|none|paru|yay|...    # value-enum alternative
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

- On Arch derivatives, the `sudo-rs` experiment is considered incompatible and is skipped unless `--no-compatibility-check` is provided. The defaults still ensure the system remains usable on derivatives (e.g., stock `sudo`).
- AUR fallback behavior: after attempting `pacman -S`, any available AUR helpers found in `PATH` (e.g., `paru`, `yay`) are tried in order. The CLI currently doesn’t provide a strict “disable AUR” mode; to avoid AUR usage, ensure no helper is installed/available in `PATH` and rely solely on pacman. You can influence helper choice/order via `--package-manager` or `--aur-helper`.
- Override flags (`--package`, `--bin-dir`, `--unified-binary`) are parsed by the CLI but not yet plumbed into all experiments; current defaults come from the registry in `src/experiments/mod.rs`. Expect overrides to be ignored until wiring is completed.

## Experiments

- coreutils (`UutilsExperiment`)
  - Arch: package `uutils-coreutils`; bin dir `/usr/lib/uutils/coreutils`; unified binary candidates include `/usr/lib/uutils/coreutils/coreutils`, `/usr/lib/cargo/bin/coreutils`, `/usr/bin/coreutils.uutils` (`src/experiments/uutils/constants.rs`).
  - Derivatives: keep distro defaults (`/usr/bin`).

- findutils (`UutilsExperiment`)
  - Arch: package `uutils-findutils` (or `uutils-findutils-bin`); paths wired in registry (`src/experiments/mod.rs`).
  - Derivatives: keep distro defaults.

- sudo-rs (`SudoRsExperiment`)
  - Arch only: installs `sudo-rs` and wires stable aliases like `/usr/bin/sudo.sudo-rs`, then links `/usr/bin/sudo -> /usr/bin/sudo.sudo-rs` and similarly for `su`. `visudo` target lives in `/usr/sbin/visudo` (`src/experiments/sudors.rs`).
  - Derivatives: experiment is incompatible → skipped by default.

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
sudo go run . --test-filter=disable-in-german    # run one YAML suite
sudo go run . -v                                 # verbose; -vv for trace
```

Environment toggles (propagated into the container):

- `VERBOSE` controls runner verbosity (0–3)
- `FULL_MATRIX=1` enforces fail-fast on skipped suites (locale/distro misconfiguration shows up as failure rather than silent skip)

See `test-orch/host-orchestrator/README.md` and `test-orch/container-runner/README.md` for full options and details.

## Known notes

- Root is required for non-dry-run `enable`/`disable` since targets live under `/usr/bin` and `/usr/sbin`.
- On Arch derivatives, locale data may be stripped in minimal Docker images. Locale-dependent suites (e.g., `tests/disable-in-german/`) are designed to fail fast in full-matrix mode if locales are missing. See `FULL_MATRIX_TESTING_PLAN.md`.

## License

This project is licensed under the GNU General Public License, version 3 or (at your option) any later version.
 
Copyright (C) 2025 veighnsche
 
See the `LICENSE` file for the full text of the GPL-3.0-or-later license.
