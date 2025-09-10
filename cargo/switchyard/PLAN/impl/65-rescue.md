# Rescue Profile & Fallback Toolset (Planning Only)

Defines the rescue profile guarantees and verification of fallback toolsets available on PATH.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §2.6 Rescue`
- Requirements: `REQ-RC1..RC3`
- Also see: `impl/00-structure.md` (adds `rescue.rs`), `impl/30-pseudocode.md` (preflight integration)

## Goals

- A rescue profile (backup symlink set) always remains available. (REQ-RC1)
- Preflight verifies at least one functional fallback path. (REQ-RC2)
- At least one fallback binary set (GNU or BusyBox) remains executable and present on PATH. (REQ-RC3)

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

struct RescueProfileCheckResult {
    has_rescue_symlinks: bool,
    fallback_toolset: Option<String>,  // "gnu" | "busybox"
    toolset_ok: bool,
    notes: Vec<String>,
}

fn verify_rescue_profile(path: &PathResolver) -> RescueProfileCheckResult {
    let mut res = RescueProfileCheckResult{ has_rescue_symlinks: false, fallback_toolset: None, toolset_ok: false, notes: vec![] };

    // 1) Check rescue symlink set exists and resolves (policy-defined location)
    res.has_rescue_symlinks = probe_rescue_symlink_set();

    // 2) Verify fallback toolset presence on PATH
    let gnu_ok = ["ls","cp","mv","rm","ln","stat","readlink","sha256sum","sort","date"].iter().all(|b| path.resolve(b).is_ok());

    if gnu_ok { res.fallback_toolset = Some("gnu".into()); res.toolset_ok = true; }
    else if busybox_present(path) { res.fallback_toolset = Some("busybox".into()); res.toolset_ok = true; }

    if !res.toolset_ok { res.notes.push("No fallback toolset found on PATH".into()); }
    res
}
```

## Preflight Integration

- `preflight()` MUST include a row summarizing rescue checks.
- If rescue checks fail and policy requires rescue guarantees, preflight MUST fail-closed (E_POLICY) unless explicitly overridden.

## Facts & Provenance

- Facts SHOULD record which fallback toolset was verified and whether rescue symlinks exist.
- Provenance MAY include PATH snapshot or sanitized indicators to support debugging.

## Tests & Evidence

- BDD: `locking_rescue.feature` scenarios validate presence of rescue toolset.
- Unit: mock `PathResolver` to simulate GNU/BusyBox presence/absence.
