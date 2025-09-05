# Testing Requirements

Below is a precise, file-by-file explanation of every test under tests/, including what each one sets up, runs, and asserts, plus any cleanup and noteworthy requirements the test enforces.

This document outlines the expected test coverage and scenarios for the refactored architecture under `src/experiments/` and `src/utils/`, tailored for Arch Linux (pacman + optional AUR helper such as paru/yay).

## Scope

- Validate experiment enable/disable flows for both unified-binary and per-binary strategies.
- Validate distro compatibility gating and the `--no-compatibility-check` skip flag.
- Validate package manager interactions for Arch (pacman/AUR), with room to add Ubuntu/apt in future.
- Validate backup/restore and symlink semantics.
- Validate CLI parsing and selection logic for `--all`, `--experiments`, `--experiment` (deprecated), `--assume-yes`, `--no-update`, and `--no-compatibility-check`.

## Unit Tests

- Experiments
  - UutilsExperiment (unified):
    - Given bin directory with sample files, ensure `list_targets()` yields `/usr/bin/<name>` targets.
    - `enable()`: creates backups for existing targets, then symlinks all targets to the unified binary path.
    - `disable()`: restores backups and removes package.
  - UutilsExperiment (non-unified):
    - `enable()`: symlinks each bin file individually to the corresponding target.
    - `disable()`: restores each target.
  - SudoRsExperiment:
    - `enable()`: installs `sudo-rs` and replaces `sudo`, `su`, `visudo` with symlinks; `disable()` restores and removes package.

- Compatibility gating
  - With `distribution().id = "arch", release = "rolling"`, `check_compatible()` returns true; otherwise false.
  - `--no-compatibility-check` allows `enable()` to proceed regardless of gating result.

- Worker behaviors
  - `replace_file_with_symlink()` creates backup at `.<name>.oxidizr.bak`, preserves permissions, and is idempotent for existing symlinks.
  - `restore_file()` restores when backup exists and warns when absent.
  - `which()` resolution preferred; falls back to `/usr/bin/<name>`.

## CLI Tests

- Parsing
  - `--all` selects all from registry.
  - `--experiments coreutils,findutils` selects both in any order.
  - Deprecated `--experiment coreutils` still works to select one.
  - `--no-update` prevents package list updates.
  - `--assume-yes` suppresses confirmation prompts.
  - `--no-compatibility-check` allows enable on incompatible distro.

- Execution wiring (with mock worker)
  - Enable/Disable iterates over selection and dispatches to each experiment.
  - ListTargets prints experiment name and each target path.
  - Check prints compatibility per experiment.

## Test Infrastructure

- Introduce `utils::worker_mock` behind `#[cfg(test)]`:
  - In-memory store for files, backups, and symlinks.
  - Captures invoked commands and package ops (e.g., `pacman -S/Rns -Sy`, `paru -S -Qi`).
  - Provides `distribution()` stubbing and `which()` resolution rules.

## Non-goals (for now)

- Ubuntu/apt path is not implemented yet; add when Ubuntu gating is introduced.
- E2E tests that make real system changes; rely on dry-run and mocks in CI.

## Coverage Target

- Aim for high-confidence unit coverage on experiment flows and worker semantics. Minimum: 80% coverage on experiments and worker modules.

Test harness conventions these files rely on
- Runner: These YAML files are Spread tasks. The execute block is a shell script. restore runs after execute for cleanup (when not in CI).
- Assertions: MATCH and NOMATCH are assertion helpers provided by the harness. A command like cmd | MATCH "text" passes when “text” appears in output; NOMATCH passes when “text” does not appear.
- Helpers and fixtures:
  - ${SPREAD_PATH}/tests/lib/uutils.sh and sudo-rs.sh define ensure_* functions that check for package presence, symlinks, backups, and outputs.
  - ${SPREAD_PATH}/tests/lib/rust-coreutils-bins.txt enumerates binaries expected to be handled under rust-coreutils.
  - tests/lib/cloud-config.yaml creates the non-root user “spread” with passwordless sudo for the environment.

