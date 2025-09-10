# Switchyard Specification (Reproducible v1.1)

## 0. Domain & Purpose

Switchyard is a **Rust library crate** (not a CLI) that provides **safe, atomic, reversible filesystem swaps** (typically for binaries).
It is **OS-agnostic**: it only manipulates filesystem paths and relies on adapters for policy and environment-specific logic.

---

## 1. Core Guarantees

### 1.1 Atomicity

* REQ-A1: A swap **MUST** be atomic with respect to crashes.
* REQ-A2: At no time **MUST** a user-visible broken or missing path exist.
* REQ-A3: All-or-nothing per plan: either all actions succeed, or no visible changes remain.

### 1.2 Rollback

* REQ-R1: Every change **MUST** be reversible by rollback.
* REQ-R2: Rollback **MUST** restore the exact prior link/file topology.
* REQ-R3: Rollback **MUST** be idempotent.

### 1.3 Safety Preconditions

* REQ-S1: Paths **MUST NOT** contain `..` or escape allowed roots.
* REQ-S2: Operations **MUST** fail if target FS is read-only, `noexec`, or immutable.
* REQ-S3: Source files **MUST** be root-owned and not world-writable, unless policy override.
* REQ-S4: If `strict_ownership=true`, targets **MUST** be package-owned (via adapter).

### 1.4 Observability

* REQ-O1: Every step **MUST** emit a structured fact (JSON).
* REQ-O2: Dry-run facts **MUST** be byte-identical to real-run facts.
* REQ-O3: Facts schema **MUST** be versioned and stable.
* REQ-O4: Attestations (signatures, SBOM-lite fragments) **MAY** be attached via adapters.

### 1.5 Locking

* REQ-L1: Only one `apply()` **MUST** mutate at a time.
* REQ-L2: If no lock manager, concurrent `apply()` is **UNSUPPORTED** (dev/test only) and a WARN fact **MUST** be emitted.
* REQ-L3: Lock acquisition **MUST** use a bounded wait with timeout → `E_LOCKING`, and facts **MUST** record `lock_wait_ms`.
* REQ-L4: In production deployments, a `LockManager` **MUST** be present. Omission is permitted only in development/testing.

### 1.6 Rescue

* REQ-RC1: A rescue profile (backup symlink set) **MUST** always remain available.
* REQ-RC2: Preflight **MUST** verify at least one functional fallback path.

### 1.7 Determinism

* REQ-D1: `plan_id` and `action_id` are UUIDv5 values derived from the normalized plan input using a project-defined, stable namespace.
* REQ-D2: Dry-run redactions are pinned: timestamps are zeroed (or expressed as monotonic deltas). Dry-run facts **MUST** be byte-identical to real-run facts after redaction.

---

## 2. Interfaces

### 2.1 Public API

```rust
fn plan(input: PlanInput) -> Plan;
fn preflight(plan: &Plan) -> PreflightReport;
fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport;
fn plan_rollback_of(report: &ApplyReport) -> Plan;
```

All path-carrying fields within `PlanInput` and `Plan` **MUST** be typed as `SafePath`. Mutating entry points do not accept `PathBuf`.

### 2.2 Adapters

```rust
trait OwnershipOracle { fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo>; }
trait LockManager { fn acquire_process_lock(&self) -> Result<LockGuard>; }
trait PathResolver { fn resolve(&self, bin: &str) -> Result<SafePath>; }
trait Attestor { fn sign(&self, bundle: &[u8]) -> Result<Signature>; }
trait SmokeTestRunner { fn run(&self, plan: &Plan) -> Result<(), SmokeFailure>; }
```

### 2.3 SafePath

* Constructed via `SafePath::from_rooted(root, candidate)`.
* Rejects `..` after normalization.
* Opened with `O_NOFOLLOW` parent dir handles (TOCTOU defense).
* All mutating public APIs **MUST** take `SafePath` (not `PathBuf`). Any non-mutating APIs that accept raw paths **MUST** immediately normalize to `SafePath`.
* TOCTOU-safe syscall sequence is normative for every mutation: open parent with `O_DIRECTORY|O_NOFOLLOW` → `openat` on final component → `renameat` → `fsync(parent)`.

---

## 3. Facts & Audit Schema

