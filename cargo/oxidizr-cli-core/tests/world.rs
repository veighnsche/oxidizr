#[cfg(feature = "bdd")]
#[derive(Debug, Default, cucumber::World)]
pub struct World {
    pub root: Option<tempfile::TempDir>,
    pub artifact_path: Option<std::path::PathBuf>,
    pub last_vec: Vec<String>,
    pub last_result: Option<Result<(), Vec<String>>>,
    pub distro_map: std::collections::HashMap<oxidizr_cli_core::PackageKind, Vec<String>>,
}

#[cfg(feature = "bdd")]
impl World {
    pub fn ensure_root(&mut self) -> &std::path::Path {
        if self.root.is_none() {
            self.root = Some(tempfile::TempDir::new().expect("temp root"));
        }
        self.root.as_ref().unwrap().path()
    }

    pub fn under_root<P: AsRef<std::path::Path>>(&mut self, rel: P) -> std::path::PathBuf {
        let r = self.ensure_root().to_path_buf();
        let rel = rel.as_ref();
        if rel.is_absolute() {
            r.join(rel.strip_prefix("/").unwrap())
        } else {
            r.join(rel)
        }
    }

    pub fn ensure_dir<P: AsRef<std::path::Path>>(&mut self, rel: P) {
        let p = self.under_root(rel);
        std::fs::create_dir_all(&p).expect("mkdir");
    }

    pub fn write_file_exec<P: AsRef<std::path::Path>>(&mut self, rel: P, contents: &str) {
        let p = self.under_root(&rel);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&p, contents).expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&p).unwrap().permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&p, perms);
        }
    }

    pub fn set_distro(&mut self, pkg: oxidizr_cli_core::PackageKind, names: Vec<String>) {
        self.distro_map.insert(pkg, names);
    }

    pub fn make_adapter(&self) -> MockAdapter {
        MockAdapter {
            map: self.distro_map.clone(),
        }
    }
}

#[cfg(feature = "bdd")]
pub struct MockAdapter {
    map: std::collections::HashMap<oxidizr_cli_core::PackageKind, Vec<String>>,
}

#[cfg(feature = "bdd")]
impl oxidizr_cli_core::DistroAdapter for MockAdapter {
    fn enumerate_package_commands(
        &self,
        _root: &std::path::Path,
        pkg: oxidizr_cli_core::PackageKind,
    ) -> Vec<String> {
        self.map.get(&pkg).cloned().unwrap_or_default()
    }
}