tests/disable-all/task.yaml
- Purpose: Verify that “disable everything” actually removes all managed experiments after they were enabled.
- Setup:
  - source tests/lib/uutils.sh and tests/lib/sudo-rs.sh to get ensure_* helpers.
- Actions:
  - oxidizr enable --yes --all
  - oxidizr disable --yes --all
- Assertions:
  - ensure_coreutils_absent:
    - Package query must indicate uutils-coreutils is not installed (`paru -Qi`/`yay -Qi`/`pacman -Qi` should fail).
    - /usr/bin/date must not be a symlink to /usr/bin/coreutils; no backup file .date.oxidizr.bak should remain.
    - date --help must contain “GNU” (verifies GNU coreutils restored).
  - ensure_findutils_absent:
    - Package query must indicate uutils-findutils is not installed.
    - /usr/bin/find must not link to /usr/lib/cargo/bin/findutils/find; no .find.oxidizr.bak file should remain.
    - find --help must contain “GNU”.
    - Likewise for xargs: not symlinked, no backup; xargs --help shows “GNU”.
  - ensure_diffutils_absent:
    - Package query must indicate uutils-diffutils is not installed.
    - It also checks that /usr/bin/find is not linked to a diffutils path and that “find” help shows GNU (note: the test asserts absence via “find”; this mirrors the file’s current content).
  - ensure_sudors_absent:
    - Package query must indicate sudo-rs is not installed.
    - /usr/bin/sudo, /usr/bin/su, /usr/sbin/visudo must not be symlinks to their /usr/lib/cargo/bin counterparts; their backup files (.sudo/.su/.visudo.oxidizr.bak) must not remain.
    - sudo --version and su --version must not contain “sudo-rs”/“su-rs”.
- Cleanup: none (the test already ends in the “all disabled” state).

tests/disable-default/task.yaml
- Purpose: Verify disabling the default experiment set (not “all”) removes only the defaults.
- Setup: source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes (default set)
  - oxidizr disable --yes (default set)
- Assertions:
  - ensure_coreutils_absent (same checks as above)
  - ensure_sudors_absent (same checks as above)
- Implicit requirement: “default” includes at least coreutils and sudo-rs; findutils/diffutils are not checked here and thus are not implied defaults.
- Cleanup: none (final state already disabled).

tests/disable-in-german/task.yaml
- Purpose: Ensure behavior is correct under a non-English locale when selectively disabling a subset.
- Setup:
  - Enable the de_DE.UTF-8 locale by uncommenting in /etc/locale.gen then locale-gen.
  - Export LC_ALL, LANGUAGE, LANG=de_DE.UTF-8 for the session.
  - source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes (default set)
  - oxidizr disable --yes --experiments coreutils (selective disable in a non-English locale)
  - Reset locale to en_GB.UTF-8 for subsequent assertions.
- Assertions:
  - ensure_coreutils_absent (GNU date restored, no rust-coreutils package, no symlink/backups).
  - ensure_sudors_installed (sudo-rs still enabled):
    - package query for sudo-rs reports installed
    - /usr/bin/sudo -> /usr/lib/cargo/bin/sudo; /usr/bin contains .sudo.oxidizr.bak; sudo --version shows “sudo-rs”
    - /usr/bin/su -> /usr/lib/cargo/bin/su; .su.oxidizr.bak exists; su --version shows “su-rs”
    - /usr/sbin/visudo -> /usr/lib/cargo/bin/visudo; .visudo.oxidizr.bak exists
- Cleanup (restore): If not CI, run oxidizr disable --yes --all.

tests/disable-partial/task.yaml
- Purpose: Selectively disable a subset (coreutils) while leaving others (sudo-rs) enabled.
- Setup: source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes (default set)
  - oxidizr disable --yes --experiments coreutils
