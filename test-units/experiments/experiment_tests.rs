use coreutils_switch::error::Result;
use coreutils_switch::experiments::uutils::model::UutilsExperiment;
use coreutils_switch::worker::Worker;
use coreutils_switch::utils::Distribution;
use std::fs;
use std::io::Write;
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Main test entry point or additional tests can be added here if needed.

#[cfg(test)]
mod tests {
    use super::*;

    // Any additional integration tests or overarching tests can be added here.
    
    /// Ensure backup preserves permission bits (e.g., sticky bit) when created.
    #[test]
    fn backup_preserves_permissions_bits() -> Result<()> {
        let mut w = MockWorker::new();
        let bin_dir = w.path_in_root("uutils/coreutils");
        fs::create_dir_all(&bin_dir)?;
        let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;

        let date = w.path_in_root("bin/date");
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
            unified_binary: Some(w.path_in_root("bin/coreutils")),
            bin_directory: bin_dir.clone(),
        };

        exp.enable(&w, true, false)?;

        let backup = backup_path(&date, &w);
        assert!(backup.exists(), "Backup file should exist at {}", backup.display());
        let backup_mode = fs::metadata(&backup)?.permissions().mode();
        assert_eq!(backup_mode & 0o7000, (mode | 0o1000) & 0o7000, "Backup should preserve permissions");
        Ok(())
    }

    #[test]
    fn enable_creates_symlinks_and_backups_unified() -> Result<()> {
        // Arrange experiment
        let mut w = MockWorker::new();
        let bin_dir = w.path_in_root("uutils/coreutils");
        fs::create_dir_all(&bin_dir)?;
        // replacement binaries
        let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
        let sort_rust = bin_dir.join("sort"); fs::write(&sort_rust, b"rust-sort")?;

        // targets in bin
        let usr_bin = w.path_in_root("bin"); fs::create_dir_all(&usr_bin)?;
        let date = usr_bin.join("date"); fs::write(&date, b"system-date")?;
        let sort = usr_bin.join("sort"); fs::write(&sort, b"system-sort")?;

        // which map
        w.add_which("date", date.clone());
        w.add_which("sort", sort.clone());

        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            unified_binary: Some(w.path_in_root("bin/coreutils")),
            bin_directory: bin_dir.clone(),
        };

        // Act
        exp.enable(&w, true, false)?;

        // Assert symlinks at targets
        assert!(fs::symlink_metadata(&date)?.file_type().is_symlink(), "Target should be a symlink at {}", date.display());
        assert!(fs::symlink_metadata(&sort)?.file_type().is_symlink(), "Target should be a symlink at {}", sort.display());
        // backups exist
        let backup_date = backup_path(&date, &w);
        let backup_sort = backup_path(&sort, &w);
        assert!(backup_date.exists(), "Backup file should exist at {}", backup_date.display());
        assert!(backup_sort.exists(), "Backup file should exist at {}", backup_sort.display());

        Ok(())
    }

    #[test]
    fn reentrant_enable_is_idempotent() -> Result<()> {
        let mut w = MockWorker::new();
        let bin_dir = w.path_in_root("uutils/coreutils");
        fs::create_dir_all(&bin_dir)?;
        let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
        let date = w.path_in_root("bin/date"); fs::create_dir_all(date.parent().unwrap())?; fs::write(&date, b"system-date")?;
        w.add_which("date", date.clone());

        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            unified_binary: Some(w.path_in_root("bin/coreutils")),
            bin_directory: bin_dir.clone(),
        };

        // First enable
        exp.enable(&w, true, false)?;
        let backup = backup_path(&date, &w);
        assert!(fs::symlink_metadata(&date)?.file_type().is_symlink(), "Target should be a symlink at {}", date.display());
        assert!(backup.exists(), "Backup should exist after first enable at {}", backup.display());
        let meta1 = fs::metadata(&backup)?;

        // Second enable (should be idempotent for existing symlink targets)
        exp.enable(&w, true, false)?;
        assert!(fs::symlink_metadata(&date)?.file_type().is_symlink(), "Target should be a symlink at {}", date.display());
        let meta2 = fs::metadata(&backup)?;

        // Backup still present and unchanged in size
        assert_eq!(meta1.len(), meta2.len(), "Backup size should remain unchanged after second enable");
        Ok(())
    }

    #[test]
    fn disable_restores_originals() -> Result<()> {
        // Arrange as in previous test
        let mut w = MockWorker::new();
        let bin_dir = w.path_in_root("uutils/coreutils");
        fs::create_dir_all(&bin_dir)?;
        let date_rust = bin_dir.join("date"); fs::write(&date_rust, b"rust-date")?;
        let date = w.path_in_root("bin/date"); fs::create_dir_all(date.parent().unwrap())?; fs::write(&date, b"system-date")?;
        w.add_which("date", date.clone());
        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            unified_binary: Some(w.path_in_root("bin/coreutils")),
            bin_directory: bin_dir.clone(),
        };
        exp.enable(&w, true, false)?;
        // precondition: target is symlink, backup exists
        assert!(fs::symlink_metadata(&date)?.file_type().is_symlink(), "Target should be a symlink at {}", date.display());
        let backup = backup_path(&date, &w);
        assert!(backup.exists(), "Backup should exist at {}", backup.display());

        // Act
        exp.disable(&w, false)?;

        // Assert restored
        assert!(fs::metadata(&date)?.is_file(), "Original file should be restored at {}", date.display());
        assert!(!fs::symlink_metadata(&date)?.file_type().is_symlink(), "Symlink should be removed at {}", date.display());
        Ok(())
    }

    #[test]
    fn check_incompatible_distro_fails_gate() -> Result<()> {
        struct NonArchWorker;
        impl Worker for NonArchWorker {
            fn distribution(&self) -> Result<Distribution> { Ok(Distribution { id: "debian".into(), release: "12".into() }) }
            fn update_packages(&self) -> Result<()> { Ok(()) }
            fn install_package(&self, _package: &str) -> Result<()> { Ok(()) }
            fn remove_package(&self, _package: &str) -> Result<()> { Ok(()) }
            fn check_installed(&self, _package: &str) -> Result<bool> { Ok(false) }
            fn which(&self, _name: &str) -> Result<Option<PathBuf>> { Ok(None) }
            fn list_files(&self, _dir: &Path) -> Result<Vec<PathBuf>> { Ok(vec![]) }
            fn replace_file_with_symlink(&self, _source: &Path, _target: &Path) -> Result<()> { Ok(()) }
            fn restore_file(&self, _target: &Path) -> Result<()> { Ok(()) }
        }

        let w = NonArchWorker;
        let exp = UutilsExperiment {
            name: "coreutils".into(),
            package: "uutils-coreutils".into(),
            unified_binary: None,
            bin_directory: w.path_in_root("uutils/coreutils"),
        };
        assert!(!exp.check_compatible(&w)?);
        Ok(())
    }
}

