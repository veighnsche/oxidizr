use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use crate::{DistroAdapter, PackageKind};
use crate::packages::static_fallback_applets;

/// Interrogate the replacement binary and return the set of applets it claims to support,
/// intersected with the provided `allow` list. Falls back to `allow` if probing fails
/// or finds an obviously tiny set (< 3).
pub fn discover_applets_with_allow(source_bin: &Path, allow: &[String]) -> Vec<String> {
    fn parse(stdout: &str, allow: &HashSet<&str>) -> Vec<String> {
        let mut out = Vec::new();
        for token in stdout.split(|c: char| c.is_whitespace() || [',', ';', '|', '/'].contains(&c)) {
            let t = token.trim();
            if !t.is_empty() && allow.contains(t) {
                out.push(t.to_string());
            }

/// Resolve the set of applets to link during `use` for a given package, by
/// interrogating the replacement binary, intersecting with distro-provided
/// commands when available, and falling back to the static list if needed.
pub fn resolve_applets_for_use<A: DistroAdapter>(
    adapter: &A,
    root: &Path,
    pkg: PackageKind,
    source_bin: &Path,
) -> Vec<String> {
    let allow = static_fallback_applets(pkg);
    let repl = discover_applets_with_allow(source_bin, &allow);
    let distro = adapter.enumerate_package_commands(root, pkg);
    if distro.is_empty() {
        repl
    } else {
        intersect_distro_with_replacement(&distro, &repl)
    }
}

/// Preflight coverage for `replace`: require that the replacement supports all
/// distro-provided commands for the given package; returns Err(missing) if not.
pub fn coverage_preflight<A: DistroAdapter>(
    adapter: &A,
    root: &Path,
    pkg: PackageKind,
    source_bin: &Path,
) -> Result<(), Vec<String>> {
    let allow = static_fallback_applets(pkg);
    let repl = discover_applets_with_allow(source_bin, &allow);
    let distro = adapter.enumerate_package_commands(root, pkg);
    if distro.is_empty() {
        // No distro enumeration (non-live root). Accept preflight; downstream code
        // should still perform post-apply smoke checks.
        return Ok(());
    }
    coverage_check(&distro, &repl)
}
        }
        out.sort();
        out.dedup();
        out
    }
    let allow_set: HashSet<&str> = allow.iter().map(|s| s.as_str()).collect();

    if let Ok(out) = Command::new(source_bin).arg("--list").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let names = parse(&s, &allow_set);
            if names.len() >= 3 {
                return names;
            }
        }
    }
    if let Ok(out) = Command::new(source_bin).arg("--help").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let names = parse(&s, &allow_set);
            if names.len() >= 3 {
                return names;
            }
        }
    }
    allow.iter().cloned().collect()
}

/// Intersect distro-provided commands with replacement-supported applets.
pub fn intersect_distro_with_replacement(distro: &[String], repl: &[String]) -> Vec<String> {
    let r: HashSet<&str> = repl.iter().map(|s| s.as_str()).collect();
    let mut out: Vec<String> = distro.iter().filter(|d| r.contains(d.as_str())).cloned().collect();
    out.sort();
    out.dedup();
    out
}

/// Return Ok(()) if replacement covers all distro commands; Err(missing) otherwise.
pub fn coverage_check(distro: &[String], repl: &[String]) -> Result<(), Vec<String>> {
    let r: HashSet<&str> = repl.iter().map(|s| s.as_str()).collect();
    let missing: Vec<String> = distro
        .iter()
        .filter(|g| !r.contains(g.as_str()))
        .cloned()
        .collect();
    if missing.is_empty() { Ok(()) } else { Err(missing) }
}
