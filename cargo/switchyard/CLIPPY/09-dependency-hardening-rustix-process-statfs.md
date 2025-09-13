# CLIPPY Remediation Plan: dependency hardening (rustix process/statfs, EXDEV, locks)

- Lint: N/A (dependency hygiene, portability, and unsafe removal)

## Proof (code reference)

- Effective UID detection currently parses `/proc`:
  - `src/fs/meta.rs::effective_uid_is_root()`
- Audit envmeta uses `libc` + `unsafe`:
  - `src/logging/audit.rs` (feature `envmeta`): `libc::getppid()`, `libc::geteuid()`, `libc::getegid()`
- EXDEV mapping relies on libc errno:
  - `src/api/apply/handlers.rs` maps `raw_os_error() == libc::EXDEV`
- Mount flags parsed from `/proc/self/mounts`:
  - `src/fs/mount.rs::ProcStatfsInspector::parse_proc_mounts`
- File locking via `fs2::FileExt` with truncation pattern:
  - `src/adapters/lock/file.rs` (`FileLockManager`)

## Goals

- Eliminate `/proc` parsing and direct `libc` calls in favor of `rustix`.
- Remove `unsafe` usage in audit identity collection.
- Prefer `std` or `rustix` error kinds over raw errno comparisons.
- Improve mount flag detection using `statfs(2)` via `rustix`.
- Optionally migrate file locks from `fs2` to `fd-lock` (fcntl-based, rustix-backed).
- Keep all observable behavior and JSON fields identical.

## Architecture alternative (preferred): unify on rustix for low-level primitives

- Enable `rustix` `process` + `fs` features and remove `libc` after call sites are migrated.
- Replace `/proc` textual parsers with direct syscalls (`geteuid`, `getegid`, `getppid`, `statfs`).
- Use `std::io::ErrorKind::CrossesDevices` for EXDEV instead of raw errno.

### Implementation plan (preferred, granular)

- [ ] Cargo: enable rustix process APIs, prepare to remove libc
  - [ ] In `Cargo.toml`: `rustix = { version = "0.38", features = ["fs", "process"] }`
  - [ ] Grep for `libc::` usages and note remaining sites; plan to drop `libc` after all migrations

- [ ] `src/fs/meta.rs`: replace `/proc` parser with rustix
  - [ ] Implement `effective_uid_is_root()` using `rustix::process::geteuid().as_raw() == 0`
  - [ ] Add a unit test that asserts non-root returns `false` (skipped if running as root)

- [ ] `src/logging/audit.rs` (feature `envmeta`): remove unsafe libc calls
  - [ ] Replace `libc::getppid()` with `rustix::process::getppid().as_raw()`
  - [ ] Replace `libc::geteuid()`/`getegid()` with rustix equivalents
  - [ ] Remove `unsafe` blocks and `#[allow(unsafe_code, ...)]` annotations here

- [ ] `src/api/apply/handlers.rs`: EXDEV mapping via std kind
  - [ ] Change mapping to `if e.kind() == std::io::ErrorKind::CrossesDevices { ... }`
  - [ ] Keep behavior of error_id mapping and messages identical

- [ ] `src/fs/mount.rs`: statfs-based flags
  - [ ] Replace `parse_proc_mounts` with `rustix::fs::statfs(path)`
  - [ ] Map flags: `read_only = flags.contains(FsFlags::RDONLY)`, `no_exec = flags.contains(FsFlags::NOEXEC)`
  - [ ] Remove `/proc/self/mounts` parsing; simplify `ProcStatfsInspector`
  - [ ] Add unit tests to validate `ensure_rw_exec()` semantics

- [ ] Optional: `src/adapters/lock/file.rs`: migrate to `fd-lock`
  - [ ] Add `fd-lock` dependency
  - [ ] Replace `fs2::FileExt` with `fd_lock::RwLock` (or write lock) and remove `.truncate(true)` open mode
  - [ ] Ensure `Drop` unlock semantics preserved; update tests in `tests/locking/*`

- [ ] Optional: consistency hardening for symlink ops
  - [ ] `src/fs/backup/snapshot.rs`: replace `std::os::unix::fs::symlink` with `rustix::fs::symlinkat` relative to parent dirfd
  - [ ] `src/fs/meta.rs::resolve_symlink_target()`: consider `readlinkat` via `open_dir_nofollow()` (pure probe; optional)

- [ ] Remove `libc` dependency
  - [ ] After migrations, drop `libc = "0.2"` from `Cargo.toml` if no call sites remain

- [ ] Documentation updates
  - [ ] Update `SPEC/` or README sections to note syscall sources (statfs, process ids) and dependency simplification
  - [ ] Note that mount detection no longer relies on `/proc` text parsing

## Acceptance criteria

- [ ] All unit and integration tests pass unchanged (including `tests/apply/*`, `tests/preflight/*`, `tests/locking/*`).
- [ ] No `unsafe` in `src/logging/audit.rs` envmeta section; no remaining `libc::` call sites.
- [ ] `effective_uid_is_root()` no longer performs I/O or parse `/proc`.
- [ ] `ProcStatfsInspector` uses `statfs`; behavior of `ensure_rw_exec()` remains consistent.
- [ ] EXDEV detection uses `ErrorKind::CrossesDevices` (or rustix errno) with identical error id mapping.
- [ ] If adopted, `fd-lock`-based locking passes all locking tests with no regressions.
- [ ] `Cargo.toml` simplified (remove `libc` if unused; add rustix `process` feature, and `fd-lock` if chosen).

## Test & verification notes

- Run full suite: `cargo test -p switchyard`.
- Add/verify unit tests:
  - `fs/meta.rs`: euid root check (guarded/skipped under root).
  - `fs/mount.rs`: flag mapping and `ensure_rw_exec()` success/failure cases.
  - `locking/*`: contention, timeout, and wait-fact stability.
- Validate end-to-end error mapping:
  - Ensure EXDEV scenarios still map to `E_EXDEV` and non-EXDEV swap failures map to `E_ATOMIC_SWAP`.
- Confirm no `/proc` reads are required for correctness after changes (only optional env metadata like `/proc/version` remains).
