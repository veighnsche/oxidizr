# Switchyard Specification (RFC-2119)

## 0. Domain & Purpose

Switchyard is a **Rust library crate** (not a CLI) that provides **safe, atomic, reversible filesystem swaps** (typically for binaries).
It is **OS-agnostic**: it only manipulates filesystem paths and relies on adapters for policy and environment-specific logic.

---

## 1. Main Guarantees

- Atomic, crash-safe swaps with no user-visible broken/missing path.
- Complete, idempotent rollback; automatic reverse-order rollback on any apply failure; partial restoration state reported if rollback fails.
- SafePath everywhere for mutations; TOCTOU-safe sequence.
- Deterministic plans and outputs: UUIDv5 IDs over normalized inputs; dry-run facts byte-identical after timestamp redactions (see §5 Audit Facts).
- Locking required in production with bounded wait → E_LOCKING; facts include lock_wait_ms.
- Lock fairness telemetry: apply.attempt facts include lock_attempts (approximate retry count inferred from lock_wait_ms and poll interval).
- Rescue profile always available; at least one fallback toolset (GNU/BusyBox) present on PATH.
- Auditable, tamper-evident facts (schema v2): SHA-256 before/after hashes; signed attestation bundles; secret masking; complete provenance.
- Apply summary includes a perf object aggregating timing signals (hash_ms, backup_ms, swap_ms) for observability (see §5 Audit Facts).
- Backup durability is enforced: backup payloads and sidecars are synced to disk, and the parent directory is fsynced after artifact creation/rename to survive crashes.
- Conservative by default: dry-run mode; fail-closed on critical compatibility differences unless policy overrides.
- Health verification required: minimal smoke suite runs post-apply; failure triggers auto-rollback (unless explicitly disabled).
- Cross-filesystem safety: EXDEV fallback with degraded-mode policy and telemetry. When degraded fallback is disallowed and EXDEV occurs, apply fails with `exdev_fallback_failed` and facts include `degraded=false` with a stable reason marker.

## 2. Normative Requirements

### 2.1 Atomicity

- REQ-A1: A swap **MUST** be atomic with respect to crashes.
- REQ-A2: A user-visible broken or missing path **MUST NOT** exist at any time.
- REQ-A3: All-or-nothing per plan: either all actions succeed, or no visible changes remain.

### 2.2 Rollback

- REQ-R1: Every change **MUST** be reversible by rollback.
- REQ-R2: Rollback **MUST** restore the exact prior link/file topology.
- REQ-R3: Rollback **MUST** be idempotent.
- REQ-R4: On any apply failure, already-applied actions **MUST** be rolled back in reverse plan order automatically.
- REQ-R5: If rollback itself fails, facts **MUST** capture partial restoration state and guidance for operator recovery.

### 2.3 Safety Preconditions

- REQ-S1: Paths **MUST NOT** contain `..` or escape allowed roots.
- REQ-S2: Operations **MUST** fail if the target filesystem is read-only, `noexec`, or immutable.
- REQ-S3: Source files **MUST** be root-owned and not world-writable, unless policy override.
- REQ-S4: If `strict_ownership=true`, targets **MUST** be package-owned (via adapter).
- REQ-S5: Preservation gating: filesystem capabilities for ownership, mode, timestamps, xattrs/ACLs/caps **MUST** be probed during preflight; if required by policy but unsupported, preflight **MUST** STOP (fail-closed) unless explicitly overridden.
- REQ-S6: Backup sidecars SHOULD record a payload_hash when a backup payload exists (sidecar v2). On restore, if policy requires sidecar integrity and a payload_hash is present, the engine MUST verify the backup payload hash and fail restore on mismatch.


### 2.4 Observability & Audit

