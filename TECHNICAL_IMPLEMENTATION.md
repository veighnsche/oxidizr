Here’s a deep-dive of every file in src at commit 132c889d, with responsibilities, key APIs, behavior, and notable implementation details. You can browse the tree here: https://github.com/jnsgruk/oxidizr/tree/132c889dc31fc84e9e56ed496746b807ab003f5b/src

## Project requirements compliance (current repository: rust_coreutils_switch)

This repository is a scaffold inspired by the referenced implementation but targets an Arch-like environment (AUR helpers, pacman) rather than Ubuntu. Based on the current sources under `src/`:

- Files present: `src/main.rs`, `src/lib.rs`, `src/cli.rs`, `src/error.rs`, `src/experiment/`, `src/worker/`, `src/core/`
- Files absent (from the referenced deep-dive): `src/experiments/mod.rs`, `src/experiments/uutils.rs`, `src/experiments/sudors.rs`, `src/utils/mod.rs`, `src/utils/command.rs`, `src/utils/worker.rs`, `src/utils/worker_mock.rs`

Compliance overview:

- CLI entrypoint exists: `src/main.rs` and `src/cli.rs` provide argument parsing with `clap`, root checks, prompts, and subcommands `Enable|Disable|Check|ListTargets`. Partially compliant with described behavior but tailored to Arch (`paru`/`yay`) rather than apt-based systems.
- Experiment abstraction: Present as `src/experiment` (singular) with a `UutilsExperiment` scaffold. Missing multi-experiment registry and type-erased enum (`Experiment`) that unifies uutils families and `sudo-rs`.
- System/Worker abstraction: Present under `src/worker` but not split into `utils` with `command.rs` and `worker.rs` as in the deep-dive; mock worker is not present behind `#[cfg(test)]` in this repo.
- Distro gating: The scaffold assumes a "rolling" release and Arch-like environment; the Ubuntu gating and release allow-list from the deep-dive is not implemented.
- Utilities/helpers: `utils::vecs_eq`, `utils::command`, and the exact backup/symlink semantics from the deep-dive are not present. Backup/restore behavior should be verified in `worker` once aligned.
- Tests: The testing requirements doc `TESTING_REQUIREMENTS.MD` is currently empty, and there is no integrated `utils::worker_mock`-style test harness. Unit tests in `src/lib.rs` are minimal and do not cover enable/disable flows.

Conclusion: The project is a functional scaffold but does not yet meet the full requirements and structure defined in the referenced technical implementation. Key gaps are the experiments registry, utils/worker split (with command wrapper, backup/symlink semantics), mock-based tests, and Ubuntu-based compatibility gating.

## Migration plan to match the technical implementation structure

Objective: Align `rust_coreutils_switch` to the structure and behaviors described in this document while keeping Arch support as a configurable variant.

Step-by-step plan:

1) Module layout refactor
   - Create `src/experiments/` with:
     - `mod.rs`: registry exposing `Experiment<'a>`, `all_experiments(&impl Worker)`, and re-exports.
     - `uutils.rs`: generic `UutilsExperiment<'a>` with fields: `name`, `package`, `supported_releases`, `unified_binary: Option<PathBuf>`, `bin_directory: PathBuf>`; methods: `name`, `enable`, `disable`, `check_compatible`, `supported_releases`, `check_installed`.
     - `sudors.rs`: `SudoRsExperiment<'a>` for `sudo-rs` install and link management.
   - Introduce `src/utils/` with:
     - `mod.rs`: re-export `command` and `worker`, define `Distribution { id, release }`, and helper `vecs_eq`.
     - `command.rs`: small command wrapper `Command::build`, `Command::command()` for logging/mocking.
     - `worker.rs`: `Worker` trait and concrete `System` implementation with `run`, `which`, `list_files`, `install/remove/update/check_installed`, file backup/restore/symlink helpers.
     - `worker_mock.rs` behind `#[cfg(test)]` providing an in-memory `MockSystem`.
   - Update `src/lib.rs` to export `experiments` and `utils` modules and re-exports used by `cli` and tests.

