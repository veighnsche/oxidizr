# Flaky: `sudo-rs` package not found in Arch-based CI containers

Last updated: 2025-09-08 15:21:22+02:00

## Summary

Intermittently, Arch-based container runs (Arch, Manjaro, EndeavourOS) fail to locate the `sudo-rs` package during test execution. When this occurs, the `sudo-rs` experiment is disabled, and downstream suites (notably `enable-partial`) sometimes fail as a consequence. This appears to be an infrastructure/mirror consistency issue rather than a deterministic code bug.

## Affected environments

- `test-orch/` matrix runs for:
  - Arch Linux container
  - Manjaro container
  - EndeavourOS container

## Impact

- `sudo-rs` experiment gets disabled unexpectedly with message: `error: package 'sudo-rs' was not found`.
- Suite `enable-partial` may report FAIL when expectations implicitly assume `sudo-rs` is available.
- Overall matrix stability is reduced due to a flake, not a deterministic failure.

## Observed logs

```
[manjaro] Optional For    : None
[manjaro] Conflicts With  : None
[manjaro] Replaces        : None
[manjaro] Installed Size  : 12.43 MiB
[manjaro] Packager        : Tobias Powalowski <tpowa@archlinux.org>
[manjaro] Build Date      : Mon 26 May 2025 03:21:27 PM UTC
[manjaro] Install Date    : Mon 08 Sep 2025 12:58:28 PM UTC
[manjaro] Install Reason  : Explicitly installed
[manjaro] Install Script  : No
[manjaro] Validated By    : Signature
[manjaro] 
[manjaro] checking dependencies...
[manjaro] 
[manjaro] Packages (1) uutils-coreutils-0.1.0-1
[manjaro] 
[manjaro] Total Removed Size:  12.43 MiB
[manjaro] 
[manjaro] :: Do you want to remove these packages? [Y/n] 
[manjaro] :: Processing package changes...
[manjaro] removing uutils-coreutils...
[manjaro] :: Running post-transaction hooks...
[manjaro] (1/1) Arming ConditionNeedsUpdate...
[manjaro] Disabled experiment: coreutils
[manjaro] :: Synchronizing package databases...
[arch]  core downloading...
[arch]  extra downloading...
[arch] error: package 'sudo-rs' was not found
[arch] Disabled experiment: sudo-rs
[arch] 2025/09/08 12:58:32 [8/9] FAIL suite: enable-partial
[arch] 2025/09/08 12:58:32 in-container runner failed: YAML test suites failed: exit status 1
[endeavouros] Starting tests...
RUN> docker run -v /home/vince/Projects/oxidizr-arch:/workspace --name oxidizr-arch-test-oxidizr-endeavouros-989e983b9915 oxidizr-endeavouros:989e983b9915 internal-runner
WARN: [arch] docker run failed: docker run (CLI) failed: exit status 1
WARN: [manjaro] docker run failed: docker run (CLI) failed: signal: killed
WARN: [endeavouros] docker run failed: docker run (CLI) failed: context canceled
==> Some tests failed.
```

## Likely root causes (hypotheses)

- Mirror desynchronization or stale package databases during container runs.
- Repository channel differences between Arch and derivatives (package may exist in `extra/testing` or be delayed/absent on Manjaro/EndeavourOS mirrors).
- Minimal base images with nonstandard or truncated mirrorlists causing transient 404/404-like index states where the package is temporarily invisible.
- Less likely: keyring/signature problems that surface as "not found" (no signature errors observed in logs above).

## Proposed mitigations

- Mirror refresh and retry logic (low cost):
  - Before attempting install, run `pacman -Syy --noconfirm`.
  - If `sudo-rs` is reported missing, wait briefly (e.g., 5s) and retry with `pacman -Syyu --noconfirm`, up to 2 attempts.
  - Optionally pin a reliable mirror (e.g., `https://geo.mirror.pkgbuild.com/$repo/os/$arch`) for Arch runs when flake rate is high.

Preflight evidence collection (not gating):
  - Collect evidence with `pacman -Syy --noconfirm` followed by `pacman -Si sudo-rs` (or `pacman -Ss '^sudo-rs$'` if `-Si` fails) prior to install.
  - This preflight does not gate or skip tests. If sudo-rs is unavailable in an in-scope environment, that is a failure signal to investigate (product first), not a soft-disable.

- Distros differences handling:
  - Differences in derivatives must be addressed explicitly in documented Supported Scope and base images, not by ad hoc skipping. Do not assume absence is "expected" without approved scope documentation.
  - Avoid AUR fallbacks that could mask product behavior unless explicitly declared as production-supported and captured in provenance. The default is to rely on official repos.

- Test suite resilience:
  - Ensure suites that are unrelated to sudo-rs do not implicitly depend on it. Suites that require sudo-rs must fail if it is unavailable in-scope, or be explicitly out-of-scope by policy (not by runtime gating).

- Observability:
  - When `sudo-rs` is reported missing, log `pacman.conf` and active mirrorlist (`/etc/pacman.d/mirrorlist`) and the exact commands/exit codes to support root-cause analysis.

## Next actions

- Product-first audit:
  - Review `src/experiments/sudors.rs` to validate how the product detects, installs, and links sudo-rs, and how it behaves when binaries are not found post-install.
  - Verify Worker implementations of `install_package` and error handling paths to ensure failures surface clearly (no masking).
- Evidence-only preflight in `test-orch/container-runner/`:
  - Log `pacman -Syy`, `pacman -Si sudo-rs` (or `-Ss`), config files, and exit codes before install attempts—without gating/skipping.
- Keep bounded retries around mirror refresh with `-Syy`/`-Syyu` to mitigate transient index issues, while treating persistent unavailability in-scope as failure.
- Ensure suites are properly decoupled: generic suites must not implicitly require sudo-rs; suites that require sudo-rs must either be in-scope and fail on unavailability or be documented out-of-scope by policy.
- Re-run the full matrix and record outcomes, attaching logs and configs to the Proof Bundle.

## Related code paths

- `test-orch/container-runner/runner.go` (install flow, experiment toggling)
- `test-orch/container-runner/util/util.go` (general helpers for shell exec)
- YAML suites under `tests/` (e.g., `tests/enable-partial/task.yaml`) that may assume availability

## Status

- Tracking ID: FLK-SUDORS-NOTFOUND
- Owner: test-orch infra
- State: Open (mitigations pending)
