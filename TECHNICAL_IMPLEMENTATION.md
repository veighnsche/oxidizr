# oxidizr-arch Technical Implementation (Scaffolding Plan)

This document defines the technical plan for an oxidizr-inspired tool that safely switches Arch Linux core utilities to Rust-based replacements (e.g., uutils) via Pacman/AUR. It matches the architecture described in oxidizr and maps directly to the scaffolding currently in this repository. Use this as a checklist and guide to complete the implementation.

## Scope

- Safe, reversible switching of utilities families (coreutils, findutils, diffutils, …) to Rust alternatives.
- Arch-focused with explicit gating (rolling) and AUR helper integration.
- Idempotent operations and atomic restoration.
- This repository currently contains a compiling scaffolding; the real system calls are intentionally left as TODOs behind a `Worker` abstraction.

## Repository Structure

- `src/experiment/mod.rs` — Experiment orchestration (enable/disable/check/list targets)
- `src/worker/mod.rs` — `Worker` trait + `System` placeholder (replace with real Arch impl using pacman/AUR helper)
- `src/cli.rs` — CLI surface: `enable`, `disable`, `check`, `list-targets`
- `src/core/mod.rs` — Placeholder for optional per-command abstractions
- `src/error.rs` — Error types (`CoreutilsError`, `Result`)
- `src/lib.rs` — Module exports, basic test
- `src/main.rs` — Binary entrypoint

## Architecture

### Experiment Abstraction

`UutilsExperiment` encapsulates one family of utilities:
- `name`: e.g., `"coreutils"`
- `package`: e.g., `"rust-coreutils"`
- `supported_releases`: e.g., `["rolling"]`
- `unified_binary`: optional path to a single dispatch binary (e.g., `/usr/bin/coreutils`)
- `bin_directory`: directory containing the replacement binaries (e.g., `/usr/lib/cargo/bin/coreutils`)

Responsibilities:
- `check_compatible(worker)`: returns `true` if distro is Arch and release is in `supported_releases`.
- `enable(worker, assume_yes, update_lists)`: package install + symlink swap-in (with safety via `Worker`).
- `disable(worker, update_lists)`: restoration from backups + package removal.
- `list_targets(worker)`: compute target paths that would be affected.

### System Interface

`Worker` trait abstracts system operations. Implement `System` for real Arch behavior.

Required methods:
- `distribution() -> (String, String)`
  - Parse `/etc/os-release` (e.g., `ID=arch`) and assume `rolling` release label.
- `update_packages()`
  - `pacman -Sy` (sync package databases) or delegate to AUR helper if needed.
- `install_package(package)`
  - `pacman -S --noconfirm <package>` if in repos; for AUR, call helper (e.g., `paru -S --noconfirm <package>`).
- `remove_package(package)`
  - `pacman -R --noconfirm <package>`; for AUR-installed packages, pacman still manages the installed package.
- `check_installed(package) -> bool`
  - `pacman -Qi <package>` returns 0 when installed.
- `which(name) -> Option<PathBuf>`
  - Use the `which` crate or invoke `which`.
- `list_files(dir) -> Vec<PathBuf>`
  - Validate directory existence, enumerate entries; skip non-regular files.
- `replace_file_with_symlink(source, target)`
  - If `target` is symlink: skip (idempotent)
  - Else:
    - `backup_file(target)` → sibling `/.<name>.oxidizr.bak` preserving permissions and special bits.
    - Remove original `target`.
    - `create_symlink(source, target)`.
- `restore_file(target)`
  - If backup exists: atomic rename backup → `target`.
  - Else: warn and continue (fails safe).

Helper semantics (can be private methods of `System`):
- `backup_file(target)`
  - Copy to `/.<filename>.oxidizr.bak`, re-apply original mode/ownership to backup to preserve SUID/SGID/sticky.
- `create_symlink(source, target)`
  - Remove any leftover `target`, create symlink `source -> target`.

## Safety Gates Before Changes (Arch)

- Distribution compatibility:
  - `check_compatible()` validates Arch and supported release list (e.g., `rolling`).
- Run-as-root and confirmation:
  - CLI intended to be run as root. Prompt for confirmation unless `--assume-yes`.
- Package list update:
  - CLI runs `pacman -Sy` (or AUR helper sync) unless `--no-update` is supplied.

## Enable Flow (Switch to Rust Utilities)

1. Compatibility and Preconditions
   - `check_compatible()` must be `true`.
   - Confirm if not `--assume-yes`.
   - Optionally `update_packages()`.

2. Install Replacement Package (Arch/AUR)
   - `install_package(self.package)` using pacman/AUR helper.

3. Discover Binaries to Replace
   - `list_files(self.bin_directory)` → candidate binaries: e.g., `date`, `sort`, …

4. Compute Target for Each Command Name
   - For each file `f`:
     - `filename = f.file_name()`.
     - `target = which(filename)` else `/usr/bin/<filename>` fallback.

5. Swap-in Symlink with Backup
   - Unified binary present (`self.unified_binary = Some(p)`):
     - `replace_file_with_symlink(p, target)` for each `filename`.
   - Non-unified binary:
     - `replace_file_with_symlink(f, target)` for each `filename`.

