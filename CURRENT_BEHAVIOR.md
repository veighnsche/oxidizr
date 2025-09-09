# Product Behavior Documentation (oxidizr-arch)

This document captures observable behavior of the Rust product under `src/` and related manifests. It focuses on inputs, outputs, side effects, invariants, logging, and error semantics as implemented in the current codebase.

Scope limits: this excludes host/container test orchestration behavior under `test-orch/`. It documents only the Rust binary/library behavior and its direct runtime file/command effects.

## High-level overview

- The binary `oxidizr-arch` controls “experiments” that safely switch selected system tools to Rust replacements via package installation and symlink swapping, with atomic backups and restore.
- Supported experiment families:
  - `coreutils` → `uutils-coreutils`
  - `findutils` → `uutils-findutils-bin`
  - `sudo-rs` → `sudo-rs`
  - `checksums` → flips checksum applets (presence-aware) using `uutils-coreutils` if needed
- Operations are executed through a `Worker` that performs distro detection, pacman/AUR actions, filesystem symlink operations, and logging.

## Binaries, library, features

- Crate name: `oxidizr_arch` (library at `src/lib.rs`).
- Binary: `oxidizr-arch` (entrypoint `src/main.rs`).
- Feature flags in `Cargo.toml` define `arch` but the current code paths do not gate functionality by features; behavior is implemented directly.

## CLI inputs and subcommands

Defined in `src/cli/parser.rs`, handled in `src/cli/handler.rs`.

- Global flags and arguments:
  - `--assume-yes | -y | --yes`: non-interactive mode; answers “yes” to confirmations and chooses default destructive options on disable.
  - `--no-update`: skip `pacman -Sy` before actions.
  - `--all`: select all experiments (ordered; see selection).
  - `--experiments <comma,sep>`: select multiple experiments by name.
  - `--experiment <name>`: legacy single selector.
  - `--skip-compatibility-check | --no-compatibility-check`: bypass distro gating.
  - `--aur-helper <auto|none|paru|yay|trizen|pamac>`: select AUR helper policy (defaults to `auto`).
  - `--package-manager <string>`: forces a specific helper name; overrides `--aur-helper` for detection preference.
  - `--package <string>`: override provider package name for the selected experiment(s).
  - `--bin-dir <path>`: override replacement bin directory for uutils experiments.
  - `--unified-binary <path>`: specify unified dispatcher path (e.g., `/usr/bin/coreutils`).
  - `--dry-run`: print intended actions; make no changes.
  - `--wait_lock <secs>`: wait up to N seconds for pacman DB lock to clear.

- Subcommands:
  - `enable`: install provider(s) if needed and swap targets to provider binaries (symlink with backup).
  - `disable`: restore targets from backups; leaves provider installed.
  - `check`: print per-experiment compatibility for current distro.
  - `list-targets`: print the target paths that the selected experiments would affect.

- Experiment selection resolution in handler:
  - If `--all`: use full registry order from `experiments::all_experiments()`.
  - Else if `--experiments` non-empty: filter by those names.
  - Else if `--experiment` provided: use that single name.
  - Else: defaults to `coreutils` and `sudo-rs` (sorted) as the default set.

- Orchestration note: when both `findutils` and `coreutils` are selected, logs hint that `findutils` should be enabled first so checksum tools remain available for AUR builds; `--all` already orders this accordingly.

### Disable command behavior (prompt vs. removal)

- When running the `disable` subcommand, the CLI handler prompts whether to Disable (restore originals, keep package installed) or Remove (uninstall the package and restore originals). In non-interactive/assume-yes mode (`--assume-yes`), it defaults to Remove. See `src/cli/handler.rs` (`enforce_root`, prompt, and `do_remove` logic).

## Experiments and their behaviors

Registry and enum in `src/experiments/mod.rs`.

