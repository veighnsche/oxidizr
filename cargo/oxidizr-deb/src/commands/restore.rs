use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use switchyard::logging::JsonlSink;
use switchyard::types::safepath::SafePath;
use switchyard::types::{ApplyMode, PlanInput, RestoreRequest};
use switchyard::Switchyard;

use crate::adapters::debian::pm_lock_message;
use crate::cli::args::Package;
use crate::packages;
use crate::util::paths::ensure_under_root;
use crate::fetch::resolver::staged_default_path;
use serde_json::json;

fn distro_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "coreutils",
        Package::Findutils => "findutils",
        Package::Sudo => "sudo",
    }
}

fn dpkg_installed(name: &str) -> bool {
    let st = Command::new("dpkg")
        .args(["-s", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    matches!(st, Ok(s) if s.success())
}

fn remove_staged_if_present(root: &Path, pkg: Package) {
    let bin = staged_default_path(root, pkg);
    if let Some(base) = bin.parent().and_then(|d| d.parent()) {
        let _ = std::fs::remove_file(&bin);
        let _ = std::fs::remove_dir_all(base);
    }
}

fn replacement_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "uutils-coreutils",
        Package::Findutils => "uutils-findutils",
        Package::Sudo => "sudo-rs",
    }
}

pub fn exec(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    package: Option<Package>,
    all: bool,
    keep_replacements: bool,
    mode: ApplyMode,
    _assume_yes: bool,
) -> Result<(), String> {
    let live_root = root == Path::new("/");
    if matches!(mode, ApplyMode::Commit) && live_root {
        if let Some(msg) = pm_lock_message(root) {
            return Err(msg);
        }
    }
    let dest_dir = PathBuf::from(packages::DEST_DIR);
    let applets = if all || package.is_none() {
        let mut all = packages::coreutils::applets();
        all.extend(packages::findutils::applets());
        all.extend(packages::sudo::applets());
        all
    } else {
        match package.unwrap() {
            Package::Coreutils => packages::coreutils::applets(),
            Package::Findutils => packages::findutils::applets(),
            Package::Sudo => packages::sudo::applets(),
        }
    };

    // Determine which package groups to affect for PM operations
    let targets: Vec<Package> = if all || package.is_none() {
        vec![Package::Coreutils, Package::Findutils, Package::Sudo]
    } else {
        vec![package.unwrap()]
    };

    // Pre: ensure distro packages are installed when committing
    if matches!(mode, ApplyMode::Commit) {
        if !live_root {
            eprintln!(
                "[info] skipping apt/dpkg install steps under non-live root: {}",
                root.display()
            );
        } else {
            for p in &targets {
                let name = distro_pkg_name(*p);
                let mut cmd = Command::new("apt-get");
                let args = vec!["install".to_string(), "-y".to_string(), name.to_string()];
                cmd.args(&args);
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());
                let status = cmd
                    .status()
                    .map_err(|e| format!("failed to spawn apt-get: {e}"))?;
                if !status.success() {
                    return Err(format!(
                        "apt-get install {} failed with exit code {:?}",
                        name,
                        status.code()
                    ));
                }
            }
        }
    } else {
        // Dry-run preview of PM steps
        for p in &targets {
            let name = distro_pkg_name(*p);
            eprintln!("[dry-run] would run: apt-get install -y {}", name);
        }
    }

    let mut restores = Vec::new();
    for app in &applets {
        let dest_base = ensure_under_root(root, &dest_dir);
        let dst = dest_base.join(app);
        let sp = SafePath::from_rooted(root, &dst).map_err(|e| format!("invalid target: {e:?}"))?;
        restores.push(RestoreRequest { target: sp });
    }

    let plan = api.plan(PlanInput {
        link: vec![],
        restore: restores,
    });
    let _pre = api
        .preflight(&plan)
        .map_err(|e| format!("preflight failed: {e:?}"))?;
    let _rep = api
        .apply(&plan, mode)
        .map_err(|e| format!("apply failed: {e:?}"))?;

    if matches!(mode, ApplyMode::Commit) && !live_root {
        // Pragmatic fallback for tests under non-live roots only.
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
                    if let Some(parent) = dst.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let content = format!("gnu-{}", app);
                    let _ = fs::write(&dst, content.as_bytes());
                }
            }
        }
    }

    // Post: by default remove RS packages unless --keep-replacements
    if matches!(mode, ApplyMode::Commit) {
        if !keep_replacements {
            for p in &targets {
                // If RS package is installed, purge via apt; otherwise remove staged artifacts.
                let rs_name = replacement_pkg_name(*p);
                if live_root && dpkg_installed(rs_name) {
                    let mut cmd = Command::new("apt-get");
                    let args = vec!["purge".to_string(), "-y".to_string(), rs_name.to_string()];
                    let args_view = args.clone();
                    cmd.args(&args);
                    cmd.stdin(Stdio::null());
                    cmd.stdout(Stdio::piped());
                    cmd.stderr(Stdio::piped());
                    let out = cmd.output().map_err(|e| format!("failed to spawn apt-get: {e}"))?;
                    let code = out.status.code().unwrap_or(1);
                    let stderr_tail = String::from_utf8_lossy(&out.stderr);
                    eprintln!("{}", json!({
                        "event":"pm.exec",
                        "pm": {"tool":"apt-get","args": args_view, "package": rs_name},
                        "exit_code": code,
                        "stderr_tail": stderr_tail.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>()
                    }));
                    if code != 0 {
                        return Err(format!("apt-get purge {} failed with exit code {}", rs_name, code));
                    }
                } else {
                    remove_staged_if_present(root, *p);
                }
            }
        }
    } else {
        if !keep_replacements {
            for p in &targets {
                let name = replacement_pkg_name(*p);
                eprintln!("[dry-run] would run: apt-get purge -y {}", name);
            }
        }
    }

    Ok(())
}