2) Behavior alignment
   - Implement Ubuntu compatibility gating as default behavior (per deep-dive) with an override flag to skip checks. Preserve Arch support by parameterizing package operations via `Worker` implementations or feature flags:
     - For apt-based systems (Ubuntu): use `apt-get` commands and `dpkg-query` for `check_installed`.
     - For Arch-based systems: use `pacman`/AUR helper commands. Choose at runtime based on `Distribution.id` or via a `--package-manager` flag.
   - Implement unified vs non-unified symlink strategy in `UutilsExperiment`:
     - Coreutils: symlink `/usr/bin/{date,sort,...}` -> unified binary path (e.g., `/usr/bin/coreutils`).
     - Findutils/diffutils: per-file symlinks from the uutils bin directory.
   - Ensure backup/restore semantics: hidden `.name.oxidizr.bak` file naming, permissions preserved, idempotent symlink creation.

3) CLI refactor
   - Keep `clap`-based `Cli`, but route actions through the `Experiment` enum:
     - Add `--all`, `--experiments`, `--yes`, and `--no-compatibility-check` aligned with the deep-dive.
     - Add selection logic `selected_experiments(all, selected, &system)` using `utils::vecs_eq`.
     - Add `default_experiments()` returning sorted defaults (e.g., `["coreutils", "sudo-rs"]`).
   - Maintain `--dry-run`, Arch flags like `--aur-helper`, but gate them under Arch mode or when `Distribution.id == "Arch"`.

4) Testing
   - Populate `TESTING_REQUIREMENTS.MD` with unit test coverage expectations:
     - Enable/disable flows call correct package manager commands.
     - Backup/restore lists and symlink targets are correct (both unified and non-unified cases).
     - Distro compatibility gating behavior, including skip flag.
   - Add unit tests using `utils::worker_mock` to simulate filesystem and package operations.
   - Add integration-style tests for CLI argument parsing and selection logic.

5) Migration execution order
   - Phase 1: Introduce `utils::worker` and migrate current `worker` code; add `utils::command` wrapper.
   - Phase 2: Move `UutilsExperiment` into `experiments/uutils.rs`; add `experiments/mod.rs` and wire registry; keep existing CLI temporarily calling the new locations.
   - Phase 3: Add `experiments/sudors.rs` and initial support for `sudo-rs` experiment behind a flag.
   - Phase 4: Refactor `cli.rs` to add selection logic and Ubuntu gating; add `--no-compatibility-check` and `--all/--experiments`.
   - Phase 5: Introduce `utils::worker_mock` and expand tests; fill `TESTING_REQUIREMENTS.MD`.
   - Phase 6: Clean-up: remove legacy paths (`src/experiment`, `src/worker`) after ensuring imports updated.

6) Risk and rollback
   - Ensure `System::backup_file` and `restore_file` are robust and idempotent. Keep `--dry-run` for safe previews.
   - During rollout, prefer not to remove legacy code until tests pass. Provide a `Disable` command that fully restores originals.

Deliverables
   - New module structure under `src/experiments` and `src/utils`.
   - Updated `cli` and `lib` wiring.
   - Tests and mock system with coverage of enable/disable logic and distro gating.

---

