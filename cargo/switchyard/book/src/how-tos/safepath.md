# Use SafePath

```rust
use switchyard::types::SafePath;

let td = tempfile::tempdir()?;
let root = td.path();
let safe = SafePath::from_rooted(root, &root.join("usr/bin/ls"))?;
assert!(safe.as_path().starts_with(root));
```

Citations:
- `src/types/safepath.rs`