- REQ-O1: Every step **MUST** emit a structured fact (JSON).
- REQ-O2: Dry-run facts **MUST** be byte-identical to real-run facts.
- REQ-O3: Facts schema **MUST** be versioned and stable.
- REQ-O4: Attestations (signatures, SBOM-lite fragments) **MUST** be generated and signed for each apply bundle.
- REQ-O5: For every mutated file, `before_hash` and `after_hash` **MUST** be recorded using SHA-256 (`hash_alg=sha256`).
- REQ-O6: Secret masking **MUST** be enforced across all audit sinks; no free-form secrets are permitted.
- REQ-O7: Provenance **MUST** include origin (repo/AUR/manual), helper, uid/gid, and confirmation of environment sanitization. Policy gating for external sources is out of scope of this core spec and MAY be enforced by adapters.
- REQ-O8: Summary events (preflight, apply.result, and rollback.summary) **MUST** include a summary_error_ids array on failures that lists stable error identifiers, with a top-level classification (e.g., E_POLICY) and any specific causes (e.g., E_LOCKING, E_SMOKE) when determinable.

### 2.5 Locking

- REQ-L1: Only one `apply()` **MUST** mutate at a time.
- REQ-L2: If no lock manager, concurrent `apply()` is **UNSUPPORTED** (dev/test only) and a WARN fact **MUST** be emitted.
- REQ-L3: Lock acquisition **MUST** use a bounded wait with timeout → `E_LOCKING`, and facts **MUST** record `lock_wait_ms`.
- REQ-L5: For observability, apply.attempt facts **SHOULD** include an approximate `lock_attempts` count.
- REQ-L4: In production deployments, a `LockManager` **MUST** be present. Omission is permitted only in development/testing.

### 2.6 Rescue

- REQ-RC1: A rescue profile (backup symlink set) **MUST** always remain available.
- REQ-RC2: Preflight **MUST** verify at least one functional fallback path.
- REQ-RC3: At least one fallback binary set (GNU or BusyBox) **MUST** remain executable and present on `PATH` for recovery.

Policy guidance: Deployments MAY set a `require_rescue` policy knob. When `require_rescue=true`, preflight verifies that a functional fallback exists; failing this check causes preflight to STOP (fail‑closed) unless explicitly overridden. Minimal verification guidance: PASS when `busybox` is present on PATH; otherwise require a deterministic subset of GNU tools (cp, mv, rm, ln, stat, readlink, sha256sum, sort, date, ls) with a threshold (e.g., ≥6/10) without executing external commands during preflight.

### 2.7 Determinism

- REQ-D1: `plan_id` and `action_id` are UUIDv5 values derived from the normalized plan input using a project-defined, stable namespace (see `src/constants.rs::NS_TAG`).
- REQ-D2: Dry-run redactions are pinned: timestamps are zeroed (or expressed as monotonic deltas). Dry-run facts **MUST** be byte-identical to real-run facts after redaction.

### 2.8 Conservatism & Modes

- REQ-C1: Dry-run is the default mode; side effects require explicit operator approval (e.g., `--assume-yes`).
- REQ-C2: Critical compatibility violations (e.g., ownership, policy, filesystem capability) **MUST** fail closed unless explicitly overridden by policy.

### 2.9 Health Verification

- REQ-H1: A minimal post-apply smoke suite **MUST** run (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date) with specified arguments.
- REQ-H2: Any mismatch or non-zero exit **MUST** trigger automatic rollback unless explicitly disabled by policy.
- REQ-H3: Health verification is part of commit; it is not optional.

### 2.10 Filesystems & Degraded Mode

- REQ-F1: When staging and target parents reside on different filesystems (EXDEV), the engine **MUST** use a safe copy + fsync + rename fallback that preserves atomic visibility.
- REQ-F2: If degraded fallback is used and policy allows, facts **MUST** record `degraded=true`; if policy disallows degraded operation, apply **MUST** fail.
- REQ-F3: Supported/tested filesystems include ext4, xfs, btrfs, and tmpfs; semantics are verified in acceptance tests.

---

## 3. Public Interfaces

### 3.1 Public API

