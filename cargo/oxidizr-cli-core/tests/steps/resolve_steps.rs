use cucumber::when;

use crate::world::World;

#[when(regex = r#"^I call resolve_applets_for_use for package \"(coreutils|findutils|sudo)\"$"#)]
pub async fn call_resolve(world: &mut World, pkg: String) {
    let src_rel = world
        .artifact_path
        .as_ref()
        .cloned()
        .expect("artifact path set");
    let abs = world.under_root(src_rel);
    let adapter = world.make_adapter();
    let root = world.ensure_root().to_path_buf();
    let kind = match pkg.as_str() {
        "coreutils" => oxidizr_cli_core::PackageKind::Coreutils,
        "findutils" => oxidizr_cli_core::PackageKind::Findutils,
        "sudo" => oxidizr_cli_core::PackageKind::Sudo,
        _ => unreachable!(),
    };
    let out = oxidizr_cli_core::resolve_applets_for_use(&adapter, &root, kind, &abs);
    world.last_vec = out;
}
