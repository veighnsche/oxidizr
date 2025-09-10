# SafePath Design (Planning Only)

This document defines the `SafePath` type, constructors, invariants, and TOCTOU-safe handle usage for mutating filesystem operations.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §3.3 SafePath`, `§2.3 Safety Preconditions`
- Requirements: `REQ-API1`, `REQ-TOCTOU1`, `REQ-S1..S5`
- Preflight schema: `cargo/switchyard/SPEC/preflight.yaml`

## Invariants

- All mutating APIs accept `SafePath` only (not `PathBuf`). (REQ-API1)
- `SafePath` is rooted; after normalization it MUST NOT escape its root. (REQ-S1)
- `..` segments are rejected; `.` segments are removed during normalization. (REQ-S1)
- Parent directory handles are opened with `O_DIRECTORY|O_NOFOLLOW`. (REQ-TOCTOU1)
- Final component operations use `openat` + `renameat` and end with `fsync(parent)` ≤50ms. (REQ-TOCTOU1, REQ-BND1)

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode; not actual code

struct SafePath {
    root: PathBuf,     // absolute, trusted root
    rel:  String,      // normalized relative path from root
}

impl SafePath {
    // Construct from a trusted root and a candidate path
    fn from_rooted(root: &Path, candidate: &Path) -> Result<SafePath, Error> {
        assert root.is_absolute();
        // 1) Join then normalize (without following symlinks)
        joined   = root.join(candidate);
        norm     = normalize_no_symlink(joined);
        // 2) Reject paths that escape the root or contain `..`
        if contains_dotdot(candidate) { return Err(Error{ kind: E_POLICY, msg: "dotdot" }); }
        if !norm.starts_with(root)    { return Err(Error{ kind: E_POLICY, msg: "escape" }); }
        // 3) Store normalized relative from root
        rel = norm.strip_prefix(root).to_string();
        Ok(SafePath{ root: root.to_path_buf(), rel })
    }

    // Open parent directory with TOCTOU-safe flags
    fn open_parent_dir_no_follow(&self) -> Result<DirHandle, Error> {
        parent_rel = parent_of(self.rel);
        parent_abs = self.root.join(parent_rel);
        // O_DIRECTORY | O_NOFOLLOW
        open_dir_no_follow(parent_abs)
    }

    // Compute absolute path lazily (for non-mutating reads only)
    fn abs(&self) -> PathBuf { self.root.join(&self.rel) }
}
```

Notes:

- `normalize_no_symlink(p)` canonicalizes `.` and `..` and collapses separators but does not resolve symlinks. TOCTOU safety comes from using directory file descriptors and `O_NOFOLLOW` when opening the parent, then `openat`/`renameat` on the final component. (REQ-TOCTOU1)
- Mutating entry points only accept `SafePath`. Any helper that accepts raw `Path` normalizes immediately to `SafePath` or refuses the operation. (REQ-API1)

## Normalization Algorithm (sketch)

```text
input: root (abs), candidate (may be abs or rel)
1. if candidate is abs: effective = candidate.strip_prefix(root) else effective = candidate
2. split effective by '/'; stack = []
3. for each seg in parts:
   - if seg == '' or seg == '.': continue
   - if seg == '..': return Err(E_POLICY)
   - else push(seg)
4. rel = join(stack, '/')
5. norm = root + '/' + rel
6. if !norm.starts_with(root): Err(E_POLICY)
7. return SafePath{root, rel}
```

## TOCTOU-Safe Usage Pattern

All mutations must follow this sequence (normative):

```text
parent = SafePath::open_parent_dir_no_follow(path)
fd     = openat(parent, final_component(path))              // do not follow if not intended
staged = create_staged_artifact(parent, final_component)
renameat(staged, parent, final_component)
fsync(parent) within 50ms                                    // REQ-BND1
```

## Error Mapping

- Invalid `SafePath` → `E_POLICY` (maps to `policy_violation` exit code). See `impl/60-errors-and-exit-codes.md`.
- Attempts that require following symlinks on parent are rejected by design.

## SPEC/Test Mapping

- `safety_preconditions.feature`: rejects escaping paths and unsupported FS states. (REQ-S1..S2)
- `api_toctou.feature`: SafePath-only APIs and TOCTOU-safe sequence. (REQ-API1, REQ-TOCTOU1)
- Unit tests: `types/safepath.rs` normalization, rejection of `..`, root-escape checks.
