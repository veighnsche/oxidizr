# Switchyard Specifications & Requirements

Version: 1.2 (2025-09-10)
Status: Draft for formal review

1. Introduction & Purpose
1.1 Objective

- Define the complete, auditable, implementation-agnostic specification for `switchyard/`, a reusable safety kernel extracted from `oxidizr-arch`.
- Specify the externally observable guarantees, interfaces, and policies for safe system-level binary switching, independent of any distro or experiment bundle.

1.2 Rationale

- `Switchyard` centralizes safety-critical mechanisms (planning, preflight, atomic commit, backups, rollback, and audit facts) so higher-level products (e.g., `oxidizr-arch`) can compose them without duplicating safety logic.

1.3 Sources (Traceability)

- `PROJECT_PLANS/SWITCHYARD_PLAN.md`
- `AUDIT_CHECKLIST.md`
- `SAFETY_MEASURES.md`
- `PROJECT_PLANS/SAFETY_DECISIONS_AUDIT.md`

1.4 Implementation Status Snapshot (0.1.0)

- Current crate/API: `switchyard` 0.1.0 (`switchyard/Cargo.toml`).
- Scope implemented today (see `switchyard/src/`):
  - Atomic symlink replacement with parent `fsync` via `fs_ops::atomic_symlink_swap()`.
  - Per-target backup/restore with `.oxidizr.bak` suffix; regular-file backups preserve permissions (mode).
  - Preflight checks: path traversal defense (`symlink::is_safe_path`), mount `rw` and not `noexec` (`preflight::ensure_mount_rw_exec`), immutable bit (`preflight::check_immutable`), source trust (`preflight::check_source_trust`), allow/forbid path gates.
  - Planner/executor: simple ordered list of actions; `DryRun` supported.
  - Policy fields available: `allow_roots`, `forbid_paths`, `strict_ownership` (defined, not enforced), `force_untrusted_source`, `force_restore_best_effort`.
- Not yet implemented (planned; see sections marked Planned):
  - DAG planning with stable IDs; multi-action rollback orchestration.
  - Structured facts/audit emission wired into plan/preflight/apply.
  - Ownership/provenance checks (strict ownership), locking, attestation, smoke tests.
  - Extended metadata (owner/times/xattrs/ACLs/capabilities) preservation.

2. System Boundaries
2.1 In Scope (library responsibilities)

- Planning of file-transition actions with deterministic, stable identifiers. [Status: Planned]
- Preflight validation for each planned action (mount suitability, immutability, trust/ownership policy, path confinement, optional policy scanners).
- Atomic commit of actions with crash-consistency guarantees and idempotence.
- Backup creation and rollback mechanisms for all destructive transitions.
- Observability via structured facts and human-readable audit messages.
- Policy application (allow/deny roots, strict ownership, trust overrides, metadata-preservation policy).

2.1.1 Implementation status (0.1.0)

- Planning: simple ordered list (no DAG, no IDs yet) implemented in `api.rs::plan()`.
- Preflight: implemented (basic checks) in `preflight.rs`; `strict_ownership` is not enforced yet.
- Atomic commit: implemented for symlink swap via `fs_ops.rs` (temp symlink + `renameat` + parent `fsync`).
- Backup/restore: implemented per-target (no cross-action rollback orchestration).
- Observability: traits exist (`FactsEmitter`, `AuditSink`) but are not wired into execution.
- Policy: `allow_roots`/`forbid_paths`/`force_untrusted_source`/`force_restore_best_effort` enforced; `strict_ownership` planned.

2.2 Out of Scope (delegated to host application or environment)

- Package management, AUR/apt/yum operations, network access.
- Initramfs preparation or rescue image provisioning.
- Signing key management and SBOM generation (interfaces may be provided; implementations are host-specific).
- CLI UX, interactive prompts, and distro-specific compatibility checks.

3. Functional Requirements
3.1 Atomicity & Crash-Consistency

- Commit of each action is a single atomic directory-entry update at its final stage.
- Parent directory metadata is made durable after commit.
- No intermediate state leaves a partially written target visible on crash.
- On EXDEV (cross-device rename not possible): perform copy + `fsync(temp)` + atomic rename-at to the final path; mark the action result with `exdev: true` in reports/facts.
- Filesystems lacking atomic directory entry updates MUST be stopped in preflight unless a policy knob explicitly enables degraded mode.

