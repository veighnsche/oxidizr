# Stream D — Supply Chain Policy (Repo-first/AUR opt-in) + Lock Wait UX

## 1) Scope

- Enforce repo-first installs; AUR only with explicit opt-in and user.
- Sanitize env for helper calls; record provenance in audit.
- Improve pacman lock-wait UX with bounded waits and periodic progress.

Touched modules (validated):

- `src/system/worker/packages.rs::{install_package, ensure_aur_preflight, repo_has_package, check_installed}`
- `src/system/worker/distro.rs::{extra_repo_available}`
- `src/cli/{parser.rs, handler.rs}` (flags: `--allow-aur`, `--aur-user`) — add `--allow-aur`
- `src/logging/audit.rs::{audit_event_fields}` for provenance

## Reuse existing infrastructure

Implement policy and UX enhancements in the existing modules:

- Enforce repo-first/AUR‑opt‑in exclusively inside `src/system/worker/packages.rs::install_package`. Add gates (e.g., `--allow-aur`) to the CLI and thread them into the worker; do not add a second installation path.
- AUR helper discovery must use `Worker.which()`; avoid direct calls to external crates. Reuse `ensure_aur_preflight` and `aur_helper_name` instead of adding helper probes elsewhere.
- Lock wait behavior belongs in the `Worker` (see `wait_for_pacman_lock_clear`); add bounded waits and progress messages there, not via a parallel loop.
- Provenance and command logging continue via `audit_event_fields`; do not introduce a new logging sink.

## 2) Rationale & Safety Objectives

- Reduce trusted surface; enforce explicit consent for AUR.
- Better operator UX while waiting on pacman locks.

## 3) Architecture & Design

- Keep official-first policy; for AUR-only packages require `--allow-aur` and a configured helper/user.
- Construct minimal env (`LC_ALL=C`, pinned PATH) and log exact command.
- Enhance `wait_for_pacman_lock_clear` to emit concise progress lines; bound by `--wait_lock`.
- Safety decisions integration (see `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`):
  - Provenance capture for every install path (official repo vs AUR) with explicit actor/helper and command in audit.
  - Env sanitization for helper execution and explicit audit fields for provenance.
  - After Stream C lands, per-op attestation and selective hashing will include changed/untrusted targets.

## 4) Failure Modes & Guarantees

- Missing helper or denied AUR path → error with emitted command.
- Lock not cleared within timeout → `Error::PacmanLockTimeout` with progress breadcrumbs.

## 5) Preflight & Post-Verification

- Preflight: verify `pacman`, `pacman-conf`, and helper presence when needed.
- Post: verify installation and record owner via `query_file_owner`.

## 6) Migration Plan

1. Add `--allow-aur` and wire policy gates into `install_package`.
2. Add progress logging to lock-wait; tune human INFO verbosity.

## 7) Testing Strategy

- Matrix: repo present/absent, with/without `--allow-aur`, helper present/absent.
- E2E in Docker matrix.

## 8) Acceptance Criteria

- No AUR execution without explicit opt-in; provenance logged.
- Lock waits show simple periodic progress.
- Audit events include helper name, effective command, and sanitized env indicators; repo presence checks are recorded.
- When Stream C is enabled, attestation bundle includes selective hashes for AUR/untrusted changes.
- No duplicate install or lock-wait paths exist: policy and progress are implemented in `src/system/worker/packages.rs` and related `Worker` helpers; CLI only provides flags.

## 9) References

- `src/system/worker/{packages.rs, distro.rs}`
- `src/cli/{parser.rs, handler.rs}`
- `PROJECT_PLANS/5_PACKAGE_SUPPLY_CHAIN_...md`, `9_UX_CLI_VERBOSITY_...md`