**JSON Schema (`/SPEC/audit_event.schema.json`):**

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SwitchyardAuditEvent",
  "type": "object",
  "required": ["ts","plan_id","stage","decision","path"],
  "properties": {
    "ts": {"type":"string","format":"date-time"},
    "plan_id": {"type":"string"},
    "schema_version": {"type":"integer","enum":[1]},
    "action_id": {"type":"string"},
    "stage": {"enum":["plan","preflight","apply.attempt","apply.result","rollback"]},
    "decision": {"enum":["success","failure","warn"]},
    "severity": {"enum":["info","warn","error"]},
    "degraded": {"type":["boolean","null"]},
    "path": {"type":"string"},
    "current_kind": {"type":"string"},
    "planned_kind": {"type":"string"},
    "before_hash": {"type":"string"},
    "after_hash": {"type":"string"},
    "provenance": {
      "type":"object",
      "properties": {
        "origin": {"enum":["repo","aur","manual"]},
        "uid": {"type":"integer"},
        "gid": {"type":"integer"},
        "pkg": {"type":"string"}
      }
    },
    "exit_code": {"type":["integer","null"]},
    "duration_ms": {"type":["integer","null"]},
    "lock_wait_ms": {"type":["integer","null"]}
  }
}
```

---

## 4. Error Taxonomy & Exit Codes

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

Errors are emitted in facts as stable identifiers (e.g. `E_POLICY`, `E_LOCKING`).

---

## 5. Preflight Diff

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
```

Dry-run output must match real-run preflight rows byte-for-byte.

---

## 6. Formal Safety Model

### 6.1 Invariants (TLA+ sketch)

* **NoBrokenLinks:** No symlink points to a missing file.
* **Rollbackable:** After any sequence of steps, `CanRollback` holds.

### 6.2 Properties

* **AtomicReplace:** property-based tests ensure no intermediate “missing” visible.
* **IdempotentRollback:** applying rollback twice yields same FS state.

---

## 7. Acceptance Tests (BDD-style)

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
```

---

## 8. Operational Bounds

* `fsync(parent)` MUST occur ≤50ms after rename.
* Plan size default max = 1000 actions (configurable).

---

## 9. Compliance Mapping

* REQ-A1/A2/A3 → Atomic tests, TLA invariants.
* REQ-R1/R2/R3 → Rollback tests, property-based tests.
* REQ-S\* → Preflight validations.
* REQ-O\* → Audit JSON schema compliance.
* REQ-L\* → Lock tests.
* REQ-RC\* → Rescue profile tests.

---

## 10. Filesystems & Degraded Mode

Supported filesystems (tested):

* ext4 — native rename semantics
* xfs — native rename semantics
* btrfs — native rename semantics
* tmpfs — native rename semantics

EXDEV path is explicitly exercised: when staging and target parents are on different filesystems, the engine MUST fall back to a safe copy+fsync+rename strategy. A policy flag `allow_degraded_fs` controls acceptance:

* When `allow_degraded_fs=true`, facts MUST include `degraded=true` and the operation proceeds.
* When `allow_degraded_fs=false`, the apply MUST fail with `exdev_fallback_failed`.

## 11. Smoke Tests (Normative Minimal Suite)

After apply, the `SmokeTestRunner` MUST, at minimum, run the following commands with specified arguments and treat any non-zero exit or mismatch as failure (which triggers auto-rollback unless explicitly disabled by policy):

* ls -l PATHS
* cp --reflink=auto SRC DST
* mv --no-target-directory SRC DST
* rm -f PATHS
* ln -sT TARGET LINK
* stat -c %a,%u,%g PATHS
* readlink -e PATH
* sha256sum -c CHECKFILE (tiny checkfile included in test bundle)
* sort -V INPUT
* date +%s

Policy MAY disable auto-rollback explicitly (e.g., `policy.disable_auto_rollback=true`); otherwise, smoke failure → auto-rollback.

## 12. Golden Fixtures & Zero-SKIP CI Gate

Golden JSON fixtures MUST exist for plan, preflight, apply, and rollback facts. CI MUST fail if:

* Any required test is SKIP.
* Any fixture diff is not byte-identical to the golden reference.

## 13. Schema Versioning & Migration

* Facts include `schema_version` (current=v1). Schema changes bump this field.
* v1→v2 requires a dual-emit period and corresponding fixture updates.
* Secret-masking rules are explicit and tested: any potentially sensitive strings in facts (e.g., environment-derived values, external command args) MUST be redacted according to policy before emission; tests assert no secrets leak.

## 14. Thread-safety

Core types (e.g., `Plan`, apply engine) are `Send + Sync`. `apply()` MAY be called from multiple threads, but only one mutator proceeds at a time under the `LockManager`. Without a `LockManager`, concurrent apply is unsupported and intended only for dev/test.

---

✅ This spec is **reproducible**:

* Human-readable requirements (RFC-2119).
* Machine-checkable YAML/TOML/JSON schemas.
* Formalizable invariants (TLA+).
* BDD acceptance tests.
* Property-based tests for atomicity.
