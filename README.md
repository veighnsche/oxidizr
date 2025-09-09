# oxidizr-arch (Arch + AUR)

A Rust tool for safely switching Arch Linux core utilities (e.g., coreutils) to Rust-based replacements (e.g., uutils) using Pacman/AUR, inspired by oxidizr. Operations are performed through a `Worker` abstraction with safety guards; non-dry-run mode performs real system changes.
Modernized, distribution-aware CLI for safely switching Arch Linux core utilities to vetted Rust-based replacements, with Docker-backed, matrix-tested orchestration. This tool performs real system operations behind a well-defined `Worker` interface with safety guards (atomic backups, audit logging, lock handling, and dry-run mode).

## What this project does

The CLI manages “experiments” that switch selected tool families to Rust implementations using a safe symlink strategy and package management. Project policy: Arch-family support includes Arch, Manjaro, CachyOS, EndeavourOS. Outside this set, experiments are treated as incompatible unless you explicitly override checks via `--skip-compat-check`.

- coreutils → uutils-coreutils
- findutils → uutils-findutils-bin
- sudo → sudo-rs
- checksums → presence-aware flipping of checksum tools (b2sum, md5sum, sha1..sha512sum)

Experiments are selected from a registry in `src/experiments/mod.rs`. There are no default experiments: you must explicitly select them using `--experiments …` or `--all`.

## Highlights

- Distribution-aware experiment registry (`src/experiments/mod.rs`)
- Real, safe switching with link-aware backups and atomic renames
  - Symlink ops in `src/symlink/ops.rs`
  - Backups live next to targets as `.<name>.oxidizr.bak`
  - If the target was a symlink, the backup is also a symlink; restore recreates the link
- Package management via pacman; optional AUR helper (paru/yay/trizen/pamac)
  - Auto-detect helper or select with `--aur-helper`
  - Use `--aur-user <name>` to run the helper as a specific user; otherwise it runs as the invoking user
- Safety and ops ergonomics
  - `--dry-run` prints intended actions without changing the system
  - `--wait-lock=<secs>` waits for the pacman DB lock (`/var/lib/pacman/db.lck`)
  - `--no-progress` disables progress bars even on TTY
  - `--skip-compat-check` bypasses distro gating (dangerous)
  - Audit logging to `/var/log/oxidizr-arch-audit.log` with fallback to `~/.oxidizr-arch-audit.log` (`src/logging/audit.rs`)
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
oxidizr-arch --experiments coreutils,sudo-rs check

# List target paths that would be affected
oxidizr-arch --all list-targets

# Enable explicitly selected experiments (no defaults)
sudo oxidizr-arch --experiments coreutils,sudo-rs enable

# Disable (restore-only) selected experiments
sudo oxidizr-arch --experiments coreutils disable

# Remove: restore and uninstall provider packages
sudo oxidizr-arch --experiments coreutils remove
```

Common flags (global):

```bash
# Select experiments
--all                                   # operate on all known experiments
--experiments coreutils,findutils       # comma-separated list
--experiment coreutils                  # legacy single selector

# Safety and execution control
--dry-run                               # print, don’t change
--no-update                             # skip pacman -Sy prior to actions
--assume-yes                            # skip confirmations (automation)
--wait-lock 30                          # wait up to 30s for pacman DB lock
--skip-compat-check                     # bypass distro gating (dangerous)
--no-progress                           # disable progress bars even on TTY

# Packaging / AUR selection
--aur-helper auto|paru|yay|trizen|pamac # select helper (auto-detect by default)
--aur-user builder                      # run AUR helper as this user (optional)
--package uutils-coreutils              # override package name per experiment
--bin-dir /usr/lib/uutils/coreutils     # override replacement bin directory
--unified-binary /usr/bin/coreutils     # override unified dispatcher path
```

Examples:

```bash
# Dry-run enabling all experiments, waiting for pacman lock
oxidizr-arch --all --dry-run --wait-lock 30 enable

# Enable only coreutils, skip pacman -Sy, and select paru AUR helper
sudo oxidizr-arch --experiments coreutils --no-update --aur-helper paru enable

# Flip checksum tools explicitly (presence-aware)
sudo oxidizr-arch --experiments checksums enable

# Disable only coreutils (restore-only; sudo remains in its current state)
sudo oxidizr-arch --experiments coreutils disable

