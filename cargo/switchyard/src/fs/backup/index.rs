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
    let rd = fs::read_dir(parent).ok()?;

    // Track best (timestamp, base_path) as we scan — no Vec, no sort.
    let mut best: Option<(u128, PathBuf)> = None;

    for entry in rd.flatten() {
        let fname = entry.file_name();
        let Some(s) = fname.to_str() else { continue }; // skip non-UTF-8

        // Two modes:
        // - Tagged: expect ".{name}.{tag}.{ts}.bak[.meta.json]"
        // - Untagged (wildcard): accept any tag: ".{name}.<any>.<ts>.bak[.meta.json]"
        let (rest_opt, wildcard) = if tag.is_empty() {
            let pre = format!(".{name}.");
            (s.strip_prefix(&pre), true)
        } else {
            let pre = format!(".{name}.{tag}.");
            (s.strip_prefix(&pre), false)
        };
        let Some(rest) = rest_opt else { continue };

        // Accept both payload and sidecar filenames.
        let core = if let Some(core) = rest.strip_suffix(".bak") {
            core
        } else if let Some(core) = rest.strip_suffix(".bak.meta.json") {
            core
        } else {
            continue;
        };

        // When wildcard, `core` has the form "<ts>" only if the tag had no body.
        // We want the last dotted segment to be the timestamp.
        let ts_part = if wildcard {
            core.rsplit('.').next().unwrap_or(core)
        } else {
            core
        };
        let Ok(ts) = ts_part.parse::<u128>() else {
            continue;
        };

        // Build the base path back. For wildcard we cannot reconstruct tag; however, we only need base path.
        // Reconstruct from the actual filename: drop any ".meta.json" suffix if present.
        let base = if s.ends_with(".bak.meta.json") {
            parent.join(&s[..s.len() - ".meta.json".len()])
        } else {
            parent.join(s)
        };

        let is_better = match best.as_ref() {
            None => true,
            Some((cur, _)) => ts > *cur,
        };
        if is_better {
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

    let mut seen = HashSet::<u128>::new();
    let mut stamps: Vec<(u128, PathBuf)> = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|s| {
            // Prefix acceptance
            let ok_prefix = if tag.is_empty() {
                s.strip_prefix(&format!(".{name}.")).is_some()
            } else {
                s.strip_prefix(&format!(".{name}.{tag}.")).is_some()
            };
            if !ok_prefix {
                return None;
            }

            // Strip sidecar suffix if present
            let rest_opt = s
                .strip_suffix(".bak")
                .or_else(|| s.strip_suffix(".bak.meta.json"))
                .map(ToString::to_string);
            let core = rest_opt?;

            // Timestamp is the last dotted segment
            let ts_s = core.rsplit('.').next().unwrap_or("");
            let Ok(ts) = ts_s.parse::<u128>() else {
                return None;
            };

            if !seen.insert(ts) {
                return None;
            }
            // Base path is without .meta.json when present
            let base = if s.ends_with(".bak.meta.json") {
                parent.join(&s[..s.len() - ".meta.json".len()])
            } else {
                parent.join(&s)
            };
            Some((ts, base))
        })
        .collect();

    if stamps.len() < 2 {
        return None;
    }
    stamps.sort_unstable_by_key(|(ts, _)| *ts);
    let (_ts, base) = stamps.get(stamps.len() - 2)?.clone();

    let sidecar = sidecar_path_for_backup(&base);
    let backup_present = if base.exists() { Some(base) } else { None };
    Some((backup_present, sidecar))
}