Result:
- System resolves common tools (`date`, `sort`, …) to Rust implementation via symlinks.
- Unified binary dispatches by `argv[0]` (symlink name).

## Disable Flow (Revert Safely)

1. Optionally `update_packages()`.
2. Discover the same set of filenames via `list_files(self.bin_directory)`.
3. For each `filename`, compute `target` with `which()` fallback.
4. `restore_file(target)` → atomic rename backup back to `target` if backup exists; else warn.
5. `remove_package(self.package)` (pacman).

## Idempotence and Rollback Guarantees

- Re-entrant enable:
  - Skip if `target` is already a symlink (idempotent).
- Safe disable:
  - Only restore if backup exists; otherwise warn and leave as-is.
- Atomic restore:
  - Use `rename` to atomically swap backup into place.
- Deterministic targets:
  - Fallback to `/usr/bin/<filename>` when `which()` fails.

## Unified vs Non-Unified Binaries

- Unified (e.g., coreutils):
  - A single binary (e.g., `/usr/bin/coreutils`) symlinked to many target names.
- Non-unified (e.g., findutils-style):
  - Each replacement binary symlinked directly to its own target.

## CLI Surface (Scaffolding)

Commands (see `src/cli.rs`):
- `enable` — Installs package via pacman/AUR helper and swaps-in symlinks via `Worker`.
- `disable` — Restores originals and removes the package.
- `check` — Prints compatibility boolean.
- `list-targets` — Prints computed target paths.

Flags:
- `--assume-yes` — Skip prompts (TODO: implement prompt in CLI).
- `--no-update` — Do not run `pacman -Sy` pre-step.
- `--experiment <name>` — Defaults to `coreutils`. Scaffolding constructs:
  - `name = "coreutils"`
  - `package = "uutils-coreutils"` (AUR/extra)
  - `supported_releases = ["rolling"]`
  - `unified_binary = /usr/bin/coreutils`
  - `bin_directory = /usr/lib/uutils/coreutils`
- `--aur-helper <helper>` — AUR helper to use (default `paru`).
- `--package <name>` — Override package name.
- `--bin-dir <path>` — Override replacement binaries directory.
- `--unified-binary <path>` — Override unified dispatch binary.

## Mapping: Requirements → Code

- Experiment orchestration → `src/experiment/mod.rs` (`UutilsExperiment`)
- System interface → `src/worker/mod.rs` (`Worker` trait)
- CLI orchestration → `src/cli.rs`
- Error handling → `src/error.rs`
- Tests (basic) → `src/lib.rs` test

## Implementation TODOs (Checklist)

- [x] Enforce root execution in CLI (uid check) and add confirmation prompts unless `--assume-yes`.
- [x] Implement `System` on Arch (feature-gated under `arch`):
  - [x] `distribution()` via `/etc/os-release`.
  - [x] `update_packages()` via `pacman -Sy`.
  - [x] `install_package()` via `pacman -S --noconfirm` or AUR helper (`paru -S --noconfirm`).
  - [x] `remove_package()` via `pacman -R --noconfirm`.
  - [x] `check_installed()` via `pacman -Qi`.
  - [x] `which()` via `which` crate.
  - [x] `list_files()` validating `bin_directory`.
  - [x] `replace_file_with_symlink()` with backup and permissions preservation.
  - [x] `restore_file()` using atomic rename; warn if backup missing.
- [x] Add logging/tracing for operations (info/warn levels).
- [x] Extend experiments for other families (findutils, diffutils) with non-unified binaries. (Added `findutils` scaffold.)
- [x] Unit/integration tests with a tmpfs or sandboxed FS:
  - [x] Enable flow idempotence (existing symlink → skip).
  - [x] Backup creation and special bits preservation.
  - [x] Disable flow restores originals atomically.
  - [x] Compatibility gate prevents unsupported releases.
- [x] Feature-gate the real `System` implementation (e.g., `--features arch`) to keep default build safe.

## Notes on Permissions and Special Bits

- When backing up targets, ensure the backup preserves:
  - File mode (including SUID/SGID/sticky).
  - Ownership if applicable (may require `chown`, careful in tests).
- When restoring, atomic `rename` ensures the swap is single-operation on the same filesystem.

## Error Handling

- All public APIs return `Result<T, CoreutilsError>`.
- Introduced variants:
  - `Io(std::io::Error)`
  - `Incompatible(String)`
  - `ExecutionFailed(String)`
  - `Other(String)`

## Security Considerations

- Root-required operations must be explicit and visible.
- Confirmation prompts reduce accidental modifications.
- Idempotent symlink creation prevents repeated destructive changes.
- Backups are placed as hidden siblings to avoid PATH shadowing and accidental invocation.

## Future Enhancements

- Dry-run mode to print planned operations without modifying the system.
- Rollback-on-error strategy if any step fails mid-enable.
- Rich reporting (JSON output) for automation.
- Telemetry/metrics hooks (optional, opt-in).

---

This document is the source of truth for the implementation. Keep it updated as you implement each TODO. The current codebase is a scaffolding that compiles and is structured to let you incrementally add the real system interactions while preserving safety and testability.
