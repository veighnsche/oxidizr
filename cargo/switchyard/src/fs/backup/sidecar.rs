use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BackupSidecar {
    pub(crate) schema: String,     // "backup_meta.v1" | "backup_meta.v2"
    pub(crate) prior_kind: String, // "file" | "symlink" | "none"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prior_dest: Option<String>, // for symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mode: Option<String>, // octal string for file, e.g. "100644"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payload_hash: Option<String>, // sha256 of payload for v2
}

pub(crate) fn sidecar_path_for_backup(backup: &Path) -> PathBuf {
    let s = backup.as_os_str().to_owned();
    use std::ffi::OsString;
    let mut s2 = OsString::from(s);
    s2.push(".meta.json");
    PathBuf::from(s2)
}

pub(crate) fn write_sidecar(backup: &Path, sc: &BackupSidecar) -> std::io::Result<()> {
    let sc_path = sidecar_path_for_backup(backup);
    if let Some(parent) = sc_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let f = std::fs::File::create(&sc_path)?;
    serde_json::to_writer_pretty(&f, sc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // Ensure sidecar durability as well
    let _ = f.sync_all();
    let _ = crate::fs::atomic::fsync_parent_dir(&sc_path);
    Ok(())
}

pub(crate) fn read_sidecar(sc_path: &Path) -> std::io::Result<BackupSidecar> {
    let f = std::fs::File::open(sc_path)?;
    serde_json::from_reader(f).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

// Return (backup_path_if_present, sidecar_path) for the latest timestamped pair.
pub(crate) fn find_latest_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut best: Option<(u128, PathBuf)> = None; // timestamp, base path (without .meta.json)
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            let fname = e.file_name();
            if let Some(s) = fname.to_str() {
                // Accept either .bak or .bak.meta.json
                if let Some(rest) = s.strip_prefix(&prefix) {
                    if let Some(num_s) = rest
                        .strip_suffix(".bak")
                        .or_else(|| rest.strip_suffix(".bak.meta.json"))
                    {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if best.as_ref().map(|(b, _)| num > *b).unwrap_or(true) {
                                // compute base path ending with .bak
                                let base = parent
                                    .join(format!("{}.bak", prefix.clone() + &num.to_string()));
                                best = Some((num, base));
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some((_, base)) = best {
        let sc = sidecar_path_for_backup(&base);
        let backup_present = if base.exists() { Some(base) } else { None };
        Some((backup_present, sc))
    } else {
        None
    }
}

// Return the previous (second newest) backup pair if present.
pub(crate) fn find_previous_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut stamps: Vec<(u128, PathBuf)> = Vec::new();
    let mut seen: std::collections::HashSet<u128> = std::collections::HashSet::new();
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            let fname = e.file_name();
            if let Some(s) = fname.to_str() {
                if let Some(rest) = s.strip_prefix(&prefix) {
                    if let Some(num_s) = rest
                        .strip_suffix(".bak")
                        .or_else(|| rest.strip_suffix(".bak.meta.json"))
                    {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if seen.insert(num) {
                                // base path ending with .bak
                                let base = parent
                                    .join(format!("{}.bak", prefix.clone() + &num.to_string()));
                                stamps.push((num, base));
                            }
                        }
                    }
                }
            }
        }
    }
    if stamps.len() < 2 {
        return None;
    }
    stamps.sort_by(|a, b| a.0.cmp(&b.0));
    let (_ts, base) = stamps[stamps.len() - 2].clone();
    let sc = sidecar_path_for_backup(&base);
    let backup_present = if base.exists() { Some(base) } else { None };
    Some((backup_present, sc))
}

/// Public helper for preflight/tests: check if there are backup artifacts (payload and/or sidecar)
/// for the given target and tag.
pub fn has_backup_artifacts(target: &Path, tag: &str) -> bool {
    if let Some((payload, sc)) = find_latest_backup_and_sidecar(target, tag) {
        payload.is_some() || sc.exists()
    } else {
        false
    }
}
