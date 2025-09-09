# TODO — Align behavior with DELTA_BEHAVIOR.md

This document tracks code and docs changes required to make the current implementation under `src/` comply with the expected behavior described in `DELTA_BEHAVIOR.md`.

Statuses: [ ] pending, [~] in-progress, [x] done

## High-priority (behavioral correctness)

- [x] CLI selection defaults: require explicit experiment selection
  - Remove implicit defaults in `src/cli/handler.rs` (`default_experiments()` + fallback branch). When no selector is provided, print a helpful message and exit with code 1.
  - Update help texts in `src/cli/parser.rs` to reflect no defaults.

- [x] Introduce `remove` subcommand (Disable is restore-only)
  - Add `Remove` to `Commands` in `src/cli/parser.rs` and route in `src/cli/handler.rs`.
  - Make `Disable` always restore only; remove the interactive "Disable or Remove" prompt and the `assume-yes => Remove` behavior.
  - Wire `Remove` to call each experiment’s `remove()` (which already exists).

- [x] Checksums: zero-discovery should be a distinct non-zero (Exit 20)
  - In `src/experiments/checksums.rs`, when no applets remain after ensure+rediscover, return a typed error (e.g., `Error::NothingToLink`) so the process exits with code 20.
  - Add an audit event `nothing_to_link`.

- [x] Findutils: remove canonical “synthesis” fallback
  - In `src/experiments/findutils.rs`, delete the block that copies binaries into a canonical dir when discovery fails.
  - If, after install, nothing can be discovered, return `Error::NothingToLink` (Exit 20) with guidance in the message and an audit `nothing_to_link` event.

- [x] Repo gating: add explicit `repo_gate_failed` exit code mapping (Exit 40)
  - `src/experiments/mod.rs::check_download_prerequisites()` currently returns `Error::ExecutionFailed(...)` on gate failures. Introduce a specific variant (e.g., `Error::RepoGateFailed { package, details }`) and emit a dedicated audit event `repo_gate_failed{required_repo,pkg,checks[...]}`.

- [x] Distro incompatibility exit code (Exit 10)
  - Keep current `Error::Incompatible`, but map it to Exit 10 in `src/main.rs`.

- [x] Restore behavior: missing backup should be an error (Exit 30) unless a force flag is set
  - In `src/symlink/ops.rs::restore_file()`, change the "no backup" path from WARN+success to an error `Error::RestoreBackupMissing(target)`.
  - Add a global CLI flag (e.g., `--force-restore-best-effort`) that, when set, preserves current best-effort behavior.
  - Propagate the flag into `Worker` and `restore_file()` signatures if needed.

- [x] Link-aware backups and atomic swaps
  - In `src/symlink/ops.rs::replace_file_with_symlink()`:
    - If target is a symlink, back up the symlink itself (create a backup symlink pointing to the same destination), not the resolved file contents.
    - Perform temp-path creation plus atomic `rename` into place. Fsync parent directory to strengthen crash consistency. Mirror this during restore.
  - Update audit events to include: `link_started`, `backup_created`, `link_done`.

- [x] Enumerated exit codes in `src/main.rs`
  - Map error variants to the DELTA codes:
    - `Incompatible` → 10
    - `NothingToLink` → 20
    - `RestoreBackupMissing` → 30
    - `RepoGateFailed` → 40
    - default → 1
  - Ensure these are surfaced as process exit codes.

## Medium-priority (CLI and UX consistency)

- [x] CLI flag normalization and cleanup (`src/cli/parser.rs`, `src/cli/handler.rs`)
  - Deduplicate skip-compat: keep `--skip-compat-check` (preserve old as hidden alias if desired).
  - Remove `--package-manager`; rely on `--aur-helper` only.
  - Add `--aur-user <name>`; validate user exists. Use when invoking AUR helper (see below).
  - Rename `--wait_lock` → `--wait-lock` (accept old as hidden alias for compatibility).
  - Add `--no-progress` to disable progress bars even on TTY.

- [x] Progress behavior toggles (`src/ui/progress.rs`)
  - Honor `--no-progress` by short-circuiting `new_bar()`.
  - Keep env overrides as secondary (as-is).

