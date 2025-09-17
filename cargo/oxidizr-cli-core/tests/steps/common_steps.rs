use cucumber::given;

use crate::world::World;

#[given(regex = r"^a staging root at .*$")]
pub async fn staging_root_at(world: &mut World) {
    world.ensure_dir("/usr/bin");
}

#[given(regex = r#"^a replacement artifact lists applets \"([^\"]+)\" at `(/.+)`$"#)]
pub async fn artifact_lists_applets_at(world: &mut World, applets: String, path: String) {
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--list\" ] || [ \"$1\" = \"--help\" ]; then\n  echo {}\n  exit 0\nfi\nexit 0\n",
        applets
    );
    world.write_file_exec(&path, &script);
    world.artifact_path = Some(std::path::PathBuf::from(path));
}

#[given(
    regex = r#"^the distro commands for package \"(coreutils|findutils|sudo)\" are \"([^\"]*)\"$"#
)]
pub async fn set_distro(world: &mut World, pkg: String, names: String) {
    let kind = match pkg.as_str() {
        "coreutils" => oxidizr_cli_core::PackageKind::Coreutils,
        "findutils" => oxidizr_cli_core::PackageKind::Findutils,
        "sudo" => oxidizr_cli_core::PackageKind::Sudo,
        _ => unreachable!(),
    };
    let list: Vec<String> = names
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    world.set_distro(kind, list);
}
