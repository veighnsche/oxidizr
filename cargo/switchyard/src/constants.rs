//! Shared crate-wide constants for Switchyard.
//!
//! Centralizes magic values and default labels used across modules.
//! Adjusting these here will propagate through the crate.

/// Default logical tag used for naming backup artifacts and sidecar files.
/// Example filenames: `.<name>.<tag>.<millis>.bak` and `.<name>.<tag>.<millis>.bak.meta.json`.
pub const DEFAULT_BACKUP_TAG: &str = "switchyard";

/// Temporary filename suffix used for atomic symlink swap staging within a directory.
/// The temporary name is constructed as `.{{fname}}{TMP_SUFFIX}`; e.g., `.ls.switchyard.tmp`.
pub const TMP_SUFFIX: &str = ".switchyard.tmp";

/// Threshold in milliseconds above which an fsync duration is annotated with a WARN severity
/// in Audit v2. See `api/apply.rs`.
pub const FSYNC_WARN_MS: u64 = 50;

/// Poll interval in milliseconds for the file-backed lock manager (see `adapters/lock_file.rs`).
pub const LOCK_POLL_MS: u64 = 25;

/// Default lock timeout used by `Switchyard::new()` unless overridden by `with_lock_timeout_ms()`.
pub const DEFAULT_LOCK_TIMEOUT_MS: u64 = 5_000;

/// UUIDv5 namespace tag for deterministic plan/action IDs.
/// Derived from SPEC Reproducible v1.1 guidance; see `SPEC/SPEC.md` § Determinism.
pub const NS_TAG: &str = "https://oxidizr-arch/switchyard";

/// Heuristic for rescue tool availability when BusyBox is not present.
/// At least `RESCUE_MIN_COUNT` of the `RESCUE_MUST_HAVE` tools must be found on PATH.
pub const RESCUE_MUST_HAVE: &[&str] = &[
    "cp",
    "mv",
    "rm",
    "ln",
    "stat",
    "readlink",
    "sha256sum",
    "sort",
    "date",
    "ls",
];
pub const RESCUE_MIN_COUNT: usize = 6;