- Common contract (`ExperimentOps` as methods on concrete types, wrapped by enum):
  - `name() -> &str`
  - `package_name() -> &str` (n/a for `checksums`)
  - `check_compatible(&Distribution) -> Result<bool>`
  - `enable(&Worker, assume_yes, update_lists)`
  - `disable(&Worker, assume_yes, update_lists)`
  - `remove(&Worker, assume_yes, update_lists)`
  - `list_targets() -> Vec<PathBuf>`

Note on implementation detail: although a trait `ExperimentOps` is defined in `src/experiments/mod.rs`, the concrete experiment structs do not currently implement this trait. Instead, they expose methods directly, and the `Experiment` enum dispatches to each concrete struct via `match`. Treat the trait as a conceptual contract (documentation aid) and the enum as the actual unified interface.

- Compatibility gating on `enable`:
  - Always reads current distro (`Worker.distribution()` parsing `/etc/os-release`).
  - If skip flag not provided, calls experiment’s `check_compatible`; returns `Error::Incompatible` when unsupported.
  - Supported IDs: `arch`, `endeavouros`, `cachyos`, `manjaro` (see `checks::SUPPORTED_DISTROS`).

### Enable command confirmation

- On `enable`, when not in `--assume-yes` mode, the CLI asks for confirmation: "Enable and switch to Rust replacements?". Answering no aborts the operation before any changes. Root enforcement is applied unless `--dry-run` is set.

- Experiment execution order for `--all`: `findutils`, `coreutils`, `sudo-rs`, `checksums`.

### Coreutils (`src/experiments/coreutils.rs`)

- Inputs:
  - Optionally overridden `package` (default `uutils-coreutils`).
  - Optional `--unified-binary` and `--bin-dir` overrides.
  - `--dry-run`, `--no-update`, `--assume-yes`, `--wait_lock`.

- Enable behavior:
  - If `update_lists`: runs `pacman -Sy`.
  - Calls `check_download_prerequisites` (repo/AUR gate; see below), then installs package via `Worker.install_package`.
  - Discovers applets from unified dispatcher (preferred: provided path or `which coreutils`) or per-applet locations under effective bin dir and common fallbacks.
  - Excludes checksum applets from linking by policy (`PRESERVE_BINS`), to be handled by the dedicated `checksums` experiment.
  - For each selected applet, creates a symlink at `/usr/bin/<applet>` to the discovered provider source with atomic backup logic (see Symlink ops).

- Disable behavior:
  - If `update_lists`: `pacman -Sy`.
  - Restores non-checksum applets from backups at their `/usr/bin/<applet>` targets.

- Remove behavior:
  - First calls `disable`.
  - Guard: refuses removal if any checksum applet target is still a symlink; instructs user to disable `checksums` first.
  - Removes package via pacman, then verifies it is absent.

- Targets:
  - All coreutils applets from a compile-time list included via `include_str!("../../tests/lib/rust-coreutils-bins.txt")` minus checksum applets.

### Findutils (`src/experiments/findutils.rs`)

- Inputs: `--dry-run`, `--no-update`, `--assume-yes`.
- Enable:
  - If `update_lists`: `pacman -Sy`.
  - Gates download, then installs `uutils-findutils-bin`.
  - Warns if `sha256sum` not found on PATH (helps AUR build visibility).
  - Discovers `find` and `xargs` in canonical bin dir or common fallbacks, then links them to `/usr/bin/find` and `/usr/bin/xargs` with backup.
  - If discovery initially finds nothing, it attempts to synthesize canonical sources by copying binaries from known locations (e.g., `/usr/bin/uu-<name>`, `/usr/lib/cargo/bin/<name>`) into the canonical bin dir (`/usr/lib/cargo/bin/findutils`) before reusing them for linking; permissions are preserved on copies.
- Disable: restore `find` and `xargs` from backups.
- Remove: `disable` then remove package; verifies absence.
- Targets: `/usr/bin/find`, `/usr/bin/xargs`.

