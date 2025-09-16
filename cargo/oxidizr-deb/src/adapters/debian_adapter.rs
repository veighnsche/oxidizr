use std::path::Path;
use std::process::Command;

use oxidizr_cli_core::{DistroAdapter, PackageKind};

pub struct DebianAdapter;

impl DistroAdapter for DebianAdapter {
    fn enumerate_package_commands(&self, root: &Path, pkg: PackageKind) -> Vec<String> {
        if root != Path::new("/") {
            return Vec::new();
        }
        let name = match pkg {
            PackageKind::Coreutils => "coreutils",
            PackageKind::Findutils => "findutils",
            PackageKind::Sudo => "sudo",
        };
        let out = match Command::new("dpkg-query").args(["-L", name]).output() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };
        if !out.status.success() {
            return Vec::new();
        }
        let s = String::from_utf8_lossy(&out.stdout);
        let mut names = Vec::new();
        for line in s.lines() {
            if let Some(n) = line.strip_prefix("/usr/bin/").or_else(|| line.strip_prefix("/bin/")) {
                if !n.is_empty() && !n.ends_with('/') {
                    names.push(n.to_string());
                }
            }
        }
        names.sort();
        names.dedup();
        names
    }
}
