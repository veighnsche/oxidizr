# Facts & Logging (Planning Only)

Defines how facts are built, redacted, ordered, and emitted as JSONL records that conform to `SPEC/audit_event.schema.json`.

References:

- SPEC: `cargo/switchyard/SPEC/SPEC.md §2.4 Observability & Audit`, `§5 Audit Facts`, `§13 Schema Versioning`
- Schema: `cargo/switchyard/SPEC/audit_event.schema.json`
- Requirements: `REQ-O1..O7`, `REQ-VERS1`, `REQ-D2`

## Goals

- Every step emits a structured fact (JSON). (REQ-O1)
- Dry-run facts are byte-identical to real-run facts after redactions. (REQ-O2, REQ-D2)
- Facts carry `schema_version = 1`. (REQ-VERS1)
- Include attestation, provenance, preservation, exit code, timings. (REQ-O3..O7)

## Rust-like Pseudocode (non-compilable)

```rust
// Planning-only pseudocode

struct Fact {
    ts: String,                     // RFC3339; zeroed or redacted in dry-run
    plan_id: Uuid,
    schema_version: u8,             // == 1
    stage: Stage,                   // plan | preflight | apply.attempt | apply.result | rollback
    decision: Decision,             // success | failure | warn
    severity: Severity,             // info | warn | error
    degraded: Option<bool>,

    // Referents
    action_id: Option<Uuid>,
    path: Option<String>,           // string path; SafePath.abs() stringified
    current_kind: Option<String>,
    planned_kind: Option<String>,

    // Integrity
    hash_alg: Option<String>,       // "sha256"
    before_hash: Option<String>,
    after_hash: Option<String>,

    // Attestation
    attestation: Option<Attestation>,

    // Provenance
    provenance: Option<Provenance>,

    // Preservation capabilities and policy
    preservation: Option<Preservation>,
    preservation_supported: Option<bool>,

    // Execution
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
    lock_wait_ms: Option<i64>,
}

struct Attestation { sig_alg: String, signature: String, bundle_hash: String, public_key_id: String }
struct Provenance { origin: String, helper: String, uid: i32, gid: i32, pkg: String, env_sanitized: bool }
struct Preservation { owner: bool, mode: bool, timestamps: bool, xattrs: bool, acls: bool, caps: bool }

enum Stage { Plan, Preflight, ApplyAttempt, ApplyResult, Rollback }
enum Decision { Success, Failure, Warn }
enum Severity { Info, Warn, Error }
```

## Emission Flow

```rust
fn emit_fact(mut f: Fact) {
    f.schema_version = 1;                      // REQ-VERS1

    // Redaction & normalization (REQ-D2, REQ-O6)
    redacted = redact(f);                      // zero timestamps in dry-run; mask secrets
    ordered  = stable_order(redacted);         // deterministic field ordering

    // Serialize and append to JSONL
    line = serialize_json(ordered);
    append_jsonl(facts_sink(), line);
}

fn redact(mut f: Fact) -> Fact {
    if policy.mode == DryRun {
        f.ts = "1970-01-01T00:00:00Z";        // or 0 delta; exact policy in ADR
    }
    // Mask secrets potentially appearing in provenance/helper/env
    f.provenance = f.provenance.map(|p| mask_provenance(p));
    return f;
}

fn stable_order(f: Fact) -> Fact {
    // Ensure map/object fields are emitted in a stable key order.
    // Implementation detail left to serializer config; planning-only note.
    f
}
```

## Where Facts Are Emitted

- `plan()` — one summary fact per action (stage=`plan`).
- `preflight()` — one row per action (stage=`preflight`).
- `apply()` — for each action: `apply.attempt` then `apply.result` (success/failure).
- `apply()` — one summary fact with attestation after successful completion.
- `rollback()` — per-step facts during rollback with partial restoration info on failure. (REQ-R5)

## Attestation & Bundles

- After a successful `apply()`, compute a bundle hash of the facts and plan summary and request a signature from `Attestor`. (REQ-O4)
- Include `{sig_alg=ed25519, signature, bundle_hash, public_key_id}` in the final `apply.result` summary fact.

## Provenance & Masking

- Provenance must include origin, helper, uid, gid, pkg, and a boolean `env_sanitized`. (REQ-O7)
- Mask potentially sensitive values before emission. (REQ-O6)

## Determinism

- The combination of stable ordering and redaction MUST ensure byte-identical dry-run vs real-run facts. (REQ-D2)

## CI Hooks

- Validate emitted facts against `SPEC/audit_event.schema.json`.
- Compare facts JSONL to golden fixtures byte-for-byte.