### Sudo-rs (`src/experiments/sudors.rs`)

- Inputs: `--dry-run`, `--no-update`, `--assume-yes`.
- Enable:
  - If `update_lists`: `pacman -Sy`.
  - Gates download, then installs `sudo-rs` and verifies installation.
  - For each of `sudo`, `su`, `visudo`:
    - Resolves provider binary from explicit locations (`/usr/lib/cargo/bin/<name>`, `/usr/bin/<name>-rs`) or PATH (`<name>-rs`).
    - Creates a stable alias symlink at `/usr/bin/<name>.sudo-rs` pointing to the real provider binary.
    - Replaces the target (`/usr/bin/<name>` or `/usr/sbin/visudo`) with a symlink pointing to the alias. Both alias and target creations are verified as symlinks; errors are hard failures.
- Disable:
  - Restores the three targets from backups, then verifies they are no longer symlinks.
- Remove: `disable` then remove package; verifies absence.
- Targets: `/usr/bin/sudo`, `/usr/bin/su`, `/usr/sbin/visudo`.

### Checksums (`src/experiments/checksums.rs`)

- Inputs: `--dry-run`, `--no-update`, `--assume-yes`, `--package`, `--unified-binary`, `--bin-dir`.
- Enable:
  - If `update_lists`: `pacman -Sy`.
  - Discovers checksum applets from unified dispatcher or per-applet directories; if none found, ensures provider (default `uutils-coreutils`) is installed, then re-discovers.
  - Logs skipped names that are not present on this build/distro.
  - Links discovered applets to `/usr/bin/<name>` with backup.
  - If no checksum applets are discovered even after ensuring the provider is installed, the operation logs that nothing was found and returns success without changes (no-op).
- Disable: restores checksum targets.
- Remove: no package removal; same as `disable` with a clarifying log.
- Targets: the checksum set `{b2sum, md5sum, sha1sum, sha224sum, sha256sum, sha384sum, sha512sum}`.

## Package and repository behaviors (`src/system/worker/packages.rs`)

- Update (`update_packages`):
  - Honors `dry_run` (emits `[dry-run] pacman -Sy`).
  - Waits for pacman DB lock if `wait_lock_secs` is set; otherwise fails fast when lock file exists.
  - Executes `pacman -Sy [--noconfirm]`; on non-zero exit returns `Error::ExecutionFailed`.

- Installation (`install_package`):
  - Rejects unsafe or invalid package names.
  - Honors `dry_run` (no changes).
  - If already installed (`pacman -Qi`), no-op with success logs.
  - Lock handling as above.
  - For most packages: attempts `pacman -S [--noconfirm] <pkg>` and returns error if not installed afterwards.
  - Special policy for `uutils-findutils-bin`:
    - Skips `pacman` when `repo_has_package` reports absent (AUR-only).
    - Falls back to available AUR helpers (`paru`, `yay`, `trizen`, `pamac`), running via `su - builder -c '<helper> ... -S --needed <pkg>'`.
    - On failure with no helpers available, returns guidance error.
  - When `--assume-yes` is set, AUR helpers are invoked with batch flags (e.g., `--batchinstall --noconfirm` for `paru`) before `-S --needed` to make installs non-interactive.
  - Helper candidate ordering respects `--package-manager` when provided by placing it first, followed by the default order `paru`, `yay`, `trizen`, `pamac`.

- Removal (`remove_package`):
  - Rejects invalid names; honors `dry_run`.
  - If not installed, logs and returns success.
  - Runs `pacman -R [--noconfirm] <pkg>`; then verifies absence.

- Repository presence:
  - `repo_has_package` checks `pacman -Si` success.
  - `extra_repo_available` uses heuristics: `pacman-conf --repo-list`, `pacman -Sl extra`, `pacman-conf -l`, and finally scanning `/etc/pacman.conf`.
  - For `uutils-coreutils` and `sudo-rs`, `check_download_prerequisites()` requires the `extra` repository to be available and explicitly verifies `pacman -Si <pkg>` succeeds. If the package is not found in repos, it fails early with guidance to refresh mirrors (`pacman -Syy`) or adjust repo configuration.

