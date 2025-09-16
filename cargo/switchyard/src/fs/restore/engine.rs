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
#[allow(
    clippy::too_many_lines,
    reason = "Will split into RestorePlanner plan/execute in PR6"
)]
pub fn restore_impl(
    target: &SafePath,
    sel: SnapshotSel,
    opts: &RestoreOptions,
) -> std::io::Result<()> {
    let target_path = target.as_path();
    // In DryRun, avoid any filesystem planning/probing and return success immediately.
    // This guarantees no errors in dry-run even when backups are missing.
    if opts.dry_run {
        return Ok(());
    }
    match RestorePlanner::plan(&target_path, sel, opts) {
        Ok((_backup_opt, _sidecar_opt, action)) => RestorePlanner::execute(&target_path, action),
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound && matches!(sel, SnapshotSel::Previous) =>
        {
            // Fallback: if no previous snapshot exists (e.g., first snapshot just captured),
            // attempt restore from the latest snapshot instead.
            match RestorePlanner::plan(&target_path, SnapshotSel::Latest, opts) {
                Ok((_b2, _s2, action2)) => RestorePlanner::execute(&target_path, action2),
                Err(e2) => Err(e2),
            }
        }
        Err(e) => Err(e),
    }
}

/// Planned action for restore execution.
#[derive(Debug, Clone)]
pub enum RestoreAction {
    Noop,
    FileRename {
        backup: PathBuf,
        mode: Option<u32>,
    },
    SymlinkTo {
        dest: PathBuf,
        cleanup_backup: Option<PathBuf>,
    },
    EnsureAbsent {
        cleanup_backup: Option<PathBuf>,
    },
    LegacyRename {
        backup: PathBuf,
    },
}

/// Planner facade that selects the correct restore action and validates integrity/idempotence.
struct RestorePlanner;

impl RestorePlanner {
    #[allow(
        clippy::too_many_lines,
        reason = "Will be split into RestorePlanner plan/execute in follow-up PR"
    )]
    fn plan(
        target: &Path,
        sel: SnapshotSel,
        opts: &RestoreOptions,
    ) -> std::io::Result<(
        Option<PathBuf>,
        Option<crate::fs::backup::sidecar::BackupSidecar>,
        RestoreAction,
    )> {
        // Locate backup payload and sidecar based on selector
        let pair = match sel {
            SnapshotSel::Latest => selector::latest(target, &opts.backup_tag),
            SnapshotSel::Previous => selector::previous(target, &opts.backup_tag),
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
            return Ok((None, None, RestoreAction::Noop));
        };
        // Read sidecar if present
        let sc = read_sidecar(&sidecar_path).ok();
        if let Some(ref side) = sc {
            // Idempotence
            if idempotence::is_idempotent(
                target,
                side.prior_kind.as_str(),
                side.prior_dest.as_deref(),
            ) {
                return Ok((backup_opt, sc, RestoreAction::Noop));
            }
        }
        if let Some(side) = sc.clone() {
            let action = match side.prior_kind.as_str() {
                "file" => {
                    let backup: PathBuf = if let Some(p) = backup_opt.clone() {
                        p
                    } else {
                        if opts.force_best_effort {
                            return Ok((backup_opt, Some(side), RestoreAction::Noop));
                        }
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    };
                    if let Some(ref expected) = side.payload_hash {
                        if !integrity::verify_payload_hash_ok(&backup, expected.as_str()) {
                            if opts.force_best_effort {
                                return Ok((backup_opt, Some(side), RestoreAction::Noop));
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
                    RestoreAction::FileRename { backup, mode }
                }
                "symlink" => {
                    if let Some(dest) = side.prior_dest.as_ref() {
                        RestoreAction::SymlinkTo {
                            dest: PathBuf::from(dest),
                            cleanup_backup: backup_opt.clone(),
                        }
                    } else if let Some(backup) = backup_opt.clone() {
                        RestoreAction::LegacyRename { backup }
                    } else if opts.force_best_effort {
                        RestoreAction::Noop
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                }
                "none" => RestoreAction::EnsureAbsent {
                    cleanup_backup: backup_opt.clone(),
                },
                _ => {
                    if let Some(backup) = backup_opt.clone() {
                        RestoreAction::LegacyRename { backup }
                    } else if opts.force_best_effort {
                        RestoreAction::Noop
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "backup payload missing",
                        ));
                    }
                }
            };
            return Ok((backup_opt, Some(side), action));
        }
        // No sidecar; legacy rename if backup exists
        if let Some(backup) = backup_opt.clone() {
            Ok((backup_opt, None, RestoreAction::LegacyRename { backup }))
        } else if opts.force_best_effort {
            Ok((None, None, RestoreAction::Noop))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "backup missing",
            ))
        }
    }

    fn execute(target: &Path, action: RestoreAction) -> std::io::Result<()> {
        match action {
            RestoreAction::Noop => Ok(()),
            RestoreAction::FileRename { backup, mode } => {
                steps::restore_file_bytes(target, &backup, mode)
            }
            RestoreAction::SymlinkTo {
                dest,
                cleanup_backup,
            } => {
                steps::restore_symlink_to(target, &dest)?;
                if let Some(b) = cleanup_backup {
                    let _ = std::fs::remove_file(b);
                }
                Ok(())
            }
            RestoreAction::EnsureAbsent { cleanup_backup } => {
                steps::ensure_absent(target)?;
                if let Some(b) = cleanup_backup {
                    let _ = std::fs::remove_file(b);
                }
                Ok(())
            }
            RestoreAction::LegacyRename { backup } => steps::legacy_rename(target, &backup),
        }
    }
}
