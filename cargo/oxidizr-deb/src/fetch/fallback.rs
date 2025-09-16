use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::args::Package;
use crate::fetch::resolver::{staged_default_path};

fn home_bin() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".cargo/bin")
}

fn is_exe(p: &Path) -> bool {
    match fs::metadata(p) {
        Ok(md) => md.is_file() && (md.permissions().mode() & 0o111) != 0,
        Err(_) => false,
    }
}

fn run(cmd: &str, args: &[&str]) -> Result<(i32, String), String> {
    let mut c = Command::new(cmd);
    c.args(args);
    c.stdin(Stdio::null());
    c.stdout(Stdio::piped());
    c.stderr(Stdio::piped());
    match c.output() {
        Ok(out) => {
            let code = out.status.code().unwrap_or(1);
            let mut s = String::new();
            s.push_str(&String::from_utf8_lossy(&out.stdout));
            s.push_str(&String::from_utf8_lossy(&out.stderr));
            Ok((code, s))
        }
        Err(e) => Err(format!("failed to spawn {}: {}", cmd, e)),
    }
}

fn apt_install(pkg: &str) -> Result<(), String> {
    let (code, out) = run("apt-get", &["update"])?;
    if code != 0 {
        return Err(format!("apt-get update failed: {}", out));
    }
    let (code, out) = run("apt-get", &["install", "-y", pkg])?;
    if code != 0 {
        return Err(format!("apt-get install {} failed: {}", pkg, out));
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

fn ensure_rustup_and_cargo() -> Result<(), String> {
    if Command::new("cargo").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok() {
        return Ok(());
    }
    // Install rustup non-interactively
    let (code, out) = run("bash", &["-lc", "curl https://sh.rustup.rs -sSf | sh -s -- -y"]) ?;
    if code != 0 { return Err(format!("rustup install failed: {}", out)); }
    Ok(())
}

fn cargo_install(crate_name: &str) -> Result<(), String> {
    // Ensure cargo in PATH by prefixing HOME/.cargo/bin
    let home_bin = home_bin();
    let mut cmd = Command::new("bash");
    cmd.arg("-lc");
    let mut env_path = OsString::from(home_bin.to_string_lossy().to_string());
    env_path.push(OsString::from(":"));
    env_path.push(std::env::var_os("PATH").unwrap_or_default());
    cmd.env("PATH", env_path);
    cmd.arg(format!("cargo install {}", crate_name));
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    match cmd.status() {
        Ok(st) if st.success() => Ok(()),
        Ok(st) => Err(format!("cargo install {} exited with {}", crate_name, st.code().unwrap_or(1))),
        Err(e) => Err(format!("failed to run cargo install {}: {}", crate_name, e)),
    }
}

fn is_root() -> bool { unsafe { libc::geteuid() == 0 } }

fn chown_root(p: &Path) -> Result<(), String> {
    // Try to set owner to root:root using `chown` command to avoid extra deps
    let st = Command::new("chown")
        .args(["root:root", &p.to_string_lossy()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("failed to spawn chown: {}", e))?;
    if !st.success() {
        return Err("chown root:root failed".to_string());
    }
    Ok(())
}

fn stage_into(root: &Path, pkg: Package, src: &Path, setuid_root: bool) -> Result<PathBuf, String> {
    let dest = staged_default_path(root, pkg);
    if let Some(parent) = dest.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::copy(src, &dest).map_err(|e| format!("copy {} -> {} failed: {}", src.display(), dest.display(), e))?;
    let mut perm = fs::metadata(&dest).map_err(|e| e.to_string())?.permissions();
    if setuid_root { perm.set_mode(0o4755); } else { perm.set_mode(0o755); }
    fs::set_permissions(&dest, perm).map_err(|e| e.to_string())?;
    if setuid_root && is_root() {
        let _ = chown_root(&dest);
    }
    Ok(dest)
}

pub fn ensure_artifact_available(root: &Path, pkg: Package, commit: bool) -> Result<PathBuf, String> {
    let setuid = matches!(pkg, Package::Sudo);
    let root_is_live = root == Path::new("/");

    // 1) Try apt if in commit and on live root
    if commit && root_is_live {
        let apt_pkg = match pkg {
            Package::Coreutils => "uutils-coreutils",
            Package::Findutils => "uutils-findutils",
            Package::Sudo => "sudo-rs",
        };
        if let Err(e) = apt_install(apt_pkg) {
            eprintln!("[info] apt path failed: {}", e);
        } else {
            // locate installed unified binary
            let candidates: Vec<&str> = match pkg {
                Package::Coreutils => vec!["uutils", "coreutils"],
                Package::Findutils => vec!["uutils-findutils"],
                Package::Sudo => vec!["sudo-rs", "sudo"],
            };
            if let Some(installed) = dpkg_locate_binary(apt_pkg, &candidates) {
                return stage_into(root, pkg, &installed, setuid);
            }
        }
    }

    // 2) Fallback: cargo install and stage
    if matches!(pkg, Package::Coreutils | Package::Findutils) {
        ensure_rustup_and_cargo()?;
        let (crate_name, bin_candidates): (&str, Vec<&str>) = match pkg {
            Package::Coreutils => ("coreutils", vec!["coreutils", "uutils"]),
            Package::Findutils => ("uutils-findutils", vec!["uutils-findutils"]),
            _ => unreachable!(),
        };
        cargo_install(crate_name)?;
        let hb = home_bin();
        for b in bin_candidates {
            let p = hb.join(b);
            if is_exe(&p) {
                return stage_into(root, pkg, &p, setuid);
            }
        }
        return Err(format!("cargo installed {}, but no expected binary found in {}", crate_name, hb.display()));
    }

    // 3) Fallback for sudo: GitHub releases
    if matches!(pkg, Package::Sudo) {
        // Fetch latest release JSON
        let (code, body) = run("curl", &["-sSL", "https://api.github.com/repos/oxidecomputer/sudo-rs/releases/latest"])?;
        if code != 0 { return Err(format!("curl failed: {}", body)); }
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("parse json: {}", e))?;
        let assets = v.get("assets").and_then(|a| a.as_array()).ok_or_else(|| "missing assets array".to_string())?;
        let mut url: Option<String> = None;
        for a in assets {
            let name = a.get("name").and_then(|s| s.as_str()).unwrap_or("");
            if name.contains("x86_64") && (name.contains("linux") || name.contains("gnu")) && (name.ends_with(".tar.gz") || name.ends_with(".tar.xz")) {
                url = a.get("browser_download_url").and_then(|s| s.as_str()).map(|s| s.to_string());
                if url.is_some() { break; }
            }
        }
        let url = url.ok_or_else(|| "no suitable sudo-rs asset found".to_string())?;
        let tmpd = tempfile::tempdir().map_err(|e| e.to_string())?;
        let tar_path = tmpd.path().join("sudo-rs.tarchive");
        let mut f = fs::File::create(&tar_path).map_err(|e| e.to_string())?;
        // Stream download
        let mut curl = Command::new("curl");
        curl.arg("-L").arg(&url);
        curl.stdout(Stdio::piped());
        let mut child = curl.spawn().map_err(|e| format!("curl spawn: {}", e))?;
        if let Some(mut so) = child.stdout.take() { std::io::copy(&mut so, &mut f).map_err(|e| e.to_string())?; }
        let status = child.wait().map_err(|e| e.to_string())?;
        if !status.success() { return Err(format!("curl download failed: {}", status)); }
        // Extract
        let tarfile = tar_path.to_string_lossy().to_string();
        let (code, out) = if url.ends_with(".tar.gz") {
            run("bash", &["-lc", &format!("tar -C '{}' -xzf '{}'", tmpd.path().display(), tarfile)])
        } else {
            run("bash", &["-lc", &format!("tar -C '{}' -xJf '{}'", tmpd.path().display(), tarfile)])
        }?;
        if code != 0 { return Err(format!("tar extract failed: {}", out)); }
        // Locate sudo-rs binary in tmpd
        let mut found: Option<PathBuf> = None;
        for entry in walkdir::WalkDir::new(tmpd.path()) {
            let e = entry.map_err(|e| e.to_string())?;
            if e.file_type().is_file() {
                let fname = e.file_name().to_string_lossy();
                if fname == "sudo-rs" || fname.starts_with("sudo-rs-") {
                    if is_exe(e.path()) { found = Some(e.into_path()); break; }
                }
            }
        }
        let found = found.ok_or_else(|| "sudo-rs binary not found in archive".to_string())?;
        return stage_into(root, pkg, &found, setuid);
    }

    Err("unreachable path in ensure_artifact_available".to_string())
}
