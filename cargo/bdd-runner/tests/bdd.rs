#![allow(dead_code)]

#[path = "../src/world.rs"]
mod world;
#[path = "../src/steps_common.rs"]
mod steps_common;

use world::TestWorld;
use cucumber::World as _;
use cucumber::writer::Json;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Resolve features directory relative to this crate's manifest dir.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let features_dir: String = std::env::var("BDD_FEATURES_DIR").unwrap_or_else(|_| {
        Path::new(&manifest_dir)
            .join("../switchyard/SPEC/features")
            .to_string_lossy()
            .into_owned()
    });

    // Prepare JSON output writer (useful for CI and local inspection)
    let default_out_dir = Path::new(&manifest_dir).join("features_out");
    let _ = fs::create_dir_all(&default_out_dir);
    let json_path: PathBuf = std::env::var("BDD_JSON_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_out_dir.join("report.json"));
    if let Some(parent) = json_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json_file = File::create(&json_path).expect("create report.json");

    println!("Running features from: {}", features_dir);
    println!("JSON report: {}", json_path.display());

    TestWorld::cucumber()
        .with_writer(Json::new(json_file))
        .with_default_cli()
        .run(features_dir)
        .await;
}
