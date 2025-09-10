# Switchyard Specification (Reproducible v1.0)

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
* REQ-L2: If no lock manager, concurrent apply is undefined but a WARN fact **MUST** be emitted.
* REQ-L3: Lock acquisition **MUST** support bounded wait with timeout → error.

### 1.6 Rescue

* REQ-RC1: A rescue profile (backup symlink set) **MUST** always remain available.
* REQ-RC2: Preflight **MUST** verify at least one functional fallback path.

---

## 2. Interfaces

### 2.1 Public API

```rust
fn plan(input: PlanInput) -> Plan;
fn preflight(plan: &Plan) -> PreflightReport;
fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport;
fn plan_rollback_of(report: &ApplyReport) -> Plan;
```

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
    "action_id": {"type":"string"},
    "stage": {"enum":["plan","preflight","apply.attempt","apply.result","rollback"]},
    "decision": {"enum":["success","failure","warn"]},
    "severity": {"enum":["info","warn","error"]},
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
    "duration_ms": {"type":["integer","null"]}
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

✅ This spec is **reproducible**:

* Human-readable requirements (RFC-2119).
* Machine-checkable YAML/TOML/JSON schemas.
* Formalizable invariants (TLA+).
* BDD acceptance tests.
* Property-based tests for atomicity.
