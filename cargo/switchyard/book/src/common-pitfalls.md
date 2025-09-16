# Common Pitfalls

Avoid these integration mistakes to keep operations safe and predictable.

- Missing LockManager in Commit mode → concurrent mutations; expect `WARN` facts and risk of contention.
- Accepting raw `PathBuf` at API boundaries → path traversal and TOCTOU hazards; always construct `SafePath` first.
- Disabling preflight gates in production (`apply.override_preflight=true`) → surprises at apply time.
- No rescue profile on PATH → limited break-glass options during recovery.
- EXDEV not anticipated → degraded mode disabled causes apply failure on cross-filesystem operations.
- Mount options not verified → `ro`, `noexec`, or immutable attributes will cause STOPs.
- SUID/SGID and hardlink hazards ignored → enforce policy and preflight checks for these node types.
- No audit sink configured → low observability; set durable sinks and validate against schema v2.
- Secrets leakage in facts → ensure strict redaction policy and validate in CI.
- Golden fixtures drifting → enforce byte-identical redacted facts across DryRun and Commit.
