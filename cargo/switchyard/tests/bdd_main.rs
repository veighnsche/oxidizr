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
    // Run all features under SPEC/features/
    let features = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SPEC/features");
    bdd_world::World::cucumber()
        .fail_on_skipped()
        .run_and_exit(features)
        .await;
}
