use cucumber::given;

use crate::bdd_world::World;

#[given(regex = r"^dpkg/apt locks are present$")]
pub async fn dpkg_apt_locks_present(world: &mut World) {
    let p = world.under_root("/var/lib/dpkg/lock-frontend");
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&p, b"lock").unwrap();
}

#[given(regex = r"^non-root sudo owner is allowed in tests$")]
pub async fn allow_nonroot_sudo_owner(world: &mut World) {
    world.envs.push((
        "OXIDIZR_DEB_TEST_ALLOW_NONROOT_SUDO_OWNER".into(),
        "1".into(),
    ));
}

#[given(regex = r"^EXDEV degraded fallback is forced via env$")]
pub async fn force_exdev(world: &mut World) {
    world
        .envs
        .push(("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES".into(), "1".into()));
    world
        .envs
        .push(("SWITCHYARD_FORCE_EXDEV".into(), "1".into()));
}
