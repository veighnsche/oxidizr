# Plan: SPEC Traceability and Documentation for oxidizr-deb

## 1) Goals

- Prove compliance against `SPEC/SPEC.md` and `SPEC/DEBIAN_UX.md` using a machine-checkable trace.
- Keep docs up-to-date and CI-enforced.

## 2) Artifacts

- `SPEC_TRACE.toml` in `cargo/oxidizr-deb/` mapping REQ IDs → code/tests.

  ```toml
  [REQ-C1] # SafePath boundary under --root
  code = ["src/cli/args.rs", "src/util/paths.rs"]
  tests = ["tests/features/dry_run_default.feature"]

  [REQ-PKG-1] # coreutils package policy (no degraded EXDEV)
  code = ["src/packages/coreutils.rs", "src/packages/findutils.rs"]
  tests = ["tests/features/degraded_fs_policy.feature", "tests/features/findutils_use_restore.feature"]

  [REQ-FETCH-1] # fetch and verify before mutation
  code = ["src/fetch/mod.rs", "src/fetch/verifier.rs"]
  tests = ["tests/features/fetch_and_verify.feature"]

  [REQ-SUDO-1] # sudo setuid hardening
  code = ["src/packages/sudo.rs", "src/adapters/preflight.rs"]
  tests = ["tests/features/sudo_use_guard.feature"]

  [REQ-PERM-1] # persist selection across upgrades after 'use --commit'
  code = ["src/commands/use.rs", "src/cli/handler.rs"]
  tests = ["tests/features/persistence.feature"]

  [REQ-CLEAN-1] # cleanup artifacts on restore
  code = ["src/commands/restore.rs"]
  tests = ["tests/features/cleanup_after_restore.feature"]
  ```

- `docs/` additions: README, manpage (generated), completions.
- Golden fixtures under `tests/fixtures/`.

## 3) Process

- When a REQ is added/changed, update `SPEC_TRACE.toml` and add/adjust tests.
- PR template requires listing touched REQ IDs.
- CI job validates that every REQ ID appears at least once in code and tests (regex scan + allowlist for N/A).

## 4) CI Checks

- `trace-check`: parse `SPEC_TRACE.toml`, confirm paths exist, run a lightweight `rg` on REQ tags in code/test headers.
- `fixtures-check`: confirm golden files exist for marked scenarios and match byte-for-byte.
- `doc-sync`: verify README and manpage reference only package-level commands (`rustify`, `restore`, `status`).

## 5) Documentation

- `README.md` (CLI quickstart, safety notes, examples).
- `MANPAGE.md` generation via clap or a template.
- Link to the engine SPEC sections for schemas and invariants.

## 6) Acceptance

- All REQs in `SPEC/SPEC.md` and `SPEC/DEBIAN_UX.md` mapped in `SPEC_TRACE.toml`.
- CI trace-check green; fixtures-check green; zero SKIP.