struct MockWorker {
    root: TempDir,
    which_map: Vec<(String, PathBuf)>,
}

impl MockWorker {
    fn new() -> Self {
        Self { root: TempDir::new().unwrap(), which_map: vec![] }
    }

    fn root_path(&self) -> &Path { self.root.path() }

    fn path_in_root(&self, rel: &str) -> PathBuf { self.root.path().join(rel) }

    fn add_which(&mut self, name: &str, path: PathBuf) { self.which_map.push((name.to_string(), path)); }
}

impl Worker for MockWorker {
    fn distribution(&self) -> Result<Distribution> { Ok(Distribution { id: "arch".into(), release: "rolling".into() }) }
    fn update_packages(&self) -> Result<()> { Ok(()) }
    fn install_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn remove_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn check_installed(&self, _package: &str) -> Result<bool> { Ok(true) }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        println!("[MockWorker] which called for: {}", name);
        for (n, p) in &self.which_map { if n == name { return Ok(Some(p.clone())); } }
        Ok(None)
    }

    fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        println!("[MockWorker] list_files called for: {}", dir.display());
        let mut out = vec![];
        if !dir.exists() { return Ok(out); }
        for e in fs::read_dir(dir)? { 
            let p = e?.path(); 
            if p.is_file() { 
                println!("[MockWorker] found file: {}", p.display());
                out.push(p); 
            } 
        }
        Ok(out)
    }

    fn replace_file_with_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        println!("[MockWorker] replace_file_with_symlink called: source={} target={}", source.display(), target.display());
        if fs::symlink_metadata(target).map(|m| m.file_type().is_symlink()).unwrap_or(false) { 
            println!("[MockWorker] target already a symlink, skipping");
            return Ok(()); 
        }
        if target.exists() {
            let backup = backup_path(target, self);
            println!("[MockWorker] creating backup: {}", backup.display());
            if let Some(parent) = backup.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(target, &backup)?;
            let meta = fs::metadata(target)?;
            fs::set_permissions(&backup, meta.permissions())?;
            println!("[MockWorker] removing original file: {}", target.display());
            fs::remove_file(target)?;
        }
        if let Some(parent) = target.parent() { 
            println!("[MockWorker] creating parent dir: {}", parent.display());
            fs::create_dir_all(parent)?; 
        }
        let _ = fs::remove_file(target);
        println!("[MockWorker] creating symlink: {} -> {}", source.display(), target.display());
        unix_fs::symlink(source, target)?;
        Ok(())
    }

    fn restore_file(&self, target: &Path) -> Result<()> {
        println!("[MockWorker] restore_file called for: {}", target.display());
        let backup = backup_path(target, self);
        if backup.exists() {
            println!("[MockWorker] backup exists, restoring: {}", backup.display());
            let _ = fs::remove_file(target);
            fs::rename(backup, target)?;
        } else {
            println!("[MockWorker] no backup found for: {}", target.display());
        }
        Ok(())
    }
}

fn backup_path(target: &Path, worker: &MockWorker) -> PathBuf {
    worker.root_path().join(format!(".{}.oxidizr.bak", target.file_name().unwrap().to_str().unwrap()))
}
