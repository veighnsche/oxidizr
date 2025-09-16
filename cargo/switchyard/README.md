# Switchyard (Library Crate)

[![Crates.io](https://img.shields.io/crates/v/switchyard.svg)](https://crates.io/crates/switchyard)
[![docs.rs](https://img.shields.io/docsrs/switchyard)](https://docs.rs/switchyard)
[![CI](https://github.com/veighnsche/switchyard/actions/workflows/test.yml/badge.svg)](https://github.com/veighnsche/switchyard/actions/workflows/test.yml)
[![mdBook](https://img.shields.io/badge/book-mdBook-blue)](https://veighnsche.github.io/switchyard/)
[![License: Apache-2.0/MIT](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)](./LICENSE)
[![MSRV 1.75+](https://img.shields.io/badge/MSRV-1.75%2B-informational)](./Cargo.toml)

> Operator & Integrator Guide (mdBook): <https://veighnsche.github.io/switchyard/>
>
> API docs on docs.rs: <https://docs.rs/switchyard>

Switchyard is a Rust library that provides a safe, deterministic, and auditable engine for applying system changes with:

- Atomic symlink replacement with backup/restore
- Preflight policy gating and capability probes
- Deterministic IDs and redaction for DryRun==Commit parity
- Rescue profile verification and fail-closed behavior
- Optional post-apply smoke checks with auto-rollback
- Structured facts and audit emission with provenance and exit codes

Switchyard can be used standalone or embedded by higher‑level CLIs. In some workspaces it may be included as a Git submodule.

Status: Core flows implemented with structured audit and locking; some features are intentionally minimal while SPEC v1.1 evolves. See the mdBook for coverage and roadmap.

---

## Features

- `SafePath` and TOCTOU-safe FS ops via capability-style handles (`rustix`)
- Transactional symlink replacement with backup/restore (reverse-ordered rollback)
- Cross-filesystem degraded fallback for symlink replacement (EXDEV → unlink+symlink when policy allows)
- Locking with bounded wait; timeouts emit `E_LOCKING` and include `lock_wait_ms`
- Deterministic `plan_id` and `action_id` (`UUIDv5`)
- Facts emission (JSON) with minimal provenance and optional attestation bundle
- Redaction layer: removes timing/severity and masks secrets for canon comparison
- Rescue policy: `require_rescue` verification (`BusyBox` or ≥6/10 GNU tools on PATH), fail-closed gates
- Optional smoke runner; default deterministic subset validates symlink targets resolve to sources

---

## Module Overview

- `src/types/`: core types (`Plan`, `Action`, `ApplyMode`, `SafePath`, IDs, reports)
- `src/fs/`: filesystem ops (atomic swap, backup/sidecar, restore engine, mount checks)
- `src/policy/`: policy `Policy` config, `types`, `gating` (apply parity), and `rescue` helpers
- `src/adapters/`: integration traits (`LockManager`, `OwnershipOracle`, `PathResolver`, `Attestor`, `SmokeTestRunner`) and defaults
- `src/api/`: facade (`Switchyard`) delegating to `plan`, `preflight`, `apply`, `rollback`
- `src/logging/`: `StageLogger` audit facade, `FactsEmitter`/`AuditSink`, and `redact`

## Documentation

- Operator & Integrator Guide (mdBook): <https://veighnsche.github.io/oxidizr-arch/>
  - Quickstart, Concepts, How‑Tos, and Reference live in the book.
- API docs on docs.rs: <https://docs.rs/switchyard>

### Examples

Run examples locally:

```bash
cargo run -p switchyard --example 01_dry_run
cargo run -p switchyard --example 02_commit_with_lock
cargo run -p switchyard --example 03_rollback
cargo run -p switchyard --example 04_audit_and_redaction
cargo run -p switchyard --example 05_exdev_degraded
```

## Quick Start

Build and run tests for this crate only:

```bash
cargo test
```

Add as a dependency (when used standalone):

```toml
[dependencies]
switchyard = "0.1"
```

### Minimal Example

```rust,ignore
use switchyard::api::{ApiBuilder, Switchyard};
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::{PlanInput, LinkRequest, SafePath, ApplyMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Provide your emitters
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();
    let policy = Policy::default();

    // Construct using the builder (or use Switchyard::builder(...))
    let api: Switchyard<_, _> = ApiBuilder::new(facts.clone(), audit, policy)
        .with_lock_timeout_ms(500)
        .build();

    // All mutating paths must be SafePath rooted under a directory you control
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
        eprintln!("Preflight failed: {:?}", preflight.stops);
        std::process::exit(10);
    }

    let report = api.apply(&plan, ApplyMode::Commit)?;
    println!(
        "Apply decision: {}",
        if report.errors.is_empty() { "success" } else { "failure" }
    );
    Ok(())
}
```

Alternate entrypoint:

```rust,ignore
use switchyard::api::Switchyard;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;

let facts = JsonlSink::default();
let audit = JsonlSink::default();
let policy = Policy::default();
let api = Switchyard::builder(facts, audit, policy)
    .with_lock_timeout_ms(500)
    .build();
```

---

## Core Concepts

### `SafePath` and TOCTOU Safety

- All mutating APIs accept `SafePath` to avoid path traversal (`..`) and ensure operations anchor under a known root.
- Filesystem operations use TOCTOU-safe sequences (open parent `O_DIRECTORY|O_NOFOLLOW` → `openat` → `renameat` → `fsync(parent)`).

### Preflight Policy and Preservation

- One preflight row per action with deterministic ordering and fields:
  - `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`
  - `preservation { owner, mode, timestamps, xattrs, acls, caps }`, and `preservation_supported`
- Enforced STOPs:
  - `require_preservation=true` but unsupported → STOP
  - `require_rescue=true` and rescue verification fails → STOP
- Additional gates cover mount `rw+exec`, immutability, roots/forbid paths, and ownership via `OwnershipOracle` when `strict_ownership=true`.

### Degraded Symlink Semantics (Cross-FS)

- For symlink replacement across filesystems (EXDEV), atomic rename does not apply to the symlink itself.
- When `allow_degraded_fs=true`, Switchyard uses unlink + `symlinkat` as a best-effort degraded fallback and records `degraded=true` in facts.
- When disallowed, the operation fails with `E_EXDEV` and no visible change.

### Locking and Exit Codes (Silver Tier)

- Provide a `LockManager` to serialize apply operations. On timeout, Switchyard emits `apply.attempt` failure with:
  - `error_id=E_LOCKING` and `exit_code=30`
  - `lock_wait_ms` included (redacted in canon)
- Other covered IDs include `E_POLICY`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`.

### Smoke Tests and Auto-Rollback

- Switchyard accepts an optional `SmokeTestRunner`. When provided, smoke tests run after apply (Commit mode) and on failure:
  - Emit `E_SMOKE`
  - Auto-rollback unless `policy.disable_auto_rollback=true`
- The default runner provides a deterministic subset: validate that `EnsureSymlink` targets resolve to their sources.

### Determinism and Redaction

- `plan_id` and `action_id` are `UUIDv5` for stable ordering.
- `DryRun` timestamps are zeroed (`1970-01-01T00:00:00Z`); volatile fields like durations and severity are removed; secrets are masked.
- Use the `redact_event()` helper to compare facts for `DryRun` vs `Commit` parity.

### Provenance and Attestation

- Minimal provenance is included on all facts (`ensure_provenance()` ensures presence); apply per-action can be enriched with `{uid,gid,pkg}` when an `OwnershipOracle` is available.
- Attestation bundle scaffolding exists; sensitive fields are masked for canon comparison.

---

## Testing and Goldens

Run all tests for this crate:

```bash
cargo test -p switchyard
```

Useful environment variables:

- `GOLDEN_OUT_DIR`: if set, certain tests will write canon files (e.g., `locking-timeout/canon_apply_attempt.json`).
- `SWITCHYARD_FORCE_RESCUE_OK`: testing override for rescue verification (`0`/`1`). Do not use outside tests.

Conformance and acceptance:

- Some golden fixtures live under `tests/golden/*`.
- A non-blocking CI job runs `SPEC/tools/traceability.py` and publishes coverage artifacts.

See the mdBook for testing guidance, troubleshooting, and conventions.

---

## Cargo Features

- `file-logging`: enables a file‑backed JSONL sink (`logging::facts::FileJsonlSink`) for facts/audit emission.

---

## Integration Notes

### Construction options

- Construct via `ApiBuilder` or `Switchyard::builder`:
  - `with_lock_manager(Box<dyn LockManager>)`
  - `with_ownership_oracle(Box<dyn OwnershipOracle>)`
  - `with_attestor(Box<dyn Attestor>)`
  - `with_smoke_runner(Box<dyn SmokeTestRunner>)`
  - `with_lock_timeout_ms(u64)`

- `Switchyard::new(facts, audit, policy)` remains available for compatibility and delegates to the builder internally.

### Naming conventions

- Facts/Audit sinks: `facts`, `audit`
- Policy: `policy`
- API instance: `api`
- Builder variable (if kept): `builder`

- **Emitters**: Provide your own `FactsEmitter` and `AuditSink` implementations to integrate with your logging/telemetry stack. `JsonlSink` is bundled for development/testing.
- **Adapters**: Implement or wire in `OwnershipOracle`, `LockManager`, `PathResolver`, `Attestor`, and `SmokeTestRunner` as needed.
- **Policy**: Start from `Policy::default()` or a preset (`Policy::production_preset()`, `Policy::coreutils_switch_preset()`). Key knobs are grouped:
  - `policy.rescue.{require, exec_check, min_count}`
  - `policy.apply.{exdev, override_preflight, best_effort_restore, extra_mount_checks, capture_restore_snapshot}`
  - `policy.risks.{ownership_strict, source_trust, suid_sgid, hardlinks}`
  - `policy.durability.preservation`
  - `policy.governance.{locking, smoke, allow_unlocked_commit}`
  - `policy.scope.{allow_roots, forbid_paths}`
  - `policy.backup.tag`, `policy.retention_count_limit`, `policy.retention_age_limit`

---

## Production Policy Preset (with builder)

Use the hardened preset and wire required adapters. Adjust EXDEV behavior per environment.

```rust,ignore
use switchyard::api::ApiBuilder;
use switchyard::policy::{Policy, types::ExdevPolicy};
use switchyard::adapters::{FileLockManager, DefaultSmokeRunner};
use switchyard::logging::JsonlSink;
use std::path::PathBuf;

let facts = JsonlSink::default();
let audit = JsonlSink::default();

let mut policy = Policy::production_preset();
policy.apply.exdev = ExdevPolicy::DegradedFallback;

let api = ApiBuilder::new(facts.clone(), audit, policy)
    .with_lock_manager(Box::new(FileLockManager::new(PathBuf::from("/var/lock/switchyard.lock"))))
    .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
    .build();
```

This preset ensures:

- Lock manager is required in Commit; absence fails with `E_LOCKING` (exit code 30).
- Smoke verification is required in Commit; missing or failing runner yields `E_SMOKE` and triggers auto‑rollback (unless explicitly disabled by policy).
- Rescue profile is verified (presence and X bits) before mutation.

---

## Prune Backups

Prune backup artifacts for a target under the current retention policy. Emits a `prune.result` fact.

```rust,ignore
use switchyard::types::SafePath;

let target = SafePath::from_rooted(root, &root.join("usr/bin/ls"))?;
let res = api.prune_backups(&target)?;
println!("pruned={}, retained={}", res.pruned_count, res.retained_count);
```

Knobs: `policy.retention_count_limit: Option<usize>`, `policy.retention_age_limit: Option<Duration>`.

---

## Rescue How‑To (Manual Rollback)

When the library cannot run (e.g., toolchain broken) you can manually restore using BusyBox/GNU coreutils.

Backup artifacts next to the target use:

- Backup payload: `.<name>.<tag>.<millis>.bak`
- Sidecar (JSON): `.<name>.<tag>.<millis>.bak.meta.json`

The sidecar schema (`backup_meta.v1`) includes:

- `prior_kind`: `file` | `symlink` | `none`
- `prior_dest`: original symlink destination (for `symlink`)
- `mode`: octal string for file mode (for `file`)

Steps (run in the parent directory of the target):

1) Locate the latest pair (highest `<millis>`):

```sh
ls -1a .<name>.*.bak* | sort -t '.' -k 4,4n | tail -n 2
```

2) Read `prior_kind` (and `prior_dest`/`mode`) from the sidecar:

```sh
jq -r '.prior_kind,.prior_dest,.mode' .<name>.<tag>.<millis>.bak.meta.json
```

3) Restore according to `prior_kind`:

- file

```sh
mv .<name>.<tag>.<millis>.bak <name>
[ -n "$(jq -r '.mode' .<name>.<tag>.<millis>.bak.meta.json)" ] && \
  chmod "$(jq -r '.mode' .<name>.<tag>.<millis>.bak.meta.json)" <name>
sync
```

- symlink

```sh
rm -f <name>
ln -s "$(jq -r '.prior_dest' .<name>.<tag>.<millis>.bak.meta.json)" <name>
sync
```

- none

```sh
rm -f <name>
sync
```

Notes:

- Relative `prior_dest` values are relative to the parent directory of `<name>`.
- The sidecar is retained to allow idempotent retries; do not delete it.
- Prefer `busybox jq` (or ship a minimal `jq`) for convenience; if `jq` is unavailable, you can inspect the JSON manually.

## Documentation and Change Control

- Baseline SPEC: `SPEC/SPEC.md`
- Immutable updates: `SPEC/SPEC_UPDATE_*.md`
- Checklist and compliance: `SPEC_CHECKLIST.md`
- Deep dives and topics: `DOCS/` and `INVENTORY/`
- Refactors and ongoing work: `zrefactor/`
- Gaps and tasks: `TODO.md`, `a-test-gaps.md`

When introducing normative behavior changes:

1. Add a `SPEC_UPDATE_####.md` entry
2. Update relevant docs (`DOCS/`, `INVENTORY/`) and checklist
3. Update tests/goldens accordingly

---

## License

This crate is dual-licensed under either:
- Apache License, Version 2.0 — see repository root `LICENSE`
- MIT License — see repository root `LICENSE-MIT`
at your option.

Minimum Supported Rust Version (MSRV): 1.75
