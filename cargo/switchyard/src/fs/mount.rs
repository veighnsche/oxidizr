//! Filesystem mount inspection and policy helpers.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags {
    pub read_only: bool,
    pub no_exec: bool,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum MountError {
    #[error("unknown or ambiguous mount state")]
    Unknown,
}

pub trait MountInspector {
    fn flags_for(&self, path: &Path) -> Result<MountFlags, MountError>;
}

/// Production inspector. Prefer kernel syscalls when available; fall back to parsing /proc/self/mounts.
pub struct ProcStatfsInspector;

impl ProcStatfsInspector {
    fn parse_proc_mounts(path: &Path) -> Result<MountFlags, MountError> {
        // Canonicalize best-effort; if it fails, still proceed with the raw path
        let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let content = std::fs::read_to_string("/proc/self/mounts").map_err(|_| MountError::Unknown)?;
        let mut best: Option<(PathBuf, String)> = None;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 { continue; }
            let mnt = PathBuf::from(parts[1]);
            if p.starts_with(&mnt) {
                let opts = parts[3].to_ascii_lowercase();
                match &best {
                    None => best = Some((mnt, opts)),
                    Some((b, _)) => {
                        if mnt.as_os_str().len() > b.as_os_str().len() {
                            best = Some((mnt, opts));
                        }
                    }
                }
            }
        }
        if let Some((_mnt, opts)) = best {
            let has_rw = opts.split(',').any(|o| o == "rw");
            let noexec = opts.split(',').any(|o| o == "noexec");
            Ok(MountFlags { read_only: !has_rw, no_exec: noexec })
        } else {
            Err(MountError::Unknown)
        }
    }
}

impl MountInspector for ProcStatfsInspector {
    fn flags_for(&self, path: &Path) -> Result<MountFlags, MountError> {
        // For now, rely on /proc/self/mounts. A future improvement can add rustix::fs::statfs mapping.
        Self::parse_proc_mounts(path)
    }
}

/// Policy helper: ensure the target mount is rw and exec-capable.
pub fn ensure_rw_exec(inspector: &impl MountInspector, path: &Path) -> Result<(), MountError> {
    match inspector.flags_for(path) {
        Ok(flags) => {
            if flags.read_only || flags.no_exec {
                return Err(MountError::Unknown);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockInspector { flags: Result<MountFlags, MountError> }
    impl MountInspector for MockInspector {
        fn flags_for(&self, _path: &Path) -> Result<MountFlags, MountError> { self.flags.clone() }
    }

    #[test]
    fn ensure_rw_exec_passes_on_rw_exec() {
        let ins = MockInspector { flags: Ok(MountFlags { read_only: false, no_exec: false }) };
        assert!(ensure_rw_exec(&ins, Path::new("/tmp")).is_ok());
    }

    #[test]
    fn ensure_rw_exec_fails_on_ro_or_noexec() {
        let ins1 = MockInspector { flags: Ok(MountFlags { read_only: true, no_exec: false }) };
        assert!(ensure_rw_exec(&ins1, Path::new("/tmp")).is_err());
        let ins2 = MockInspector { flags: Ok(MountFlags { read_only: false, no_exec: true }) };
        assert!(ensure_rw_exec(&ins2, Path::new("/tmp")).is_err());
    }

    #[test]
    fn ensure_rw_exec_fails_on_ambiguous() {
        let ins = MockInspector { flags: Err(MountError::Unknown) };
        assert!(ensure_rw_exec(&ins, Path::new("/tmp")).is_err());
    }
}
