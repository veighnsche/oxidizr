# Switchyard (Library Crate)

Switchyard is a Rust library that provides a safe, deterministic, and auditable engine for applying system changes with:

- Atomic symlink replacement with backup/restore
- Preflight policy gating and capability probes
- Deterministic IDs and redaction for DryRun==Commit parity
- Rescue profile verification and fail-closed behavior
- Optional post-apply smoke checks with auto-rollback
- Structured facts and audit emission with provenance and exit codes

This crate lives inside the `oxidizr-arch` monorepo and is designed to be embedded by higher-level CLIs.

Status: Silver-tier coverage for exit codes and core flows; some features are stubbed or intentionally minimal to ensure determinism. See `PLAN/90-implementation-tiers.md`.

---

## Features

- SafePath and TOCTOU-safe FS ops via capability-style handles (`rustix`)
- Transactional symlink replacement with backup/restore (reverse-ordered rollback)
- Cross-filesystem degraded fallback for symlink replacement (EXDEV → unlink+symlink when policy allows)
- Locking with bounded wait; timeouts emit `E_LOCKING` and include `lock_wait_ms`
- Deterministic `plan_id` and `action_id` (UUIDv5)
- Facts emission (JSON) with minimal provenance and optional attestation bundle
- Redaction layer: removes timing/severity and masks secrets for canon comparison
- Rescue policy: `require_rescue` verification (BusyBox or ≥6/10 GNU tools on PATH), fail-closed gates
- Optional smoke runner; default deterministic subset validates symlink targets resolve to sources

---

## Quick Start

Build and run tests for this crate only:

```bash
cargo test -p switchyard
```

Add as a dependency (when used standalone):

```toml
[dependencies]
switchyard = { path = "./cargo/switchyard" }
```

### Minimal Example

```rust
use switchyard::Switchyard;
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::types::plan::{PlanInput, LinkRequest};
use switchyard::types::safepath::SafePath;
use switchyard::types::ApplyMode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Facts sink and audit sink; replace with your own emitters
    let facts = JsonlSink::default();
    let audit = JsonlSink::default();

    // Policy: require rescue tools and allow degraded cross-FS symlink fallback
    let mut policy = Policy::default();
    policy.require_rescue = true;
    policy.allow_degraded_fs = true;

    let api = Switchyard::new(facts.clone(), audit, policy)
        .with_lock_timeout_ms(500);

    // All mutating paths must be SafePath rooted under a directory you control
    let td = tempfile::tempdir()?;
    let root = td.path();
    std::fs::create_dir_all(root.join("usr/bin"))?;
    std::fs::write(root.join("usr/bin/ls"), b"old")?;
    std::fs::write(root.join("bin/new"), b"new")?;

    let source = SafePath::from_rooted(root, &root.join("bin/new"))?;
    let target = SafePath::from_rooted(root, &root.join("usr/bin/ls"))?;

    let plan = api.plan(PlanInput { link: vec![LinkRequest { source, target }], restore: vec![] });

    // Preflight applies policy gating (mount rw/exec, ownership, preservation, rescue, etc.)
    let preflight = api.preflight(&plan)?;
    if !preflight.ok {
        eprintln!("Preflight failed: {:?}", preflight.stops);
        std::process::exit(10);
    }

    // Apply in Commit mode; in DryRun mode timestamps are zeroed for determinism
    let report = api.apply(&plan, ApplyMode::Commit)?;
    println!("Apply decision: {}", report.decision);
    Ok(())
}
```

---

## Core Concepts

### SafePath and TOCTOU Safety

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

- `plan_id` and `action_id` are UUIDv5 for stable ordering.
- DryRun timestamps are zeroed (`1970-01-01T00:00:00Z`); volatile fields like durations and severity are removed; secrets are masked.
- Use the `redact_event()` helper to compare facts for DryRun vs Commit parity.

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

See `docs/testing/TESTING_POLICY.md` for project-wide rules (zero SKIPs, harness constraints, and CI behavior).

---

## Integration Notes

- Emitters: Provide your own `FactsEmitter` and `AuditSink` implementations to integrate with your logging/telemetry stack. `JsonlSink` is bundled for development/testing.
- Adapters: Implement or wire in `OwnershipOracle`, `LockManager`, `PathResolver`, `Attestor`, and `SmokeTestRunner` as needed.
- Policy: Start from `Policy::default()` and enable gates:
  - `require_rescue`, `require_preservation`, `allow_degraded_fs`, `strict_ownership`, `force_untrusted_source`, `disable_auto_rollback`.

---

## Production Policy Preset

For production, enable the following policy toggles to satisfy SPEC v1.1 requirements (L4, H3, RC1) and harden rescue checks:

```rust
use switchyard::policy::Policy;

let mut policy = Policy::default();
// Rescue required, with executability checks (BusyBox or ≥6/10 GNU tools must be present and executable)
policy.require_rescue = true;
policy.rescue_exec_check = true;
// Locking and health gates in Commit
policy.require_lock_manager = true;
policy.require_smoke_in_commit = true;
// Optional depending on environment
policy.allow_degraded_fs = true; // allow EXDEV fallback with telemetry
// Keep fail-closed behavior (override_preflight=false) unless explicitly overridden for dev

let api = switchyard::Switchyard::new(facts.clone(), audit, policy)
    .with_lock_manager(Box::new(your_lock_manager))
    .with_smoke_runner(Box::new(your_smoke_runner))
    .with_ownership_oracle(Box::new(your_ownership_oracle));
```

These toggles ensure:

- Lock manager is mandatory in Commit; absence fails with `E_LOCKING` (exit code 30).
- Smoke verification is mandatory in Commit; missing or failing runner yields `E_SMOKE` with auto‑rollback.
- Rescue profile is verified (presence and X bits) before mutate when required.

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

3) Restore according to prior_kind:

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
- Immutable updates: `SPEC/SPEC_UPDATE_*.md` (see `SPEC_UPDATE_0002.md` for rescue and degraded symlink clarifications)
- ADRs: `PLAN/adr/*.md` (see `ADR-0015-exit-codes-silver-and-ci-gates.md`)
- Implementation tiers and roadmap: `PLAN/90-implementation-tiers.md`

When introducing normative behavior changes:

1. Add a `SPEC_UPDATE_####.md` entry
2. Add or update an ADR
3. Update PLAN notes and tests/goldens accordingly

---

## License

GPL-3.0-or-later. See the repository root `LICENSE` file.