3.2 Idempotence

- Re-applying the same plan on an unchanged system produces a no-op (no additional backups, no changes) and identical facts output.
- Symlink replacement that already matches the desired target is a no-op.

3.3 Deterministic Planning

- Planned: Plan is a DAG of actions with stable, reproducible ordering for identical inputs and environment.
- Planned: Each action carries a stable identifier derived from normalized inputs.
- Current (0.1.0): Plan is a simple ordered list of actions as provided; no action IDs yet.

3.4 Rollback Guarantees

- For all destructive transitions, a rollback path exists that restores the last pre-state from a local backup.
- Rollback does not remove unrelated files and is safe to invoke repeatedly (idempotent).

Status (0.1.0): Partial — rollback is available per-target via the `RestoreFromBackup` action; there is no multi-action transaction rollback orchestration.

Added in v1.2:

- On any mid-plan failure during `apply`, the library MUST attempt complete, automatic rollback of already-committed actions, restoring pre-state from backups in reverse plan order.
- Rollback attempts MUST be captured in facts/audit with per-target outcomes and a final summary decision.

3.5 Metadata Handling

- Regular-file backups preserve permissions (mode). Ownership and timestamps are not preserved or recorded.
- Symlink backups preserve linkness and link destination.
- Planned: Policy-driven handling of xattrs/ACLs/capabilities: at minimum detect-and-warn; optional preserve modes are exposed via policy knobs.

Added in v1.2:

- Commit to preserving for regular files: owner, mode, and timestamps at minimum.
- For symlinks: preserve linkness and target only (no owner/timestamps enforced on link itself).
- Expose preservation knobs for ACLs/xattrs/capabilities; when bits are requested but unsupported by the FS, preflight MUST emit WARN or STOP per policy (see §5.4).

3.6 Preflight Validation

- Mount suitability: target path resides on a read-write, executable filesystem.
- Immutability: preflight fails when target or its parent is immutable unless overridden by policy.
- Path confinement: targets and sources must pass SafePath invariants (no parent traversal; normalized within allowed roots).
- Trust/Ownership checks: untrusted sources (world-writable, non-root-owned, or under HOME) and unowned targets are rejected unless policy permits.
- Policy gates: allowlist roots and forbid paths are enforced.
- Optional scanners: capability/ACL/xattr presence, provenance signals, and compatibility hints may emit warnings or stops per policy.

3.7 Locking

- Library accepts an optional `LockManager` adapter.
- `apply()` MUST acquire a process-wide lock if `LockManager` is supplied; otherwise proceed but emit a WARN fact indicating "unlocked apply".
- Optional per-target locks MAY be acquired per action when provided; actions execute in plan order; no nested target locks.
- Deadlock avoidance: no lock re-entry; acquisition order is strictly the plan order.
- Implementation status (0.1.0): Locking not yet implemented; this is a normative contract for the adapter when added.

3.8 Observability

- Library MUST emit observability records at the following checkpoints:
  - `plan` (plan created), `preflight` (per-action results),
  - `apply.attempt` (before each action), `apply.result` (after each action),
  - `restore.result` (after restore actions).
- Each payload includes: `{ schema_version, plan_id, action_id?, stage, severity }` plus provenance fields `{ actor, source_version, origin, owner_pkg, helper }` and other stable fields relevant to the event.
- When no sinks are provided, the library MUST still build these records and expose them in the returned `PreflightReport`/`ApplyReport` structures.
- Implementation status (0.1.0): Traits exist (`FactsEmitter`, `AuditSink`) but wiring is pending.

Added in v1.2:

- Every mutated target MUST include `before_hash` and `after_hash` (cryptographic, e.g., SHA-256). Selective hashing is allowed; cache eligibility by `(dev,inode,mtime,size)`.
- Attestation bundles (per-operation JSONL) MUST be produced with detached signatures; see §3.16 for schema and signed bytes.

3.9 Error Taxonomy & Mapping

