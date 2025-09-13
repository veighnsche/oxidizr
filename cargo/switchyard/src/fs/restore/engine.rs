use std::path::{Path, PathBuf};

use super::{
    idempotence, integrity, selector, steps,
    types::{RestoreOptions, SnapshotSel},
};
use crate::fs::backup::sidecar::read_sidecar;
use crate::types::safepath::SafePath;

/// Restore a file from its backup. When no backup exists, return an error unless `force_best_effort` is true.
///
/// # Errors
///
/// Returns an IO error if the backup file cannot be restored.
pub fn restore_file(
    target: &SafePath,
    dry_run: bool,
    force_best_effort: bool,
    backup_tag: &str,
) -> std::io::Result<()> {
    let opts = RestoreOptions {
        dry_run,
        force_best_effort,
        backup_tag: backup_tag.to_string(),
    };
    restore_impl(target, SnapshotSel::Latest, &opts)
}

/// Restore from the previous (second newest) backup pair. Used when a fresh snapshot
/// was just captured pre-restore and we want to restore to the state before snapshot.
///
/// # Errors
///
/// Returns an IO error if the backup file cannot be restored.
pub fn restore_file_prev(
    target: &SafePath,
    dry_run: bool,
    force_best_effort: bool,
    backup_tag: &str,
) -> std::io::Result<()> {
    let opts = RestoreOptions {
        dry_run,
        force_best_effort,
        backup_tag: backup_tag.to_string(),
    };
    restore_impl(target, SnapshotSel::Previous, &opts)
}

/// Engine entry that performs restore given a selector and options.
///
/// # Errors
///
/// Returns an IO error if the backup file cannot be restored.
#[allow(clippy::too_many_lines, reason = "deferred refactoring")]
pub fn restore_impl(
    target: &SafePath,
    sel: SnapshotSel,
    opts: &RestoreOptions,
) -> std::io::Result<()> {
    let target_path = target.as_path();
    // Locate backup payload and sidecar based on selector
    let pair = match sel {
        SnapshotSel::Latest => selector::latest(&target_path, &opts.backup_tag),
        SnapshotSel::Previous => selector::previous(&target_path, &opts.backup_tag),
    };
    let (backup_opt, sidecar_path): (Option<PathBuf>, PathBuf) = if let Some(p) = pair {
        p
    } else {
        if !opts.force_best_effort {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "backup missing",
            ));
        }
        return Ok(());
    };
    // Read sidecar if present
    let sc = read_sidecar(&sidecar_path).ok();
    if let Some(ref side) = sc {
        // Idempotence
        if idempotence::is_idempotent(
            &target_path,
            side.prior_kind.as_str(),
            side.prior_dest.as_deref(),
        ) {
            return Ok(());
        }
    }
    if opts.dry_run {
        return Ok(());
    }
    if let Some(side) = sc {
        match side.prior_kind.as_str() {
            "file" => {
                let backup: PathBuf = if let Some(p) = backup_opt {
                    p
                } else {
                    if opts.force_best_effort {
                        return Ok(());
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "backup payload missing",
                    ));
                };
                if let Some(ref expected) = side.payload_hash {
                    if !integrity::verify_payload_hash_ok(&backup, expected.as_str()) {
                        if opts.force_best_effort {
                            return Ok(());
                        }
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload hash mismatch",
                        ));
                    }
                }
                let mode = side
                    .mode
                    .as_ref()
                    .and_then(|ms| u32::from_str_radix(ms, 8).ok());
                steps::restore_file_bytes(&target_path, &backup, mode)?;
            }
            "symlink" => {
                if let Some(dest) = side.prior_dest.as_ref() {
                    steps::restore_symlink_to(&target_path, Path::new(dest))?;
                    if let Some(b) = backup_opt.as_ref() {
                        let _ = std::fs::remove_file(b);
                    }
                } else if let Some(backup) = backup_opt {
                    steps::legacy_rename(&target_path, &backup)?;
                } else if !opts.force_best_effort {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "backup payload missing",
                    ));
                }
            }
            "none" => {
                steps::ensure_absent(&target_path)?;
                if let Some(b) = backup_opt.as_ref() {
                    let _ = std::fs::remove_file(b);
                }
            }
            _ => {
                if let Some(backup) = backup_opt {
                    steps::legacy_rename(&target_path, &backup)?;
                } else if !opts.force_best_effort {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "backup payload missing",
                    ));
                }
            }
        }
        return Ok(());
    }
    // No sidecar; legacy rename if backup exists
    if let Some(backup) = backup_opt {
        if opts.dry_run {
            return Ok(());
        }
        steps::legacy_rename(&target_path, &backup)
    } else if opts.force_best_effort {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "backup missing",
        ))
    }
}
