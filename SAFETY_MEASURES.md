# SAFETY_MEASURES

This document summarizes the concrete risk-mitigation measures implemented in the oxidizr-arch codebase today. Citations refer to files and symbols under `src/`.

## Transactionality and Atomicity

- Atomic symlink swaps using `renameat(2)`
  - `symlink/ops.rs::atomic_symlink_swap()` creates a temp symlink in the same directory and atomically renames it into place.
  - Parent directory is opened with `O_DIRECTORY|O_NOFOLLOW` via `open_dir_nofollow()` to reduce TOCTOU risk; parent is `fsync`'d after rename (`fsync_parent_dir()`).
- Consistent backups before mutation
  - Regular files are backed up to a hidden path with suffix `.<name>.oxidizr.bak` before replacement; permissions are preserved (`replace_file_with_symlink()`).
  - Existing symlinks are backed up as symlinks pointing to their current destination (link-aware backups).
- Restore is also atomic
  - `symlink/ops.rs::restore_file()` renames the backup into place with `renameat`, using `O_NOFOLLOW` on parent and an `fsync` of the directory after the operation.

## Dry‑Run and Explicit Consent

- Safe dry‑run mode
  - `--dry-run` disables state persistence (`state/mod.rs::save_state()` short‑circuits), audit file sink (`logging/init.rs::init_logging()`), and avoids any mutating filesystem calls in high‑level flows (e.g., `InstallHook` prints the plan instead of writing).
- Clear operator confirmation gates
  - Mutating subcommands prompt for confirmation unless `--assume-yes` is provided (`cli/handler.rs::handle_cli()`).
- Root enforcement for mutations
  - Mutating commands enforce root unless in dry‑run (`cli/handler.rs::enforce_root()`), mapped to a dedicated exit code (`error.rs::Error::RootRequired`).

## Filesystem and Ownership Safety

- Filesystem suitability checks
  - Require `rw` and not `noexec` mounts for targets and `/usr` (`system/fs_checks.rs::ensure_mount_rw_exec()`).
  - Detect immutable files (best effort via `lsattr`) and fail with remediation guidance (`check_immutable()`).
- Source trust policy
  - Enforce that sources are not world‑writable, are root‑owned, and are on an executable mount; disallow sources under `$HOME` unless `--force` (`system/fs_checks.rs::check_source_trust()`).
- Package ownership policy
  - Warn or fail when target is not owned by a package, depending on `--strict-ownership` (`system/worker/packages.rs::verify_owner_for_target()`).

## Package and Supply Chain Safeguards

- Repository and AUR gating
  - Verify official `extra` repo availability using multiple probes (`system/worker/distro.rs::extra_repo_available()`), and gate package operations accordingly (`experiments/mod.rs::check_download_prerequisites()`).
  - For packages absent from official repos (e.g., `uutils-findutils-bin`), fall back to available AUR helpers with clear audit logs (`system/worker/packages.rs::install_package()`).
- Pacman lock handling
  - Respect and wait for pacman DB locks with timeouts, returning a typed error (`Error::PacmanLockTimeout`).
- Least‑privilege AUR invocation
  - Optionally run AUR helpers under a specified user (`--aur-user`), using `su - <user> -c ...` (`system/worker/packages.rs`).

## Distro Compatibility and Selection Safety

- Compatibility checks
  - Support is limited to Arch family (`checks/compat.rs`); CLI `Check` aggregates incompatibilities and fails clearly (`cli/handler.rs`).
- Safe experiment selection and ordering
  - CLI enforces mutually exclusive selection flags (`cli/parser.rs`).
  - Registry order is respected; visibility logs warn when findutils+coreutils are both selected to preserve checksum tools during AUR builds (`cli/handler.rs`).

## State and Hooks

- Idempotent state management with dry‑run gating
  - Enabled experiments and managed targets are persisted under `/var/lib/oxidizr-arch/state.json`; writes are skipped in dry‑run (`state/mod.rs`).
  - `relink_managed()` restores previously managed symlinks from state, tolerating unknown entries and cleaning them up.
- Pacman hook safety
  - Hook body is deterministic and can be previewed without install; hook installation is explicitly invoked and logged (`system/hook.rs`).

## Audit Logging and Observability

- Structured, machine‑readable JSONL audit trail
  - Audit events include timestamp, environment identifiers, and operation metadata (`logging/audit.rs::audit_event_fields()`), emitted to `/var/log/oxidizr-arch-audit.log` or `$HOME/.oxidizr-arch-audit.log` fallback (`logging/init.rs`).
  - Audit sink is disabled automatically under `--dry-run` to avoid recording non‑actions.
- Human logs with stable verbosity contract
  - Human‑readable logs include a distro prefix and map to explicit verbosity levels (`logging/init.rs::HumanFormatter`, `VERBOSE` env).

## Recovery Measures

- One‑step rollback of managed targets
  - `experiments/util.rs::restore_targets()` orchestrates restore of a list of targets; `symlink/ops.rs::restore_file()` performs the atomic restore operation.
- Post‑enable verification for sudo‑rs
  - `experiments/sudors.rs::verify_post_enable()` verifies setuid/ownership on the real installed binary, ensures PAM config presence, and can run an optional `sudo -n true` smoke test for a configured user; on failure, it reverts the changes.

## Concurrency and Process Safety

- Single‑instance process lock
  - A PID‑independent lock under `/run/lock/oxidizr-arch.lock` prevents concurrent mutating operations (`system/lock.rs`).
- Directory `fsync` after renames
  - Parent directory is `sync_all()`'d after `renameat` to reduce metadata loss risk on crashes (`symlink/ops.rs::fsync_parent_dir()`).

## Minimal Intrusion by Design

- Symlink‑first strategy
  - The tool swaps symlinks at well‑known targets under `/usr/bin` rather than modifying binaries or package databases directly.
- Idempotent operations and informative logs
  - Existing correct symlinks are left untouched; all steps emit informative logs with context and structured audit fields for traceability.

---

These measures are in place today. Additional planned work to strengthen safety (e.g., transactional rollbacks across multi‑target operations, automatic smoke tests, cryptographic hash logging, tamper‑evident audit chaining, and metadata/xattrs preservation) is tracked in `TODO_LIST_V2.md`.
