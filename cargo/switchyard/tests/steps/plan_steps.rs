use cucumber::{given, when};

use crate::bdd_world::World;
use crate::bdd_support::env::EnvGuard;
use switchyard::policy::types::SmokePolicy;

#[given(regex = r"^(/.+) is a symlink to (.+)$")]
pub async fn given_symlink(world: &mut World, link: String, dest: String) {
    world.mk_symlink(&link, &dest);
}

#[given(regex = r"^a plan with at least one action$")]
pub async fn given_plan_min(world: &mut World) {
    let link = world
        .last_link
        .clone()
        .unwrap_or_else(|| "/usr/bin/ls".to_string());
    let src = world
        .last_src
        .clone()
        .unwrap_or_else(|| "providerB/ls".to_string());
    world.build_single_swap(&link, &src);
}

#[given(regex = r"^the target and staging directories reside on different filesystems$")]
pub async fn given_exdev_env(world: &mut World) {
    world
        .env_guards
        .push(EnvGuard::new("SWITCHYARD_TEST_ALLOW_ENV_OVERRIDES", "1"));
    world
        .env_guards
        .push(EnvGuard::new("SWITCHYARD_FORCE_EXDEV", "1"));
}

#[given(regex = r"^the minimal post-apply smoke suite is configured$")]
pub async fn given_smoke(world: &mut World) {
    world.policy.governance.smoke = SmokePolicy::Require {
        auto_rollback: true,
    };
    let api = switchyard::api::Switchyard::builder(
        world.facts.clone(),
        world.audit.clone(),
        world.policy.clone(),
    )
    .with_smoke_runner(Box::new(switchyard::adapters::DefaultSmokeRunner))
    .build();
    world.api = Some(api);
}

#[when(regex = r"^I plan a swap to (\S+)$")]
pub async fn when_plan_swap(world: &mut World, provider: String) {
    let link = world
        .last_link
        .clone()
        .unwrap_or_else(|| "/usr/bin/ls".to_string());
    let src = format!("{}/ls", provider);
    world.build_single_swap(&link, &src);
}

#[given(regex = r"^a plan that mutates a file$")]
pub async fn given_plan_mutates(world: &mut World) {
    // Ensure a regular file exists at target so swap creates backup and after_kind changes
    let root = world.ensure_root().to_path_buf();
    let link = "/usr/bin/cp";
    let target = crate::bdd_support::util::under_root(&root, link);
    if let Some(p) = target.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let _ = std::fs::write(&target, b"old");
    world.mk_symlink(link, "providerA/cp");
    world.build_single_swap(link, "providerB/cp");
}

#[given(regex = r"^a plan with environment-derived values that may be sensitive$")]
pub async fn given_plan_env_sensitive(world: &mut World) {
    world
        .env_guards
        .push(EnvGuard::new("SWITCHYARD_HELPER", "paru"));
    given_plan_min(world).await;
}

#[given(regex = r"^a plan that uses an external helper$")]
pub async fn given_plan_external_helper(world: &mut World) {
    given_plan_env_sensitive(world).await
}
