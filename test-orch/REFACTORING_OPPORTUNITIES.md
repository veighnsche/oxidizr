# Refactoring Opportunities for Test Orchestrator

Goal: Run a streamlined test matrix across Arch, CachyOS, Manjaro, EndeavourOS with no unintended skips. Only allow `disable-in-german` to be skipped when the matrix runs distros in parallel (flakiness under parallel cross-distro execution), while running everything else.

## Findings

- __EndeavourOS naming inconsistency__
  - Host orchestrator default flag and mapping use `endeavoros` (missing `u`), while the distro ID and test YAML use `endeavouros`.
  - Impact: Confusing UX and potential drift in docs/flags.
  - Refactor: Unify to `endeavouros` across flags and mapping keys in `host-orchestrator/main.go`.

- __Locale handling is probe-only__
  - `container-runner/setup/locales.go` should not gate execution; locales are probed/logged for visibility only.
  - Impact: Avoids conflating locale availability with test pass/fail semantics.
  - Refactor: Keep locale setup non-fatal; tests remain responsible for their own assertions.

- __FULL_MATRIX semantics skip policy__
  - `yamlrunner/yamlrunner.go` treats any skipped suite as fatal in `FULL_MATRIX` mode.
  - Desired: Only `disable-in-german` should be allowed to skip (and not be fatal) when the matrix runs distros in parallel (flakiness under parallel execution). It passes in isolation/serialized.
  - Refactor: Add an exception/allowlist for this suite specifically under parallel cross-distro runs.

- __Missing current distro accessor__
  - `util.ShouldRunOnDistro` parses `/etc/os-release` but there is no exported `CurrentDistroID()` helper.
  - Refactor: Add `util.CurrentDistroID()` so other code can read the current distro cleanly.

- __Probe summary for visibility__
  - There is no single summary log of key environment facts per container run (distro, locale presence, AUR helper presence).
  - Refactor: Add a preflight probe summary to print (and thus persist in CI logs) the current distro ID, whether `de_DE.UTF-8` locale is present, and whether `paru`/`yay` is detected.

- __Optional: AUR helper detection breadth__
  - We currently detect & skip installing only when `paru` exists. Some distros may ship `yay` instead.
  - Decision: Keep installation tied to `paru` for consistency, but include `yay` in the probe summary so we have data. (Future: treat either as sufficient.)

## Plan of Action

1. Unify EndeavourOS spelling in `host-orchestrator/main.go` (flag default and `distroMap` key).
2. Keep `setup/locales.go` probe-only and non-fatal (log-only), even under `FULL_MATRIX`.
3. Add `util.CurrentDistroID()`.
4. In `yamlrunner`, allow a skip exception for the `disable-in-german` suite only when the matrix runs distros in parallel, even when `FULL_MATRIX=1`.
5. Add a startup probe summary in `setup/preflight.go` to log distro, locale status, and AUR helper detection.

## Acceptance Criteria

- Running the matrix with default flags builds and runs across all four distros.
- Only `disable-in-german` is skipped when running the matrix in parallel; all other suites run.
- FULL_MATRIX remains enabled; skip of `disable-in-german` on non-Arch does not fail the run.
- CI logs contain a one-shot probe summary showing distro, locale presence, and AUR helper detection.
- Docs reflect unified spelling and reference this plan (linked from `README.md`).

## Status

- [x] Unify EndeavourOS spelling and normalize aliases in `host-orchestrator/main.go`; docs updated.
- [x] Locale handling: switched to probe-only and removed active setup from `setup/setup.go`; `setup/locales.go` retained but unused.
- [x] Add `util.CurrentDistroID()`.
- [x] FULL_MATRIX exception in `yamlrunner` for `disable-in-german` under parallel cross-distro runs.
- [x] Preflight probe summary logs (distro, locale presence, and AUR helper detection).
- [x] Avoid duplicate AUR logic: `setup/rust.go` only sets rustup default; AUR helper install remains in `setup/users.go`.
- [x] Documentation aligned with probe-only locale policy and corrected SKIP rationale (parallel-run flakiness).

Open items (optional):
- [ ] Empirically verify `yay`/`paru` presence and `de_DE` availability on Manjaro and EndeavourOS base images used in CI.
- [ ] Consider deleting `setup/locales.go` or hiding it behind a disabled-by-default flag if we want a future "force locales" mode.