- AUR helper detection (`aur_helper_name`):
  - Considers `--package-manager`/`aur_helper` preference and checks presence via `which`.
  - If no helper is found, audit logs record `not_found`; operations requiring AUR will fail with a clear error.

- Already-installed reuse prompt (`experiments::check_download_prerequisites`):
  - If a package is detected as installed, the system prompts to reuse the existing install or reinstall. In non-interactive mode (`--assume-yes`), it defaults to reuse.
  - An audit event records the decision (`reuse` or `reinstall_requested`). Informational logs are emitted accordingly.

## Distro and compatibility (`src/checks/compat.rs`, `src/system/worker/distro.rs`)

- Distribution resolution reads `/etc/os-release` (`ID`, `ID_LIKE`), defaulting to `arch` and `rolling` on missing.
- Compatibility check returns `Error::Incompatible` unless distro ID is within the supported set, unless the CLI skip flag is used.

## Symlink and backup behavior (`src/symlink/ops.rs`)

- Backup path naming: `.<name>.oxidizr.bak` in the same directory as the target (e.g., `/usr/bin/.ls.oxidizr.bak`).
- Path safety guard: rejects sources/targets containing parent directory components or traversal tokens; returns `Error::ExecutionFailed`.
- Replace with symlink (`replace_file_with_symlink`):
  - No-op if `source == target`.
  - Honors `dry_run` (logs intent and returns success).
  - If target is an existing symlink:
    - Resolves current destination (canonicalizing relative paths using the target’s parent directory).
    - If already pointing to the desired source (canonicalized), no-op.
    - Else, attempts to back up the current resolved destination file (if it exists) to the target’s backup path, preserving permissions, then replaces the symlink to point to the new source.
  - If target is a regular file:
    - Copies the file to backup path, preserves permissions, removes original, ensures parent directory exists.
    - Removes any leftover file and creates a symlink pointing to the source.
  - Emits audit log for created symlink; INFO-level human logs are suppressed while a progress bar is active (see UI behavior).

- Restore (`restore_file`):
  - If backup exists: honors `dry_run`, else removes the target symlink/file (best-effort) and renames the backup into place; emits audit log.
  - If no backup exists: logs a warning and leaves target as-is.

## Logging and audit behavior (`src/logging/*`, `src/main.rs`)

- Initialization: `init_logging()` called at process start.
- Human logs to stderr (tracing subscriber) with level controlled by `VERBOSE` env:
  - `VERBOSE=0 → ERROR`, `1 → INFO` (default), `2 → DEBUG`, `3 → TRACE`.
  - ANSI colors are enabled on TTY.
- Audit logs: JSON Lines written to `/var/log/oxidizr-arch-audit.log` (fallback to `$HOME/.oxidizr-arch-audit.log`).
  - `audit_event()` emits fields: timestamp (RFC3339 millis), component, event, decision, inputs, outputs, exit_code.
  - Free-form strings are passed through a basic secret masker (masks common token/password forms).
- On fatal CLI error, main logs `fatal_error` with the error and exits code 1.

## Progress and UI behavior (`src/ui/progress.rs`, used by experiments/util)

- Progress bars are created on TTY stderr, with a limited redraw rate when not TTY.
- Environment variables `OXI_PROGRESS=1`, `OXIDIZR_PROGRESS=1`, or `PROGRESS=1` can force progress behavior in non-TTY environments.
- While a progress bar is active, per-item symlink INFO logs are suppressed (warnings/errors still show).
- Emits host progress protocol lines to stdout in the form `PB> <current>/<total> <label>` for external renderers.

## Outputs and side effects (filesystem/process)

