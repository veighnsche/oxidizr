use crate::error::Result;
use crate::utils::{Distribution, Worker};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

pub struct MockWorker {
    pub which_map: Vec<(String, PathBuf)>,
}

impl MockWorker {
    pub fn new() -> Self {
        MockWorker { which_map: Vec::new() }
    }
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

impl Worker for MockWorker {
    fn distribution(&self) -> Result<Distribution> { Ok(Distribution { id: "arch".into(), release: "rolling".into() }) }
    fn update_packages(&self) -> Result<()> { Ok(()) }
    fn install_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn remove_package(&self, _package: &str) -> Result<()> { Ok(()) }
    fn check_installed(&self, _package: &str) -> Result<bool> { Ok(true) }

    fn which(&self, name: &str) -> Result<Option<PathBuf>> {
        println!("[MockWorker] which called for: {}", name);
        for (n, p) in &self.which_map { if n == name { return Ok(Some(p.clone())); } }
        if cfg!(test) {
            Ok(Some(PathBuf::from("bin").join(name)))
        } else {
            Ok(None)
        }
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
            let backup = backup_path(target);
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
        let backup = backup_path(target);
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