- Errors use stable identifiers to avoid churn in logs and consumer tooling. Canonical set reserved by the library:
  - `E_POLICY`, `E_PREFLIGHT_ENV`, `E_PREFLIGHT_OWNERSHIP`, `E_LOCKING`,
  - `E_ATOMIC_SWAP`, `E_EXDEV_FALLBACK`, `E_BACKUP`, `E_RESTORE`, `E_OBSERVABILITY`, `E_SMOKE_FAIL`.
- Host CLIs map these identifiers to exit codes; the library guarantees stable identifiers in facts/audit payloads.
- Current (0.1.0): `ApplyReport.errors` is `Vec<String>`; taxonomy will be adopted by future versions without renaming identifiers.

3.10 Preflight Diff (normative)

- `preflight(plan)` MUST produce a deterministic per-action diff row including:
  - `action_id` (stable), `path`
  - `current_kind`: `missing` | `regular_file` | `directory` | `symlink`, and `current_dest` when kind is `symlink`
  - `planned_kind`: e.g., `symlink` | `restore_from_backup`, and `planned_dest` when applicable
  - `policy`: `{ strict_ownership_passed: bool, trust_ok: bool, roots_ok: bool, forbids_hit: bool }`
  - `provenance`: `{ uid: u32, gid: u32, package?: string }`
  - `notes`: `string[]` (compat hints/warnings)
- Serialization MUST be stable (sorted keys, deterministic action ordering) so dry-run artifacts are byte-identical across runs with identical inputs.

3.11 Default Dry-Run & Preflight (Added in v1.2)

- Dry-run/preflight is the default posture unless explicitly overridden by a host-level confirmation flag (e.g., `--assume-yes`).
- Preflight MUST produce deterministic plan rows (see §3.10) for CI/PR comparison.
- In dry-run, no filesystem mutations are performed; facts/audit payloads remain byte-identical across runs with identical inputs.

3.12 Compatibility Matrix Hook (Added in v1.2)

- The library MUST support a pluggable compatibility matrix (JSON/TOML rules) that flags risky semantics or patterns.
- Critical compatibility violations MUST fail closed during preflight unless explicitly overridden by policy.

3.13 Profiles & Atomic Pointer Flip (Added in v1.2)

- Support an atomic profile-pointer flip model with a stable `active` pointer (e.g., `active -> profiles/<current>`), and profile bins under `profiles/<name>/bin`.
- The audit event schema MUST include `{ from_profile, to_profile }` when a profile flip occurs.
- Backup semantics:
  - Regular files: preserve owner, mode, timestamps.
  - Symlinks: preserve linkness and target only.
- A `canary` profile MUST be supported as an escape valve for safety validation prior to flipping.

3.14 Rescue & Recovery (Added in v1.2)

- At least one "rescue profile" (e.g., GNU or busybox) MUST remain available in `PATH` at all times.
- Rollback MUST remain single-step and idempotent.
- Operator-facing recovery and verification documentation is a required deliverable.

3.15 Supply Chain & Provenance (Added in v1.2)

- Provenance MUST include: origin (`repo` vs `aur`), package owner, and helper/command used for acquisition.
- AUR execution MUST require an explicit policy override (`allow_aur`).
- Environment for provenance checks/helpers MUST be sanitized (e.g., `LC_ALL=C`, pinned `PATH`).
- Lock-wait UX MUST be bounded and expose progress breadcrumbs via facts/audit.

3.16 Facts/Audit Schema v1 (Added in v1.2)

- Purpose: Freeze field names and ordering for byte-stable JSONL facts, define masking, and specify attestation bytes.

- Envelope (ordered):
  - `schema_version`, `ts`, `plan_id`, `action_id?`, `stage`, `severity`, `run_id?`, `actor?`, `container_id?`, `distro`, `component`, `subsystem`, `event`, `decision`.

- Provenance (ordered):
  - `origin` (`repo`|`aur`|`manual`), `repo_present?`, `owner_pkg?`, `helper?`, `helper_cmd?` (sanitized), `uid`, `gid`.

- Targeting (ordered):
  - `path`, `current_kind`, `current_dest?`, `planned_kind`, `planned_dest?`.

- Result (ordered):
  - `exit_code`, `duration_ms`, `lock_wait_ms?`, `exdev?`, `before_hash?`, `after_hash?`.

- Secret masking: any values matching credential patterns (e.g., `token=...`, `password=...`, `Authorization: Bearer ...`) MUST be masked before emission.

