# Quickstart (Debian/Ubuntu)

This chapter shows the minimal steps to create a plan, preflight it, and apply in Commit mode with locking.

```rust
use switchyard::api::{ApiBuilder, Switchyard};
use switchyard::logging::JsonlSink;
use switchyard::policy::{Policy, types::ExdevPolicy};
use switchyard::types::{PlanInput, LinkRequest, SafePath, ApplyMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();

    let mut policy = Policy::production_preset();
    policy.apply.exdev = ExdevPolicy::DegradedFallback;

    let api: Switchyard<_, _> = ApiBuilder::new(facts, audit, policy)
        .with_lock_timeout_ms(500)
        .build();

    let td = tempfile::tempdir()?;
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin"))?;
    std::fs::write(root.join("usr/bin/ls"), b"old")?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::write(root.join("bin/new"), b"new")?;

    let source = SafePath::from_rooted(root, &root.join("bin/new"))?;
    let target = SafePath::from_rooted(root, &root.join("usr/bin/ls"))?;

    let plan = api.plan(PlanInput { link: vec![LinkRequest { source, target }], restore: vec![] });

    let preflight = api.preflight(&plan)?;
    if !preflight.ok {
        eprintln!("Preflight stops: {:?}", preflight.stops);
        std::process::exit(10);
    }

    let report = api.apply(&plan, ApplyMode::Commit)?;
    if !report.errors.is_empty() {
        eprintln!("Apply errors: {:?}", report.errors);
    }
    Ok(())
}
```

Citations:
- `cargo/switchyard/src/api/mod.rs`
- `cargo/switchyard/src/policy/config.rs`
- `cargo/switchyard/src/adapters/lock/file.rs`
