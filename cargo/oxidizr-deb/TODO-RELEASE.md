# oxidizr-deb Release Blockers and Tasks

This document tracks blockers and tasks required for a safe, spec-compliant release of `oxidizr-deb`.

## Blockers (must fix)

- [ ] Sudo guard ordering
  - Move `sudo_guard()` in `src/commands/use_cmd.rs` to run after artifact acquisition (APT/fallback) and before linking.
  - Only run on `--commit` and live `/`.

- [ ] Restore test fallback must not run on live root
  - In `src/commands/restore.rs`, gate the "pragmatic fallback" that rewrites `gnu-<applet>` to non-live roots (or an explicit test env).
  - Never touch real binaries on live `/`.

- [ ] Restore cleanup aware of staged replacements
  - In `restore.rs`, if RS packages are not installed via dpkg, remove staged `/opt/oxidizr/replacements/<pkg>` instead of `apt-get purge`.
  - Always remove staged artifacts after restore.

- [ ] Provider availability invariants (pre/post APT)
  - In `replace.rs` and `restore.rs`: use `dpkg-query`/`dpkg -s` and staged presence to enforce at least one provider remains before and after purge/install.

- [ ] Sudo fallback hardening
  - In `fetch/fallback.rs`: if `sudo` staged, require owner `root:root` and `mode=4755` or error.
  - Select GitHub release asset by architecture (x86_64, arm64/aarch64) and fail clearly if unsupported.

- [ ] Structured PM event logs
  - Emit JSON events for all apt/dpkg invocations with `tool`, `args`, `exit_code`, `stderr_tail`, `package`.

## High priority (should fix)

- [ ] Doctor lock detection parity
  - Make `doctor.rs` use fs2 exclusive lock attempts, same as `adapters/debian.rs`.

- [ ] Minimal smoke gating
  - After `apply()` in `use_cmd.rs` under `--commit`, verify several applet symlinks resolve to the staged/unified binary; for `sudo` verify `4755 root:root`.

- [ ] Cleanup warnings and stubs
  - Remove dead code and unused imports in `util/diagnostics.rs`, `fetch/sources.rs`, `fetch/verifier.rs`, etc.

- [ ] Docs: fallback trust model
  - README/SPEC must clearly state PM path is preferred; fallback retrieval is online and surfaced to the operator.

## Acceptance

- [ ] CI job to run `scripts/ubuntu_dev_proof.sh` and `scripts/debian_live_proof.sh` to prove APT-first flows (fallback only when necessary).

---

Implementation starts immediately below this line.