- Dry-run: external sinks MUST NOT persist facts; records MUST still be constructed and embedded in `PreflightReport`/`ApplyReport` with exact key order for byte-stable diffing.

- Attestation: the per-operation JSONL record (newline-terminated UTF-8, with the exact key order above) MUST be signed with a detached Ed25519 signature (`.sig`). SBOM-lite fragments MAY be generated and linked.

4. Non-Functional Requirements
4.1 Deterministic Dry-Run

- Dry-run produces byte-for-byte stable plan outputs and facts for identical inputs.
- Dry-run must not mutate filesystem state or external resources.
- Use content-derived IDs (e.g., UUIDv5 over normalized plan inputs) for `plan_id` and `action_id` to ensure stability.
- Redact or normalize volatile fields in dry-run outputs (e.g., timestamps as 0 or monotonic deltas) to keep byte-stable artifacts.
- Facts emitted in dry-run MUST be stable and byte-identical for identical inputs and environment.

Status (0.1.0): Dry-run mode is supported (no mutations), but facts emission and golden fixtures are not yet provided.

4.2 Minimal Trusted Surface

- Only essential dependencies and system calls are assumed; no ambient network access.
- Behavior is auditable by a single engineer in reasonable time.

4.3 Human Auditability

- Facts and logs are structured and consistent; schemas are stable and versioned.
- Messages include actionable remediation for common failures (e.g., immutability guidance).

Status (0.1.0): Traits for facts/logging exist but are not yet integrated; schemas are not defined in-code.

4.4 Safety Under Interruption

- SIGINT/kill/power-loss during commit preserves atomicity and recoverability (either old or new state is fully present; backups not destroyed prematurely).

4.5 Portability & Filesystems

- Guarantees are upheld on standard Linux filesystems (e.g., ext4, xfs, btrfs, tmpfs) when they provide atomic directory entry updates.

4.6 Dependency Surface (Added in v1.2)

- External dependencies MUST be minimized; heavy crates MUST be feature-gated.
- PATH resolution MUST be unified via a central adapter (see `PathResolver` trait); duplicate helpers are not permitted.

4.7 Verbosity Contract (Added in v1.2)

- Fixed levels `v0..v3` with CLI mapping: v0 critical/summary; v1 default lifecycle; v2 verbose (command echoing begins at v2); v3 trace.
- Audit and user-facing logs MUST respect these semantics; levels are intrinsic to messages and not derived from flags.
- Prefix format SHOULD follow `[<distro>][v<level>][<scope>] message`.

5. Interfaces & Contracts
5.1 Public API (0.1.0)

- `Switchyard<E: FactsEmitter, A: AuditSink>::new(facts, audit, policy) -> Switchyard` — Construct with configured policy and sinks (sinks are currently no-ops).
- `plan(input: PlanInput) -> Plan` — Converts requested link/restore operations into an ordered `Plan` of `Action`s.
- `preflight(plan: &Plan) -> PreflightReport` — Validates each `Action` with safety checks; produces warnings and stops.
- `apply(plan: &Plan, mode: ApplyMode) -> ApplyReport` — Executes actions in sequence; honors `DryRun`; records executed actions and errors.
- `plan_rollback_of(report: &ApplyReport) -> Plan` — Convenience: derive inverse actions as a `Plan` to rollback a previous apply (modeled as `RestoreFromBackup` actions).
- Note: There is no imperative rollback function; rollback remains modeled as planned actions.

5.2 Traits (Adapters)

- `FactsEmitter` — Present today; emits machine-readable facts. Currently a no-op default; wiring is planned.
- `AuditSink` — Present today; emits human-readable lines. Currently a no-op default; wiring is planned.
- `OwnershipOracle` — Planned: provenance (owner package, repo presence) and strict-ownership enforcement.
- `LockManager` — Planned: process-wide and optional per-target locks.

```rust
pub trait LockManager {
    fn acquire_process_lock(&self) -> Result<LockGuard, SwitchyardError>;
    fn acquire_target_lock(&self, path: &std::path::Path) -> Result<LockGuard, SwitchyardError>; // optional
}
```

- `PathResolver` — Provides unified PATH resolution for binaries; centralizes `which` semantics and avoids duplicate helpers.

