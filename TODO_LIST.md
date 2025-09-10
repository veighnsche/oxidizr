# TODO — Verified Gaps & Bugs

Legend:

- Status: Implemented | Partially Implemented | Needs Work | Bug Verified
- Evidence cites files/symbols where behavior was inspected.

## A. CLI & UX

1. Exit codes are incomplete / inconsistent

- Status: Implemented
- Evidence: Centralized mapping via `src/error.rs::impl Error::exit_code()`. `src/main.rs` delegates to it. Added new error variants for CLI misuse, pacman lock timeout, root required, filesystem unsuitable, and hook install error.
- Action: None (refinement pass later as planned).

2. `--dry-run` leaks side-effects

- Status: Implemented
- Evidence: `src/state/mod.rs` now threads `dry_run` through `save_state`/`set_enabled` and skips writes; `src/cli/handler.rs` suppresses `write_state_report` under dry-run; `src/system/hook.rs` exposes `hook_body()`/`hook_path()` and `InstallHook` prints the plan in dry-run; `src/logging/init.rs` disables the audit file sink in dry-run.
- Action: None.

3. Ambiguous experiment selection (`--all` and `--experiments` together)

- Status: Implemented
- Evidence: `src/cli/parser.rs` adds `conflicts_with_all` so `--all` conflicts with `--experiments`/`--experiment`.
- Action: None.

4. Non-interactive `--assume-yes` suppression

- Status: Implemented
- Evidence: Prompts are gated with `assume_yes` in `src/cli/handler.rs::handle_cli()` and `src/experiments/mod.rs::check_download_prerequisites()`.
- Action: None.

## B. Safety & Privilege

5. Root enforcement not uniform

- Status: Implemented
- Evidence: Mutating subcommands enforce root when not `--dry-run`; `InstallHook` now prints plan in dry-run. `enforce_root()` maps to `Error::RootRequired`.
- Action: None.

6. Immutable/attr checks best-effort only

- Status: Partially Implemented
- Evidence: `src/system/fs_checks.rs` maps immutable/noexec/ro mount issues to `Error::FilesystemUnsuitable` with remediation hints. Further IO error mapping at other edges can be added later.
- Action: Consider broader IO error mapping in symlink operations and worker edges.

7. TOCTOU on restore/link

- Status: Implemented
- Evidence: `src/symlink/ops.rs::restore_file()` now uses `open_dir_nofollow()` and `renameat` with parent directory fsync.
- Action: None.

## C. Filesystem Linking & Restore

8. Backup policy drift

- Status: Implemented
- Evidence: Suffix is `.oxidizr.bak` via `src/symlink/ops.rs::BACKUP_SUFFIX` and `backup_path()`. Overwrites are idempotent.
- Action: None.

9. Best-effort restore can leave unsafe state

- Status: Needs Work (non-sudo suites)
- Evidence: `sudo-rs` verifies restored state; other suites do not yet.
- Action: Add post-restore verifiers for coreutils/findutils/checksums.

10. Path traversal defenses

- Status: Implemented
- Evidence: `restore_file()` now mirrors no-follow parent open and atomic rename.
- Action: None.

11. Unified binary detection brittle

- Status: Implemented
- Evidence: Coreutils/checksums prefer unified dispatcher (`/usr/bin/coreutils` or PATH), else fallback to per-applet and last-resort PATH (`src/experiments/coreutils.rs`, `checksums.rs`).
- Action: Optionally log exact per-applet path when falling back to PATH.

## D. Package Manager / AUR

12. Pacman DB lock handling

- Status: Implemented
- Evidence: `update_packages`/`install_package`/`remove_package` return `Error::PacmanLockTimeout` on lock timeout; exit code 50 via centralized mapper.
- Action: Optional jitter/cancel hooks.

13. AUR helper selection & privilege

- Status: Partially Implemented
- Evidence: Runs helpers via `su - <user> -c`, see `install_package()`; no explicit env sanitization.
- Action: Consider controlled env (HOME/PATH minimal) when invoking via `Command` or `su -c`.

14. “Reinstall requested” no-op

- Status: Implemented
- Evidence: `check_download_prerequisites()` returns a boolean reinstall flag; `install_package()` accepts `reinstall` and omits `--needed` to force reinstall via pacman.
- Action: None.

15. Repo gating for derivatives

