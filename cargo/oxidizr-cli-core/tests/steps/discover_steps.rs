use cucumber::when;

use crate::world::World;

#[when(regex = r#"^I call discover_applets_with_allow with allow \"([^\"]*)\"$"#)]
pub async fn call_discover(world: &mut World, allow: String) {
    let src_rel = world
        .artifact_path
        .as_ref()
        .cloned()
        .expect("artifact path set");
    let allow_vec: Vec<String> = allow
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let abs = world.under_root(src_rel);
    let out = oxidizr_cli_core::discover_applets_with_allow(&abs, &allow_vec);
    world.last_vec = out;
}
