use std::path::{Path, PathBuf};

/// Determine whether the current target state already matches the sidecar's prior state,
/// in which case a restore would be a no-op.
#[must_use]
pub fn is_idempotent(target_path: &Path, prior_kind: &str, prior_dest: Option<&str>) -> bool {
    let kind_now = match std::fs::symlink_metadata(target_path) {
        Ok(md) => {
            let ft = md.file_type();
            if ft.is_symlink() {
                "symlink"
            } else if ft.is_file() {
                "file"
            } else {
                "other"
            }
        }
        Err(_) => "none",
    };

    match prior_kind {
        "file" if kind_now == "file" => true,
        "symlink" if kind_now == "symlink" => {
            if let Some(want_str) = prior_dest {
                if let Ok(cur) = std::fs::read_link(target_path) {
                    let want = PathBuf::from(want_str);
                    // Compare resolved forms for robustness
                    let mut cur_res = cur.clone();
                    if cur_res.is_relative() {
                        if let Some(parent) = target_path.parent() {
                            cur_res = parent.join(cur_res);
                        }
                    }
                    let mut want_res = want.clone();
                    if want_res.is_relative() {
                        if let Some(parent) = target_path.parent() {
                            want_res = parent.join(want_res);
                        }
                    }
                    let cur_res = std::fs::canonicalize(&cur_res).unwrap_or(cur_res);
                    let want_res = std::fs::canonicalize(&want_res).unwrap_or(want_res);
                    return cur_res == want_res;
                }
            }
            false
        }
        "none" if kind_now == "none" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn td() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn idempotent_when_file_and_prior_file() {
        let t = td();
        let tgt = t.path().join("usr/bin/app");
        std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
        std::fs::write(&tgt, b"data").unwrap();
        assert!(is_idempotent(&tgt, "file", None));
    }

    #[test]
    fn idempotent_when_symlink_matches_prior_dest() {
        let t = td();
        let root = t.path();
        let dest = root.join("bin");
        std::fs::create_dir_all(&dest).unwrap();
        let tgt = root.join("usr/bin/app");
        std::fs::create_dir_all(tgt.parent().unwrap()).unwrap();
        // absolute symlink to dest
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&dest, &tgt).unwrap();
        }
        let ok = is_idempotent(&tgt, "symlink", Some(dest.to_str().unwrap()));
        assert!(ok);
    }

    #[test]
    fn idempotent_when_none_and_target_missing() {
        let t = td();
        let tgt = t.path().join("usr/bin/missing");
        assert!(is_idempotent(&tgt, "none", None));
    }
}
