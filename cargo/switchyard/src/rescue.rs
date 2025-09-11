use std::env;

/// Verify that at least one rescue toolset is available on PATH (BusyBox or GNU core utilities).
/// This is a minimal, non-invasive check for Sprint 3.
pub fn verify_rescue_tools() -> bool {
    // Test override knobs:
    if let Some(v) = env::var_os("SWITCHYARD_FORCE_RESCUE_OK") {
        if v == "1" { return true; }
        if v == "0" { return false; }
    }
    // Prefer BusyBox (single binary) as a compact rescue profile
    if which_on_path("busybox").is_some() {
        return true;
    }
    // Fallback: require a tiny subset of GNU core tools to be present
    let must_have = ["cp", "mv", "rm", "ln", "stat", "readlink", "sha256sum", "sort", "date", "ls"];
    let mut found = 0usize;
    for bin in must_have.iter() {
        if which_on_path(bin).is_some() { found += 1; }
    }
    // Heuristic: at least 6/10 present counts as available for rescue in this minimal check
    found >= 6
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
