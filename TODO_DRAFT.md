# TODO — Gaps & Bugs to Triage

## A. CLI & UX

1. **Exit codes are incomplete / inconsistent** — Add a centralized enum → code map and return codes for all top-level failure classes (permission, pacman lock, missing backup, “nothing to link”, AUR helper missing, compat fail, hook install fail, dry-run attempts real writes). **(VERIFY IN CODE)**
   *Action:* Implement `thiserror` variants + `From<anyhow::Error>` → `ExitCode`.

2. **`--dry-run` leaks side-effects** — Ensure *every* mutating path checks `dry_run` **before** filesystem writes, package invocations, or hook creation. Pay special attention to: pacman hook writer, state.json writes, audit file creation, and symlink/restore helpers. **(VERIFY IN CODE)**
   *Action:* Add `dry_run_guard!()` macro; unit test with a temp root.

3. **Ambiguous experiment selection** — If both `--all` and `--experiments`/`--experiment` provided, define precedence and error/warn instead of silently choosing one. **(VERIFY IN CODE)**
   *Action:* Clap `conflicts_with_all` or explicit validation.

4. **Non-interactive behavior drift** — `--assume-yes` should consistently suppress *all* prompts (reuse existing package, confirmation banners, etc.). **(VERIFY IN CODE)**
   *Action:* Grep for `stdin` reads; gate on `assume_yes`.

## B. Safety & Privilege

5. **Root enforcement not uniform** — Some subcommands (e.g., `install-hook`, `relink-managed`) may attempt privileged writes in dry-run or forget to check euid before file ops. **(VERIFY IN CODE)**
   *Action:* Central `require_root_or_dry_run(cmd)` helper; integrate early.

6. **Immutable/attr checks are best-effort** — If `lsattr` is missing or fails, code may proceed and then hard-fail on write. Add a reliable fallback (e.g., attempt write + precise error) and clearer guidance (`chattr -i`). **(CONFIRMED risk class)**
   *Action:* Treat `lsattr` absence as “unknown”; proceed with a guarded write and map EPERM with hint.

7. **TOCTOU on restore/link** — Ensure atomic rename+fsync for both link and restore paths, and that parent directory handles are opened with `O_DIRECTORY|O_NOFOLLOW`. **(VERIFY IN CODE)**
   *Action:* Audit both “replace” and “restore” code paths for identical atomicity and fsync coverage.

## C. Filesystem Linking & Restore

8. **Backup policy drift** — Symlink targets vs. regular file backups differ; verify backups always end with a single, predictable suffix (`.oxidizr.bak`) and are pruned or idempotent on re-runs. **(VERIFY IN CODE)**
   *Action:* Normalize backup naming; add `ensure_backup_absent_then_create()`.

9. **Best-effort restore can leave unsafe state** — If `--force_restore_best_effort` skips missing backups, some commands (notably `sudo-rs`) should *hard fail* if critical binaries remain symlinks. Enforce consistent policy across experiments. **(VERIFY IN CODE)**
   *Action:* After restore, assert targets are **not** symlinks for critical suites; fail otherwise.

10. **Path traversal defenses incomplete** — Validate both `source` and **parent** components of `target` for `..` and symlinked directories. **(VERIFY IN CODE)**
    *Action:* Use `openat2`-like discipline: open parent dir `O_NOFOLLOW`, operate via FDs.

11. **Unified binary detection brittle** — If the “unified” applet (à la `uu-coreutils`) is absent, fallback must try all well-known install paths and PATH; log which path was chosen. **(VERIFY IN CODE)**
    *Action:* Harden discovery order and log the winning strategy.

## D. Package Manager / AUR

12. **Pacman DB lock handling** — If `--wait-lock` isn’t set, operations may fail immediately; if set, polling interval/backoff should be sane and cancelable. **(VERIFY IN CODE)**
    *Action:* Implement `wait_for_pacman_lock(timeout)` with jitter and clear exit codes.

13. **AUR helper selection & privilege** — Running helpers as root can be hazardous; ensure `--aur-user` reliably drops privileges and that environment is sanitized. **(VERIFY IN CODE)**
    *Action:* Execute via `su -c` with a controlled env; verify `$HOME`, `$PATH` minimal.

14. **“Reinstall requested” no-op** — If user declines reuse and wants reinstall, but code short-circuits on “already installed”, you silently ignore intent. **(VERIFY IN CODE)**
    *Action:* Pass `--overwrite`/`--needed` flags correctly or uninstall then install when user opted to reinstall.

