//! Preflight checks and helpers.
//!
//! This module provides best-effort filesystem and policy gating checks used by the
//! higher-level API. It also exposes a small helper to render a `PreflightReport`
//! into a SPEC-aligned YAML sequence for fixtures and artifacts.
use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn mount_entry_for(path: &Path) -> Option<(PathBuf, String)> {
    // Parse /proc/self/mounts and select the longest mountpoint that prefixes `path`.
    let mut f = match fs::File::open("/proc/self/mounts") {
        Ok(f) => f,
        Err(_) => return None,
    };
    let mut s = String::new();
    if f.read_to_string(&mut s).is_err() {
        return None;
    }
    let p = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let mut best: Option<(PathBuf, String)> = None;
    for line in s.lines() {
        // format: <src> <mountpoint> <fstype> <opts> ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let mnt = PathBuf::from(parts[1]);
        if p.starts_with(&mnt) {
            let opts = parts[3].to_string();
            match &best {
                None => best = Some((mnt, opts)),
                Some((b, _)) => {
                    if mnt.as_os_str().len() > b.as_os_str().len() {
                        best = Some((mnt, opts));
                    }
                }
            }
        }
    }
    best
}

/// Ensure the filesystem backing `path` is read-write and not mounted with noexec.
/// Returns Ok(()) if suitable; Err(String) with a human message otherwise.
pub fn ensure_mount_rw_exec(path: &Path) -> Result<(), String> {
    if let Some((_mnt, opts)) = mount_entry_for(path) {
        let opts_l = opts.to_ascii_lowercase();
        let has_rw = opts_l.split(',').any(|o| o == "rw");
        let noexec = opts_l.split(',').any(|o| o == "noexec");
        if !has_rw || noexec {
            return Err(format!(
                "Filesystem at '{}' not suitable: requires rw and exec (opts: {})",
                path.display(),
                opts
            ));
        }
    }
    Ok(())
}

/// Check immutability (best-effort via `lsattr -d`). Returns Err(String) when immutable.
pub fn check_immutable(path: &Path) -> Result<(), String> {
    let out = std::process::Command::new("lsattr")
        .args(["-d", path.as_os_str().to_string_lossy().as_ref()])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines() {
                let mut fields = line.split_whitespace();
                if let Some(attrs) = fields.next() {
                    if attrs.contains('i') {
                        return Err(format!(
                            "Target '{}' is immutable (chattr +i). Run 'chattr -i {}' to clear before proceeding.",
                            path.display(), path.display()
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Source trust checks. Returns Err(String) if untrusted and `force` is false. When `force` is true,
/// returns Ok(()) and leaves it to callers to emit warnings.
pub fn check_source_trust(source: &Path, force: bool) -> Result<(), String> {
    let meta = fs::symlink_metadata(source).map_err(|e| format!("{}", e))?;
    let mode = meta.mode();
    if (mode & 0o002) != 0 && !force {
        return Err(format!(
            "Untrusted source (world-writable): {}. Pass --force to override.",
            source.display()
        ));
    }
    if meta.uid() != 0 && !force {
        return Err(format!(
            "Untrusted source (not root-owned): {}. Pass --force to override.",
            source.display()
        ));
    }
    ensure_mount_rw_exec(source)?;
    if let Ok(home) = std::env::var("HOME") {
        let home_p = Path::new(&home);
        if source.starts_with(home_p) && !force {
            return Err(format!(
                "Untrusted source under HOME: {}. Pass --force to override.",
                source.display()
            ));
        }
    }
    Ok(())
}

/// Render a SPEC-aligned YAML sequence from a `PreflightReport` rows collection.
/// This exporter is intended for tests and artifacts and preserves only the
/// keys defined in SPEC/preflight.yaml.
pub fn to_yaml(report: &crate::types::report::PreflightReport) -> String {
    use serde_json::Value as J;
    use serde_yaml::Value as Y;
    let mut items: Vec<Y> = Vec::new();
    for row in &report.rows {
        let mut map = serde_yaml::Mapping::new();
        let get = |k: &str| row.get(k).cloned().unwrap_or(J::Null);
        let keys = [
            "action_id",
            "path",
            "current_kind",
            "planned_kind",
            "policy_ok",
            "provenance",
            "notes",
        ];
        for k in keys.iter() {
            let v = get(k);
            if !v.is_null() {
                let y: Y = serde_yaml::to_value(v).unwrap_or(Y::Null);
                map.insert(Y::String((*k).to_string()), y);
            }
        }
        items.push(Y::Mapping(map));
    }
    serde_yaml::to_string(&Y::Sequence(items)).unwrap_or_else(|_| "[]\n".to_string())
}
