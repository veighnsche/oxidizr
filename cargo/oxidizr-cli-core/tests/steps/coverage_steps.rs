use cucumber::when;

use crate::world::World;

#[when(regex = r#"^I call coverage_check with distro \"([^\"]*)\" and repl \"([^\"]*)\"$"#)]
pub async fn call_coverage_check(world: &mut World, distro: String, repl: String) {
    let parse = |s: &str| -> Vec<String> {
        s.split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    };
    let d = parse(&distro);
    let r = parse(&repl);
    world.last_result = Some(oxidizr_cli_core::coverage_check(&d, &r));
}

#[when(regex = r#"^I call coverage_preflight for package \"(coreutils|findutils|sudo)\"$"#)]
pub async fn call_coverage_preflight(world: &mut World, pkg: String) {
    let adapter = world.make_adapter();
    let root = world.ensure_root().to_path_buf();
    let src_rel = world.artifact_path.as_ref().cloned().expect("artifact");
    let abs = world.under_root(src_rel);
    let kind = match pkg.as_str() {
        "coreutils" => oxidizr_cli_core::PackageKind::Coreutils,
        "findutils" => oxidizr_cli_core::PackageKind::Findutils,
        "sudo" => oxidizr_cli_core::PackageKind::Sudo,
        _ => unreachable!(),
    };
    world.last_result = Some(oxidizr_cli_core::coverage_preflight(
        &adapter, &root, kind, &abs,
    ));
}
