// Default OwnershipOracle implementation using OS metadata (Unix-only)

use crate::adapters::OwnershipOracle;
use crate::types::OwnershipInfo;
use crate::types::errors::{Error, ErrorKind, Result};
use crate::types::safepath::SafePath;

#[derive(Copy, Clone, Debug, Default)]
pub struct FsOwnershipOracle;

impl OwnershipOracle for FsOwnershipOracle {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let md = std::fs::symlink_metadata(path.as_path()).map_err(|e| Error {
                kind: ErrorKind::Io,
                msg: format!("metadata: {}", e),
            })?;
            Ok(OwnershipInfo {
                uid: md.uid(),
                gid: md.gid(),
                pkg: String::new(),
            })
        }
        #[cfg(not(unix))]
        {
            Err(Error {
                kind: ErrorKind::Policy,
                msg: "OwnershipOracle not supported on this platform".into(),
            })
        }
    }
}
