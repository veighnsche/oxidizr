use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::packages::static_fallback_applets;
use crate::{DistroAdapter, PackageKind};

/// Interrogate the replacement binary and return the set of applets it claims to support,
/// intersected with the provided `allow` list. Falls back to `allow` if probing fails
/// or finds an obviously tiny set (< 3).
pub fn discover_applets_with_allow(source_bin: &Path, allow: &[String]) -> Vec<String> {
    fn parse(stdout: &str, allow: &HashSet<&str>) -> Vec<String> {
        let mut out = Vec::new();
        for token in stdout.split(|c: char| c.is_whitespace() || [',', ';', '|', '/'].contains(&c))
        {
            let t = token.trim();
            if !t.is_empty() && allow.contains(t) {
                out.push(t.to_string());
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
    allow.to_vec()
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

/// Intersect distro-provided commands with replacement-supported applets.
pub fn intersect_distro_with_replacement(distro: &[String], repl: &[String]) -> Vec<String> {
    let r: HashSet<&str> = repl.iter().map(|s| s.as_str()).collect();
    let mut out: Vec<String> = distro
        .iter()
        .filter(|d| r.contains(d.as_str()))
        .cloned()
        .collect();
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
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter {
        distro: Vec<String>,
    }
    impl DistroAdapter for MockAdapter {
        fn enumerate_package_commands(&self, _root: &Path, _pkg: PackageKind) -> Vec<String> {
            self.distro.clone()
        }
    }

    #[test]
    fn test_coverage_check_ok() {
        let distro = vec!["ls".into(), "cat".into()];
        let repl = vec!["ls".into(), "cat".into(), "echo".into()];
        assert!(coverage_check(&distro, &repl).is_ok());
    }

    #[test]
    fn test_coverage_check_missing_reports() {
        let distro = vec!["ls".into(), "cat".into(), "mv".into()];
        let repl = vec!["ls".into()];
        let err = coverage_check(&distro, &repl).unwrap_err();
        assert!(err.contains(&"cat".to_string()));
        assert!(err.contains(&"mv".to_string()));
        assert!(!err.contains(&"ls".to_string()));
    }

    #[test]
    fn test_intersect_distro_with_replacement_sorted_unique() {
        let distro = vec!["ls".into(), "cat".into(), "cat".into()];
        let repl = vec!["ls".into(), "echo".into()];
        let out = intersect_distro_with_replacement(&distro, &repl);
        assert_eq!(out, vec!["ls".to_string()]);
    }

    #[test]
    fn test_discover_fallback_when_binary_missing() {
        let allow = vec!["ls".into(), "cat".into()];
        let bogus = Path::new("/definitely/not/a/binary/path");
        let out = discover_applets_with_allow(bogus, &allow);
        // Missing binary → fallback to allow set
        assert_eq!(out, allow);
    }

    #[test]
    fn test_resolve_applets_for_use_intersects_with_distro_when_present() {
        let adapter = MockAdapter {
            distro: vec!["ls".into(), "cat".into()],
        };
        let root = Path::new("/");
        // Non-existent so discovery falls back to static allow; intersection should retain only the mocked distro names that are allowed.
        let source_bin = Path::new("/nonexistent/bin");
        let out = resolve_applets_for_use(&adapter, root, PackageKind::Coreutils, source_bin);
        assert!(out.iter().all(|n| n == "ls" || n == "cat"));
        assert!(!out.is_empty());
    }

    #[test]
    fn test_resolve_applets_for_use_returns_repl_when_no_distro_list() {
        let adapter = MockAdapter { distro: vec![] };
        let root = Path::new("/");
        let source_bin = Path::new("/nonexistent/bin");
        let out = resolve_applets_for_use(&adapter, root, PackageKind::Findutils, source_bin);
        // Fallback returns static set for findutils
        assert!(out.contains(&"find".to_string()));
        assert!(out.contains(&"xargs".to_string()));
    }
}
