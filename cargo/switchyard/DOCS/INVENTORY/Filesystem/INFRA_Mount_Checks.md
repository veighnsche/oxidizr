# Mount checks (rw+exec)

- Category: Infra
- Maturity: Silver

## Summary

Ensures targets reside on mounts that are writable and executable before mutations proceed.

## Implementation

- Mount inspector and helper: `cargo/switchyard/src/fs/mount.rs::{MountInspector, ProcStatfsInspector, ensure_rw_exec}`.
- Preflight wrapper: `cargo/switchyard/src/preflight/checks.rs::ensure_mount_rw_exec()`.

## Wiring Assessment

- Preflight and gating iterate additional mount roots from policy and verify target.
- Conclusion: wired correctly; fail-closed on ambiguity.

## Evidence and Proof

- Unit tests for `ensure_rw_exec` with mocked inspector.

## Gaps and Risks

- Parser relies on `/proc/self/mounts`; no statfs-based fallback yet.

## Next Steps to Raise Maturity

- Add rustix::statfs-based implementation; golden tests for common mount configs.

## Related

- Policy `extra_mount_checks`.
