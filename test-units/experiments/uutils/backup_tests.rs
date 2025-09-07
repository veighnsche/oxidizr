use crate::error::Result;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::tests::experiments::uutils::mock_worker::MockWorker;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn backup_preserves_permissions_bits() -> Result<()> {
    let temp = TempDir::new()?;
    let worker = MockWorker::new();
    let original = temp.path().join("original");
    fs::write(&original, "test content")?;
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(&original, perms.clone())?;
    let target = temp.path().join("target");
    fs::copy(&original, &target)?;
    fs::set_permissions(&target, perms)?;
    let source = temp.path().join("source");
    worker.replace_file_with_symlink(&source, &target)?;
    let backup = backup_path(&target);
    assert!(backup.exists(), "Backup file should exist at {}", backup.display());
    let backup_meta = fs::metadata(&backup)?;
    assert_eq!(backup_meta.permissions().mode() & 0o777, 0o755, "Backup should preserve permissions");
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
