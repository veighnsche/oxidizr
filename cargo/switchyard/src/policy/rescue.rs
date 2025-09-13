//! Rescue tool availability verification.
//!
//! This module verifies that a rescue profile is available before mutations happen when
//! required by policy. Two profiles are supported:
//! - BusyBox present on PATH (preferred single-binary profile)
//! - GNU core tools subset present on PATH (configurable minimum count)
//!
//! Test override knobs:
//! - `SWITCHYARD_FORCE_RESCUE_OK=1|0` forces the result for testing.
//!
use crate::constants::{RESCUE_MIN_COUNT, RESCUE_MUST_HAVE};
use std::env;
use crate::types::{RescueError, RescueStatus};

/// Verify that at least one rescue toolset is available on PATH (BusyBox or GNU core utilities).
/// Wrapper that does not enforce executability checks.
pub fn verify_rescue_tools() -> bool {
    verify_rescue(false).is_ok()
}

/// Verify rescue tooling with optional executability check.
/// When `exec_check` is true, the discovered binaries must have at least one execute bit set.
pub fn verify_rescue_tools_with_exec(exec_check: bool) -> bool {
    verify_rescue(exec_check).is_ok()
}

/// Verify rescue tooling with an explicit minimum count for the GNU subset when BusyBox is absent.
pub fn verify_rescue_tools_with_exec_min(exec_check: bool, min_count: usize) -> bool {
    verify_rescue_min(exec_check, min_count).is_ok()
}

pub fn verify_rescue(exec_check: bool) -> Result<RescueStatus, RescueError> {
    verify_rescue_min(exec_check, RESCUE_MIN_COUNT)
}

fn verify_rescue_min(exec_check: bool, min_count: usize) -> Result<RescueStatus, RescueError> {
    // Test override knobs:
    if let Ok(v) = env::var("SWITCHYARD_FORCE_RESCUE_OK") {
        let v = v.trim();
        if v == "1" {
            return Ok(RescueStatus::GNU {
                found: min_count,
                min: min_count,
            });
        }
        if v == "0" {
            return Err(RescueError::Unavailable);
        }
    }
    // Prefer BusyBox (single binary) as a compact rescue profile
    if let Some(p) = which_on_path("busybox") {
        if !exec_check || is_executable(&p) {
            return Ok(RescueStatus::BusyBox { path: p });
        }
    }
    // Fallback: require a tiny subset of GNU core tools to be present
    let must_have = RESCUE_MUST_HAVE;
    let mut found = 0usize;
    for bin in must_have.iter() {
        if let Some(p) = which_on_path(bin) {
            if !exec_check || is_executable(&p) {
                found += 1;
            }
        }
    }
    if found >= min_count {
        Ok(RescueStatus::GNU {
            found,
            min: min_count,
        })
    } else {
        Err(RescueError::Unavailable)
    }
}

fn which_on_path(bin: &str) -> Option<String> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let cand = dir.join(bin);
        if cand.exists() {
            return Some(cand.display().to_string());
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(md) = std::fs::metadata(path) {
        let mode = md.permissions().mode();
        return (mode & 0o111) != 0;
    }
    false
}

#[cfg(not(unix))]
fn is_executable(_path: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn forced_ok_env_yields_ok() {
        env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "1");
        let r = verify_rescue(false);
        env::remove_var("SWITCHYARD_FORCE_RESCUE_OK");
        assert!(r.is_ok());
    }

    #[test]
    #[serial]
    fn forced_fail_env_yields_err() {
        env::set_var("SWITCHYARD_FORCE_RESCUE_OK", "0");
        let r = verify_rescue(false);
        env::remove_var("SWITCHYARD_FORCE_RESCUE_OK");
        assert!(r.is_err());
    }
}
