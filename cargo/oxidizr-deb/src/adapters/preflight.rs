use std::path::Path;

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use switchyard::types::safepath::SafePath;

pub fn sudo_guard(root: &Path, source_bin: &Path) -> Result<(), String> {
    let sp = SafePath::from_rooted(root, source_bin)
        .map_err(|e| format!("invalid source_bin: {e:?}"))?;
    let p = sp.as_path();
    let md = std::fs::symlink_metadata(&p)
        .map_err(|e| format!("replacement missing: {}: {e}", p.display()))?;
    let mode = md.permissions().mode();
    let setuid = (mode & 0o4000) != 0;
    let uid = md.uid();
    let gid = md.gid();
    let strict_owner = std::env::var("OXIDIZR_DEB_TEST_ALLOW_NONROOT_SUDO_OWNER")
        .ok()
        .map(|s| s != "1")
        .unwrap_or(true);
    if !setuid || (strict_owner && (uid != 0 || gid != 0)) {
        return Err("sudo replacement must be root:root with mode=4755 (setuid root)".to_string());
    }
    Ok(())
}
