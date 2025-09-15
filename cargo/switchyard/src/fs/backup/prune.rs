use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::sidecar::sidecar_path_for_backup;
use crate::fs::atomic::fsync_parent_dir;

/// Prune timestamped backup pairs for target path based on count and age limits.
///
/// # Errors
///
/// Returns an IO error if the backup pruning operation fails. The newest backup is never deleted.
///
/// Retention semantics:
/// - `count_limit = Some(N)`: retain up to N newest backups in total, including the newest. N is clamped to at least 1.
/// - `count_limit = None`: no count-based pruning (age policy may still prune).
/// - `age_limit = Some(d)`: prune any backup older than `d` relative to now (ms precision), regardless of count, but still never delete the newest entry.
/// - `age_limit = None`: no age-based pruning.
///
/// Both policies apply together: a backup is pruned if it violates either count or age policy.
/// Sidecar files are deleted alongside their payloads. The directory containing entries is fsynced best‑effort.
pub fn prune_backups(
    target: &Path,
    tag: &str,
    count_limit: Option<usize>,
    age_limit: Option<Duration>,
) -> io::Result<crate::types::PruneResult> {
    let name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{name}.{tag}.");

    // Collect unique (timestamp, base .bak path) from both payload and sidecar filenames
    let mut seen = HashSet::<u128>::new();
    let mut stamps: Vec<(u128, PathBuf)> = Vec::new();

    let rd = fs::read_dir(parent)?;

    for entry_res in rd {
        let Ok(entry) = entry_res else { continue };

        // Bind the OsString so it lives past the match
        let fname = entry.file_name(); // OsString
        let Some(s) = fname.to_str() else { continue }; // skip non-UTF-8

        let Some(rest) = s.strip_prefix(&prefix) else {
            continue;
        };
        // Strip payload or sidecar suffix and parse the numeric timestamp part
        let core_opt = rest
            .strip_suffix(".bak")
            .or_else(|| rest.strip_suffix(".bak.meta.json"));
        let Some(core) = core_opt else { continue };
        let ts_s = core.rsplit('.').next().unwrap_or("");
        let Ok(ts) = ts_s.parse::<u128>() else {
            continue;
        };
        if !seen.insert(ts) {
            continue;
        }
        // Construct the base .bak path; sidecar resolved later
        let base = parent.join(format!("{prefix}{ts}.bak"));
        stamps.push((ts, base));
    }

    if stamps.is_empty() {
        return Ok(crate::types::PruneResult {
            pruned_count: 0,
            retained_count: 0,
        });
    }

    // Sort newest → oldest by embedded timestamp
    stamps.sort_unstable_by_key(|(ts, _)| std::cmp::Reverse(*ts));

    // Compute age threshold in milliseconds (embedded ts is ms since epoch)
    let now_ms: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let age_cutoff_ms: Option<u128> = age_limit.map(|d| d.as_millis());

    // Determine how many newest to retain by count policy.
    // We never delete the newest, so clamp minimum to 1.
    let desired_keep_by_count = count_limit.map_or(usize::MAX, |n| n.max(1));

    let mut to_delete: Vec<PathBuf> = Vec::new();
    let mut retained: usize = 0;

    for (idx, (ts, base)) in stamps.iter().enumerate() {
        // Always retain the newest
        if idx == 0 {
            retained += 1;
            continue;
        }

        // Count policy: keep the first `desired_keep_by_count` total entries.
        let count_violation = idx >= desired_keep_by_count;

        // Age policy: delete if older than cutoff (if provided)
        let age_violation = age_cutoff_ms.is_some_and(|cut| now_ms.saturating_sub(*ts) > cut);

        if count_violation || age_violation {
            to_delete.push(base.clone());
        } else {
            retained += 1;
        }
    }

    // Delete selected backups + their sidecars
    let mut pruned = 0usize;
    for base in to_delete {
        // Best-effort deletions; ignore individual errors so we continue cleaning others
        let _ = fs::remove_file(&base);
        let sc = sidecar_path_for_backup(&base);
        let _ = fs::remove_file(&sc);
        pruned += 1;
    }

    // Fsync the directory that contains the entries (same dir as `target`)
    let _ = fsync_parent_dir(target);

    Ok(crate::types::PruneResult {
        pruned_count: pruned,
        retained_count: retained,
    })
}