- Assertions:
  - ensure_coreutils_absent (as above).
  - ensure_sudors_installed (as above).
- Cleanup (restore): If not CI, run oxidizr disable --yes --all.

tests/enable-all/task.yaml
- Purpose: Verify enabling “all” activates all managed experiments.
- Setup: source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes --all
- Assertions:
  - ensure_coreutils_installed:
    - package query for uutils-coreutils reports installed
    - For every binary in tests/lib/rust-coreutils-bins.txt:
      - which $bin gives the path; that path must be a symlink to /usr/bin/coreutils (multi-call frontend).
      - A backup file .$bin.oxidizr.bak should be present alongside (the helper tolerates printing a note if absent).
      - $bin --help must NOT include the GNU coreutils URL, demonstrating the Rust replacement is active.
  - ensure_findutils_installed:
    - package query for uutils-findutils reports installed
    - /usr/bin/find -> /usr/lib/cargo/bin/findutils/find; .find.oxidizr.bak present; find --help must not show GNU URL.
    - /usr/bin/xargs -> /usr/lib/cargo/bin/findutils/xargs; .xargs.oxidizr.bak present; xargs --help must not show GNU URL.
  - ensure_diffutils_installed (conditionally):
    - If supported on this Arch snapshot, package query for uutils-diffutils reports installed
    - /usr/bin/diff -> /usr/lib/cargo/bin/diffutils/diff; .diff.oxidizr.bak present; diff --help must not show GNU URL.
  - ensure_sudors_installed (as described above).
- Cleanup (restore): If not CI, oxidizr disable --yes --all.

tests/enable-default/task.yaml
- Purpose: Verify enabling the default experiment set.
- Setup: source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes (defaults)
- Assertions:
  - ensure_coreutils_installed (as above).
  - ensure_sudors_installed (as above).
- Cleanup (restore): If not CI, oxidizr disable --yes --all.
- Implicit requirement: “default” includes coreutils and sudo-rs.

tests/enable-no-compatibility-check/task.yaml
- Purpose: Ensure enabling with the compatibility gate bypass actually proceeds and results are installed.
- Setup: source uutils.sh and sudo-rs.sh.
  - oxidizr enable --yes --no-compatibility-check --experiments coreutils findutils sudo-rs
- Assertions:
  - ensure_coreutils_installed, ensure_findutils_installed, ensure_sudors_installed (as detailed above, with pacman/paru query semantics).
- Cleanup (restore):
  - If not CI, oxidizr disable --yes --experiments coreutils findutils sudo-rs --no-compatibility-check
  - Requirements this enforces:
  - The flag --no-compatibility-check must bypass normal version/compatibility gating so the enable proceeds and the artifacts (packages, symlinks, backups) appear.

tests/enable-partial/task.yaml
- Purpose: Enable only a specific subset (coreutils) and verify the others remain untouched/absent.
- Setup: source uutils.sh and sudo-rs.sh.
- Actions:
  - oxidizr enable --yes --experiments coreutils
- Assertions:
  - ensure_coreutils_installed (present).
  - ensure_findutils_absent, ensure_diffutils_absent, ensure_sudors_absent (not present; GNU tools show for find/xargs/date; no symlinks/backups).
- Cleanup (restore): If not CI, oxidizr disable --yes --all.

tests/non-root/task.yaml
- Purpose: Verify the application refuses to run as non-root with a clear error.
- Actions:
  - sudo -u spread oxidizr enable --yes 2>&1 | MATCH "Error: This program must be run as root"
  - sudo -u spread oxidizr disable --yes 2>&1 | MATCH "Error: This program must be run as root"
- Assertions:
  - Both commands must emit that exact error string when run as the unprivileged “spread” user.
  - Ensures both enable and disable subcommands are protected by the root check.

Library files referenced by tests