- [x] Audit taxonomy expansion (`src/logging/audit.rs` + call sites)
  - Implemented events at call sites: `enabled`, `removed_and_restored`, `link_started`, `link_done`, `backup_created`, `restore_started`, `restore_done`, `repo_gate_failed{…}`, `nothing_to_link`, `install_package.*`, `remove_package`.
  - Structured events are emitted regardless of progress bar state.

## Medium-priority (AUR and package flows)

- [x] Avoid hardcoded `builder` user for AUR (`src/system/worker/packages.rs`)
  - Respect `--aur-user`. If set, run helper as that user; else run as invoking user (or maintain current behavior behind a compatibility flag/env if needed).
  - Replace `su - builder -c` with `runuser -u <user> -- <helper> ...` or `sudo -u <user> -- <helper> ...` depending on availability.
  - Improve error messages when helper/user validation fails.

- [x] Remove `--package-manager` preference plumbing
  - Simplify `effective_helper` in `src/cli/handler.rs` and candidate ordering in `src/system/worker/aur.rs` to rely solely on `--aur-helper` (with `auto|none|paru|yay|trizen|pamac`).

## Medium-priority (assets and targets)

- [x] Move coreutils applet list into assets
  - Create `src/assets/coreutils-bins.txt` and change `include_str!` in `src/experiments/coreutils.rs` to point there.
  - Update `list-targets` behavior if it depends on asset paths; ensure tests are adjusted accordingly.

## Low-priority (codebase hygiene and docs)

- [x] Error enum cleanup (`src/error.rs`)
  - Added new typed variants: `NothingToLink`, `RestoreBackupMissing`, `RepoGateFailed { package: String, details: String }`. (Legacy variants retained for compatibility.)

- [ ] README and behavior docs
  - Update `README.md` to reflect: no default experiments; `remove` subcommand; link-aware backups; AUR requirement for findutils; enumerated exit codes; new flags.
  - Move/refresh behavior notes in `CURRENT_BEHAVIOR.md` after implementation to match DELTA.

- [ ] Tests
  - Add/adjust tests for:
    - Exit codes (10/20/30/40) mapping in `src/tests/`.
    - Checksums zero-discovery path.
    - Findutils no-synthesis path.
    - Link-aware backup/restore across symlink and regular-file cases, including missing-backup error and `--force-restore-best-effort`.
    - CLI: new `remove` subcommand; `--no-progress`; `--wait-lock` kebab-case; `--aur-user`.

## File-by-file pointers

- `src/cli/parser.rs` — flags and subcommands.
- `src/cli/handler.rs` — selection defaults, disable/remove flow, root enforcement.
- `src/experiments/mod.rs` — repo gating errors and audit, `all_experiments()` order (already matches), typed error returns.
- `src/experiments/checksums.rs` — zero-discovery → `NothingToLink` + audit.
- `src/experiments/findutils.rs` — remove synthesis block; zero-discovery → `NothingToLink`.
- `src/experiments/coreutils.rs` — move applet list include to `src/assets/...`.
- `src/symlink/ops.rs` — link-aware backup/restore, atomic swaps, fsync parent, missing backup error.
- `src/system/worker/packages.rs` — install/remove flows; AUR invocation user handling; audit taxonomy.
- `src/system/worker/aur.rs` — simplify candidate ordering after `--package-manager` removal.
- `src/logging/audit.rs` — add helpers/constants for event taxonomy.
- `src/ui/progress.rs` — `--no-progress` integration.
- `src/main.rs` — map error variants to enumerated exit codes.
- `src/error.rs` — introduce new error variants and remove unused ones (or wire them).

## Acceptance criteria (must-haves)

- No default experiment selection; `oxidizr-arch enable` without selectors exits non-zero with a clear message.
- `disable` never uninstalls; `remove` exists and uninstalls after restore.
- Checksums and Findutils return Exit 20 when nothing to link after provider ensure.
- Repo gating failures emit `repo_gate_failed{…}` and exit with code 40.
- Distro incompatibility exits with code 10.
- Restore without a backup exits with code 30 unless forced.
- Symlink operations back up/restores links as links and use atomic renames; audit events emitted at each step.
- CLI supports `--aur-helper` and `--aur-user`, `--no-progress`, and `--wait-lock` (kebab-case).
- Coreutils applet list lives under `src/assets/`.
- README and CURRENT_BEHAVIOR updated to reflect new behavior.
