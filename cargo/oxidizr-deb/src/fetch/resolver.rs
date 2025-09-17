use std::path::{Path, PathBuf};

use crate::cli::args::Package;

pub fn resolve_artifact(
    root: &Path,
    pkg: Package,
    offline: bool,
    use_local: Option<&PathBuf>,
) -> PathBuf {
    let candidate = if offline {
        if let Some(p) = use_local {
            p.clone()
        } else if let Ok(e) = std::env::var("OXIDIZR_DEB_LOCAL_ARTIFACT") {
            PathBuf::from(e)
        } else {
            match pkg {
                Package::Coreutils => PathBuf::from("/opt/uutils/uutils"),
                Package::Findutils => PathBuf::from("/opt/uutils-findutils/uutils-findutils"),
                Package::Sudo => PathBuf::from("/opt/sudo-rs/sudo-rs"),
            }
        }
    } else {
        match pkg {
            Package::Coreutils => PathBuf::from("/usr/bin/uutils"),
            Package::Findutils => PathBuf::from("/usr/bin/uutils-findutils"),
            Package::Sudo => PathBuf::from("/usr/bin/sudo-rs"),
        }
    };
    if candidate.is_absolute() {
        if candidate.starts_with(root) {
            candidate
        } else {
            let rel = candidate
                .strip_prefix(std::path::Path::new("/"))
                .unwrap_or(&candidate);
            root.join(rel)
        }
    } else {
        root.join(candidate)
    }
}

pub fn staged_default_path(root: &Path, pkg: Package) -> PathBuf {
    let (sub, bin) = match pkg {
        Package::Coreutils => ("uutils-coreutils", "uutils"),
        Package::Findutils => ("uutils-findutils", "uutils-findutils"),
        Package::Sudo => ("sudo-rs", "sudo-rs"),
    };
    root.join("opt/oxidizr/replacements")
        .join(sub)
        .join("bin")
        .join(bin)
}
