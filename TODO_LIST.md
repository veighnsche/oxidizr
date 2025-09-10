# TODO — Verified Gaps & Bugs

Legend:

- Status: Implemented | Partially Implemented | Needs Work | Bug Verified
- Evidence cites files/symbols where behavior was inspected.

## A. CLI & UX

1. Exit codes are incomplete / inconsistent

- Status: Needs Work
- Evidence: `src/main.rs` maps only {10,20,30,40} for `Incompatible`, `NothingToLink`, `RestoreBackupMissing`, `RepoGateFailed`; all others -> 1.
- Action: Introduce centralized exit code mapping (e.g., `impl Error { fn exit_code(&self)->i32 }`) and cover full table (1 general; 2 CLI misuse; 10 incompatible; 20 nothing to link; 30 missing backup; 40 repo/AUR gating; 50 pacman lock timeout; 70 root required; 80 immutable/noexec; 90 hook install error).

2. `--dry-run` leaks side-effects

- Status: Bug Verified
- Evidence: `src/system/hook.rs::install_pacman_hook()` writes unconditionally; `src/state/mod.rs::save_state()` and `write_state_report()` always write; audit writer `src/logging/init.rs` always opens/creates sink.
- Action: Gate writes under dry-run. Options:
  - Add `dry_run_guard!()` or thread `dry_run` to `state`/`hook` modules; in dry-run, print planned path/diff only; suppress `state.json`/`state-report.txt` writes. Consider making audit optional or log-only to stderr during dry-run.

3. Ambiguous experiment selection (`--all` and `--experiments` together)

- Status: Needs Work
- Evidence: `src/cli/handler.rs` silently prefers `--all` when true; no warning/error.
- Action: Add clap validation (`conflicts_with_all`) or explicit check to warn/error when both are set.

4. Non-interactive `--assume-yes` suppression

- Status: Implemented
- Evidence: Prompts are gated with `assume_yes` in `src/cli/handler.rs::handle_cli()` and `src/experiments/mod.rs::check_download_prerequisites()`.
- Action: None.

## B. Safety & Privilege

5. Root enforcement not uniform

- Status: Partially Implemented
- Evidence: `src/cli/handler.rs` enforces root for mutating commands when not `--dry-run`. However, `InstallHook` still writes during dry-run → privileged write without euid check.
- Action: Introduce `require_root_or_dry_run("cmd")` early per subcommand and ensure no writes occur in dry-run paths.

6. Immutable/attr checks best-effort only

- Status: Needs Work
- Evidence: `src/system/fs_checks.rs::check_immutable()` uses `lsattr` if available; on absence/failure, proceeds; later ops may fail with opaque EPERM/EROFS.
- Action: On write failure, map `EPERM/EROFS` to friendly hint (e.g., immutable bit, mount flags). Treat missing `lsattr` as unknown, but wrap IO errors with guidance.

7. TOCTOU on restore/link

- Status: Partially Implemented
- Evidence: Linking uses `renameat` + dir `O_NOFOLLOW` + parent fsync in `src/symlink/ops.rs::atomic_symlink_swap()`/`open_dir_nofollow()`. Restore uses `fs::rename` + fsync but not `O_NOFOLLOW`/`renameat`.
- Action: Mirror atomic discipline in restore: `renameat` with open parent fd and `O_DIRECTORY|O_NOFOLLOW`.

## C. Filesystem Linking & Restore

8. Backup policy drift

- Status: Implemented
- Evidence: Suffix is `.oxidizr.bak` via `src/symlink/ops.rs::BACKUP_SUFFIX` and `backup_path()`. Overwrites are idempotent.
- Action: None.

9. Best-effort restore can leave unsafe state

- Status: Needs Work (non-sudo suites)
- Evidence: `sudo-rs` disable checks targets are not symlinks after restore (`src/experiments/sudors.rs`), but coreutils/findutils/checksums do not assert post-restore state.
- Action: After restore, verify critical targets are not symlinks (at least for sudo and other critical suites). Consider policy per experiment.

10. Path traversal defenses

- Status: Partially Implemented
- Evidence: `replace_file_with_symlink()` validates paths and uses `open_dir_nofollow()`; `restore_file()` lacks equivalent `O_NOFOLLOW` guard.
- Action: Add no-follow parent open on restore path as well.

11. Unified binary detection brittle

- Status: Implemented
- Evidence: Coreutils/checksums prefer unified dispatcher (`/usr/bin/coreutils` or PATH), else fallback to per-applet and last-resort PATH (`src/experiments/coreutils.rs`, `checksums.rs`).
- Action: Optionally log exact per-applet path when falling back to PATH.

