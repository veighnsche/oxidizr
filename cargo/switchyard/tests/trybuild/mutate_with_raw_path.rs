// compile-fail: using raw PathBuf where SafePath is required by the public API
use switchyard::api::Switchyard;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;

fn main() {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let api = Switchyard::builder(facts, audit, Policy::default()).build();

    // Raw path, not SafePath
    let target = std::path::PathBuf::from("/tmp/whatever");

    // Should not compile: prune_backups expects &SafePath
    let _ = api.prune_backups(&target);
}
