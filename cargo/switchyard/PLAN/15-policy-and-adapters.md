# Policy Flags & Adapters Bundle (Planning Only)

Defines the policy configuration surface and an `Adapters` bundle that supplies environment-specific behavior to the library.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §3.2 Adapters`, `§2.8 Conservatism & Modes`
- Requirements: `REQ-C1`, `REQ-C2`, `REQ-L1..L4`, `REQ-RC2`
- Related docs: `impl/20-adapters.md`, `impl/45-preflight.md`, `impl/50-locking-concurrency.md`

## Policy Flags (Planning)

```rust
// Planning-only pseudocode; not actual code

struct PolicyFlags {
    allow_degraded_fs: bool,      // EXDEV fallback allowed (REQ-F2)
    strict_ownership: bool,       // enforce strict package ownership (REQ-S4)
    disable_auto_rollback: bool,  // allow skipping auto-rollback on smoke failure (REQ-H2)
    require_rescue: bool,         // require rescue profile & fallback toolset (REQ-RC2)
    override_preflight: bool,     // allow operator to override certain gates (dangerous)
    require_preservation: bool,   // require FS preservation support (owner/mode/timestamps/xattrs/acl/caps)
    backup_tag: String,           // selects the CLI/user namespace for backups (e.g., "switchyard" by default)
}
```

Defaults (current implementation):

- `ApplyMode::DryRun` by default (REQ-C1).
- `allow_degraded_fs = false` unless explicitly set.
- `strict_ownership = false`.
- `disable_auto_rollback = false`.
- `require_rescue` not yet enforced in code; planned for production.
- `override_preflight = false`.
- `require_preservation = false`.
- `backup_tag = "switchyard"` by default. CLI integrators MUST override this tag to their own namespace when multiple CLIs co-exist and share Switchyard.

Recommended production profile:

- `strict_ownership = true`
- `require_rescue = true`
- `require_preservation = true`
- `allow_degraded_fs = false` (unless cross-fs operations are common and acceptable)

### Backup Tagging

- Each mutating operation that creates a backup writes to a file named:
  `.basename.<backup_tag>.<unix_millis>.bak`
- On restore, Switchyard selects the latest matching backup for the given `backup_tag` only. This avoids cross-CLI interference.
- The tag is provided via `Policy.backup_tag` and SHOULD be set by the embedding application/CLI.

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
- Filesystem syscalls are made via `rustix` with capability-style directory handles; this crate forbids `unsafe` (`#![forbid(unsafe_code)]`).

## Stability & Semver (Planning)

- `PolicyFlags` fields are part of the public configuration surface and must follow semver stability rules.
- Additive changes require a minor bump; removals or semantic changes require a major bump.

## Tests & Evidence

- Unit: default policy instantiation and behavior toggles.
- BDD: policy-driven scenarios in `conservatism_ci.feature` and `atomic_swap.feature` (EXDEV policy).
