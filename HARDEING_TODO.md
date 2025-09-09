# Hardening TODO — oxidizr-arch

Inputs: HARDENING_TASKS.md + code survey of src/ as of 2025-09-09.

This document lists production-minimum hardening work, maps each item to concrete code locations, and proposes acceptance tests. Items are prioritized and include suggested modules/APIs to touch.

## Summary of current state

- Audit logging
  - Structured JSONL via `logging/audit.rs::audit_event_fields()` is wired across product code. `audit_op()` calls the structured helper. Good.
- Repo gating and install flow
  - `experiments/mod.rs::check_download_prerequisites()` gates on `[extra]` and package presence for official packages, and on AUR helper for `uutils-findutils-bin`.
  - `system/worker/packages.rs` implements pacman ops and AUR fallback only for findutils. Good baseline, but missing AUR preflight tooling checks.
- Symlink operations
  - `symlink/ops.rs` does atomic rename with a temp symlink and backs up existing files/symlinks. Uses `symlink_metadata()` to avoid simple TOCTOU, but does not use `openat/fstatat(O_NOFOLLOW)` or reject symlinked parents. `is_safe_path()` is naïve. Needs no-follow/dir policy.
- sudo-rs
  - `experiments/sudors.rs` links `sudo|su|visudo` via an alias, but does not perform post-enable verification (setuid root, ownership, PAM, `sudo -n true`).
- State + relink hook
  - No persistent state of managed experiments/targets; no pacman post-transaction hook installer.
- Concurrency lock
  - Pacman lock wait exists, but no single-instance lock for oxidizr-arch itself.
- Owner verification
  - No `pacman -Qo` checks for targets.
- Writable/exec + immutable preflight
  - No `/usr` mount flags or `chattr +i` checks.
- Removal guard
  - Present: `experiments/coreutils.rs::remove()` refuses removal when checksum applets appear linked. Keep and test.
- Risky patterns
  - No `unsafe` blocks found. Minimal `unwrap()` at `ui/progress.rs` (template parsing). No `panic!/todo!` in product paths.

## Actionable TODO (by HARDENING_TASKS.md item)

1) Pacman post-transaction relink hook (clobber-proofing)

Status: DONE

Implemented:

- Persistent state + relink entrypoint.
- CLI subcommand `relink-managed` (parser + handler) calls `experiments::relink_managed()` which re-enables persisted experiments with compat-check skip.
- Hook installer `system/hook.rs::install_pacman_hook()` writes `/usr/share/libalpm/hooks/oxidizr-arch-relink.hook` with Exec `oxidizr-arch relink-managed --assume-yes --no_update --no_progress`.
Remaining: Add e2e upgrade test in container runner.

2) sudo-rs post-enable verifier (setuid, ownership, PAM)

Status: DONE

Implemented:

- Added `verify_post_enable()` in `experiments/sudors.rs` and invoked post-enable.
- Verifies real binary via alias, uid=0,gid=0,setuid bit, PAM file presence, and optional `sudo -n true` smoke test via `--sudo-smoke-user`.
- On failure: restores originals, removes alias symlinks, and errors out.
  - Checks (follow final target to real binary):
    - `uid=0,gid=0` and `mode & 0o4000 != 0` for `sudo`, `su`, `visudo`.
    - `/etc/pam.d/sudo` exists; warn or gate per distro if PAM-less expected.
    - Smoke: attempt `sudo -n true` for a sudoer user when available.
      - CLI flag: `--sudo-smoke-user <name>` to specify a test user; if absent, skip but warn.
  - On failure: restore originals (`restore_targets()`), remove alias symlinks, return `Error::ExecutionFailed` with remediation.
  - Where: `src/experiments/sudors.rs` + small helpers in `system/worker/fs_ops.rs` for getting mode/owner.
- Tests: post-enable, validators pass; inject negative cases to ensure fail-fast + revert.

3) Race-safe, no-follow filesystem operations

Status: PARTIAL

Implemented:

- Rejects symlinked parent directories (no-follow policy) and uses `symlink_metadata` on leaf.
- Atomic swap within same directory using temp symlink + `fs::rename` and parent directory fsync.
Remaining:
- If desired, further harden using `openat/fstatat/renameat` syscalls for parent/leaf path handling.
- Maintain backups and audit logs as today.
- Tests: symlink-race scenarios; ensure no-follow enforcement blocks malicious parents.

4) AUR preflight for findutils

Status: DONE

Implemented:

- `Worker::ensure_aur_preflight(assume_yes)` ensures/installs prerequisites under `-y`, else prints exact pacman command and aborts.
- Invoked from `findutils::enable()` after prerequisites gating.
  - Implement `Worker::ensure_aur_preflight(assume_yes)` in `system/worker/packages.rs`:
    - If `assume_yes`: `pacman -S --needed base-devel git fakeroot` (and ensure `makepkg` via `pacman -Q makepkg` or group check).
    - Else: print exact pacman command and return `Error::ExecutionFailed` with guidance.
  - Call from `experiments/findutils.rs::enable()` after `check_download_prerequisites()`.
- Tests: run enable on minimal image without base-devel; verify auto-install under `-y` and helpful abort otherwise.

5) Writable/exec + immutability preflight

Status: DONE

Implemented:

- `system/fs_checks.rs`: `ensure_mount_rw_exec`, `check_immutable`, `check_source_trust` (root-owned, not world-writable, exec mount, not under $HOME unless `--force`).
- Integrated into `Worker::replace_file_with_symlink` and `Worker::restore_file` preflights.
  - `check_mount_rw_exec("/usr")` by parsing `/proc/self/mounts` (or `/proc/mounts`) for the mount containing `/usr`; require `rw` and not `noexec`.
  - `check_immutable(path)` via `lsattr -d` parsing or ioctl (`FS_IOC_GETFLAGS`); if immutable, return error with `chattr -i` hint.
- Invoke before any linking in `create_symlinks()` and before restores in `restore_targets()`; or at experiment start over all computed targets.
- Tests: simulate ro/noexec/immutable and ensure we abort with clear messages.

6) Persist minimal state + targeted relink + final state report

Status: DONE

Implemented:

- `src/state/mod.rs` created with JSON persistence and state report writer.
- Experiments update state on enable/disable; CLI writes a final state report after mutating commands.
  - `State { enabled: Vec<String>, managed: Vec<String>, ts: String }` serialized to `/var/lib/oxidizr-arch/state.json`.
  - `update_state_on_enable/disable()` called by each experiment after success.
  - `compute_managed_targets(&[Experiment])` using `list_targets()`.
  - `write_state_report()` to `/var/log/oxidizr-arch/state-report.txt` listing each managed path as `regular|symlink|missing` (+ link target when symlink).
- Add `cli` flag `--state-dir` for tests to redirect persistence.
- Hook (item 1) reads this state for targeted relinking.

7) Package owner verification (warn by default; strict mode: block)

Status: DONE

Implemented:

- CLI adds `--strict-ownership`; plumbed through `Worker`.
- `Worker::query_file_owner` and `verify_owner_for_target` added.
- Enforcement integrated before linking operations.
- Tests: link to a path not owned by expected package; assert warn vs block.

8) Single-instance process lock

Status: DONE

Implemented:

- `system/lock.rs` with fs2-based lock.
- Enforced for mutating commands in CLI handler.
- Tests: background process holds lock; second invocation exits fast.

9) Source path trust checks for overrides

Status: DONE

Implemented:

- `fs_checks::check_source_trust()` enforces ownership, world-writable, exec-mount, and not under `$HOME` unless `--force`.
- Used before linking.
  - Verify resolved absolute sources are:
    - Not world-writable, on an `exec` mount, owned by root, and not under `$HOME` unless `--force`.
  - Add `--force` flag in CLI to bypass with explicit user consent.
  - Log resolved absolute source for each applet; already logged via `create_symlinks()` but add canonicalization where missing.
- Tests: world-writable dir, noexec mount; ensure rejection w/ remediation.

10) Removal guard already present → keep it strict

Status: DONE (pre-existing)

- Already present in `experiments/coreutils.rs::remove()` for checksum applets linked. Add explicit test coverage and clear remediation message.

## Cross-cutting improvements

- Replaced `unwrap()` in `ui/progress.rs` style builder with a safe fallback.

Status: DONE

- Expand audit events to include decision outcomes consistently: use `begin/success/failure` across symlink and restore stages (already largely present) and include `artifacts` where helpful (e.g., hook paths, state files).
- Ensure all external command strings passed to audit are secret-scrubbed (masking already applied in `audit_event`; continue using `audit_event_fields` with `cmd`).

## Proposed file/module additions

- `src/state/mod.rs` — state persistence and reporting.
- `src/system/hook.rs` — pacman hook installer utilities.
- `src/system/fs_checks.rs` — mount and immutability preflights.
- `src/system/lock.rs` — single-instance lock.

## Acceptance criteria (ship checklist)

Status snapshot:

- Relink hook: Implemented; manual verification pending e2e automation.
- sudo-rs verifier: Implemented with revert-on-fail.
- FS ops: No-follow parent and atomic swap implemented; syscall-level hardening optional.
- AUR preflight: Implemented.
- RO/immutable preflights: Implemented.
- State + report: Implemented.
- Owner verification: Implemented (warn by default; strict blocks).
- Concurrency lock: Implemented (mutating commands).
- Override trust checks: Implemented with `--force`.

- Relink hook prevents clobbering in upgrade tests.
- sudo-rs verifier blocks unsafe states; `sudo -n true` works when configured.
- FS ops are no-follow/atomic; symlink-race tests safe.
- AUR preflight enforced or auto-installed under `-y`.
- RO/immutable preflights abort with clear messages.
- State persisted; hook relinks managed set; state report matches reality.
- Owner verification warns by default; blocks with `--strict-ownership`.
- Concurrency lock prevents parallel runs.
- Override trust checks enforced (or bypassed only with `--force`).

## Implementation order (suggested)

1. State persistence + CLI `relink-managed` + state report.
2. Pacman hook installer + e2e upgrade test.
3. sudo-rs verifier with revert-on-fail.
4. AUR preflight for findutils.
5. Single-instance lock.
6. Owner verification + strict mode flag.
7. Source trust checks + `--force`.
8. FS preflights (mount/immutable).
9. Symlink no-follow refactor (bigger change; follow with targeted tests).
10. Minor: remove `unwrap()` in progress style.
