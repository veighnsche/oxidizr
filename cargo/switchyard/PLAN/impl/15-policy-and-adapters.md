# Policy Flags & Adapters Bundle (Planning Only)

Defines the policy configuration surface and an `Adapters` bundle that supplies environment-specific behavior to the library.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §3.2 Adapters`, `§2.8 Conservatism & Modes`
- Requirements: `REQ-C1`, `REQ-C2`, `REQ-L1..L4`, `REQ-RC2`
- Related docs: `impl/20-adapters.md`, `impl/55-preflight.md`, `impl/80-locking-concurrency.md`

## Policy Flags (Planning)

```rust
// Planning-only pseudocode; not actual code

struct PolicyFlags {
    allow_degraded_fs: bool,      // EXDEV fallback allowed (REQ-F2)
    strict_ownership: bool,       // enforce strict package ownership (REQ-S4)
    disable_auto_rollback: bool,  // allow skipping auto-rollback on smoke failure (REQ-H2)
    require_rescue: bool,         // require rescue profile & fallback toolset (REQ-RC2)
    override_preflight: bool,     // allow operator to override certain gates (dangerous)
}
```

Defaults are conservative:

- `ApplyMode::DryRun` by default (REQ-C1).
- `allow_degraded_fs = false` unless explicitly set.
- `strict_ownership = true`.
- `disable_auto_rollback = false`.
- `require_rescue = true` by default in production.
- `override_preflight = false`.

## Adapters Bundle

```rust
// Planning-only pseudocode

struct Adapters {
    ownership: Box<dyn OwnershipOracle + Send + Sync>,
    lock: Option<Box<dyn LockManager + Send + Sync>>,   // None in dev/test (REQ-L4)
    path: Box<dyn PathResolver + Send + Sync>,
    attest: Box<dyn Attestor + Send + Sync>,
    smoke: Box<dyn SmokeTestRunner + Send + Sync>,
}
```

Notes:

- All adapter traits are `Send + Sync` to support thread-safe use (SPEC §14).
- In production deployments, `lock` MUST be present (REQ-L4); omission is allowed only in dev/test where WARN facts are emitted (REQ-L2).

## Stability & Semver (Planning)

- `PolicyFlags` fields are part of the public configuration surface and must follow semver stability rules.
- Additive changes require a minor bump; removals or semantic changes require a major bump.

## Tests & Evidence

- Unit: default policy instantiation and behavior toggles.
- BDD: policy-driven scenarios in `conservatism_ci.feature` and `atomic_swap.feature` (EXDEV policy).
