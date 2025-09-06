# oxidizr-arch Scaffolding (Arch + AUR)

A Rust scaffolding for safely switching Arch Linux core utilities (e.g., coreutils) to Rust-based replacements (e.g., uutils) using Pacman/AUR, inspired by oxidizr. This project is intentionally non-destructive and leaves real system calls as TODOs behind a `Worker` abstraction.

See `TECHNICAL_IMPLEMENTATION.md` for the complete design and checklist.

## Features (Scaffold)

- Arch Linux focused ("rolling") compatibility gate.
- Experiment model for families (e.g., `coreutils`).
- AUR-friendly configuration (package/helper overridable).
- Idempotent enable/disable flows via symlink strategy (implemented via `Worker` stubs).

## Install (Local Build)

```bash
cargo install --path .
```

This installs the binary `oxidizr-arch`.

## CLI Usage

Commands:

```bash
# Check compatibility with the current system (expects Arch/rolling in scaffold)
oxidizr-arch check

# List target paths that would be affected (computed via which() fallback)
oxidizr-arch list-targets

# Enable (install package + symlink swap-in) — scaffold does not perform real system ops yet
oxidizr-arch enable

# Disable (restore backups + remove package) — scaffold does not perform real system ops yet
oxidizr-arch disable
```

Common flags:

```bash
# Override package/helper/paths
oxidizr-arch \
  --experiment coreutils \
  --package uutils-coreutils \
  --aur-helper paru \
  --bin-dir /usr/lib/uutils/coreutils \
  --unified-binary /usr/bin/coreutils \
  enable

# Skip update step
oxidizr-arch --no-update enable

# Skip confirmations (to be implemented)
oxidizr-arch --assume-yes enable
```

Defaults for `--experiment coreutils`:

- `package`: `uutils-coreutils`
- `bin-dir`: `/usr/lib/uutils/coreutils`
- `unified-binary`: `/usr/bin/coreutils`
- compatibility: `Arch` + `rolling`

## Library Status

The `core` module is a placeholder. The intended extension surface is via `experiment` (switch orchestration) and `worker` (system operations). Implement `Worker` with real Arch logic to make the tool functional.

## Development

Read `TECHNICAL_IMPLEMENTATION.md` for the Arch/AUR plan, including safety requirements, idempotence, backups, and atomic restore behavior.

## Testing

There are three recommended ways to test:

1) Rust unit tests (fast, no system changes)

```bash
cargo test
```

This runs `tests/cli_tests.rs` (CLI parsing/UX) and `tests/experiment_tests.rs` (backup/symlink semantics and Arch gating with a mock worker).

2) Manual, isolated testing in an Arch LXD container

Use an ephemeral Arch container to exercise the Arch scripts and assertions without touching your host:

```bash
# Install and initialize LXD once on the host
sudo pacman -Sy --noconfirm lxd
sudo systemctl enable --now lxd
lxd init --auto

# Launch an ephemeral Arch container and mount the repo
name=oxidizr-arch-test
lxc launch images:archlinux $name -e
lxc exec $name -- bash -lc 'pacman -Sy --noconfirm base-devel git sudo curl'

# Push project into the container
lxc file push -r . $name/root/project

# Build and expose expected binary name (oxidizr-arch)
lxc exec $name -- bash -lc '
  cd /root/project/rust_coreutils_switch && \
  pacman -Sy --noconfirm rustup && \
  rustup default stable && \
  cargo build --release && \
  ln -sf "$PWD/target/release/oxidizr-arch" /usr/local/bin/oxidizr-arch
'

# Run the Arch shell assertions
lxc exec $name -- bash -lc '
  cd /root/project/rust_coreutils_switch && \
  source tests/lib/uutils.sh && \
  source tests/lib/sudo-rs.sh && \
  oxidizr-arch enable --yes --all && \
  ensure_coreutils_installed && \
  ensure_findutils_installed || true && \
  ensure_diffutils_installed_if_supported && \
  ensure_sudors_installed && \
  oxidizr-arch disable --yes --all && \
  ensure_coreutils_absent && \
  ensure_findutils_absent && \
  ensure_diffutils_absent && \
  ensure_sudors_absent
'

# Destroy the container when done
lxc delete -f $name
```

Notes:

- The helper scripts assume Arch packages `uutils-coreutils`, `uutils-findutils`, `uutils-diffutils`, and `sudo-rs`.
- If you prefer a specific helper, pass `--package-manager yay` (or `--aur-helper yay`). To disable AUR entirely, use `--aur-helper none`.
- Our shell assertions now use the binary named `oxidizr-arch`.

3) Spread runner (optional)

The `tests/*/task.yaml` files are written in a Spread-compatible style. If you want a full suite runner, set up Spread and LXD, then add a `spread.yaml` that targets an Arch image and points the `tests/` directory as a suite. Example high-level steps:

```bash
# Install Spread (via Go toolchain)
go install github.com/snapcore/spread/cmd/spread@latest

# Ensure LXD is set up as above, then run:
spread -v
```

If you want, I can commit a ready-to-run `spread.yaml` configured for `images:archlinux` and wire environment like `SPREAD_PATH` so the helpers resolve.

## License

This project is licensed under the GNU General Public License, version 3 or (at your option) any later version.

Copyright (C) 2025 veighnsche

See the `LICENSE` file for the full text of the GPL-3.0-or-later license.
