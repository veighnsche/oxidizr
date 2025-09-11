use std::env;
use crate::constants::{RESCUE_MUST_HAVE, RESCUE_MIN_COUNT};

/// Verify that at least one rescue toolset is available on PATH (BusyBox or GNU core utilities).
/// Wrapper that does not enforce executability checks.
pub fn verify_rescue_tools() -> bool {
    verify_rescue_tools_with_exec(false)
}

/// Verify rescue tooling with optional executability check.
/// When `exec_check` is true, the discovered binaries must have at least one execute bit set.
pub fn verify_rescue_tools_with_exec(exec_check: bool) -> bool {
    // Test override knobs:
    if let Some(v) = env::var_os("SWITCHYARD_FORCE_RESCUE_OK") {
        if v == "1" { return true; }
        if v == "0" { return false; }
    }
    // Prefer BusyBox (single binary) as a compact rescue profile
    if let Some(p) = which_on_path("busybox") {
        if !exec_check || is_executable(&p) { return true; }
    }
    // Fallback: require a tiny subset of GNU core tools to be present
    let must_have = RESCUE_MUST_HAVE;
    let mut found = 0usize;
    for bin in must_have.iter() {
        if let Some(p) = which_on_path(bin) {
            if !exec_check || is_executable(&p) { found += 1; }
        }
    }
    // Heuristic: at least RESCUE_MIN_COUNT present counts as available for rescue in this minimal check
    found >= RESCUE_MIN_COUNT
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
