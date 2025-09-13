use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use super::sidecar::sidecar_path_for_backup;

/// Return (`backup_path_if_present`, `sidecar_path`) for the latest timestamped pair.
pub(crate) fn find_latest_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?; // relies on UTF-8 file name
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{name}.{tag}.");

    // Track best (timestamp, base_path) as we scan — no Vec, no sort.
    let mut best: Option<(u128, PathBuf)> = None;

    let rd = fs::read_dir(parent).ok()?;
    for entry in rd.flatten() {
        let fname = entry.file_name(); // OsString
        let Some(s) = fname.to_str() else { continue }; // skip non-UTF-8

        // Must start with the prefix
        let Some(rest) = s.strip_prefix(&prefix) else {
            continue;
        };

        // Accept ".bak" or ".bak.meta.json"
        let Some(num_s) = rest
            .strip_suffix(".bak")
            .or_else(|| rest.strip_suffix(".bak.meta.json"))
        else {
            continue;
        };

        let Ok(ts) = num_s.parse::<u128>() else {
            continue;
        };

        // If this timestamp is newer, keep it
        let is_better = best.as_ref().is_none_or(|(cur, _)| ts > *cur);
        if is_better {
            // Construct the .bak base path (sidecar is derived later)
            let base = parent.join(format!("{prefix}{ts}.bak"));
            best = Some((ts, base));
        }
    }

    let (_, base) = best?;
    let sidecar = sidecar_path_for_backup(&base);
    let backup_present = if base.exists() { Some(base) } else { None };
    Some((backup_present, sidecar))
}

/// Return the previous (second newest) backup pair if present.
pub(crate) fn find_previous_backup_and_sidecar(
    target: &Path,
    tag: &str,
) -> Option<(Option<PathBuf>, PathBuf)> {
    let name = target.file_name()?.to_str()?;
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let prefix = format!(".{name}.{tag}.");

    let mut seen = HashSet::<u128>::new();

    // Collect unique (timestamp, base_path) pairs
    let mut stamps: Vec<(u128, PathBuf)> = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|s| {
            // Guard: must start with prefix
            let rest = s.strip_prefix(&prefix)?;
            // Accept either ".bak" or ".bak.meta.json"
            let num_s = rest
                .strip_suffix(".bak")
                .or_else(|| rest.strip_suffix(".bak.meta.json"))?;
            let num: u128 = num_s.parse().ok()?;
            // Deduplicate timestamps
            if !seen.insert(num) {
                return None;
            }
            let base = parent.join(format!("{prefix}{num}.bak"));
            Some((num, base))
        })
        .collect();

    if stamps.len() < 2 {
        return None;
    }

    // Second newest by timestamp
    stamps.sort_unstable_by_key(|(ts, _)| *ts);
    let (_ts, base) = stamps.get(stamps.len() - 2)?.clone();

    let sidecar = sidecar_path_for_backup(&base);
    let backup_present = if base.exists() { Some(base) } else { None };

    Some((backup_present, sidecar))
}