```rust
pub trait PathResolver {
    fn which(&self, name: &str) -> Option<std::path::PathBuf>;
}
```

Locking contract additions (Added in v1.2):

- Process-wide lock acquisition SHOULD have a bounded wait; facts MUST include `lock_wait_ms`.
- On timeout, map to `E_LOCKING` category; host CLIs MUST map to a stable exit code. Concurrent `apply()` is undefined behavior without a `LockManager`.

5.3 Path Safety Invariants

- Current (0.1.0): Paths are plain `PathBuf`; traversal defense via `symlink::is_safe_path` and `fs_ops::open_dir_nofollow` for parent directories.
- Planned: Introduce a `SafePath` newtype validated at API boundaries (no `..` after normalization; confined to allowed roots; prevent parent traversal via no-follow directory operations).
- `SafePath` is constructed only via `SafePath::from_rooted(root, candidate)` which enforces confinement and rejects traversal after normalization.
- Mutating operations MUST open parent directories with `O_DIRECTORY | O_NOFOLLOW` (TOCTOU defense) before acting on final path components.

5.4 Policy Knobs

- Implemented (0.1.0):
  - `allow_roots: Vec<PathBuf>` — restrict target scope to permitted roots (if non-empty).
  - `forbid_paths: Vec<PathBuf>` — hard-deny specific subtrees.
  - `strict_ownership: bool` — defined but not enforced yet.
  - `force_untrusted_source: bool` — when true, proceed with untrusted sources but surface a warning in preflight.
  - `force_restore_best_effort: bool` — allow restore to succeed when a backup is missing.
- Planned additions:
  - `preserve_metadata` (bitflags) — owner/times/xattrs/ACLs/capabilities preservation policies.
  - `allow_aur: bool` — permit AUR-based acquisition paths when true; otherwise preflight MUST stop on AUR paths.
  - `allow_degraded_fs: bool` — permit proceeding on filesystems with uncertain atomic dirent guarantees; facts MUST record degraded mode.

```rust
bitflags::bitflags! {
    pub struct PreservedMeta: u32 {
        const OWNER  = 0b00001;
        const TIMES  = 0b00010; // atime/mtime/ctime best-effort
        const XATTRS = 0b00100;
        const ACLS   = 0b01000;
        const CAPS   = 0b10000; // Linux capabilities
    }
}
```

Runtime MUST detect filesystem support; if unsupported and a bit is set, preflight emits WARN (or STOP if policy requires strict preservation).

6. Acceptance Criteria
6.1 Testability of Safety Properties

- Each property (atomicity, idempotence, confinement, rollback) has a corresponding automated test.

6.2 Dry-Run/Commit Parity

- Outputs and facts from dry-run match commit planning; only side effects differ.
- Dry-run artifacts (facts/preflight diff) are byte-identical across runs with identical inputs.

6.3 Stable Schemas

- Planned: Facts and audit message schemas are versioned; changes require migration guidance and tests.
- Audit bundles MUST include before/after cryptographic hashes of all affected files; actor, source/version/provenance, and exit codes MUST be recorded.
- Per-operation JSONL bundles MUST be produced with optional detached signatures (`.sig`, e.g., Ed25519); selective hashing MAY be cached by inode/mtime/size.
- SBOM-lite fragments MAY be emitted per operation.

6.4 Recovery Path

- Rollback remains available for all destructive actions; absence of backup is surfaced clearly and can be policy-downgraded.
- Profile pointer flip MUST be reversible by a single-step rollback to the prior profile.
- A rescue profile MUST be present and verified in test runs.

6.5 No Regressions in Link Semantics

- Symlinks are never materialized into regular files by library operations.

6.6 Supply Chain & Lock Wait UX (Added in v1.2)

- Provenance records MUST include origin (repo vs AUR), package owner, and helper used.
- AUR paths MUST be gated by `allow_aur` policy.
- Lock waits MUST be bounded and surfaced with progress breadcrumbs in audit/facts.

6.7 Dependency Surface (Added in v1.2)

- Heavy dependencies are feature-gated; PATH resolution is centralized via `PathResolver`.

6.8 Zero-SKIP CI Policy (Added in v1.2)

