# Preflight Checks & Diff (Planning Only)

Defines preflight gating rules, capability probes, and the output schema alignment with `SPEC/preflight.yaml`.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §4 Preflight Diff`, `§2.3 Safety Preconditions`
- Schema: `cargo/switchyard/SPEC/preflight.yaml`
- Requirements: `REQ-S1..S5`, `REQ-RC2`, `REQ-C2`

## Goals

- Enforce safety preconditions before any mutation: SafePath, filesystem flags, ownership, preservation capabilities. (REQ-S1..S5)
- Fail-closed on critical violations unless explicitly overridden by policy. (REQ-C2)
- Verify at least one functional fallback path (rescue check). (REQ-RC2)

## Minimal Preflight v1 (Lean)

- Produce structured rows per `SPEC/preflight.yaml` with the following fields populated:
  - `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance.uid/gid/pkg`, `notes`, `preservation`, `preservation_supported`.
- Sort rows deterministically by (`path`, `action_id`).
- Enforce fail-closed on critical violations unless explicit policy override is set (loud logs). (REQ-C2)

### Apply Wiring Note (Sprint 02 scope)

- `apply()` MUST refuse to execute mutations when corresponding preflight rows indicate `policy_ok=false`, unless an explicit override flag in `Policy` is set (e.g., `override_preflight=true`).
- When refused, emit a failure fact with `error_id=E_POLICY` and an appropriate `exit_code` per tier (see Exit Codes Silver coverage in `PLAN/30-errors-and-exit-codes.md`).

## Capability Gating & Adapters

- Ownership: use `OwnershipOracle.owner_of(&SafePath)` and require root-owned, not world-writable by default. (REQ-S3)
- Strict target ownership: when `policy.strict_ownership=true`, targets must be package-owned; otherwise STOP unless overridden. (REQ-S4)
- Preservation: probe FS capabilities (owner, mode, timestamps, xattrs, ACLs, caps). If policy requires and unsupported, STOP (fail-closed). (REQ-S5)

## Clean Code Alignment

- Types encode invariants (`SafePath`), explicit dependencies via adapter traits; side effects isolated. See `docs/CLEAN_CODE.md §§3,6`.
- Guard clauses early; clear error messages with remediation for operators. See `docs/CLEAN_CODE.md §19`.

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

struct PreflightRow {
    action_id: Uuid,
    path: String,
    current_kind: String,
    planned_kind: String,
    policy_ok: bool,
    provenance: PreflightProvenance, // uid,gid,pkg
    notes: Vec<String>,
}

struct PreflightProvenance { uid: i32, gid: i32, pkg: String }

fn preflight(plan: &Plan, adapters: &Adapters, policy: &PolicyFlags) -> Result<PreflightReport, Error> {
    let mut rows: Vec<PreflightRow> = vec![];

    // Global rescue checks (record as a synthetic row or summary fact)
    let rescue = verify_rescue_profile(&*adapters.path);    // see impl/65-rescue.md
    if policy.require_rescue && (!rescue.has_rescue_symlinks || !rescue.toolset_ok) {
        return Err(Error{ kind: E_POLICY, msg: "rescue profile not satisfied" });
    }

    for a in &plan.actions {
        // Ownership and filesystem gating
        let owner = adapters.ownership.owner_of(&a.path)?;  // REQ-S3,S4
        let fs_ok = check_fs_flags(&a.path);                 // ro, noexec, immutable -> REQ-S2
        let pres_ok = probe_preservation_caps(&a.path);      // owner,mode,timestamps,xattrs,acls,caps -> REQ-S5

        let policy_ok = owner.ok && fs_ok && pres_ok || policy.override_preflight;

        if !policy_ok && !policy.allow_violation {           // REQ-C2
            return Err(Error{ kind: E_POLICY, msg: "preflight fail-closed" });
        }

        rows.push(PreflightRow{
            action_id: a.action_id,
            path: a.path.abs().to_string(),
            current_kind: a.metadata.current_kind.clone(),
            planned_kind: format!("{:?}", a.kind).to_lowercase(),
            policy_ok,
            provenance: PreflightProvenance{ uid: getuid(), gid: getgid(), pkg: owner.pkg },
            notes: vec![],
        });
    }

    ensure_rows_deterministic(&mut rows);                   // sort stable by path/action_id
    Ok(PreflightReport{ rows })
}
```

## Mapping to YAML Schema

Fields map 1:1 to `SPEC/preflight.yaml` keys:

- `action_id` → `action_id`
- `path` → `path`
- `current_kind` → `current_kind` (enum)
- `planned_kind` → `planned_kind` (enum)
- `policy_ok` → `policy_ok`
- `provenance.uid/gid/pkg` → nested map
- `notes` → `notes`
- `preservation` → `preservation` (object)
- `preservation_supported` → `preservation_supported` (bool)

## Tests & Evidence

- Unit: simulate various FS flags and ensure fail-closed behavior.
- BDD: `safety_preconditions.feature` negative scenarios; `locking_rescue.feature` rescue checks.
- Golden: deterministic ordering and values in preflight YAML.
