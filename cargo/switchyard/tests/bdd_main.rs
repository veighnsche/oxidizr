#![cfg_attr(
    not(feature = "bdd"),
    allow(unused_imports, unused_variables, dead_code)
)]

#[path = "bdd_support/mod.rs"]
mod bdd_support;
#[path = "bdd_world/mod.rs"]
mod bdd_world;
mod steps;

use cucumber::World as _; // bring trait into scope for World::cucumber()

#[cfg(not(feature = "bdd"))]
fn main() {}

#[cfg(feature = "bdd")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Run all features under SPEC/features/ unless a specific feature path is provided via env.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let features_env = std::env::var("SWITCHYARD_BDD_FEATURE_PATH").ok();
    let features = if let Some(p) = features_env {
        let pb = std::path::PathBuf::from(p);
        if pb.is_absolute() {
            pb
        } else {
            root.join(pb)
        }
    } else {
        root.join("SPEC/features")
    };
    bdd_world::World::cucumber()
        .fail_on_skipped()
        .run_and_exit(features)
        .await;
}
