# Download Analytics: Problem, Investigation, and Plan

This document tracks our ongoing effort to measure and reduce unnecessary downloads performed by the container runner during test orchestration.

## Problem Statement

- We observed heavy repeated downloads during container runs. This includes package installs, Rust toolchain components, Cargo crates, and AUR/git fetches.
- Repeated downloads slow down local and CI runs and add variability to the pipeline.
- Goal: Measure where downloads happen, quantify them, and reduce them toward an ideal “warm-run” baseline.

## Where downloads currently happen (code references)

- Pacman base deps: `test-orch/container-runner/setup/deps.go`
  - Installs `base-devel sudo git curl rustup which findutils`.
- Rust toolchain: `test-orch/container-runner/setup/rust.go`
  - `rustup set profile minimal` + `rustup default stable` (root-only).
- AUR helper installation: `test-orch/container-runner/setup/users.go#installAurHelper`
  - `git clone` + `makepkg` for `paru-bin` when not pre-installed.
- Build and unit tests (Cargo):
  - Build: `test-orch/container-runner/setup/build.go` → `cargo build --release`.
  - Tests: `test-orch/container-runner/runner.go` → `cargo test` (runInContainer).
- Docker base image provisioning (one time per image build): `test-orch/docker/Dockerfile`
  - `pacman -S ... rustup ...`, locale generation, and reflector mirror list setup.

## Existing caching and improvements applied

- Persistent caches (host orchestrator): `test-orch/host-orchestrator/dockerutil/dockerutil.go`
  - Non-interactive runs now mount:
    - `/root/.cargo/registry`, `/root/.cargo/git`, `/workspace/target`
    - `/var/cache/pacman`
    - `/home/builder/build` (AUR build cache)
    - `/root/.rustup` (NEW)
  - Interactive `--shell` runs: same mounts added (NEW) so shells are warm after the first run.
- Rust toolchain setup
  - `setup_shell.sh` now installs the toolchain only once (root) with `rustup set profile minimal`.
  - `setup/rust.go` does the same (root-only, minimal profile).
- Mirror selection
  - Dockerfile reflector restricted to HTTPS (`--protocol https`) to avoid unusable `rsync://` mirrors.
- Locale defaults
  - Image now defaults to `en_US.UTF-8` (German still generated but only used when explicitly requested by tests).

## NEW: Lightweight download analytics (implemented)

- Analytics package: `test-orch/container-runner/analytics/analytics.go`
  - Parses process output from `pacman`, `rustup`, `cargo`, `git`, `makepkg` to count download events and estimate sizes.
  - Approximations are intentional and good enough to flag regressions.
- Instrumentation points:
  - `util.RunCmd` and `util.RunCmdQuiet` stream stdout/stderr through scanners and forward each line to `analytics.ProcessLine(...)`.
  - On successful run completion, a Markdown report is written to `/workspace/TEST_DOWNLOADS_ANALYTICS.md`.
- What is counted today:
  - Pacman: package download count (+ example names).
  - Rustup: number of components + approximate bytes.
  - Cargo: number of crates + approximate bytes from `Downloaded ... (SIZE)` lines.
  - Git/Makepkg: clone count, `Receiving objects` size heuristic; makepkg download events.

### Report format

Each container writes a report at the end of a successful run:
- Path: `/workspace/TEST_DOWNLOADS_ANALYTICS-<distro>.md` (e.g., `TEST_DOWNLOADS_ANALYTICS-arch.md`)
- Contents:
  - Totals by category (counts + approximate bytes where available).
  - Heuristic minimum for a warm run.
  - Suggestions to further reduce downloads.

## Heuristic minimum (ideal warm run)

On a fully warmed and stable cached environment:
- Pacman: 0 new package downloads (if base dependencies are baked or pacman cache is mounted).
- Rustup: 0 components (toolchain present under `/root/.rustup`).
- Cargo: 0 crate downloads (registry/git caches mounted; incremental build in `/workspace/target`).
- Git/Makepkg: 0 (AUR helper repo cached under `/home/builder/build`).

## Early findings and hypotheses

- First run on a new image will still incur downloads; this is expected.
- Subsequent runs should trend toward the heuristic minimum. If not:
  - Cargo lockfile or registry updates may be forcing refetch.
  - Pacman `-Syyu` performed in too many places (host vs container) can trigger repeated DB refreshes.
  - New tags (image content hash changes) may vary cache keys and cause cache misses (`cargo-target/<distro>` per-distro key is used by design; ensure distro stays consistent when comparing runs).

## Plan to reduce downloads further

1. Keep improving persistence
   - Confirm that all mounts exist on host and are writable: `.rustup`, `.cargo/registry`, `.cargo/git`, `target`, `pacman`, `aur-build`.
   - For interactive `--shell`, ensure the same mounts (already done).
2. Cut first-run weight (optional)
   - Pre-bake stable `rustup default stable` into the image if startup time matters more than image size.
3. Cargo discipline
   - Prefer `cargo sparse` (default in modern Cargo) and pin versions via `Cargo.lock`.
   - Consider `CARGO_HOME` overrides if needed.
4. Pacman discipline
   - Avoid extra `-Syyu` calls inside the container after image build unless necessary.
   - If DB refreshes are needed, do it once early in the run.
5. AUR helper
   - The build is idempotent and uses a persistent cache directory. If analytics show repeated rebuilds, enforce an up-to-date `git pull --rebase` instead of reclone.
6. Thresholds & regression alarms (future work)
   - Add thresholds (env-configurable) and fail or warn when download counts/bytes exceed expectations on warm runs.
   - Emit per-distro breakdowns (tag-aware, already partially implemented via per-distro cargo target).

## How to use the analytics

- Run the host orchestrator as usual:
  - `sudo go run test-orch/host-orchestrator --run` (or `--shell` then run tests manually)
- After the run, open the per-distro analytics reports from the host filesystem:
  - `TEST_DOWNLOADS_ANALYTICS-arch.md`, `TEST_DOWNLOADS_ANALYTICS-manjaro.md`, etc. (mounted from `/workspace`).
- Compare across runs:
  - First run will be heavier. Subsequent runs should approach the heuristic minimum if caches persist and inputs are unchanged.

## Known limitations

- Parsing is heuristic and may miss some cases.
- Scanner is line-oriented; if tools print giant lines without newlines, size detection may miss.
- We currently do not analyze Docker build-time downloads (host-side). Adding host-side analytics would require instrumenting the host orchestrator build step separately.

## Next steps

- Add a small per-distro section to the report by reading the image tag (optional).
- Add warning thresholds and CI regression checks.
- Consider a host-side analytics mode to measure Docker build contributions.
