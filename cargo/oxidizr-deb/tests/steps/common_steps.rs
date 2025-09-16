use cucumber::given;
use std::path::PathBuf;

use crate::bdd_world::World;

#[given(regex = r"^a fakeroot with stock coreutils applets$")]
pub async fn fakeroot_with_stock_coreutils(world: &mut World) {
    // seed a minimal set so backup/restore has originals to snapshot
    let root = world.ensure_root().to_path_buf();
    let usr_bin = world.under_root("/usr/bin");
    std::fs::create_dir_all(&usr_bin).expect("mkdir -p usr/bin");
    // Seed a couple of representative applets
    world.write_file("/usr/bin/ls", b"gnu-ls", true);
    world.write_file("/usr/bin/cat", b"gnu-cat", true);
    // Seed findutils applets as stock
    world.write_file("/usr/bin/find", b"gnu-find", true);
    world.write_file("/usr/bin/xargs", b"gnu-xargs", true);
    // Also ensure lock dir exists
    std::fs::create_dir_all(world.under_root("/var/lock")).unwrap();
    // Debug hint
    eprintln!("[bdd] fakeroot at {}", root.display());
}

#[given(regex = r"^a staging root at .*$")]
pub async fn staging_root_at(world: &mut World) {
    // The test harness always uses a fresh TempDir; the literal path in the feature is informational only.
    // Ensure common directories exist to satisfy preflight checks.
    world.ensure_dir("/usr/bin");
    world.ensure_dir("/var/lock");
}

#[given(regex = r#"^a verified replacement artifact is available for package "(coreutils|findutils|sudo)"$"#)]
pub async fn verified_artifact_available(world: &mut World, pkg: String) {
    let (rel_path, contents): (PathBuf, &'static [u8]) = match pkg.as_str() {
        "coreutils" => (PathBuf::from("/opt/uutils/uutils"), b"uutils-binary"),
        "findutils" => (PathBuf::from("/opt/uutils-findutils/uutils-findutils"), b"uutils-findutils-binary"),
        "sudo" => (PathBuf::from("/opt/sudo-rs/sudo-rs"), b"sudo-rs-binary"),
        _ => unreachable!(),
    };
    let abs = world.under_root(&rel_path);
    if let Some(parent) = abs.parent() { std::fs::create_dir_all(parent).unwrap(); }
    std::fs::write(&abs, contents).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&abs).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&abs, perms).unwrap();
    }
    world.artifact_path = Some(rel_path);
}

#[given(regex = r"^the sudo artifact has setuid 4755$")]
pub async fn sudo_artifact_setuid(world: &mut World) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let rel = world.artifact_path.as_ref().expect("artifact path").clone();
        let abs = world.under_root(rel);
        let mut perms = std::fs::metadata(&abs).unwrap().permissions();
        perms.set_mode(0o4755);
        std::fs::set_permissions(&abs, perms).unwrap();
    }
}
