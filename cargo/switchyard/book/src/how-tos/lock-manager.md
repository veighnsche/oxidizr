# Configure a Lock Manager

```rust
use switchyard::adapters::FileLockManager;
use switchyard::api::ApiBuilder;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use std::path::PathBuf;

let api = ApiBuilder::new(JsonlSink::default(), JsonlSink::default(), Policy::default())
    .with_lock_manager(Box::new(FileLockManager::new(PathBuf::from("/var/lock/switchyard.lock"))))
    .build();
```

Citations:
- `cargo/switchyard/src/adapters/lock/file.rs`
- `cargo/switchyard/src/api/mod.rs`
