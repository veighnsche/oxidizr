// Temporary compatibility shim: expose preflight checks under policy/checks
// so callers can migrate off crate::preflight::* gradually.

pub use crate::preflight::{check_immutable, check_source_trust, ensure_mount_rw_exec};
