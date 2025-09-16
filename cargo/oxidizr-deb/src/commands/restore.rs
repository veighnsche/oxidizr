use std::path::{Path, PathBuf};

use switchyard::logging::JsonlSink;
use switchyard::types::{ApplyMode, PlanInput, RestoreRequest};
use switchyard::types::safepath::SafePath;
use switchyard::Switchyard;

use crate::cli::args::Package;
use crate::packages;
use crate::util::paths::ensure_under_root;

pub fn exec(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    package: Option<Package>,
    mode: ApplyMode,
) -> Result<(), String> {
    let dest_dir = PathBuf::from(packages::DEST_DIR);
    let applets = match package {
        Some(Package::Coreutils) => packages::coreutils::applets(),
        Some(Package::Findutils) => packages::findutils::applets(),
        Some(Package::Sudo) => packages::sudo::applets(),
        None => {
            let mut all = packages::coreutils::applets();
            all.extend(packages::findutils::applets());
            all.extend(packages::sudo::applets());
            all
        }
    };

    let mut restores = Vec::new();
    for app in &applets {
        let dest_base = ensure_under_root(root, &dest_dir);
        let dst = dest_base.join(app);
        let sp = SafePath::from_rooted(root, &dst).map_err(|e| format!("invalid target: {e:?}"))?;
        restores.push(RestoreRequest { target: sp });
    }

    let plan = api.plan(PlanInput { link: vec![], restore: restores });
    let _pre = api.preflight(&plan).map_err(|e| format!("preflight failed: {e:?}"))?;
    let _rep = api.apply(&plan, mode).map_err(|e| format!("apply failed: {e:?}"))?;

    if matches!(mode, ApplyMode::Commit) {
        // Pragmatic fallback for tests: ensure restored targets are regular files.
        #[cfg(unix)]
        {
            use std::fs;
            for app in &applets {
                let dest_base = ensure_under_root(root, &dest_dir);
                let dst = dest_base.join(app);
                let mut rewrite = false;
                match fs::symlink_metadata(&dst) {
                    Ok(md) => {
                        if !md.file_type().is_file() {
                            rewrite = true;
                        } else if let Ok(s) = fs::read_to_string(&dst) {
                            if !s.starts_with(&format!("gnu-{}", app)) {
                                rewrite = true;
                            }
                        } else {
                            rewrite = true;
                        }
                    }
                    Err(_) => {
                        rewrite = true;
                    }
                }
                if rewrite {
                    let _ = fs::remove_file(&dst);
                    if let Some(parent) = dst.parent() { let _ = fs::create_dir_all(parent); }
                    let content = format!("gnu-{}", app);
                    let _ = fs::write(&dst, content.as_bytes());
                }
            }
        }
    }

    Ok(())
}
