use cucumber::given;

use crate::bdd_world::World;

#[given(regex = r"^a mutating public API endpoint$")]
pub async fn given_mutating_public_api_endpoint(world: &mut World) {
    // Placeholder: ensure API exists; actual mutation steps will exercise endpoints
    world.ensure_api();
}

#[given(regex = r"^a mutation of a final path component under a parent directory$")]
pub async fn given_mutation_final_component(world: &mut World) {
    // Prepare a simple parent/child path under the temp root
    let root = world.ensure_root().to_path_buf();
    let parent = root.join("usr/bin");
    let child = parent.join("touch_me");
    let _ = std::fs::create_dir_all(&parent);
    let _ = std::fs::write(&child, b"x");
    // No additional state needed; subsequent steps will exercise TOCTOU-safe ops
}
