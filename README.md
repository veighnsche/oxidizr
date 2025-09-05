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

This installs the binary `coreutils-switch`.

## CLI Usage

Commands:

```bash
# Check compatibility with the current system (expects Arch/rolling in scaffold)
coreutils-switch check

# List target paths that would be affected (computed via which() fallback)
coreutils-switch list-targets

# Enable (install package + symlink swap-in) — scaffold does not perform real system ops yet
coreutils-switch enable

# Disable (restore backups + remove package) — scaffold does not perform real system ops yet
coreutils-switch disable
```

Common flags:

```bash
# Override package/helper/paths
coreutils-switch \
  --experiment coreutils \
  --package uutils-coreutils \
  --aur-helper paru \
  --bin-dir /usr/lib/uutils/coreutils \
  --unified-binary /usr/bin/coreutils \
  enable

# Skip update step
coreutils-switch --no-update enable

# Skip confirmations (to be implemented)
coreutils-switch --assume-yes enable
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

## License

Dual-licensed under MIT or Apache 2.0 at your option.
