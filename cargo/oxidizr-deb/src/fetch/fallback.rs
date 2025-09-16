use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::args::Package;
 

pub fn apt_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "rust-coreutils",
        Package::Findutils => "rust-findutils",
        Package::Sudo => "sudo-rs",
    }
}

fn run(cmd: &str, args: &[&str]) -> Result<(i32, String, String), String> {
    let mut c = Command::new(cmd);
    c.args(args);
    c.stdin(Stdio::null());
    c.stdout(Stdio::piped());
    c.stderr(Stdio::piped());
    match c.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(1);
            let so = String::from_utf8_lossy(&out.stdout).to_string();
            let se = String::from_utf8_lossy(&out.stderr).to_string();
            Ok((code, so, se))
        }
        Err(e) => Err(format!("failed to spawn {}: {}", cmd, e)),
    }
}

fn apt_install(pkg: &str) -> Result<(), String> {
    let (code, _so, se) = run("apt-get", &["update"])?;
    eprintln!("{}", serde_json::json!({
        "event":"pm.update","pm":{"tool":"apt-get","args":["update"]},"exit_code":code,
        "stderr_tail": se.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>()
    }));
    if code != 0 {
        return Err("apt-get update failed".to_string());
    }
    // Optional version pin via env
    let pin = std::env::var("OXIDIZR_DEB_APT_VERSION").ok();
    let mut install_args = vec!["install", "-y"];
    let pinned_name;
    if let Some(v) = pin.as_deref() {
        pinned_name = format!("{}={}", pkg, v);
        install_args.push(&pinned_name);
    } else {
        install_args.push(pkg);
    }
    let (code, _so, se) = run("apt-get", &install_args)?;
    eprintln!("{}", serde_json::json!({
        "event":"pm.install","pm":{"tool":"apt-get","args":install_args,"package":pkg},"exit_code":code,
        "stderr_tail": se.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>()
    }));
    if code != 0 {
        return Err(format!("apt-get install {} failed", pkg));
    }
    Ok(())
}

fn dpkg_locate_binary(pkgname: &str, candidates: &[&str]) -> Option<PathBuf> {
    if let Ok(out) = Command::new("dpkg").args(["-L", pkgname]).output() {
        let list = String::from_utf8_lossy(&out.stdout);
        for line in list.lines() {
            let p = Path::new(line.trim());
            if p.is_file() {
                let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if candidates.iter().any(|c| *c == fname) {
                    return Some(p.to_path_buf());
                }
            }
        }
    }
    None
}

// Online fallbacks removed; apt-only path is supported.

pub fn ensure_artifact_available(root: &Path, pkg: Package, commit: bool) -> Result<PathBuf, String> {
    let setuid = matches!(pkg, Package::Sudo);
    let root_is_live = root == Path::new("/");

    // 1) Try apt if in commit and on live root
    if commit && root_is_live {
        let apt_pkg = apt_pkg_name(pkg);
        if let Err(e) = apt_install(apt_pkg) {
            eprintln!("[info] apt path failed: {}", e);
        } else {
            // locate installed unified binary and use it directly (keeps updates via apt)
            let candidates: Vec<&str> = match pkg {
                Package::Coreutils => vec!["uutils", "coreutils"],
                Package::Findutils => vec!["uutils-findutils"],
                Package::Sudo => vec!["sudo-rs", "sudo"],
            };
            if let Some(installed) = dpkg_locate_binary(apt_pkg, &candidates) {
                return Ok(installed);
            }
        }
    }

    Err("replacement retrieval requires apt on --root=/; online fallbacks are disabled".to_string())
}
