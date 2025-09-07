# Testing and CI Policy

This document defines the non-negotiable rules for tests, CI, harness behavior, and product alignment in the `oxidizr-arch` project.

## Scope

- Applies to all test orchestrators and runners under `test-orch/`, `tests/`, and any CI workflows under `.github/workflows/`.
- Applies to product code under `src/` and its interaction with test runners.

## Core Principle

- The product must perform all state mutations. Tests must not mask, pre-create, or repair product-managed artifacts.

## SKIP Policy

- Only one SKIP is approved: missing locale data in derivative Docker images (e.g., `de_DE`) while infra is being fixed.
- No other SKIPs are authorized. Any additional SKIPs must be removed.
- CI treatment: SKIPs cause CI failure by default. The single locale SKIP is time-bounded, must have a blocking issue and an owner, and must be removed as soon as images are fixed.

## Allowed SKIPs Table (Docker-only constraints)

This table is the authoritative list of which tests may ever skip due to Docker image limitations. Anything not listed here must not skip.

| Test Suite                              | Allowed to SKIP? | Distros                              | Reason                                                                                 | Notes / Owner |
|-----------------------------------------|------------------|--------------------------------------|----------------------------------------------------------------------------------------|---------------|
| `tests/disable-in-german`               | Yes (temporary)  | `cachyos, manjaro, endeavouros`      | Missing locale definition files (e.g., `/usr/share/i18n/locales/de_DE`) in base images | Infra; tracked in `GERMAN_LOCALE_TEST_ISSUE.md`; remove when images fixed |
| `tests/enable-all`                      | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/enable-default`                  | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/enable-no-compatibility-check`   | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/enable-partial`                  | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/disable-all`                     | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/disable-default`                 | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/disable-partial`                 | No               | all                                  | Not locale-dependent; must run                                                         | — |
| `tests/non-root`                        | No               | all                                  | Not locale-dependent; must run                                                         | — |

Notes:
- The locale SKIP is the only SKIP not treated as a failure in `FULL_MATRIX` CI. It is strictly time-bounded and must have an active blocking issue and owner. All other SKIPs fail the run.

## CI and Matrix Policy

- FULL_MATRIX is the default for CI runs via the host orchestrator.
- Any SKIP in any matrix job fails the run, except the single allowed SKIP listed above (`tests/disable-in-german` on CachyOS/Manjaro/EndeavourOS due to missing locale files). That specific SKIP is permitted (tracked) and does not fail the run.
- Non-matrix runs must not silently appear green by skipping assertions—if the scenario cannot be executed, fail with a clear reason.

## Harness Policy (Docker and other runners)

- Do not mutate `/usr/bin/*` or related product-managed paths in the harness.
- Do not install or rely on BusyBox or similar toolsets to perform file operations for tests.
- Do not pre-create, repair, or inject applet symlinks before or around `oxidizr-arch enable/disable`.
- Do not add shell cache workarounds such as `hash -r` to make tests pass. If resolution or sequencing is incorrect, fix the product or run assertions in a fresh process.
- Docker entrypoint must follow the minimal flow: build binary → `enable` → assertions → `disable` → assertions.

## Product–Policy Alignment

- Goal: support switching across Arch-family distros (`arch, manjaro, cachyos, endeavouros`) with zero SKIPs in matrix runs (except the single locale SKIP).
- Do not distro-gate the Arch-family to make tests pass; run the same assertions across the family. Gate only OSes we explicitly do not support or have never tested.
- Registry defaults must not hard-block derivatives. If providers are available (via pacman/AUR), switching proceeds; otherwise fail loudly with a clear reason (no SKIPs).
- `SudoRsExperiment::check_compatible()` should allow derivatives when the package is installed/available; otherwise error (not skip).

## Packaging & AUR Semantics

- Assume an AUR helper (e.g., `paru`, `yay`) is available on systems that require AUR packages. In CI, the container-runner ensures a helper exists; on user systems, users must install one.
- CLI overrides (`--package`, `--bin-dir`, `--unified-binary`) must override registry defaults on all distros so tests/users can force switching deterministically.

## Registry and Probing

- Always attempt switching on supported Arch-family distros. Use `pacman` first; when packages are only in AUR, use the available helper. If installation fails, fail loudly with a clear reason (no SKIPs).
- Probing is for visibility and better messages (e.g., detect that `sudo-rs` is missing), not for gating or skipping.

## Infrastructure Policy (Containers/Images)

- Required locale data (e.g., `de_DE`) must be pre-provisioned in derivative images. Until then, the single approved SKIP applies and is time-bounded.
- Infra gaps discovered by tests must either be fixed in images or cause a hard failure with clear remediation guidance.

## Documentation and Accountability

- Any temporary workaround must be accompanied by a blocking task (issue) with an owner and removal criteria.
- Comments that label a workaround as “temporary/stupid” without removing it are forbidden—remove the workaround or file the blocking task immediately.

## References

- Product switching logic: `src/experiments/uutils/` and `src/utils/worker/` (e.g., `replace_file_with_symlink()`, `restore_file()`).
- Docker harness: `test-orch/docker/entrypoint.sh`.
- Container runner assertions: `test-orch/container-runner/assertions/`.
- YAML suites: `tests/*/task.yaml` (Spread/LXD via `spread.yaml`).

## Enforcement

- CI enforces these rules. Violations will cause failures. Do not add masking logic to bypass failures—fix the root cause.