## D. Package Manager / AUR

12. Pacman DB lock handling

- Status: Implemented (exit code mapping missing)
- Evidence: `wait_for_pacman_lock_clear()` with timeout in `src/system/worker/packages.rs`, called by update/install/remove. Lacks jitter/cancel hooks and exit code 50.
- Action: Add jitter if desired; map lock timeout to code 50.

13. AUR helper selection & privilege

- Status: Partially Implemented
- Evidence: Runs helpers via `su - <user> -c`, see `install_package()`; no explicit env sanitization.
- Action: Consider controlled env (HOME/PATH minimal) when invoking via `Command` or `su -c`.

14. “Reinstall requested” no-op

- Status: Bug Verified
- Evidence: `check_download_prerequisites()` can set `reuse=false`, but `install_package()` returns early when package already installed.
- Action: If reinstall requested, pass appropriate flags (`--overwrite`/remove then install) or force reinstall path; avoid early return.

15. Repo gating for derivatives

- Status: Implemented
- Evidence: `extra_repo_available()` probes `pacman-conf --repo-list`, `pacman -Sl extra`, `pacman-conf -l`, and `/etc/pacman.conf`; gating enforced in `check_download_prerequisites()`.
- Action: None.

## E. State & Hooks

16. State writes under dry-run

- Status: Bug Verified
- Evidence: `state::set_enabled()` and `write_state_report()` always write; invoked from enable/disable flows even when worker is dry-run.
- Action: Gate `save_state` / `write_state_report` on `!dry_run`.

17. State drift → relink

- Status: Needs Work
- Evidence: `relink_managed()` warns on unknown experiment but does not remove from state or persist cleanup.
- Action: Warn, drop unknown entries, and persist updated state.

18. Hook install ignores dry-run

- Status: Bug Verified
- Evidence: `src/cli/handler.rs` skips root check on dry-run but still calls `install_pacman_hook()` which writes.
- Action: Respect dry-run: print intended hook path and contents only; do not write.

19. Hook action hardcodes flags

- Status: Needs Work
- Evidence: `src/system/hook.rs::hook_body()` uses `--assume-yes --no_update --no_progress`. Clap long flags for these fields default to hyphenated names (e.g., `--no-update`, `--no-progress`) per `src/cli/parser.rs`. Using underscores may not be accepted by clap.
- Action: Change hook exec to use `--no-update --no-progress` (hyphenated) and add a unit test for hook body generation.

## F. Compatibility & Selection

20. Compat check inconsistencies

- Status: Needs Work
- Evidence: `Commands::Check` prints compatibility and always exits 0; no `--strict` semantics.
- Action: Aggregate and return non-zero when any incompatible, or add `--strict` flag.

21. Experiment ordering constraints

- Status: Implemented (implicit)
- Evidence: Selection respects registry order (`all_experiments()` → filter preserves order). Info log warns when coreutils+findutils both selected.
- Action: Optional: make constraints explicit with user-visible gating/errors.

## G. Logging & Audit

22. Audit sink fallback

- Status: Partially Implemented
- Evidence: Fallback from `/var/log` to `$HOME/.oxidizr-arch-audit.log` in `src/logging/init.rs`. Not announced.
- Action: Announce chosen sink once at startup.

23. Sensitive data in audit

- Status: Needs Review
- Evidence: `audit_event()` masks secrets; `audit_event_fields()` does not mask, but current call sites don’t include secrets/tokens.
- Action: Optionally apply masking to `audit_event_fields()` or audit inputs before logging.

## H. Error Messages & Mapping

24. One error → many messages

- Status: Partially Implemented
- Evidence: Many errors have context, but no consistent mapping of `io::ErrorKind` (e.g., EROFS/EPERM) to friendly guidance.
- Action: Map common kinds to actionable hints (immutable bit, mount flags, missing backup) at edges like `symlink/ops.rs` and `worker/fs_ops.rs`.

25. Uniform exit code table

- Status: Needs Work
- Evidence: See Item 1; only partial mapping implemented.
- Action: Implement full table; document in README.

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

## Quick Wins (recommended order)

1. Dry-run hardening (Items 2, 16, 18)
2. Root & exit-code unification (Items 5, 1, 25)
3. Restore/link atomicity audit (Items 7, 10)
4. AUR/pacman flow correctness (Items 12, 13, 14)
5. State/Hook resilience (Items 17, 19)
6. Compat & ordering enforcement (Items 20, 21)