```rust
fn plan(input: PlanInput) -> Plan;
fn preflight(plan: &Plan) -> PreflightReport;
fn apply(plan: &Plan, mode: ApplyMode) -> ApplyReport;
fn plan_rollback_of(report: &ApplyReport) -> Plan;
fn prune_backups(target: &SafePath) -> PruneResult; // applies retention policy (count/age) and emits prune.result
```

Adapters are configured on the host `Switchyard` object via builder methods (e.g., `with_lock_manager`, `with_ownership_oracle`) rather than passed per-call to `apply()`. This ensures deterministic behavior and clearer lifecycle management of adapters.

All path-carrying fields within `PlanInput` and `Plan` **MUST** be typed as `SafePath`. Mutating entry points do not accept `PathBuf`.

### 3.2 Adapters

```rust
trait OwnershipOracle { fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>; }
trait LockManager { fn acquire_process_lock(&self, timeout_ms: u64) -> Result<LockGuard>; }
trait PathResolver { fn resolve(&self, bin: &str) -> Result<SafePath>; }
trait Attestor { fn sign(&self, bundle: &[u8]) -> Result<Signature>; }
trait SmokeTestRunner { fn run(&self, plan: &Plan) -> Result<(), SmokeFailure>; }
```

### 3.3 SafePath

- Constructed via `SafePath::from_rooted(root, candidate)`.
- Rejects `..` after normalization.
- Opened with `O_NOFOLLOW` parent dir handles (TOCTOU defense).
- All mutating public APIs **MUST** take `SafePath` (not `PathBuf`). Any non-mutating APIs that accept raw paths **MUST** immediately normalize to `SafePath`.
- TOCTOU-safe syscall sequence is normative for every mutation: open parent with `O_DIRECTORY|O_NOFOLLOW` → `openat` on final component → `renameat` → `fsync(parent)`.

### 3.4 Retention (Prune Backups)

- The library provides `Switchyard::prune_backups(&SafePath) -> PruneResult` to prune backup artifacts under policy.
- Policy knobs: `retention_count_limit: Option<usize>` and `retention_age_limit: Option<Duration>`.
- Selection rules: backups are sorted by timestamp (newest first); the newest backup is never deleted. Count and age filters are applied to older backups. Deletions remove payload+sidecar pairs and fsync the parent directory.
- A `prune.result` event is emitted with fields including `target_path`, `policy_used`, `pruned_count`, and `retained_count`.

---

## 4. Preflight Diff (Normative Schema)

**YAML schema (`/SPEC/preflight.yaml`):**

```yaml
type: sequence
items:
  type: map
  mapping:
    action_id: { type: str }
    path: { type: str }
    current_kind: { enum: [missing, file, dir, symlink] }
    planned_kind: { enum: [symlink, restore_from_backup, skip] }
    policy_ok: { type: bool }
    provenance:
      type: map
      mapping:
        uid: { type: int }
        gid: { type: int }
        pkg: { type: str }
    notes: { type: seq, sequence: { type: str } }
    preservation:
      type: map
      mapping:
        owner: { type: bool }
        mode: { type: bool }
        timestamps: { type: bool }
        xattrs: { type: bool }
        acls: { type: bool }
        caps: { type: bool }
    preservation_supported: { type: bool }
```

 Dry-run output must match real-run preflight rows byte-for-byte.

 Rows are deterministically ordered by (`path`, `action_id`) to ensure stable diffs and goldens across environments.

---

## 5. Audit Facts (JSON Schema v2)

The normative JSON Schema for audit facts is version 2 and lives at `/SPEC/audit_event.v2.schema.json`.

Envelope (always present; best‑effort for optional fields):

- `schema_version=2`, `ts`, `plan_id`, optional `action_id` for per‑action events
- `event_id`, `run_id`, `switchyard_version`, `redaction`, `seq`, `dry_run`
- Optional host/process/actor/build metadata objects when available

Stages and required fields:

- `plan` — per‑action events include `path`
- `preflight` — per‑action events include `path`, `current_kind`, `planned_kind`
- `preflight.summary` — stage aggregate; no per‑action required fields
- `apply.attempt` — includes locking fields (`lock_backend`, `lock_attempts`, `lock_wait_ms` when known); may omit `path`
- `apply.result` — per‑action results; include before/after hashes (`hash_alg=sha256`, `before_hash`, `after_hash`) when mutated
- `rollback`, `rollback.summary` — restore and summary semantics unchanged; summaries may include `summary_error_ids`
- `prune.result` — requires `path`, `pruned_count`, `retained_count`

Selected optional fields (when measured/available): `degraded`, `degraded_reason`, `perf{hash_ms,backup_ms,swap_ms,io_bytes_*}`, `provenance`, `preservation{...}`, `backup_durable`, `sidecar_integrity_verified`, `error{kind,errno,message,remediation}`.

---

## 6. Error Taxonomy & Exit Codes

**TOML (`/SPEC/error_codes.toml`):**

```toml
[exit_codes]
success = 0
generic_error = 1
policy_violation = 10
ownership_error = 20
lock_timeout = 30
atomic_swap_failed = 40
exdev_fallback_failed = 50
backup_missing = 60
restore_failed = 70
smoke_test_failed = 80
```

Errors are emitted in facts as stable identifiers (e.g. `E_POLICY`, `E_LOCKING`). Preflight summary emits `error_id=E_POLICY` and `exit_code=10` when any STOP conditions are present.

On summary failures, `summary_error_ids` provides a best-effort chain of identifiers for analytics and routing. For example, a smoke failure may populate `["E_SMOKE","E_POLICY"]` while a locking timeout would surface `["E_LOCKING","E_POLICY"]`. Ownership-related checks may co‑emit `E_OWNERSHIP` alongside the top‑level classification to aid routing.

---

## 7. Formal Safety Model

### Invariants (TLA+ sketch)

- **NoBrokenLinks:** No symlink points to a missing file.
- **Rollbackable:** After any sequence of steps, `CanRollback` holds.

### Properties

- **AtomicReplace:** property-based tests ensure no intermediate “missing” visible.
- **IdempotentRollback:** applying rollback twice yields same FS state.

---

## 8. Acceptance Tests (BDD-style)

```gherkin
Feature: Atomic swap
  Scenario: Enable and rollback
    Given /usr/bin/ls is a symlink to providerA/ls
    When I plan a swap to providerB
    And I apply the plan
    Then /usr/bin/ls resolves to providerB/ls atomically
    And rollback restores providerA/ls

  Scenario: Cross-filesystem EXDEV fallback
    Given the target and staging directories reside on different filesystems
    When I apply a plan that replaces /usr/bin/cp
    Then the operation handles EXDEV by copy+sync+rename into place atomically
    And facts record degraded=true when policy allow_degraded_fs is enabled

  Scenario: Automatic rollback on mid-plan failure
    Given a plan with three actions A, B, C
    And action B will fail during apply
    When I apply the plan
    Then the engine automatically rolls back A in reverse order
    And facts clearly indicate partial restoration state if any rollback step fails
```

---

## 9. Operational Bounds

- `fsync(parent)` MUST occur ≤50ms after rename.
- Plan size default max = 1000 actions (configurable).

---

## 10. Filesystems & Degraded Mode

Clarification (symlink replacement): For cross‑filesystem symlink replacement where atomic copy+rename on the link itself is not possible, the engine uses an unlink+`symlinkat` best‑effort degraded fallback when `allow_degraded_fs=true`. When `allow_degraded_fs=false`, the operation fails with `exdev_fallback_failed` and performs no visible change. Facts record `degraded=true|false` and a stable `degraded_reason` (e.g., "exdev_fallback").

Supported filesystems (tested):

- ext4 — native rename semantics
- xfs — native rename semantics
- btrfs — native rename semantics
- tmpfs — native rename semantics

