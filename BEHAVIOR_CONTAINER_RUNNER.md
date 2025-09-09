# Container Runner Behavior (in-container Go runner)

This document describes the behavior of the in-container runner that executes inside the Docker image under `test-orch/container-runner/`. It focuses on stage sequencing, dependency installs, build, YAML suite execution, assertions, and artifact/log collection. It also calls out any behaviors that could harden or mask results.

## Scope

- In scope: Go-based container runner under `test-orch/container-runner/` and the test image `test-orch/docker/Dockerfile` it depends on.
- Out of scope: Rust product behavior (documented in `BEHAVIOR.md`) and the host orchestrator under `test-orch/host-orchestrator/`.

## CLI and invocation

- The runner binary supports an optional `internal-runner` token as `argv[1]`; when present, it is stripped before parsing flags. This keeps backward compatibility with older entrypoints.
- Flag `--test-filter <name>` maps to environment variable `TEST_FILTER=<name>` for the process, used by the YAML runner.

## Stage sequencing (run order)

The runner entrypoint is `runInContainer()` in `test-orch/container-runner/runner.go`.

1. __Preflight__ (`setup/preflight.go`)
   - Requires `pacman` to exist in the base image.
   - Optional network probe if `curl` is present.
   - Enforces availability of German locale definition and listing (`/usr/share/i18n/locales/de_DE` and `locale -a` includes `de_DE.*`). Fails hard if missing (policy: no locale-based skips).
   - Logs a context summary: distro ID, de_DE presence, and whether `paru`/`yay` are preinstalled.
2. __Workspace staging__ (`setup/workspace.go`)
   - Verifies `/workspace` bind mount exists and is writable by creating/removing a temp file. All subsequent stages operate in-place under `/workspace`.
3. __System dependencies__ (`setup/deps.go`)
   - Removes `cachyos-extra.db` if present to normalize repo behavior in CachyOS-based images.
   - `pacman -Syy` followed by install of `base-devel sudo git curl rustup which findutils` (with `--needed`).
4. __Users__ (`setup/users.go`)
   - Ensures `builder` and `spread` users exist.
   - Ensures `/home/builder` ownership is `builder:builder` (mounts may leave it owned by root).
   - Writes `/etc/sudoers.d/99-builder` enabling passwordless sudo for `builder`.
5. __AUR helper__ (`setup/users.go`, `installAurHelper()`)
   - Detects preinstalled `paru` via multiple methods; if absent, builds `paru-bin` as `builder` under `/home/builder/build` and installs it as root. Idempotent with `git pull --rebase` and wildcard package install.
6. __Rust toolchain__ (`setup/rust.go`)
   - Sets `rustup` profile to `minimal`. Ensures `stable` default toolchain for root.
7. __Build product__ (`setup/build.go`)
   - Operates under `/workspace`.
   - Build-stamp optimization: if `/usr/local/bin/.oxidizr_build_hash` matches current `git rev-parse HEAD` and the binary exists, skips rebuild; otherwise `cargo build --release` and installs `oxidizr-arch` to `/usr/local/bin/`.
   - Default parallelism `CARGO_BUILD_JOBS=2` unless overridden.
   - Set `FORCE_RUST_REBUILD=1` to bypass the build-stamp optimization and force a full rebuild.
8. __Rust unit tests__ (`runner.go`)
   - Runs `cargo test -q` in `/workspace`. Fails the run on non-zero exit.
9. __YAML suites__ (`yamlrunner/`) and __Assertions__ (`assertions/`)
   - Executes YAML suites (see below), then runs additional in-container assertions.
10. __Analytics report__ (`analytics/`)
    - Writes `TEST_DOWNLOADS_ANALYTICS.md` to `/workspace` (or `TEST_DOWNLOADS_ANALYTICS-<distro>.md` if `ANALYTICS_DISTRO` set).

## YAML suite executor behavior (`yamlrunner/`)

- __Discovery__: Walks all subdirectories under `/workspace/tests/` and collects every `task.yaml`. The list is sorted. Policy: no code-level skips during discovery.
- __Filtering__: If `TEST_FILTER` env is set (or `--test-filter` flag was passed), runs only the suite whose directory basename matches.
- If `TEST_FILTER` is set but matches no discovered suites, the run errors early with a clear message.
- __Distro gating__: `distro-check` field is enforced via `util.ShouldRunOnDistro()`. If current distro ID is not in the list, the suite returns an error (fail-on-skip policy).
- __Execution__: Runs the `execute:` block if present via a generated temporary `bash` script with:
  - `set -Eeuo pipefail` and traps on `ERR` and `EXIT` to echo failing command/line or exit code for visibility.
  - Default environment forces English locale: `LANG/LC_ALL/LANGUAGE=en_US.UTF-8`. Inline `VAR=... cmd` in the script still override for that subcommand.
