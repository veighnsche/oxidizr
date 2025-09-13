use std::path::{Path, PathBuf};

use crate::fs::atomic::fsync_parent_dir;
use super::sidecar::sidecar_path_for_backup;

/// Generate a unique backup path for a target file (includes a timestamp).
/// Public so callers (preflight/tests) can compute expected names.
pub fn backup_path_with_tag(target: &Path, tag: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    parent.join(format!(".{}.{}.{}.bak", name, tag, ts))
}

/// Prune backups according to retention policy: keep newest by count and by age. Never deletes the newest backup.
pub fn prune_backups(
    target: &Path,
    tag: &str,
    count_limit: Option<usize>,
    age_limit: Option<std::time::Duration>,
) -> std::io::Result<crate::types::PruneResult> {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{}.{}.", name, tag);
    let mut stamps: Vec<(u128, PathBuf)> = Vec::new();
    let mut seen: std::collections::HashSet<u128> = std::collections::HashSet::new();
    if let Ok(rd) = std::fs::read_dir(parent) {
        for e in rd.flatten() {
            if let Some(s) = e.file_name().to_str() {
                if let Some(rest) = s.strip_prefix(&prefix) {
                    if let Some(num_s) = rest
                        .strip_suffix(".bak")
                        .or_else(|| rest.strip_suffix(".bak.meta.json"))
                    {
                        if let Ok(num) = num_s.parse::<u128>() {
                            if seen.insert(num) {
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
    if stamps.is_empty() {
        return Ok(crate::types::PruneResult { pruned_count: 0, retained_count: 0 });
    }
    // Sort newest first
    stamps.sort_by(|a, b| b.0.cmp(&a.0));
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut to_delete: Vec<PathBuf> = Vec::new();
    let mut retained = 0usize;
    for (idx, (ts, base)) in stamps.iter().enumerate() {
        // Never delete the newest backup
        if idx == 0 {
            retained += 1;
            continue;
        }
        let mut delete = false;
        if let Some(limit) = count_limit {
            if idx >= limit { // keep first `limit-1` after newest? Interpret as total retain = limit
                delete = true;
            }
        }
        if !delete {
            if let Some(age) = age_limit {
                let age_ms = age.as_millis() as u128;
                if now_ms.saturating_sub(*ts) > age_ms {
                    delete = true;
                }
            }
        }
        if delete {
            to_delete.push(base.clone());
        } else {
            retained += 1;
        }
    }
    let mut pruned = 0usize;
    for base in to_delete {
        if base.exists() {
            let _ = std::fs::remove_file(&base);
        }
        let sc = sidecar_path_for_backup(&base);
        if sc.exists() {
            let _ = std::fs::remove_file(&sc);
        }
        pruned += 1;
    }
    // fsync parent for durability
    let _ = fsync_parent_dir(target);
    Ok(crate::types::PruneResult { pruned_count: pruned, retained_count: retained })
}
