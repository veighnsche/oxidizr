# Plan: Features and Gherkin/BDD Tests for oxidizr-deb

## 1) Goals
- Validate CLI surface and engine integration end-to-end at the package level.
- Cover Debian UX addendum requirements.

## 2) Test Harness Options
- Rust `cucumber` crate similar to engine BDD wiring (see `cargo/switchyard/tests/BDD_WIRING.md`).
- Place oxidizr-deb BDD under `cargo/oxidizr-deb/tests/bdd_*` with `[[test]]` harness disabled if needed.
- Alternatively, high-level YAML runner (existing test-orch) can invoke the CLI in containers; start with Rust cucumber for local speed.

## 3) Feature Suites (Draft)
- `features/dry_run_default.feature`
  - Dry-run is default; commit only with `--commit`.
- `features/coreutils_use_restore.feature`
  - `use coreutils` performs internal mapping to the replacement binary; `restore coreutils` reverts to GNU.
- `features/findutils_use_restore.feature`
  - `use findutils` performs internal mapping to the replacement binary; `restore findutils` reverts to GNU.
- `features/sudo_use_guard.feature`
  - Commit blocks if replacement cannot satisfy `root:root` and `4755` requirements.
- `features/apt_locks.feature`
  - When dpkg/apt locks present, commit refuses with friendly diagnostic.
- `features/degraded_fs_policy.feature`
  - Package policies disallow degraded EXDEV fallback (no visible change; stable reason).
- `features/fetch_and_verify.feature`
  - Fetch selects the correct artifact by arch/distro; SHA-256 (and signature when available) verified before plan.
- `features/status_reporting.feature`
  - `status` reports current active and restorable packages.

## 4) Step Definitions (Sketch)
- World: temp root directory, seeded tree with dummy binaries & perms; optional local artifact for `--offline --use-local`.
- Steps
  - Given a fakeroot with …
  - And a verified replacement artifact is available (or fetched) for package …
  - When I run `oxidizr-deb …`
  - Then exit code is …
  - And `/usr/bin/ls` resolves to … (coreutils unified mapping is internal)
  - And backups exist for …
  - And output contains …

## 5) Fixtures & Goldens
- Capture plan/preflight/apply summaries for golden diff where deterministic.
- Keep timestamps redacted per engine rules.

## 6) CI Integration
- Run BDD in CI as part of oxidizr-deb job.
- Optional container jobs for Ubuntu LTS images to validate apt lock paths and merged-/usr assumptions.

## 7) Acceptance
- Each SPEC requirement in `SPEC.md` and `DEBIAN_UX.md` mapped to ≥1 scenario.
- No SKIP in CI; flakes documented if infra-related.