- __Expected outcomes__: Optional `expect: fail|xfail` treats a non-zero exit in `execute` as PASS and a zero exit as FAIL. Default expectation is `pass`.
- __Restore__: If `restore:` is present, it always runs after `execute` in a `defer`-style block. Failures in `restore` are logged as warnings and do not fail the suite. Individual suites may themselves guard the restore with `if [ -z "${CI:-}" ]`.
- __Output streaming__: The runner streams script stdout/stderr directly without additional prefixes, so test output appears unmodified in logs.

## In-container assertions (`assertions/`)

After YAML suites, the runner performs additional checks that exercise `oxidizr-arch` end-to-end:

- Assertions are gated to Arch-family distros (`arch`, `manjaro`, `cachyos`, `endeavouros` and common variants). On other distros, assertions are skipped with a context log.
- __Enable__ experiments `coreutils,sudo-rs` with `--package-manager none` (forces repo installs) and verify:
  - `sudo-rs` and `uutils-coreutils` are installed (`pacman -Qi`).
  - `/usr/bin/sudo` symlink chain points to sudo-rs; acceptance includes either a direct link to `/usr/bin/sudo-rs` or the stable alias `/usr/bin/sudo.sudo-rs`. Backup exists at `/usr/bin/.sudo.oxidizr.bak`.
  - A minimum number of coreutils applet symlinks exist (default threshold 10; override via `COREUTILS_MIN_SYMLINKS`). For a critical subset, `--version` must not contain `GNU`.
- __Disable__ and verify absence of symlinks/backups and that GNU `date --version` contains `GNU`.

These assertions fail the run on any non-zero or unexpected state.

## Artifacts, logs, and progress

- __Analytics report__: `analytics.WriteReportMarkdown()` writes a summary to `/workspace/TEST_DOWNLOADS_ANALYTICS.md` or `...-<distro>.md`, based on parsed outputs of `pacman`, `rustup`, `cargo`, `git`, and `makepkg` observed during the run.
- __Runner logs__: Go logger prefix is `[RUNNER]`. Utility execution logs print `RUN> <cmd>` lines (`util.RunCmd`). Context breadcrumbs use `CTX>` and progress uses `PB>` for host-side renderers.
- __Docker image prep__: The Dockerfile builds and embeds the runner binary, preinstalls essentials, generates `en_US.UTF-8` and `de_DE.UTF-8`, and sets DNS to Google resolvers. Entry point is `/usr/local/bin/isolated-runner`.

## Behaviors that harden or could mask results (call-outs)

- __Expected-fail mechanism__: `task.yaml` supports `expect: fail|xfail`. When used, an `execute` failure is considered PASS. Use sparingly and document rationale in the suite.
- __Cleanup non-fatal__: Failures in `restore:` are logged as warnings and do not fail the suite. This prevents cleanup issues from flipping outcomes but can hide environment residue; suites should self-guard and print proofs before/after.
- __Forced English locale for scripts__: Test scripts run with `LANG/LC_ALL/LANGUAGE=en_US.UTF-8` by default. This stabilizes outputs but may hide locale-sensitive issues unless a suite explicitly overrides.
- __Image-level hardening__: The Dockerfile pins functional prerequisites (reinstalls `glibc`, generates locales, sets DNS to 8.8.8.8/8.8.4.4). This reduces infra flakiness but can hide host-network misconfigurations.
- __Repo normalization on derivatives__: Removing `cachyos-extra.db` and installing a standard base set nudges CachyOS-like images toward stock Arch behavior, reducing repo-related flakes at the cost of masking derivative-specific quirks.
- __AUR helper bootstrap__: Installing `paru-bin` if missing ensures AUR availability; idempotent build reduces flakiness from persistent caches.
- __Build-stamp skip__: Rebuild is skipped when the commit hash matches and a binary exists. Functionally safe for test correctness but noteworthy when diagnosing missing rebuilds.
- __Runner-set env for product commands__: `util.RunCmd` sets `RUST_BACKTRACE=1` and adds default `RUST_LOG=info` when invoking `oxidizr-arch` unless already set, increasing logs which can change timing/volume but not outcomes.

## Environment variables and toggles

- `TEST_FILTER=<name>`: run only the suite whose directory basename matches; error if no suite matches.
- `ANALYTICS_DISTRO=<distro>`: suffix analytics report filename.
- `CARGO_BUILD_JOBS=<n>`: control build parallelism (default 2).
- `FORCE_RUST_REBUILD=1`: force a rebuild even when the build stamp matches.
- `PROJECT_DIR=<path>`: override the project root for assertions that read `tests/lib/rust-coreutils-bins.txt` (default `/workspace`).

## Key paths and commands (reference)

- __Runner code__: `test-orch/container-runner/`
- __Image__: `test-orch/docker/Dockerfile`
- __YAML suites__: `/workspace/tests/**/task.yaml`
- __Analytics__: `/workspace/TEST_DOWNLOADS_ANALYTICS.md` or `...-<distro>.md`
