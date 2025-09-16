use std::path::Path;
use std::process::{Command, Stdio};

use switchyard::logging::JsonlSink;
use switchyard::types::ApplyMode;
use switchyard::Switchyard;
use oxidizr_cli_core::prompts::should_proceed;
use oxidizr_cli_core::{coverage_preflight, PackageKind};

use crate::adapters::debian::pm_lock_message;
use crate::cli::args::Package;
use serde_json::json;
use crate::fetch::fallback::apt_pkg_name;
use crate::fetch::resolver::resolve_artifact;
use crate::packages;
use crate::adapters::debian_adapter::DebianAdapter;

fn distro_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "coreutils",
        Package::Findutils => "findutils",
        Package::Sudo => "sudo",
    }
}

fn replacement_pkg_name(pkg: Package) -> &'static str { apt_pkg_name(pkg) }

fn dpkg_installed(name: &str) -> bool {
    let st = Command::new("dpkg")
        .args(["-s", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    matches!(st, Ok(s) if s.success())
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
        if !assume_yes && !should_proceed(assume_yes, root) {
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
        // Coverage preflight: replacement must cover all distro-provided applets (coreutils/findutils)
        let kind = match p {
            Package::Coreutils => Some(PackageKind::Coreutils),
            Package::Findutils => Some(PackageKind::Findutils),
            Package::Sudo => None,
        };
        if let Some(k) = kind {
            let src = resolve_artifact(root, *p, false, None);
            if let Err(missing) = coverage_preflight(&DebianAdapter, root, k, &src) {
                return Err(format!(
                    "cannot replace {:?}: replacement does not cover all applets; missing: {}",
                    p,
                    missing.join(", ")
                ));
            }
        }
    }

    // Then remove the distro packages under guardrails
    for p in &targets {
        let name = distro_pkg_name(*p);
        if matches!(mode, ApplyMode::DryRun) {
            eprintln!("[dry-run] would run: apt-get purge -y {}", name);
            continue;
        }
        // Provider invariant pre-check: ensure at least one provider remains available
        let rs_name = replacement_pkg_name(*p);
        let have_rs_pkg = dpkg_installed(rs_name);
        if !is_active(root, *p) {
            return Err(format!("invariant violation: replacement for {:?} not active before purge", p));
        }
        if !have_rs_pkg {
            return Err(format!("invariant violation: no replacement package present for {:?}", p));
        }
        let mut cmd = Command::new("apt-get");
        let args = vec!["purge".to_string(), "-y".to_string(), name.to_string()];
        let args_view = args.clone();
        cmd.args(&args);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let out = cmd.output().map_err(|e| format!("failed to spawn apt-get: {e}"))?;
        let code = out.status.code().unwrap_or(1);
        let stderr_tail = String::from_utf8_lossy(&out.stderr);
        eprintln!("{}", json!({
            "event":"pm.purge",
            "pm": {"tool":"apt-get","args": args_view, "package": name},
            "exit_code": code,
            "stderr_tail": stderr_tail.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>()
        }));
        if code != 0 {
            return Err(format!("apt-get purge {} failed with exit code {}", name, code));
        }
        // Pre-check: replacement must be active before purging GNU packages
        if matches!(mode, ApplyMode::Commit) {
            if !is_active(root, *p) {
                return Err(format!("replacement for {:?} not active before purge; aborting", p));
            }
        }
    }

    // Post-check: replacement must remain active after purging GNU packages and provider remains
    if matches!(mode, ApplyMode::Commit) {
        for p in &targets {
            if !is_active(root, *p) {
                return Err(format!("invariant violation: replacement for {:?} not active after purge", p));
            }
            let rs_name = replacement_pkg_name(*p);
            if !dpkg_installed(rs_name) {
                return Err(format!("invariant violation: no provider present for {:?} after purge", p));
            }
        }
    }

    Ok(())
}
