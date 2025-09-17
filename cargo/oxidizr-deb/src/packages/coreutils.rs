use std::path::Path;
use std::process::Command;

pub fn applets() -> Vec<String> {
    [
        "[",
        "arch",
        "b2sum",
        "base32",
        "base64",
        "basename",
        "basenc",
        "cat",
        "chcon",
        "chgrp",
        "chmod",
        "chown",
        "chroot",
        "cksum",
        "comm",
        "coreutils",
        "cp",
        "csplit",
        "cut",
        "date",
        "dd",
        "df",
        "dir",
        "dircolors",
        "dirname",
        "du",
        "echo",
        "env",
        "expand",
        "expr",
        "factor",
        "false",
        "fmt",
        "fold",
        "groups",
        "head",
        "hostid",
        "hostname",
        "id",
        "install",
        "join",
        "kill",
        "link",
        "ln",
        "logname",
        "ls",
        "md5sum",
        "mkdir",
        "mkfifo",
        "mknod",
        "mktemp",
        "mv",
        "nice",
        "nl",
        "nohup",
        "nproc",
        "numfmt",
        "od",
        "paste",
        "pathchk",
        "pinky",
        "pr",
        "printenv",
        "printf",
        "ptx",
        "pwd",
        "readlink",
        "realpath",
        "rm",
        "rmdir",
        "runcon",
        "seq",
        "sha1sum",
        "sha224sum",
        "sha256sum",
        "sha384sum",
        "sha512sum",
        "shred",
        "shuf",
        "sleep",
        "sort",
        "split",
        "stat",
        "stdbuf",
        "stty",
        "sum",
        "sync",
        "tac",
        "tail",
        "tee",
        "test",
        "timeout",
        "touch",
        "tr",
        "true",
        "truncate",
        "tsort",
        "tty",
        "uname",
        "unexpand",
        "uniq",
        "unlink",
        "uptime",
        "users",
        "vdir",
        "wc",
        "who",
        "whoami",
        "yes",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Try to discover supported applets by interrogating the replacement binary.
/// Strategy: try `--list`, then `--help`, and parse out known names intersected with our static set.
/// Falls back to the static full list if probing fails or yields an obviously incomplete set.
pub fn discover_applets_from_binary(source_bin: &Path) -> Vec<String> {
    let static_set = applets();

    // Helper to parse stdout into candidate names.
    fn parse_names(stdout: &str, allow: &[String]) -> Vec<String> {
        use std::collections::HashSet;
        let allow_set: HashSet<&str> = allow.iter().map(|s| s.as_str()).collect();
        let mut out = Vec::new();
        for token in stdout
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '|' || c == '/')
        {
            let t = token.trim();
            if !t.is_empty() && allow_set.contains(t) {
                out.push(t.to_string());
            }
        }
        out.sort();
        out.dedup();
        out
    }

    // Probe 1: --list
    if let Ok(out) = Command::new(source_bin).arg("--list").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let names = parse_names(&s, &static_set);
            if names.len() >= 10 {
                return names;
            }
        }
    }
    // Probe 2: --help
    if let Ok(out) = Command::new(source_bin).arg("--help").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let names = parse_names(&s, &static_set);
            if names.len() >= 10 {
                return names;
            }
        }
    }
    // Fallback: static full set
    static_set
}

/// Enumerate GNU coreutils applet names from dpkg on a live root.
/// Returns None if root != "/" or if dpkg query fails.
pub fn dpkg_coreutils_applets(root: &Path) -> Option<Vec<String>> {
    if root != Path::new("/") {
        return None;
    }
    let out = Command::new("dpkg-query")
        .args(["-L", "coreutils"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let mut names = Vec::new();
    for line in s.lines() {
        // Consider /bin and /usr/bin entries
        if let Some(name) = line
            .strip_prefix("/usr/bin/")
            .or_else(|| line.strip_prefix("/bin/"))
        {
            if !name.is_empty() && !name.ends_with('/') {
                names.push(name.to_string());
            }
        }
    }
    if names.is_empty() {
        return None;
    }
    names.sort();
    names.dedup();
    Some(names)
}

/// Resolve the set of applets to link during `use`.
/// Uses replacement binary discovery intersected with dpkg-provided applets on live systems.
pub fn resolved_applets_for_use(root: &Path, source_bin: &Path) -> Vec<String> {
    let repl = discover_applets_from_binary(source_bin);
    if let Some(gnu) = dpkg_coreutils_applets(root) {
        use std::collections::HashSet;
        let r: HashSet<_> = repl.iter().cloned().collect();
        let mut out: Vec<String> = gnu.into_iter().filter(|g| r.contains(g)).collect();
        out.sort();
        out.dedup();
        out
    } else {
        repl
    }
}

/// Return dpkg-derived applets for restore (live root) or full static list otherwise.
pub fn dpkg_coreutils_applets_or_static(root: &Path) -> Vec<String> {
    if let Some(gnu) = dpkg_coreutils_applets(root) {
        gnu
    } else {
        applets()
    }
}

/// Check full coverage before purge: return Ok(all_gnu) if replacement covers all dpkg applets; Err(missing) otherwise.
pub fn coverage_check(root: &Path, source_bin: &Path) -> Result<Vec<String>, Vec<String>> {
    let gnu = match dpkg_coreutils_applets(root) {
        Some(v) => v,
        None => return Ok(applets()),
    };
    let repl = discover_applets_from_binary(source_bin);
    use std::collections::HashSet;
    let r: HashSet<_> = repl.iter().collect();
    let missing: Vec<String> = gnu.iter().filter(|g| !r.contains(g)).cloned().collect();
    if missing.is_empty() {
        Ok(gnu)
    } else {
        Err(missing)
    }
}