15. **Repo gating for derivatives** — On Manjaro/EndeavourOS/CachyOS, repo names and availability differ; gating logic should detect mirrors and `pacman-conf` quirks. **(CONFIRMED risk class)**
    *Action:* Add multiple strategies (pacman-conf, pacman -Sl, read `/etc/pacman.conf`) with clear audit lines.

## E. State & Hooks

16. **State writes under dry-run** — `state.json` and `state-report.txt` should not be written during dry-run. **(VERIFY IN CODE)**
    *Action:* Gate `save_state`/`write_state_report` on `!dry_run`.

17. **State drift → relink** — `relink-managed` should degrade gracefully if an experiment is renamed/removed in code but still in state; warn, drop, and persist cleanup. **(VERIFY IN CODE)**
    *Action:* On unknown experiment names, remove from state after user-visible warning.

18. **Hook install ignores dry-run** — If `install-hook` tries to write unconditionally, that’s a bug. **(VERIFY IN CODE)**
    *Action:* Respect `dry_run`; print the path and diff instead of writing.

19. **Hook action hardcodes flags** — Ensure hook calls `relink-managed` with `--assume-yes --no-update --no-progress` and *no* interactive paths. **(VERIFY IN CODE)**
    *Action:* Unit test hook body generation.

## F. Compatibility & Selection

20. **Compat check inconsistencies** — `check` should return a *non-zero* exit when any selected experiment is incompatible (or provide `--strict` to do so). Also, `--skip-compat-check` should affect `check` consistently. **(VERIFY IN CODE)**
    *Action:* Aggregate failures; exit 0/1 based on `--strict`.

21. **Experiment ordering constraints** — Enabling `findutils` before `coreutils` (or vice versa) for AUR checksum availability should be enforced, not just logged. **(VERIFY IN CODE)**
    *Action:* Topologically sort selected experiments; hard-gate if order violates constraints unless `--force`.

## G. Logging & Audit

22. **Audit sink fallback** — If `/var/log` is unwritable, fallback to `$HOME` should be explicit and announced once; avoid partial run with missing audit. **(VERIFY IN CODE)**
    *Action:* Probe sink at startup and pin; log chosen path.

23. **Sensitive data in audit** — Scrub environment and command arguments (e.g., usernames in `--aur-user`) in audit lines if necessary; do not log tokens. **(VERIFY IN CODE)**

## H. Error Messages & Mapping

24. **One error → many messages** — Ensure lower-level IO errors are wrapped with actionable hints (e.g., immutable bit, mount flags, missing backup). **(VERIFY IN CODE)**
    *Action:* `context()` every external call; map `io::ErrorKind` → friendly messages.

25. **Uniform exit code table** — Document and implement:

    * `1` general failure; `2` CLI misuse; `10` incompatible; `20` nothing to link; `30` missing backup; `40` repo/AUR gating; `50` pacman lock timeout; `70` permission/root required; `80` immutable/noexec; `90` hook install error. **(ADD)**

## I. Tests & Idempotency

26. **Idempotency tests missing** — Repeated `enable`, repeated `disable`, partial failure then retry, and `relink-managed` after package upgrades should be covered. **(VERIFY IN CODE)**
    *Action:* Integration tests with tmpfs + chroot-like sandbox.

27. **Rollback gaps on sudo-rs** — If post-enable verification fails, ensure aliases and targets are fully rolled back even if one restore fails; return a *single* aggregated error. **(VERIFY IN CODE)**

## J. Edge Cases

28. **Non-TTY / CI modes** — Ensure `--no-progress` and TTY detection disable spinners and reduce log noise; don’t emit control codes in CI. **(VERIFY IN CODE)**

29. **Missing `pacman`, `pacman-conf`, `lsattr`** — Fail early with clear remediation if critical tools are missing. **(VERIFY IN CODE)**

30. **PATH dependence** — Discovery that falls back to `which` can pick userland binaries (HOME). Reject HOME-scoped sources unless `--force`. **(VERIFY IN CODE)**

---

## Quick Wins (order of implementation)

1. **Dry-run hardening** (items 2, 16, 18)
2. **Root & exit-code unification** (items 5, 1, 25)
3. **Restore/link atomicity audit** (items 7, 8, 9, 10)
4. **AUR/pacman flow correctness** (items 12, 13, 14)
5. **State/Hook resilience** (items 17, 19)
6. **Compat & ordering enforcement** (items 20, 21)
