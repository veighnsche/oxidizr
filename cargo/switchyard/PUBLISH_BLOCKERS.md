# Switchyard Publish Blockers

This document tracks everything we must do before publishing Switchyard as a standalone repository and on crates.io.

## Status snapshot (monorepo)

- CI workflows live at repo root: `.github/workflows/ci.yml`, `.github/workflows/book.yml`.
- CI contains Switchyard-specific guardrails and jobs that reference monorepo paths (e.g., `cargo/switchyard/**`, `scripts/bdd_filter_results.py`, `test_ci_runner.py`).
- BDD features and steps live entirely under the crate (`cargo/switchyard/SPEC/**`, `cargo/switchyard/tests/**`) and can run via Cargo with `--features bdd`.
- Root-level `.gitignore` exists; a crate-level `.gitignore` is not present in `cargo/switchyard/`.

## Blockers checklist

### 1) Repository split and layout

- [ ] Create a new GitHub repo with Switchyard content at the repo root (not under `cargo/switchyard/`).
  - [ ] Move contents of `cargo/switchyard/` to new repo root.
  - [ ] Preserve `SPEC/`, `tests/`, `DOCS/`, `README.md`, `CHANGELOG.md` (if present), `LICENSE*` files.
  - [ ] Fix all intra-repo paths that currently assume an extra `cargo/switchyard/` prefix (e.g., docs that link using that prefix).
- [ ] Ensure `Cargo.toml` has complete package metadata:
  - [ ] `name`, `version`, `description`, `license`/`license-file`, `repository`, `homepage`, `documentation`, `readme`, `keywords`, `categories`.
  - [ ] Set `rust-version` (MSRV) and match it in CI.
  - [ ] Review `[package] exclude`/`include` to keep crate small but include required docs (README, LICENSE).

### 2) CI for the new repo

- [ ] Add `.github/workflows/` in the new repo with Switchyard-only workflows:
  - [ ] `lint.yml`: fmt + clippy (warnings as errors), guardrails pertinent to Switchyard.
  - [ ] `test.yml`: unit/integration tests on stable, beta, nightly; MSRV job if pinned.
  - [ ] `bdd.yml`: run cucumber BDD (`cargo test --features bdd --test bdd -- --nocapture`).
  - [ ] `docs.yml`: build docs; optionally `RUSTDOCFLAGS=--cfg docsrs`.
  - [ ] `publish.yml` (manual): `cargo publish --dry-run`; tagged release publish.
  - [ ] Optional: `spec-traceability.yml` re-running `SPEC/tools/traceability.py` if you want artifacts.
- [ ] Remove/replace monorepo references in CI scripts:
  - [ ] Replace `python3 scripts/bdd_filter_results.py` with direct Cargo invocation or vendor the script into the new repo.
  - [ ] Replace `python3 test_ci_runner.py` with equivalent `cargo` commands or bring it along if it’s still useful.
  - [ ] Remove monorepo-only gates (e.g., checks referencing non-existent `src/` in the new repo) or adapt them to Switchyard’s structure.

### 3) Scripts and tooling referenced by CI/tests

- [ ] Vendor required helper scripts into the new repo or eliminate them:
  - [ ] `scripts/bdd_filter_results.py` (optional; can be replaced by direct BDD run).
  - [ ] `test_ci_runner.py` (optional; assess whether still needed).
  - [ ] `SPEC/tools/traceability.py` (already under the crate; keep).
  - [ ] Any `golden-diff/` tooling (if you intend to keep a golden fixtures workflow).
- [ ] Ensure all script paths in CI reference the new repo layout.

### 4) Tests are fully standalone

- [ ] Unit/integration tests: verify `cargo test` passes from the new repo root (no monorepo path assumptions).
- [ ] BDD tests:
  - [ ] Confirm `cargo test --features bdd --test bdd` works without external repo files.
  - [ ] Ensure `.feature` files are under `SPEC/features/` at repo root (they are today under the crate; keep same relative layout after move).
  - [ ] Keep tests hermetic (temp dirs only). Retain/no-op the CI “absolute path” guard if desired.
- [ ] Remove/port any tests that depend on monorepo assets or external crates in this workspace.

### 5) .gitignore and repo hygiene

- [ ] Add a repo-level `.gitignore` appropriate for a single-crate repo:
  - [ ] Ignore: `target/`, `tmp/`, `**/*.bak`, `.DS_Store`, editor folders, BDD logs (e.g., `target/bdd-lastrun.log`).
  - [ ] Do not ignore: `SPEC/**`, `DOCS/**`, `README.md`, `CHANGELOG.md`, `LICENSE*`.
- [ ] Ensure CI artifacts are excluded from the published crate via `Cargo.toml` `exclude` or a precise `include` list.

### 6) Docs and badges

- [ ] Update `README.md` badges to point to the new repo (GitHub CI, crates.io, docs.rs, license, MSRV).
- [ ] If using mdBook, either keep it in a separate `book` repo or add a dedicated workflow to publish to `gh-pages`.
- [ ] Verify links within README and docs use correct relative paths.

### 7) Licensing and changelog

- [ ] Ensure `LICENSE` (and `LICENSE-MIT` if dual-licensed) are present at repo root.
- [ ] Maintain `CHANGELOG.md`; CI may enforce changelog updates upon crate changes.

### 8) MSRV and clippy

- [ ] Enforce MSRV in CI (matrix includes the chosen MSRV).
- [ ] `cargo clippy --all-targets -- -D warnings` clean, including tests (allowances for unwrap/expect in tests are acceptable if annotated).
- [ ] `cargo fmt -- --check` clean.

### 9) Docs build and crate packaging gates

- [ ] `RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps` passes.
- [ ] `cargo package --list` shows only intended files. Run `cargo publish --dry-run` successfully.

### 10) Post-split housekeeping

- [ ] Update `oxidizr-arch` monorepo CI to remove Switchyard-specific jobs.
- [ ] Update dependents (e.g., `oxidizr-deb`) to use the published crate version or the new git URL.
- [ ] Close out any monorepo TODOs that belonged to Switchyard; move to the new repo as issues.

## Quick answers to current questions

- BDD tests standalone? Yes. They run via Cargo with `--features bdd`; `.feature` files and step code live inside the crate. The helper Python script is optional.
- Is all testing standalone? Unit/integration tests are. Any CI steps using root-level helper scripts must be ported or replaced in the new repo.
- Where is the workflows folder right now? At monorepo root: `.github/workflows/`. There is no crate-local workflows folder; the new repo will need its own.
- Why is there switchyard stuff in the workspace CI? Because the monorepo CI currently gates Switchyard (guardrails, BDD, traceability, changelog). These should be removed from the monorepo once Switchyard is split.
- Where is the `.gitignore`? Monorepo-level at repo root. Add a repo-level `.gitignore` in the new Switchyard repo.

---

Use this checklist to drive the split; once all boxes are checked, we are ready to publish Switchyard as a standalone crate and repository.