- CI MUST have zero SKIPs in required suites; Arch-family matrix is default.
- Harness discipline: tests MUST NOT pre-mutate product-managed paths; mutations are owned by the product during tests.
- This policy is a release gate.

6.9 Exit Code Mapping (Added in v1.2)

- Stable mapping used across tests and CLIs:

  - `0` — success
  - `1` — generic failure
  - `2` — CLI misuse
  - `10` — Incompatible distro
  - `20` — Nothing to link
  - `30` — Restore backup missing
  - `40` — Repo/AUR gating failure
  - `50` — Pacman DB lock timeout
  - `70` — Root required
  - `80` — Filesystem unsuitable (immutable/noexec/ro)
  - `90` — Hook install error

6.10 Minimal Smoke Suite (Added in v1.2)

- After apply, run a short health suite with canned args: `ls`, `cp`, `mv`, `rm`, `ln`, `stat`, `readlink`, `sha256sum`, `sort`, `date`.
- Any non-zero or mismatch MUST trigger automatic rollback (policy MAY explicitly disable).

7. Auditability & Test Plan
7.1 Golden Fixtures

- Planned: Golden JSON fixtures for facts across plan/preflight/apply/restore under representative scenarios.

7.2 Crash-Consistency Tests

- Fault-injection harness that interrupts between backup creation and commit; validate recovery behavior and parent directory durability.

7.3 Cross-Filesystem Validation

- Execute suites on ext4, btrfs, tmpfs (and others as available) to validate atomicity assumptions.

7.4 Property-Based Tests

- Idempotence: applying same plan twice yields no additional changes.
- Path Confinement: arbitrary fuzzed paths are rejected unless within policy roots and free of traversal.

7.5 Health Verification

- Planned: Optional smoke tests executed post-commit with rollback-on-fail path validated.

7.6 Golden Fixtures (Added in v1.2)

- Provide golden JSON fixtures for `plan`/`preflight`/`apply`/`restore` facts to assert byte-for-byte determinism.

8. Compliance Guarantees

- Transactionality: atomic commit + parent durability; idempotence; EXDEV fallback documented; no symlink materialization.
- Auditability: structured facts and human logs with stable schemas; per-operation JSONL (+ optional signature); before/after hashes recorded; actor/provenance/exit codes included; SBOM-lite optional.
- Least Intrusion: metadata preservation per policy; deterministic preflight diff enumerates exact scope of change; rescue profile always available.
- Determinism: deterministic planning and byte-stable dry-run artifacts (facts and diff); content-derived IDs.
- Conservatism: fail-closed on critical compatibility issues and policy violations; explicit overrides required (e.g., `allow_aur`, assume-yes).
- Recovery First: backups created pre-commit; one-step rollback; profile pointer flip reversible; operator recovery documentation provided.
- Supply Chain: provenance records origin (repo vs AUR), owner, helper; sanitized environment; bounded lock-wait with breadcrumbs.
- Dependency Surface: minimal external dependencies; heavy crates feature-gated; unified PATH resolution.

9. Versioning & Stability

- Semantic versioning for public APIs and fact schemas.
- Backward-compatible changes preferred; breaking changes require explicit major version.

10. Glossary

- Action: A single planned change (e.g., ensure symlink).
- Plan: Deterministic, ordered set of actions.
- Policy: Caller-supplied safety rules for gating and metadata behavior.
- Facts: Machine-readable structured records of decisions and outcomes.
- Attestation: Optional hashing/signature artifacts for audit verification.

## 11. Missing Items vNext (Added in v1.2)

The following are implementation/backlog items (non-normative) that remain after v1.2’s normative additions:

- SafePath newtype adoption across public APIs; acceptance: all mutating entrypoints take `SafePath`, and tests validate the O_DIRECTORY|O_NOFOLLOW open sequence.
- OwnershipOracle + strict ownership enforcement; acceptance: preflight STOP on unknown ownership unless policy override, with fixtures.
- LockManager adapter implementation with bounded waits; acceptance: `lock_wait_ms` recorded, timeout maps to `E_LOCKING`.
- SBOM-lite fragment schema and fixtures; acceptance: schema v1 with sample artifacts referenced from attestation bundles.
- Dependency footprint trim; acceptance: heavy crates behind feature flags and `cargo deny`/`cargo audit` pass in CI.
