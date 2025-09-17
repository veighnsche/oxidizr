#[cfg(not(feature = "bdd"))]
fn main() {}

#[cfg(feature = "bdd")]
#[path = "steps/mod.rs"]
mod steps;
#[cfg(feature = "bdd")]
#[path = "world.rs"]
mod world;

#[cfg(feature = "bdd")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    use cucumber::World as _;
    use std::path::PathBuf;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let features = root.join("tests/features");

    world::World::cucumber()
        .fail_on_skipped()
        .run_and_exit(features)
        .await;
}