# Remove coreutils (restore originals, then uninstall package)
sudo oxidizr-arch --experiments coreutils remove
```

Notes:

- Supported distros: Arch, Manjaro, CachyOS, EndeavourOS. Use `--skip-compat-check` to override on other distros (dangerous).
- AUR helpers: an AUR helper (`paru`, `yay`, `trizen`, or `pamac`) must be available for AUR-only packages. Use `--aur-helper` to select and `--aur-user` to run as a specific user.
- Override flags (`--package`, `--bin-dir`, `--unified-binary`) are parsed by the CLI and are wired end-to-end to override registry defaults across distros.

## Experiments

- coreutils → package `uutils-coreutils` (repo-gated). Discovery uses unified dispatcher if present, or per-applet binaries.
- findutils → package `uutils-findutils-bin` (AUR). Requires an AUR helper.
- sudo-rs → package `sudo-rs` (repo-gated). Uses stable alias symlinks under `/usr/bin/*.sudo-rs`.

## Enable / Disable / Remove behavior

- On `enable`:
  - If needed, `pacman -Sy` (unless `--no-update`), repository gating enforced.
  - Installs required packages (`pacman -S` or AUR for `uutils-findutils-bin`).
  - Discovers provider binaries and flips targets with link-aware, atomic symlink swaps.
  - Coreutils deliberately excludes checksum applets; use the `checksums` experiment for those.

- On `disable` (restore-only):
  - Restores targets from backups. Missing backups cause an error unless `--force-restore-best-effort` is set.
  - Packages are left installed.

- On `remove`:
  - Performs `disable`, then uninstalls provider packages and verifies absence.

## Safety model

- Link-aware backups: if the target was a symlink, the backup is a symlink. Restores recreate the original symlink.
- Atomic swaps: temp symlink + atomic rename with fsync of the parent directory.
- Idempotent switching: re-running enable updates incorrect symlinks in place; disable restores originals where backups exist.
- Path safety: traversal checks guard against `..` in paths (see `src/symlink/ops.rs::is_safe_path`).
- Audit trail: structured JSONL audit events are written to `/var/log/oxidizr-arch-audit.log` with fallback to `$HOME/.oxidizr-arch-audit.log` (`src/logging/audit.rs`).

## Exit codes

- 0  — success
- 1  — general failure
- 10 — incompatible distro (unless skipped with `--skip-compat-check`)
- 20 — nothing to link after ensuring provider (e.g., checksums/findutils discovery)
- 30 — restore backup missing (unless forced best-effort)
- 40 — repository gate failed (missing repo/helper or package absent in repos)

## Architecture overview

- CLI and entrypoints
  - `src/main.rs` wires logging and delegates to `cli::handle_cli()`
  - `src/cli/` defines flags/subcommands and drives experiments via a `Worker` implementation
- Experiments
  - Registry and routing: `src/experiments/mod.rs`
  - Coreutils/findutils/checksums: `src/experiments/*.rs`
  - sudo-rs: `src/experiments/sudors.rs`
- Worker abstraction and system implementation
  - System (Arch-centric) with pacman/AUR + symlink ops: `src/system/worker/*.rs`, `src/symlink/*.rs`
- Common utilities
  - Audit logging: `src/logging/*`

## Testing

1) Rust unit tests

```bash
cargo test
```

1) Orchestrated, isolated tests (Docker)

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

## Security modules: SELinux and AppArmor

Even when Unix permissions and ownership are correct, Mandatory Access Control (MAC) systems like SELinux and AppArmor can block execution of provider binaries or swapped targets. If a command “looks fine” but won’t execute (permission denied, EPERM, or silent failure), check your MAC layer first.

Symptoms:

- `Permission denied` when executing a file that has `+x` and readable perms
- `operation not permitted` on `execve` or while creating/renaming symlinks
- AVC/AppArmor denials in logs

How to diagnose:

- SELinux (if enabled):

  ```bash
  journalctl -t setroubleshoot -t audit --since -2h | grep -i avc || true
  ausearch -m avc -m user_avc -ts recent || true
  dmesg | grep -i avc || true
  ```

  - Inspect file context:

  ```bash
  ls -Z /usr/bin/ls /usr/lib/uutils/coreutils 2>/dev/null || true
  ```

- AppArmor (if enabled):

  - Check profile status and denials:

  ```bash
  aa-status
  journalctl -k --since -2h | grep -i apparmor || true
  ```

SELinux remediation (recommended, persistent):

If your provider binaries live under non-standard paths (e.g., `/usr/lib/uutils/...` or a custom `--bin-dir`), ensure they are labeled with the executable type (typically `bin_t`).

```bash
# Label the directory tree as executable binaries
sudo semanage fcontext -a -t bin_t "/usr/lib/uutils(/.*)?"
sudo restorecon -Rv /usr/lib/uutils

# If you use a unified dispatcher override
sudo semanage fcontext -a -t bin_t "/usr/bin/coreutils"
sudo restorecon -v /usr/bin/coreutils
```

If SELinux is enforcing and you are validating a one-off issue, you can temporarily switch to permissive to confirm the diagnosis (not a fix):

```bash
# Temporarily (until reboot) — for debugging only
sudo setenforce 0
# Re-enable enforcing immediately after validation
sudo setenforce 1
```

AppArmor remediation:

Some distros ship strict profiles for package managers or system utilities that might deny `execve` on unfamiliar paths.

- Put the affected profile into complain mode during validation (logs denials but allows actions):

  ```bash
  sudo aa-status | sed -n 's/^\s\+profiles are in enforce mode:\s*//p'
  # Example: relax a specific profile while testing
  sudo aa-complain /usr/bin/pacman || true
  ```

- For a permanent fix, extend the relevant profile to allow execution of the chosen `--bin-dir` or unified binary path, or disable that profile if appropriate for your environment:

  ```bash
  # Disable a profile (use sparingly)
  sudo aa-disable /path/to/profile
  ```

Notes for Docker-based tests (`test-orch/`):

- Host AppArmor/SELinux may confine the container. If you see denials originating from the host, adjust the Docker run profile or set `--security-opt` appropriately from the host orchestrator.
- The Arch-derived images in `test-orch/` are minimal; ensure any required MAC tooling (`audit`, `setools`, `appArmor` utilities) is present if you need to debug inside the container.

## License

This project is licensed under the GNU General Public License, version 3 or (at your option) any later version.

Copyright (C) 2025 veighnsche

See the `LICENSE` file for the full text of the GPL-3.0-or-later license.