tests/lib/uutils.sh
- ensure_coreutils_installed:
  - Package uutils-coreutils must be reported installed via `paru -Qi`/`yay -Qi` (fallback `pacman -Qi`).
  - For each binary in tests/lib/rust-coreutils-bins.txt:
    - which resolves to a path that must be a symlink to /usr/bin/coreutils.
    - A backup file .$bin.oxidizr.bak is expected (the helper prints a note if absent).
    - The command’s help must not reference the GNU coreutils URL.
- ensure_coreutils_absent:
  - uutils-coreutils not installed; /usr/bin/date not symlinked; no .date.oxidizr.bak; date --help contains “GNU”.
- ensure_findutils_installed:
  - uutils-findutils installed.
  - /usr/bin/find -> /usr/lib/cargo/bin/findutils/find; .find.oxidizr.bak exists; find --help not GNU URL.
  - /usr/bin/xargs -> /usr/lib/cargo/bin/findutils/xargs; .xargs.oxidizr.bak exists; xargs --help not GNU URL.
- ensure_findutils_absent:
  - uutils-findutils not installed; find/xargs not symlinked; no backups; their --help shows “GNU”.
- ensure_diffutils_installed:
  - Only when supported on this Arch snapshot:
    - uutils-diffutils installed.
    - /usr/bin/diff -> /usr/lib/cargo/bin/diffutils/diff; .diff.oxidizr.bak exists; diff --help not GNU URL.
- ensure_diffutils_absent:
  - uutils-diffutils not installed.
  - Also verifies that /usr/bin/find is not linked to a diffutils path, no .find.oxidizr.bak, and find --help shows “GNU” (this reflects the current assertions in the file).

tests/lib/sudo-rs.sh
- ensure_sudors_installed:
  - sudo-rs installed.
  - /usr/bin/sudo -> /usr/lib/cargo/bin/sudo; .sudo.oxidizr.bak present; sudo --version shows “sudo-rs”.
  - /usr/bin/su -> /usr/lib/cargo/bin/su; .su.oxidizr.bak present; su --version shows “su-rs”.
  - /usr/sbin/visudo -> /usr/lib/cargo/bin/visudo; .visudo.oxidizr.bak present.
- ensure_sudors_absent:
  - sudo-rs not installed; none of the above symlinks/backups exist; versions not branded “sudo-rs/su-rs”.

tests/lib/rust-coreutils-bins.txt
- A canonical list of binaries that must be redirected to the Rust coreutils multi-call binary (/usr/bin/coreutils) when uutils-coreutils is enabled. The ensure_coreutils_installed function loops over this list to validate the symlinks and non-GNU help output.

tests/lib/cloud-config.yaml
- Cloud-init snippet enabling SSH password auth and creating the “spread” user (password placeholder SPREAD_PASSWORD) with passwordless sudo. Ensures the test runner can exercise both root and non-root behaviors (e.g., tests/non-root).

Key behavioral guarantees these tests enforce
- Root requirement: All privileged operations must fail for non-root with the exact error “This command must be run as root”.
- Default vs all:
  - Default enable/disable must at least affect: uutils-coreutils and sudo-rs.
  - “All” must additionally include findutils and (on supported releases) diffutils.
- Selective operations:
  - --experiments coreutils targets only coreutils; other experiments remain as they were.
- Compatibility bypass:
  - --no-compatibility-check must actually bypass gating and yield successful installation of the specified experiments.
- System effects and reversibility:
  - Enabling installs the corresponding pacman package(s) or AUR package(s) (via configured helper), rewires system binaries via symlinks to Rust/uutils or sudo-rs paths, and creates backup files .<name>.oxidizr.bak next to originals.
  - Disabling removes packages, restores original binaries (no symlinks), and removes backup files; GNU-provided tools must be observable via their help output containing “GNU”.
- Locale robustness:
  - Operations must succeed correctly when the process locale is set to de_DE.UTF-8 (test switches back to English for assertions).