use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BackupSidecar {
    pub(crate) schema: String,     // "backup_meta.v1" | "backup_meta.v2"
    pub(crate) prior_kind: String, // "file" | "symlink" | "none"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prior_dest: Option<String>, // for symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mode: Option<String>, // octal string for file, e.g. "100644"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payload_hash: Option<String>, // sha256 of payload for v2
}

pub(crate) fn sidecar_path_for_backup(backup: &Path) -> PathBuf {
    let s = backup.as_os_str().to_owned();
    use std::ffi::OsString;
    let mut s2 = OsString::from(s);
    s2.push(".meta.json");
    PathBuf::from(s2)
}

pub(crate) fn write_sidecar(backup: &Path, sc: &BackupSidecar) -> std::io::Result<()> {
    let sc_path = sidecar_path_for_backup(backup);
    if let Some(parent) = sc_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let f = std::fs::File::create(&sc_path)?;
    serde_json::to_writer_pretty(&f, sc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // Ensure sidecar durability as well
    let _ = f.sync_all();
    let _ = crate::fs::atomic::fsync_parent_dir(&sc_path);
    Ok(())
}

pub(crate) fn read_sidecar(sc_path: &Path) -> std::io::Result<BackupSidecar> {
    let f = std::fs::File::open(sc_path)?;
    serde_json::from_reader(f).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
