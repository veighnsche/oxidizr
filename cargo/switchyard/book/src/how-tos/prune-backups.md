# Prune Backups

Prune backup artifacts for a specific target under the current retention policy. A `prune.result` fact is emitted with counts and knobs used.

```rust
use switchyard::api::ApiBuilder;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::SafePath;

let api = ApiBuilder::new(JsonlSink::default(), JsonlSink::default(), Policy::default()).build();
let td = tempfile::tempdir()?;
let root = td.path();
let target = SafePath::from_rooted(root, &root.join("usr/bin/ls"))?;
// Configure retention knobs
// api.policy.retention_count_limit and retention_age_limit are set at construction in Policy
let res = api.prune_backups(&target)?;
println!("pruned={}, retained={}", res.pruned_count, res.retained_count);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Citations:
- `src/api/mod.rs`
- `src/fs/backup/prune.rs`
- `src/policy/config.rs`