The following sections remain as a reference specification for the target architecture and behavior.
  - Link: main.rs
  - Role: CLI entrypoint and orchestrator for enabling/disabling experiments (Rust replacements for coreutils/findutils/diffutils/sudo).
  - External crates: anyhow, clap, clap_verbosity_flag, inquire, tracing, tracing_subscriber, uzers.
  - Key structures:
  - Args (clap Parser): global flags:
  - --yes/-y: skip confirmation prompts
      - --all/-a: operate on all known experiments
      - --no-compatibility-check: skip distro/release gating (dangerous)
      - --experiments/-e <list>: filter experiments (defaults to default_experiments())
    - Commands: Enable | Disable
  - Runtime flow (main):
    - Parses args (Args::parse()).
    - Safety: requires root (uzers::get_current_uid() == 0).
    - Logging: initializes tracing_subscriber with verbosity via clap_verbosity_flag.
    - Constructs System implementing Worker (System::new()?).
    - Distro gating:
      - If no_compatibility_check = false: require Ubuntu (system.distribution()?.id == "Ubuntu"), else bail.
      - If skipping check and not Ubuntu, logs warn about instability.
    - Selects experiments via selected_experiments(all, experiments, &system).
    - Dispatch:
      - Enable: confirm_or_exit, apt-get update via system.update_package_lists(), then e.enable(no_compatibility_check) for each.
      - Disable: confirm_or_exit, then e.disable() for each.
  - Helper functions:
    - enable(system, experiments, yes, no_compatibility_check) -> Result<()>:
      - Confirmation, apt update, then enabling experiments (respecting per-experiment compatibility unless globally skipped).
    - disable(experiments, yes) -> Result<()>:
      - Confirmation, then disabling experiments (per-experiment no-ops if not installed).
    - selected_experiments(all, selected: Vec<String>, system) -> Vec<Experiment>:
      - Builds all_experiments(system) then filters:
        - If all = true: uses the full set; warns if user provided a non-default selection (compares with vecs_eq to ignore order).
        - If all = false: default to default_experiments() when none provided; filters by e.name().
    - confirm_or_exit(yes: bool):
      - Skips prompt if yes is true.
      - Otherwise prompts "Continue?" with default false and a help message about risks; exits with code 1 if declined or prompt fails.
    - default_experiments() -> Vec<String>:
      - Returns sorted ["coreutils", "sudo-rs"] (ensures deterministic order for vecs_eq).

- src/experiments/mod.rs
  - Link: experiments/mod.rs
  - Role: Experiment registry and type-erased interface unifying multiple experiment families.
  - Key items:
    - Modules: sudors, uutils; re-exports SudoRsExperiment, UutilsExperiment.
    - Enum Experiment<'a> { Uutils(UutilsExperiment<'a>), SudoRs(SudoRsExperiment<'a>) }
    - Polymorphic methods:
      - name(&self) -> String
      - enable(&self, no_compatibility_check: bool) -> Result<()>
        - Checks compatibility (unless skipped), warns and skips if unsupported.
      - disable(&self) -> Result<()>
        - Skips restore if not installed (warn).
      - check_compatible(&self) -> bool
      - supported_releases(&self) -> Vec<String>
      - check_installed(&self) -> bool
    - Catalog: all_experiments(system: &impl Worker) -> Vec<Experiment>:
      - Uutils("coreutils"): package rust-coreutils; releases: 24.04, 24.10, 25.04; unified_binary Some("/usr/bin/coreutils"); bin_directory "/usr/lib/cargo/bin/coreutils"
      - Uutils("diffutils"): package rust-diffutils; releases: 24.10, 25.04; unified_binary Some("/usr/lib/cargo/bin/diffutils/diffutils"); bin_directory "/usr/lib/cargo/bin/diffutils"
      - Uutils("findutils"): package rust-findutils; releases: 24.04, 24.10, 25.04; unified_binary None; bin_directory "/usr/lib/cargo/bin/findutils"
      - SudoRs: sudo replacement (see sudors.rs).
  - Notes:
    - The unified_binary option controls whether all utility names symlink to a single binary (e.g., coreutils), vs linking each tool individually from a bin directory.

- src/experiments/sudors.rs
  - Link: experiments/sudors.rs
  - Role: Experiment for installing and wiring sudo-rs as a replacement for sudo/su/visudo.
  - Constants:
    - PACKAGE = "sudo-rs"
  - SudoRsExperiment<'a> { system: &'a dyn Worker }
  - Behavior:
    - supported_releases() -> Vec<String>: ["24.04","24.10","25.04"]
    - check_compatible(): compares Worker::distribution().release against supported list.
    - check_installed(): Worker::check_installed(PACKAGE) with unwrap_or(false).
    - name() -> "sudo-rs"
    - enable() -> Result<()>:
      - Logs "Installing and configuring sudo-rs".
      - system.install_package("sudo-rs").
      - For each path in sudors_files():
        - Derive filename (e.g., "sudo", "su", "visudo").
        - Resolve existing path via system.which(filename) or fallback to /usr/bin/<filename>.
        - Replace target with symlink to the provided cargo-installed path via system.replace_file_with_symlink(source, target).
      - Files replaced: /usr/lib/cargo/bin/{su,sudo,visudo}
    - disable() -> Result<()>:
      - For each filename, resolves the existing target and calls system.restore_file(target).
      - Removes package with system.remove_package("sudo-rs").
  - Test notes (with MockSystem):
    - Verifies package installation/removal commands.
    - Ensures backups created for existing binaries and symlinks point to expected sources/targets.
    - Confirms restore list and the behavior when restoring after installed.