- Files created/modified:
  - Backups `.<name>.oxidizr.bak` next to targets under `/usr/bin/` and `/usr/sbin/`.
  - Symlinks replacing targets (e.g., `/usr/bin/ls → /usr/lib/uutils/coreutils/ls` or unified dispatcher).
  - Audit log at `/var/log/oxidizr-arch-audit.log` (or `$HOME/.oxidizr-arch-audit.log`).
- External commands executed:
  - `pacman -Sy`, `pacman -Si`, `pacman -Qi`, `pacman -S`, `pacman -R`.
  - `pacman-conf` queries for repo presence.
  - `su - builder -c '<helper args>'` for AUR installs; AUR helper name discovery uses `which`.
  - `which` lookups for binaries during discovery.
- Exit semantics:
  - On non-recoverable errors (e.g., CLI handler returns Err), process exits with code 1 and logs a fatal error.
  - Sub-operations return structured `Error` variants (see Errors below) that propagate to the CLI handler.

## Error model (`src/error.rs` and usages)

- Error variants:
  - `CommandNotFound(String)` — declared but not currently used in code paths inspected; potential future use.
  - `ExecutionFailed(String)` — generic operational failure (package ops, symlink ops, validation errors, verification mismatches).
  - `InvalidImplementation(String)` — declared but not currently used in code paths inspected.
  - `Io(std::io::Error)` — from filesystem/process IO errors.
  - `Incompatible(String)` — distro gating or explicit compatibility failure.
  - `Other(String)` — catch-all.
- Typical error messages include clear expectations vs. observed state (e.g., “Expected: X, Received: Y”) and guidance.

## Invariants and safety constraints

- Do not replace checksum tools as part of the `coreutils` experiment; these are managed by `checksums`.
- When removing `coreutils`, refuse removal if checksum applets appear linked; require disabling `checksums` first.
- Path traversal is rejected by `is_safe_path` before creating symlinks.
- Backups preserve permissions of original files.
- `enable`/`disable` operations are idempotent:
  - Re-running `enable` updates incorrect symlinks in place.
  - `disable` restores from backups when they exist; warns if missing.
- Distro compatibility: only the Arch-family IDs listed are considered supported unless explicitly overridden by flag.
- Root requirement: `enable` and `disable` enforce root unless `--dry-run` is set.

## Environment variables

- `VERBOSE` controls human log level (0–3).
- `OXI_PROGRESS`, `OXIDIZR_PROGRESS`, `PROGRESS` can force progress output behavior in non-TTY contexts.

## Observability and auditability

- Most user-visible steps are logged at INFO with structured spans; detailed tracing is available at higher `VERBOSE`.
- Audit events capture command intents/results and decisions, useful for post-run analysis independent of human logs.

## List-targets and check outputs

- `list-targets`: prints lines of `<experiment>\t<absolute-target-path>` for each target computed by `list_targets()`.
- `check`: prints lines of `<experiment>\tCompatible: <true|false>` using `check_compatible` result.

## Notes drawn from README alignment

- README emphasizes: atomic backups next to targets; audit logging path; root requirement for non-dry-run.
- Default experiments are `coreutils` and `sudo-rs`; `checksums` is opt-in and also used as an explicit suite in tests.
- No SKIPs in product: operations either proceed or fail with explicit errors; compatibility skips are opt-in via flag.

## Known interactions and constraints (implemented semantics)

- Repository gating:
  - `coreutils` and `sudo-rs` require the `extra` repo to be present and the package to exist in repos (`pacman -Si`); else a clear error is returned.
  - `findutils` is expected from AUR (bin variant), requiring an AUR helper; absence yields a clear error.
- Progress UX: a single bar reflects link/restore item progression and the code emits per-item host protocol lines.

## Non-goals in this documentation

- This document does not describe the Go-based test orchestrator (host/runner) behavior or Docker build logic.