EXDEV path is explicitly exercised: when staging and target parents are on different filesystems, the engine MUST fall back to a safe copy+fsync+rename strategy where applicable. For symlink replacement specifically, the engine uses an unlink+`symlinkat` best‑effort degraded fallback under policy. A policy flag `allow_degraded_fs` controls acceptance:

- When `allow_degraded_fs=true`, facts MUST include `degraded=true` and the operation proceeds, with `degraded_reason="exdev_fallback"`.
- When `allow_degraded_fs=false`, the apply MUST fail with `exdev_fallback_failed`; facts MUST include `degraded=false` and `degraded_reason="exdev_fallback"` for analytics consistency.

## 11. Smoke Tests (Normative Minimal Suite)

After apply, the `SmokeTestRunner` MUST, at minimum, run the following commands with specified arguments and treat any non-zero exit or mismatch as failure (which triggers auto-rollback unless explicitly disabled by policy):

- ls -l PATHS
- cp --reflink=auto SRC DST
- mv --no-target-directory SRC DST
- rm -f PATHS
- ln -sT TARGET LINK
- stat -c %a,%u,%g PATHS
- readlink -e PATH
- sha256sum -c CHECKFILE (tiny checkfile included in test bundle)
- sort -V INPUT
- date +%s

Policy MAY disable auto-rollback explicitly (e.g., `policy.disable_auto_rollback=true`); otherwise, smoke failure → auto-rollback.

Note: The smoke runner is provided via the `SmokeTestRunner` adapter. In production deployments an adapter MUST be configured so the suite runs; omission is permitted only in development/testing contexts.

## 12. Golden Fixtures & Zero-SKIP CI Gate

Golden JSON fixtures MUST exist for plan, preflight, apply, and rollback facts. CI MUST fail if:

- Any required test is SKIP.
- Any fixture diff is not byte-identical to the golden reference.

## 13. Schema Versioning & Migration

- Facts include `schema_version` (current=v2). Schema changes bump this field and require fixture/test updates.
- Pre‑1.0 policy: no dual‑emit toggles; v2 is the only supported schema going forward. Legacy v1 is deprecated.
- Secret‑masking rules are explicit and tested: any potentially sensitive strings in facts (e.g., environment‑derived values, external command args) MUST be redacted according to policy before emission; tests assert no secrets leak.

## 14. Thread-safety

Core types (e.g., `Plan`, apply engine) are `Send + Sync`. `apply()` MAY be called from multiple threads, but only one mutator proceeds at a time under the `LockManager`. Without a `LockManager`, concurrent apply is unsupported and intended only for dev/test.

## 15. Security Requirements Summary

- Automatic reverse-order rollback on mid-plan failure; idempotent rollback; partial-restoration facts on rollback error.
- SafePath-only mutations and TOCTOU-safe sequence (open parent O_DIRECTORY|O_NOFOLLOW → openat → renameat → fsync(parent)).
- Deterministic plan/action IDs (UUIDv5); dry-run facts byte-identical after timestamp redactions.
- Production locking required; bounded wait with timeout → E_LOCKING; facts include lock_wait_ms.
- Rescue profile with at least one fallback toolset on PATH; preflight verifies availability.
- Audit facts schema v2 with SHA-256 before/after hashes; signed attestation bundles; strict secret masking; complete provenance.
- Preflight Diff schema with stable ordering and keys.
- Cross-filesystem EXDEV fallback with degraded-mode policy and telemetry.

## 16. CLI Integration Guidance (SafePath)

Integrations that expose a CLI front-end SHOULD enforce SafePath at boundaries:

- Validate inputs by constructing `SafePath::from_rooted(root, candidate)` and reject on error.
- Avoid accepting raw `PathBuf` for mutating operations. Convert to `SafePath` immediately where unavoidable.
- Document error handling for path validation and show safe examples that do not reference non-existent APIs.
- Minimal smoke suite with exact args; failure → auto-rollback unless explicitly disabled.
- Golden fixtures and zero-SKIP CI gate.