- src/experiments/uutils.rs
  - Link: experiments/uutils.rs
  - Role: Generic experiment for uutils-provided Rust replacements (coreutils, diffutils, findutils).
  - UutilsExperiment<'a> fields:
    - name: String (e.g., "coreutils")
    - system: &'a dyn Worker
    - package: String (e.g., "rust-coreutils")
    - supported_releases: Vec<String>
    - unified_binary: Option<PathBuf> (e.g., Some("/usr/bin/coreutils") for coreutils)
    - bin_directory: PathBuf (e.g., "/usr/lib/cargo/bin/coreutils")
  - Behavior:
    - new(name, system, package, supported_releases, unified_binary, bin_directory) initializes fields (clones and converts release strings).
    - check_compatible(): compares distribution.release against supported list.
    - supported_releases(): returns clone of configured releases.
    - check_installed(): defers to Worker::check_installed(package).
    - name(): returns configured name.
    - enable() -> Result<()>:
      - Logs "Installing and configuring <package>".
      - system.install_package(package).
      - Gets the list of tool files via system.list_files(bin_directory).
      - For each file:
        - Determine filename (e.g., "date", "sort").
        - Resolve existing path via system.which(filename) or default to /usr/bin/<filename>.
        - If unified_binary is Some(p): all symlinks target that single p; else symlink file-by-file from the listed bin_directory.
      - Backups and file replacement semantics handled by Worker implementation.
    - disable() -> Result<()>:
      - Lists files again, resolves existing targets, and calls system.restore_file for each.
      - Logs removal and system.remove_package(package).
  - Test notes:
    - Covers both unified and non-unified configurations:
      - Coreutils (unified): symlinks /usr/bin/{date,sort,...} -> /usr/bin/coreutils.
      - Findutils (non-unified): symlinks /usr/bin/{find,xargs} -> /usr/lib/cargo/bin/findutils/{find,xargs}.
    - Validates apt install/remove calls, backup/restore, and symlink lists.

- src/utils/mod.rs
  - Link: utils/mod.rs
  - Role: Utilities module root; re-exports and shared types/helpers.
  - Modules:
    - mod command; mod worker;
    - pub use command::*; pub use worker::*;
    - #[cfg(test)] mod worker_mock; pub use worker_mock::tests::* for tests.
  - Types:
    - Distribution { id: String, release: String }: Linux distribution metadata used by experiments for compatibility.
  - Helpers:
    - vecs_eq<T: Hash + Eq>(v1, v2) -> bool:
      - Unordered equality: compare lengths; then check every element of v2 is in a HashSet of v1’s elements.
      - Used in CLI logic to detect when --all is set but user supplied a custom experiment list (to warn and ignore).

- src/utils/command.rs
  - Link: utils/command.rs
  - Role: Minimal command wrapper for Worker::run interface and for test logging/mocking.
  - Command { command: String, args: Vec<String> }
  - Methods:
    - build(command: &str, args: &[&str]) -> Self: converts args to owned Strings.
    - command(&self) -> String: emits a "cmd arg1 arg2 ..." string (used heavily in mocks and debug logs).

