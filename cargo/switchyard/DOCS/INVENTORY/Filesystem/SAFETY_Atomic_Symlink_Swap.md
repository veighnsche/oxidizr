# Atomic symlink swap (TOCTOU-safe)

- Category: Safety
- Maturity: Silver
- Owner(s): <owner>
- Last reviewed: 2025-09-13
- Next review due: 2025-10-13
- Related PR(s): <#NNNN>

## Summary

Performs symlink replacement using a TOCTOU-safe sequence with directory handles: `open parent O_DIRECTORY|O_NOFOLLOW → symlinkat(tmp) → renameat(tmp→final) → fsync(parent)`.

Pros & Cons

| Pros | Proof (code/tests) |
| --- | --- |
| TOCTOU-safe replacement via dirfd ops | `cargo/switchyard/src/fs/atomic.rs::atomic_symlink_swap()` uses `open_dir_nofollow` and dirfd syscalls |
| Atomic rename for same-FS swaps | `renameat` used for final placement; tests in `fs/swap.rs::tests` |
| Durability improved by fsync(parent) | `fsync_parent_dir` called; perf tracked via `FSYNC_WARN_MS` |
| Degraded fallback under EXDEV (policy-gated) | Fallback to unlink+`symlinkat` when `allow_degraded_fs=true`; `degraded=true` fact |

| Cons | Notes |
| --- | --- |
| Cross-FS cannot be fully atomic | EXDEV forces degraded path if explicitly allowed; otherwise STOP with `E_EXDEV` |
| Parent fsync overhead | Small perf cost; warn threshold tracked; configurable only via code constants currently |

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

## Feature Analytics

- Complexity: Medium. Atoms in `fs/atomic.rs` + orchestration in `fs/swap.rs`; syscall sequence and error mapping.
- Risk & Blast Radius: Affects symlink mutation paths; guarded by `SafePath` and policy gating; degraded path can momentarily leave window on EXDEV.
- Performance Budget: Parent fsync adds small overhead; warn threshold via `constants::FSYNC_WARN_MS`.
- Observability: Emits `degraded`, `duration_ms` and perf fields; facts captured in `apply.result`.
- Test Coverage: Unit/integration tests in `fs/swap.rs::tests`; gap: EXDEV simulation/goldens.
- Determinism & Redaction: Facts go through redaction; DryRun timestamps zeroed.
- Policy Knobs: `allow_degraded_fs`, `require_backup_durability`.
- Exit Codes & Error Mapping: `E_ATOMIC_SWAP` (40) on swap errors; `E_EXDEV` (50) when degraded forbidden.
- Concurrency/Locking Touchpoints: Uses global apply lock; no per-target lock.
- Cross-FS/Degraded Behavior: Explicitly handled via `EXDEV` fallback under policy.
- Platform Notes: Linux-focused semantics; relies on POSIX-like syscalls.
- DX Ergonomics: Encapsulated API via `replace_file_with_symlink()`; call sites remain simple.

Policy Controls Matrix

| Flag | Default | Effect |
| --- | --- | --- |
| `allow_degraded_fs` | `false` | Permit unlink+`symlinkat` fallback on EXDEV; otherwise STOP with `E_EXDEV` |
| `require_backup_durability` | `true` | Controls whether parent fsync is attempted/recorded durable |

Exit Reasons / Error → Exit Code Map

| Error ID | Exit Code | Where mapped |
| --- | --- | --- |
| `E_ATOMIC_SWAP` | `40` | `cargo/switchyard/src/api/errors.rs::{exit_code_for, exit_code_for_id_str}` |
| `E_EXDEV` | `50` | Same mapping; emitted when cross-FS and degraded not allowed |

Observability Map

| Fact | Fields (subset) | Schema |
| --- | --- | --- |
| `apply.result` | `degraded`, `duration_ms`, `before_kind`, `after_kind`, `perf.swap_ms` | Minimal Facts v1 |

Test Coverage Map

| Path | Test name | Proves |
| --- | --- | --- |
| `src/fs/swap.rs` | basic swap tests | atomic rename path works |
| `src/fs/swap.rs` | restore round-trip | swap + restore inverse behavior |
| SPEC features | `features/atomic_swap.feature` | EXDEV degraded fallback semantics |

## Maturity & Upgrade Path

| Tier | Capabilities | Required Guarantees | Tests/Proofs | Ops/Tooling | Relationship to Previous Tier |
| --- | --- | --- | --- | --- | --- |
| Bronze | Basic swap without perf/observability | Same-FS atomicity via rename; best-effort fsync | Unit tests | None | Additive |
| Silver (current) | Dirfd TOCTOU-safe sequence; degraded path policy-gated; perf fields | Fail-closed on policy; recorded degraded; perf tracked | Unit+integration; emit fields validated | Inventory entry | Additive |
| Gold | Goldens for EXDEV path; failure injection; perf budgets characterized | Deterministic artifact/goldens; budget adherence | Goldens + CI gates | CI gates, perf alerts | Additive |
| Platinum | Cross-FS robust strategy or verified invariants; platform matrix | Strong guarantees and multi-platform validation | Property/syscall model tests | Continuous compliance | Additive |

## Gaps and Risks

- Cross-filesystem swap behavior limited to degraded unlink+symlink; no two-phase rename across mounts.
- No formal perf budget beyond `FSYNC_WARN_MS`.

## Next Steps to Raise Maturity

- Golden fixtures for EXDEV and failure paths; CI contention tests.
- Add per-filesystem coverage (tmpfs/ext4/btrfs) if in scope.

## Related

- SPEC v1.1 (TOCTOU-safe syscall sequence, degraded mode).
- `cargo/switchyard/src/constants.rs::FSYNC_WARN_MS`.
