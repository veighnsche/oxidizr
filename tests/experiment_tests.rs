use coreutils_switch::experiment::UutilsExperiment;
use coreutils_switch::worker::Worker;
use coreutils_switch::Result;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use std::os::unix::fs::PermissionsExt;

struct MockWorker {
    root: TempDir,
    which_map: Vec<(String, PathBuf)>,
}

/// Ensure backup preserves permission bits (e.g., sticky bit) when created.
#[test]
fn backup_preserves_permissions_bits() -> Result<()> {
    let mut w = MockWorker::new();
    let bin_dir = w.path_in_root("/usr/lib/uutils/coreutils");
    fs::create_dir_all(&bin_dir)?;
    let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;

    let date = w.path_in_root("/usr/bin/date");
    fs::create_dir_all(date.parent().unwrap())?;
    fs::write(&date, b"system-date")?;
    // set sticky bit on target (01000)
    let mut perms = fs::metadata(&date)?.permissions();
    let mode = perms.mode();
    perms.set_mode(mode | 0o1000);
    fs::set_permissions(&date, perms.clone())?;

    w.add_which("date", date.clone());
    let exp = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        supported_releases: vec!["rolling".into()],
        unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
        bin_directory: bin_dir.clone(),
    };

    exp.enable(&w, true, false)?;

    let backup = backup_path(&date);
    assert!(backup.exists());
    let backup_mode = fs::metadata(&backup)?.permissions().mode();
    assert_eq!(backup_mode & 0o7000, (mode | 0o1000) & 0o7000);
    Ok(())
}

#[test]
fn reentrant_enable_is_idempotent() -> Result<()> {
    let mut w = MockWorker::new();
    let bin_dir = w.path_in_root("/usr/lib/uutils/coreutils");
    fs::create_dir_all(&bin_dir)?;
    let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
    let date = w.path_in_root("/usr/bin/date"); fs::create_dir_all(date.parent().unwrap())?; fs::write(&date, b"system-date")?;
    w.add_which("date", date.clone());

    let exp = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        supported_releases: vec!["rolling".into()],
        unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
        bin_directory: bin_dir.clone(),
    };

    // First enable
    exp.enable(&w, true, false)?;
    let backup = backup_path(&date);
    assert!(fs::symlink_metadata(&date)?.file_type().is_symlink());
    assert!(backup.exists());
    let meta1 = fs::metadata(&backup)?;

    // Second enable (should be idempotent for existing symlink targets)
    exp.enable(&w, true, false)?;
    assert!(fs::symlink_metadata(&date)?.file_type().is_symlink());
    let meta2 = fs::metadata(&backup)?;

    // Backup still present and unchanged in size
    assert_eq!(meta1.len(), meta2.len());
    Ok(())
}

impl MockWorker {
    fn new() -> Self {
        Self { root: TempDir::new().unwrap(), which_map: vec![] }
    }

    fn root_path(&self) -> &Path { self.root.path() }

    fn path_in_root(&self, rel: &str) -> PathBuf { self.root.path().join(rel.trim_start_matches('/')) }

    fn add_which(&mut self, name: &str, path: PathBuf) { self.which_map.push((name.to_string(), path)); }
}

impl Worker for MockWorker {
    fn distribution(&self) -> Result<(String, String)> { Ok(("Arch".into(), "rolling".into())) }
    fn update_packages(&self) -> Result<()> { Ok(()) }
    fn install_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn remove_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn check_installed(&self, _package: &str) -> Result<bool> { Ok(true) }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        for (n, p) in &self.which_map { if n == name { return Ok(Some(p.clone())); } }
        Ok(None)
    }

    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut out = vec![];
        if !dir.exists() { return Ok(out); }
        for e in fs::read_dir(dir)? { let p = e?.path(); if p.is_file() { out.push(p); } }
        Ok(out)
    }

    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        if fs::symlink_metadata(target).map(|m| m.file_type().is_symlink()).unwrap_or(false) { return Ok(()); }
        if target.exists() {
            let backup = backup_path(target);
            fs::copy(target, &backup)?;
            let meta = fs::metadata(target)?;
            fs::set_permissions(&backup, meta.permissions())?;
            fs::remove_file(target)?;
        }
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        let _ = fs::remove_file(target);
        unix_fs::symlink(source, target)?;
        Ok(())
    }

    fn restore_file(&self, target: &Path) -> Result<()> {
        let backup = backup_path(target);
        if backup.exists() {
            let _ = fs::remove_file(target);
            fs::rename(backup, target)?;
        }
        Ok(())
    }
}

fn backup_path(target: &Path) -> PathBuf {
    let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("backup");
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{}.oxidizr.bak", name))
}

#[test]
fn enable_creates_symlinks_and_backups_unified() -> Result<()> {
    // Arrange experiment
    let mut w = MockWorker::new();
    let bin_dir = w.path_in_root("/usr/lib/uutils/coreutils");
    fs::create_dir_all(&bin_dir)?;
    // replacement binaries
    let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
    let sort_rust = bin_dir.join("sort"); fs::write(&sort_rust, b"rust-sort")?;

    // targets in /usr/bin
    let usr_bin = w.path_in_root("/usr/bin"); fs::create_dir_all(&usr_bin)?;
    let date = usr_bin.join("date"); fs::write(&date, b"system-date")?;
    let sort = usr_bin.join("sort"); fs::write(&sort, b"system-sort")?;

    // which map
    w.add_which("date", date.clone());
    w.add_which("sort", sort.clone());

    let exp = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        supported_releases: vec!["rolling".into()],
        unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
        bin_directory: bin_dir.clone(),
    };

    // Act
    exp.enable(&w, true, false)?;

    // Assert symlinks at targets
    assert!(fs::symlink_metadata(&date)?.file_type().is_symlink());
    assert!(fs::symlink_metadata(&sort)?.file_type().is_symlink());
    // backups exist
    assert!(backup_path(&date).exists());
    assert!(backup_path(&sort).exists());

    Ok(())
}

#[test]
fn disable_restores_originals() -> Result<()> {
    // Arrange as in previous test
    let mut w = MockWorker::new();
    let bin_dir = w.path_in_root("/usr/lib/uutils/coreutils");
    fs::create_dir_all(&bin_dir)?;
    let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
    let date = w.path_in_root("/usr/bin/date"); fs::create_dir_all(date.parent().unwrap())?; fs::write(&date, b"system-date")?;
    w.add_which("date", date.clone());
    let exp = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        supported_releases: vec!["rolling".into()],
        unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
        bin_directory: bin_dir.clone(),
    };
    exp.enable(&w, true, false)?;
    // precondition: target is symlink, backup exists
    assert!(fs::symlink_metadata(&date)?.file_type().is_symlink());
    assert!(backup_path(&date).exists());

    // Act
    exp.disable(&w, false)?;

    // Assert restored
    assert!(fs::metadata(&date)?.is_file());
    assert!(!fs::symlink_metadata(&date)?.file_type().is_symlink());
    Ok(())
}

#[test]
fn check_incompatible_release_fails_gate() -> Result<()> {
    let w = MockWorker::new();
    let exp = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        supported_releases: vec!["not-rolling".into()],
        unified_binary: None,
        bin_directory: PathBuf::from("/unused"),
    };
    assert!(!exp.check_compatible(&w)?);
    Ok(())
}
