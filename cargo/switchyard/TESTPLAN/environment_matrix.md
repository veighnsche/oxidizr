# Environment Matrix (Switchyard Library)

Defines environment axes relevant to the library API and maps which scenarios run on which envs. Controls combinatorial blow-up via base sets and rotation policy.

## Environment Axes

- Filesystem flavor:
  - ext4 (default), xfs, btrfs (if available), tmpfs (for edge inode behavior)
- Cross-filesystem boundary (EXDEV trigger):
  - same-fs, cross-fs (simulated via `SWITCHYARD_FORCE_EXDEV=1`)
- Permissions/ownership:
  - owner/group: uid/gid variants (root-like vs non-root),
  - mode bits: 0644, 0755, suid/sgid bits present
- Path characteristics:
  - length: short, long (255 bytes), huge (4096 bytes)
  - unicode: present/absent
  - deep nesting levels
- Symlink forms:
  - absolute vs relative symlink destinations
- Disk space thresholds:
  - normal, low-disk (simulate ENOSPC)
- Parallelism / contention:
  - single-thread, rival-lock-holder present
- Crash/kill points (fault injection):
  - none, kill-between-backup-and-rename
- Lock backends:
  - file-based lock (default), custom lock adapter
- Time and clock skew:
  - DryRun (TS_ZERO), Commit (real time)
- Backup inventory shape:
  - none, single, many, with sidecars, tampered payload hash

## Base Environment Sets

- Base-1 (CI quick):
  - ext4-like tmpfs, same-fs, normal disk, single-thread, no crash injection,
  - default ownership (uid=non-root), modes 0644/0755 for files/dirs,
  - unicode disabled, short paths, file lock manager present.

- Base-2 (Daily rotation):
  - same as Base-1 plus unicode paths and long paths, rival lock holder for contention cases.

- Base-3 (Gold/Nightly rotation):
  - cross-fs simulated (EXDEV=1), low-disk for selected cases, tampered backup for integrity checks,
  - relative symlink targets, deep nesting, suid/sgid modes where permitted.

- Base-4 (Platinum/Weekly):
  - exhaustive boundaries: huge path lengths, btrfs/xfs where available, crash injection in swap/restore, parallel stress.

## Mapping Scenarios to Envs (controlled blow-up)

- Bronze tier scenarios run only on Base-1.
- Silver tier scenarios run on Base-1 and rotate through Base-2 weekly.
- Gold tier scenarios run on Base-1 and Base-3; contention and EXDEV-specific scenarios always use Base-3.
- Platinum tier scenarios run on Base-4 only.

## Justification

- EXDEV behavior is fully simulated by `SWITCHYARD_FORCE_EXDEV=1` (see `src/fs/atomic.rs`), avoiding the need to mount multiple FS in CI for most runs.
- Low disk (ENOSPC) is rare; covered in Platinum to constrain cost.
- Filesystem diversity (xfs/btrfs) adds risk coverage but is scheduled for weekly rotation to control time.
- Crash injection focuses on non-atomic windows (between backup and rename) to validate rollback/idempotence.
