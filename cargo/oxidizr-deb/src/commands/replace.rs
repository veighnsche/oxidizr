use std::path::Path;
use std::process::{Command, Stdio};

use switchyard::logging::JsonlSink;
use switchyard::types::ApplyMode;
use switchyard::Switchyard;

use crate::adapters::debian::pm_lock_message;
use crate::cli::args::Package;

fn distro_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "coreutils",
        Package::Findutils => "findutils",
        Package::Sudo => "sudo",
    }
}

fn is_active(root: &Path, pkg: Package) -> bool {
    let path = match pkg {
        Package::Coreutils => root.join("usr/bin/ls"),
        Package::Findutils => root.join("usr/bin/find"),
        Package::Sudo => root.join("usr/bin/sudo"),
    };
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn exec(
    api: &Switchyard<JsonlSink, JsonlSink>,
    root: &Path,
    package: Option<Package>,
    all: bool,
    mode: ApplyMode,
    assume_yes: bool,
) -> Result<(), String> {
    if matches!(mode, ApplyMode::Commit) {
        if let Some(msg) = pm_lock_message(root) {
            return Err(msg);
        }
        // Live-root constraint for PM mutations
        if root != Path::new("/") {
            return Err(
                "replace operations require --root=/ (live system) for apt/dpkg changes"
                    .to_string(),
            );
        }
        // Confirm if interactive
        if !assume_yes && !crate::util::prompts::should_proceed(assume_yes, root) {
            return Err("aborted by user".to_string());
        }
    }

    let targets: Vec<Package> = if all {
        vec![Package::Coreutils, Package::Findutils, Package::Sudo]
    } else if let Some(p) = package {
        vec![p]
    } else {
        return Err("specify a package or use --all".to_string());
    };

    // First ensure RS is installed & active (use semantics)
    for p in &targets {
        // offline/use_local not applicable here; rely on system packages
        crate::commands::r#use::exec(api, root, *p, false, None, mode)?;
        // Provider pre-check: replacement must now be active
        if !is_active(root, *p) {
            return Err(format!("replacement for {:?} is not active after use; aborting replace", p));
        }
    }

    // Then remove the distro packages under guardrails
    for p in &targets {
        let name = distro_pkg_name(*p);
        if matches!(mode, ApplyMode::DryRun) {
            eprintln!("[dry-run] would run: apt-get purge -y {}", name);
            continue;
        }
        let mut cmd = Command::new("apt-get");
        let args = vec!["purge".to_string(), "-y".to_string(), name.to_string()];
        cmd.args(&args);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::piped());
        match cmd.output() {
            Ok(out) => {
                let code = out.status.code().unwrap_or(1);
                if code != 0 {
                    return Err(format!("apt-get purge {} failed with exit code {}", name, code));
                }
            }
            Err(e) => return Err(format!("failed to spawn apt-get: {e}")),
        }
        // Pre-check: replacement must be active before purging GNU packages
        if matches!(mode, ApplyMode::Commit) {
            if !is_active(root, *p) {
                return Err(format!("replacement for {:?} not active before purge; aborting", p));
            }
        }
    }

    // Post-check: replacement must remain active after purging GNU packages
    if matches!(mode, ApplyMode::Commit) {
        for p in &targets {
            if !is_active(root, *p) {
                return Err(format!("invariant violation: replacement for {:?} not active after purge", p));
            }
        }
    }

    Ok(())
}
