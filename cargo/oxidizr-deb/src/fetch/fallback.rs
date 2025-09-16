use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::args::Package;
use crate::fetch::resolver::{staged_default_path};

pub fn apt_pkg_name(pkg: Package) -> &'static str {
    match pkg {
        Package::Coreutils => "rust-coreutils",
        Package::Findutils => "rust-findutils",
        Package::Sudo => "sudo-rs",
    }
}

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
        "event":"pm.exec","pm":{"tool":"apt-get","args":["update"]},"exit_code":code,
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
        "event":"pm.exec","pm":{"tool":"apt-get","args":install_args,"package":pkg},"exit_code":code,
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

fn ensure_rustup_and_cargo() -> Result<(), String> {
    if Command::new("cargo").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok() {
        return Ok(());
    }
    // Install rustup non-interactively
    let (code, _so, se) = run("bash", &["-lc", "curl https://sh.rustup.rs -sSf | sh -s -- -y"]) ?;
    if code != 0 { return Err(format!("rustup install failed: {}", se)); }
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
    let mut install_cmd = format!("cargo install --locked {}", crate_name);
    if let Some(ver) = std::env::var("OXIDIZR_DEB_CARGO_VERSION").ok() {
        install_cmd.push_str(&format!(" --version {}", ver));
    }
    cmd.arg(install_cmd);
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
    if setuid_root {
        if !is_root() {
            return Err("sudo replacement requires root to set setuid root:root".to_string());
        }
        perm.set_mode(0o4755);
    } else {
        perm.set_mode(0o755);
    }
    fs::set_permissions(&dest, perm).map_err(|e| e.to_string())?;
    if setuid_root {
        chown_root(&dest)?;
    }
    Ok(dest)
}

pub fn ensure_artifact_available(root: &Path, pkg: Package, commit: bool) -> Result<PathBuf, String> {
    let setuid = matches!(pkg, Package::Sudo);
    let root_is_live = root == Path::new("/");

    // 1) Try apt if in commit and on live root
    if commit && root_is_live {
        let apt_pkg = apt_pkg_name(pkg);
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
        // Detect arch for asset selection
        let (arch_code, arch_out, _) = run("uname", &["-m"])?;
        if arch_code != 0 { return Err("failed to detect architecture".to_string()); }
        let arch = arch_out.trim();
        // Fetch release JSON (tagged or latest)
        let endpoint = if let Some(tag) = std::env::var("OXIDIZR_DEB_GITHUB_TAG").ok() {
            format!("https://api.github.com/repos/oxidecomputer/sudo-rs/releases/tags/{}", tag)
        } else {
            "https://api.github.com/repos/oxidecomputer/sudo-rs/releases/latest".to_string()
        };
        let (code, body, _) = run("curl", &["-sSL", &endpoint])?;
        if code != 0 { return Err(format!("curl failed: {}", body)); }
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("parse json: {}", e))?;
        let assets = v.get("assets").and_then(|a| a.as_array()).ok_or_else(|| "missing assets array".to_string())?;
        let mut url: Option<String> = None;
        let mut asset_name: Option<String> = None;
        for a in assets {
            let name = a.get("name").and_then(|s| s.as_str()).unwrap_or("");
            let arch_ok = match arch {
                "x86_64" => name.contains("x86_64"),
                "aarch64" | "arm64" => name.contains("aarch64") || name.contains("arm64"),
                _ => false,
            };
            if arch_ok && (name.contains("linux") || name.contains("gnu")) && (name.ends_with(".tar.gz") || name.ends_with(".tar.xz")) {
                url = a.get("browser_download_url").and_then(|s| s.as_str()).map(|s| s.to_string());
                asset_name = Some(name.to_string());
                if url.is_some() { break; }
            }
        }
        let url = url.ok_or_else(|| "no suitable sudo-rs asset found".to_string())?;
        let asset_name = asset_name.unwrap_or_else(|| "sudo-rs.tar".to_string());
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
        // Attempt checksum verification via sidecar asset (sha256)
        let mut sha_url: Option<String> = None;
        for a in assets {
            let name = a.get("name").and_then(|s| s.as_str()).unwrap_or("");
            if name.contains("sha256") && name.contains(&asset_name) {
                sha_url = a.get("browser_download_url").and_then(|s| s.as_str()).map(|s| s.to_string());
                break;
            }
        }
        if let Some(sha_url) = sha_url {
            let (sc, body, _) = run("curl", &["-sSL", &sha_url])?;
            if sc != 0 { return Err("failed to download checksum".to_string()); }
            let expected_hex = body.split_whitespace().next().unwrap_or("").trim().to_lowercase();
            if expected_hex.len() >= 16 { // minimal sanity
                use sha2::{Digest, Sha256};
                let mut file = fs::File::open(&tar_path).map_err(|e| e.to_string())?;
                let mut hasher = Sha256::new();
                std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
                let sum = hasher.finalize();
                let got_hex = hex::encode(sum);
                if got_hex != expected_hex {
                    return Err("checksum verification failed for sudo-rs asset".to_string());
                }
            }
        } else {
            return Err("missing checksum for sudo-rs asset; refusing to install from GitHub without verification".to_string());
        }
        // Extract
        let tarfile = tar_path.to_string_lossy().to_string();
        let (code, _so, se) = if url.ends_with(".tar.gz") {
            run("bash", &["-lc", &format!("tar -C '{}' -xzf '{}'", tmpd.path().display(), tarfile)])
        } else {
            run("bash", &["-lc", &format!("tar -C '{}' -xJf '{}'", tmpd.path().display(), tarfile)])
        }?;
        if code != 0 { return Err(format!("tar extract failed: {}", se)); }
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