- Status: Implemented
- Evidence: `extra_repo_available()` probes `pacman-conf --repo-list`, `pacman -Sl extra`, `pacman-conf -l`, and `/etc/pacman.conf`; gating enforced in `check_download_prerequisites()`.
- Action: None.

## E. State & Hooks

16. State writes under dry-run

- Status: Implemented
- Evidence: `set_enabled()`/`save_state()` accept `dry_run` and skip writes; CLI suppresses `write_state_report` under dry-run.
- Action: None.

17. State drift → relink

- Status: Implemented
- Evidence: `relink_managed()` now drops unknown experiments from state and persists the update.
- Action: None.

18. Hook install ignores dry-run

- Status: Implemented
- Evidence: CLI prints planned path and hook body under dry-run; no filesystem writes.
- Action: None.

19. Hook action hardcodes flags

- Status: Partially Implemented
- Evidence: `hook_body()` now uses hyphenated flags `--no-update --no-progress`.
- Action: Add a small unit test for hook body generation (pending).

## F. Compatibility & Selection

20. Compat check inconsistencies

- Status: Implemented
- Evidence: `Commands::Check` aggregates and returns `Error::Incompatible` when any experiment is incompatible.
- Action: None.

21. Experiment ordering constraints

- Status: Implemented (implicit)
- Evidence: Selection respects registry order (`all_experiments()` → filter preserves order). Info log warns when coreutils+findutils both selected.
- Action: Optional: make constraints explicit with user-visible gating/errors.

## G. Logging & Audit

22. Audit sink fallback

- Status: Implemented
- Evidence: Audit sink choice (primary or HOME fallback) is announced once at initialization.
- Action: None.

23. Sensitive data in audit

- Status: Needs Review
- Evidence: `audit_event()` masks secrets; `audit_event_fields()` does not mask, but current call sites don’t include secrets/tokens.
- Action: Optionally apply masking to `audit_event_fields()` or audit inputs before logging.

## H. Error Messages & Mapping

24. One error → many messages

- Status: Partially Implemented
- Evidence: Added `FilesystemUnsuitable` mapping for immutable/noexec/ro mount via `fs_checks`. Additional IO error contextualization at other edges remains to be done.
- Action: Extend mapping across more IO sites if needed.

25. Uniform exit code table

- Status: Implemented (initial table)
- Evidence: Centralized in `Error::exit_code()` with prepared codes.
- Action: Document in README (optional).

## I. Tests & Idempotency

26. Idempotency tests

- Status: Needs Work
- Evidence: Cargo tests cover unit-ish helpers; idempotency/retry/relink flows not covered here (E2E suites exist under `tests/` YAML but not as cargo tests).
- Action: Add integration tests using tmp roots/sandboxes for repeated enable/disable/partial failure and relink after upgrades.

27. Rollback gaps on sudo-rs

- Status: Partially Implemented
- Evidence: `verify_post_enable()` reverts on failure (restores and removes aliases) and returns the single error; no aggregated multi-error.
- Action: Consider collecting multiple restore errors then returning an aggregated error.

## J. Edge Cases

28. Non-TTY / CI modes

- Status: Implemented
- Evidence: `src/ui/progress.rs` relies on `atty`, has `--no-progress`, avoids ANSI when not TTY; provides “PB> …” host lines.
- Action: None.

29. Missing `pacman`, `pacman-conf`, `lsattr`

- Status: Needs Work
- Evidence: Missing tools result in `std::io::Error` bubbling up (exit 1) without clear remediation.
- Action: Preflight checks with friendly messages and clear remediation when critical tools are absent.

30. PATH dependence / HOME sources

- Status: Implemented
- Evidence: Source trust checks disallow HOME unless `--force` in `src/system/fs_checks.rs::check_source_trust()`.
- Action: None.

---

## Quick Wins (status)

1. Dry-run hardening (Items 2, 16, 18) — Completed
2. Root & exit-code unification (Items 5, 1, 25) — Completed (initial table)
3. Restore/link atomicity audit (Items 7, 10) — Completed
4. AUR/pacman flow correctness (Items 12, 13, 14) — Partially Completed (Item 13 pending)
5. State/Hook resilience (Items 17, 19) — Partially Completed (hook unit test pending)
6. Compat & ordering enforcement (Items 20, 21) — Completed (21 already implicit)
