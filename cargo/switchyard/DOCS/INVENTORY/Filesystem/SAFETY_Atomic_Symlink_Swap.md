# Atomic symlink swap (TOCTOU-safe)

- Category: Safety
- Maturity: Silver

## Summary

Performs symlink replacement using a TOCTOU-safe sequence with directory handles: `open parent O_DIRECTORY|O_NOFOLLOW → symlinkat(tmp) → renameat(tmp→final) → fsync(parent)`.

## Behaviors

- Opens parent directory with `O_DIRECTORY|O_NOFOLLOW` and operates via dirfd to avoid TOCTOU.
- Creates a temporary symlink name and atomically renames it into place.
- Fsyncs the parent directory to persist metadata and directory entry changes.
- Cleans up temporary names best-effort without failing the operation.
- When `EXDEV` occurs and `allow_degraded_fs=true`, falls back to unlink + `symlinkat` (records `degraded=true`).
- Emits perf fields (`swap_ms`) and flags (`degraded`) in `apply.result`.

## Implementation

- Core atom: `cargo/switchyard/src/fs/atomic.rs::atomic_symlink_swap()` and helpers (`open_dir_nofollow`, `fsync_parent_dir`).
- Orchestration: `cargo/switchyard/src/fs/swap.rs::replace_file_with_symlink()` snapshots state, removes prior node via dirfd, then calls the atomic swap.
- Degraded EXDEV fallback when `allow_degraded_fs=true`.

## Wiring Assessment

- Apply path: `cargo/switchyard/src/api/apply/handlers.rs::handle_ensure_symlink()` invokes `fs::replace_file_with_symlink()`.
- Policy flag `allow_degraded_fs` is threaded from `Policy` to handler to `fs` atoms.
- Facts include `degraded` and `duration_ms` with fsync timing.
- Conclusion: wired correctly; degraded path and perf captured, policy honored.

## Evidence and Proof

- Tests: `cargo/switchyard/src/fs/swap.rs::tests` cover basic swap and round-trip with restore.
- Emit fields: `apply.result` includes `before_kind`, `after_kind`, `degraded`, `duration_ms`.

## Gaps and Risks

- Cross-filesystem swap behavior limited to degraded unlink+symlink; no two-phase rename across mounts.
- No formal perf budget beyond `FSYNC_WARN_MS`.

## Next Steps to Raise Maturity

- Golden fixtures for EXDEV and failure paths; CI contention tests.
- Add per-filesystem coverage (tmpfs/ext4/btrfs) if in scope.

## Related

- SPEC v1.1 (TOCTOU-safe syscall sequence, degraded mode).
- `cargo/switchyard/src/constants.rs::FSYNC_WARN_MS`.
