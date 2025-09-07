use crate::error::Result;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::tests::experiments::uutils::mock_worker::MockWorker;
use std::fs;
use std::path::PathBuf;

#[test]
fn enable_creates_symlinks_and_backups_unified() -> Result<()> {
    let worker = MockWorker::new();
    let u = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        unified_binary: if cfg!(test) { Some(PathBuf::from("bin/coreutils")) } else { Some(PathBuf::from("/usr/bin/coreutils")) },
        bin_directory: if cfg!(test) { PathBuf::from("bin") } else { PathBuf::from("/usr/lib/uutils/coreutils") },
    };
    u.enable(&worker, true, false)?;
    let date = PathBuf::from("bin/date");
    assert!(fs::symlink_metadata(&date).map(|m| m.file_type().is_symlink()).unwrap_or(false), "Target should be a symlink at {}", date.display());
    let backup_date = backup_path(&date);
    assert!(backup_date.exists(), "Backup file should exist at {}", backup_date.display());
    Ok(())
}

#[test]
fn reentrant_enable_is_idempotent() -> Result<()> {
    let worker = MockWorker::new();
    let u = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        unified_binary: if cfg!(test) { Some(PathBuf::from("bin/coreutils")) } else { Some(PathBuf::from("/usr/bin/coreutils")) },
        bin_directory: if cfg!(test) { PathBuf::from("bin") } else { PathBuf::from("/usr/lib/uutils/coreutils") },
    };
    u.enable(&worker, true, false)?;
    let date = PathBuf::from("bin/date");
    let backup = backup_path(&date);
    assert!(backup.exists(), "Backup should exist after first enable at {}", backup.display());
    u.enable(&worker, true, false)?;
    assert!(backup.exists(), "Backup should still exist after second enable at {}", backup.display());
    assert!(fs::symlink_metadata(&date).map(|m| m.file_type().is_symlink()).unwrap_or(false), "Symlink should still exist after second enable at {}", date.display());
    Ok(())
}

fn backup_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let backup_name = format!("{}.bak", path.file_name().unwrap().to_str().unwrap());
    if cfg!(test) {
        PathBuf::from("tmp").join(backup_name)
    } else {
        path.parent().unwrap().join(backup_name)
    }
}
