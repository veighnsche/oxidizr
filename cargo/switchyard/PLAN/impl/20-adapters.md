# Adapter Trait Contracts (Planning Only)

This document elaborates on adapter responsibilities and invariants.

## OwnershipOracle

- Responsibility: resolve package ownership or policy-relevant ownership of a target path.
- Contract:
  - `owner_of(path: &SafePath) -> Result<OwnershipInfo, Error>`
  - MUST NOT follow symlinks outside SafePath root.
  - Used during preflight to enforce REQ-S3/REQ-S4.

Rust-like planning types:

```rust
struct OwnershipInfo {
    uid: i32,
    gid: i32,
    pkg: String,              // package or provider name if applicable
    world_writable: bool,
}
```

Invariants & Errors:

- If `world_writable==true` or uid/gid mismatch under strict policy, return `E_OWNERSHIP` (maps to `ownership_error`).
- MUST operate on `SafePath` and reject if path escapes root (caught earlier by SafePath). See `impl/25-safepath.md`.

## LockManager

- Responsibility: serialize mutating `apply()` operations in production.
- Contract:
  - `acquire_process_lock(timeout_ms: u64) -> Result<LockGuard, Error>` with bounded wait.
  - On timeout, return `E_LOCKING` and record `lock_wait_ms` in facts.
  - Without a LockManager (dev/test), emit WARN fact; concurrent apply is UNSUPPORTED.

See also: `impl/50-locking-concurrency.md`.

## PathResolver

- Responsibility: resolve binaries/providers to `SafePath`.
- Contract:
  - `resolve(bin: &str) -> Result<SafePath, Error>`
  - Resolve within allowed roots only.

Invariants:

- Resolution MUST yield a `SafePath` rooted within allowed directories.
- MUST NOT follow symlinks that escape allowed roots.

## Attestor

- Responsibility: sign apply bundles; return signature metadata.
- Contract:
  - `sign(bundle: &[u8]) -> Result<Signature, Error>`
  - Use ed25519; include `public_key_id` and `bundle_hash` in facts.

Planning types:

```rust
struct Signature { sig_alg: String, signature: String, bundle_hash: String, public_key_id: String }
```

Notes:

- Attestation performed after successful `apply()`; see `impl/40-facts-logging.md` for fact fields.

## SmokeTestRunner

- Responsibility: execute the minimal smoke suite post-apply.
- Contract:
  - `run(plan: &Plan) -> Result<(), Error>`
  - Failure returns `E_SMOKE`; engine triggers auto-rollback unless disabled by policy.

Telemetry:

- Emit per-test facts or a summary fact recording command, args, and exit status (sanitized). See `SPEC §11` and `impl/40-facts-logging.md`.

## Common Invariants

- All adapters must be thread-safe where applicable (`Send + Sync`) to support concurrent callers (even if `apply()` serializes under the lock).
- Adapters must avoid leaking secrets; any provenance/env values returned must be maskable.

## Adapters Bundle & Policy

See `impl/15-policy-and-adapters.md` for the `Adapters` struct and `PolicyFlags` defaults that govern adapter use and gating rules.