- src/utils/worker.rs
  - Link: utils/worker.rs
  - Role: System abstraction defining all OS-touching behaviors and the real implementation (System).
  - Trait Worker:
    - distribution() -> Result<Distribution>:
      - Default implementation invokes run(lsb_release -is) and run(lsb_release -rs).
    - run(&self, cmd: &Command) -> Result<Output>: required by implementors.
    - list_files(&self, directory: PathBuf) -> Result<Vec<PathBuf>>: required.
    - which(&self, binary_name: &str) -> Result<PathBuf>: required.
    - Package helpers (with default implementations using run):
      - install_package(apt-get install -y <pkg>)
      - remove_package(apt-get remove -y <pkg>)
      - update_package_lists(apt-get update)
      - check_installed(dpkg-query -s <pkg>): returns Ok(true) on success, Ok(false) otherwise.
    - File ops (required):
      - replace_file_with_symlink(source, target)
      - backup_file(file)
      - restore_file(file)
      - create_symlink(source, target)
  - System: concrete implementation
    - run(): uses std::process::Command; bails with stderr if status is non-success; logs debug with full command string.
    - list_files(directory):
      - Validates path exists and is a directory.
      - Collects entry paths into Vec<PathBuf>.
    - which(): uses which::which crate.
    - replace_file_with_symlink(source, target):
      - If target exists:
        - If it’s already a symlink, skip idempotently.
        - Else backup_file(target) and remove original file.
      - Then create_symlink(source, target).
    - backup_file(file):
      - Computes backup filename via backup_filename(file) => "/parent/.<name>.oxidizr.bak" (hidden and suffixed).
      - Copies file to backup, then applies the same permissions (preserves SUID/SGID/sticky bits).
    - restore_file(file):
      - Computes matching backup filename; renames it back if present, otherwise warn and continue.
    - create_symlink(source, target):
      - Ensures target is removed if present, then symlink(source -> target).
    - Internal helpers:
      - backup_filename(file: &Path) -> PathBuf:
        - E.g., "/etc/hosts" => "/etc/.hosts.oxidizr.bak"; "./config" => ".config.oxidizr.bak"
      - remove_file_if_exists(file: &PathBuf) -> Result<()>
    - Tests:
      - test_backup_filename validates the naming for regular, no-parent, and dotfile cases.
  - Logging:
    - Uses tracing debug/trace/warn to provide detailed behavior.

- src/utils/worker_mock.rs (test-only)
  - Link: utils/worker_mock.rs
  - Role: A fully in-memory Worker mock for unit tests (gated under #[cfg(test)]).
  - Availability:
    - Declared as #[cfg(test)] pub mod tests { ... }
    - Re-exported by utils/mod.rs in tests so modules can use MockSystem directly.
  - MockSystem:
    - Tracks:
      - commands: Vec<String> of built command strings
      - files: HashMap<PathBuf, (contents: String, primary_in_path: bool)>
      - installed_packages: Vec<String>
      - created_symlinks: Vec<(src, dst)>
      - restored_files: Vec<String>
      - backed_up_files: Vec<String>
      - mocked_commands: HashMap<command_string, stdout>
    - Defaults: sets mocked lsb_release outputs to Ubuntu 24.04.
    - Helpers:
      - mock_files([(path, contents, primary_in_PATH)]) to populate virtual FS and PATH resolution preference.
      - mock_install_package(pkg) to mark as “installed”.
      - mock_command(cmd, stdout) to stub run() outputs.
  - Worker impl (MockSystem):
    - run(): records command; returns mocked Output using mocked_commands (stdout only).
    - check_installed(pkg): uses installed_packages.
    - list_files(directory): returns keys that start with the given directory string.
    - which(binary_name): searches files by file_name == binary_name and primary_in_PATH == true; bails if not found.
    - replace_file_with_symlink(source, target): backs up if the target exists in the mocked file map; then delegates to create_symlink.
    - create_symlink(): pushes into created_symlinks.
    - backup_file(), restore_file(): append to respective vectors.