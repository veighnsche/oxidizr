# Switchyard Glossary

Authoritative terminology for the `cargo/switchyard` crate. Sources: `SPEC/SPEC.md`, `SPEC/requirements.yaml`, `SPEC/audit_event.schema.json`, and planning docs under `PLAN/`. Section references point to those files.

- __Apply__ — Execution of a `Plan` to mutate the filesystem. Emits `apply.attempt` and `apply.result` audit facts per action and may trigger rollback on failure. See `SPEC/SPEC.md §3.1, §2.2, §2.4`.

- __Apply Mode (`ApplyMode`)__ — Controls side effects: `DryRun` (default) vs `Commit`/`RealRun`. Dry-run emits byte-identical facts after redaction. See `src/types/plan.rs`, `SPEC §2.8`, `SPEC §2.4`.

- __Apply Report (`ApplyReport`)__ — Summary of an `apply()` run, including decision, degraded status, exit codes, and partial restoration when rollback is incomplete. See `PLAN/10-types-traits.md`.

- __Attestation__ — Signed bundle metadata for an `apply` success, containing signature, bundle hash, and public key id. Emitted in summary facts. See `SPEC §2.4`, `SPEC §5`, `audit_event.schema.json`.

- __Attestor__ — Adapter responsible for producing an attestation signature (e.g., ed25519). See `SPEC §3.2`, `PLAN/10-types-traits.md`.

- __Audit Fact (Fact)__ — Structured JSON event emitted for each step (`plan`, `preflight`, `apply.attempt`, `apply.result`, `rollback`). Versioned by `schema_version`. See `SPEC §2.4`, `SPEC §5`, `audit_event.schema.json`.

- __Audit Sink / Facts Emitter (`AuditSink`, `FactsEmitter`)__ — Abstractions for emitting and persisting facts (typically JSONL). Redaction policy is applied before emission. See `src/api/audit.rs`, `src/logging/redact.rs`, `PLAN/40-facts-logging.md`.

- __Backup__ — Copy of the replaced artifact enabling rollback. Filenames include a tag and timestamp to avoid collisions across CLIs. See “backup_tag”.

- __`backup_tag`__ — Policy-provided tag included in backup filenames to isolate multiple CLIs using Switchyard on the same host. Format: `.basename.<backup_tag>.<unix_millis>.bak`. Only backups with the current tag are considered for restore. See `SPEC/SPEC_UPDATE_0001.md`, `TODO.md §5.1`.

- __Bounded Wait__ — Lock acquisition must have a timeout. On timeout: error `E_LOCKING` and fact includes `lock_wait_ms`. See `SPEC §2.5`, `PLAN/50-locking-concurrency.md`.

- __`lock_wait_ms`__ — Telemetry field recording how long lock acquisition took before success or timeout. Included in `apply.attempt` facts; must be recorded on success and (when possible) on timeout. See `src/api/apply.rs`.

- __Capabilities / Capability-based Handles__ — Security model of opening parent directories with `O_DIRECTORY|O_NOFOLLOW` and using `*at` syscalls to avoid TOCTOU. See “TOCTOU-safe sequence”, `SPEC §3.3`.

- __Degraded Mode__ — Cross-filesystem fallback path (EXDEV): safe copy + fsync + rename, allowed by policy. Facts set `degraded=true`. If disallowed, apply fails. See `SPEC §2.10`.

- __`degraded` Flag__ — Boolean field in apply facts indicating an EXDEV degraded fallback was used (non‑atomic visibility). Controlled by `Policy.allow_degraded_fs`. See `src/api/apply.rs`, `src/fs/atomic.rs`.

- __Determinism__ — Stable, reproducible outputs: `plan_id`/`action_id` via UUIDv5 over normalized inputs; facts identical between dry-run and real-run after redaction. See `SPEC §2.7`, `PLAN/35-determinism.md`.

- __Dry-run__ — Side-effect-free mode. Required to be conservative by default; emits facts with timestamps zeroed or normalized. See `SPEC §2.8`, `SPEC §2.4`.

- __Fail‑closed__ — Safety stance that operations must STOP when a critical precondition fails (e.g., unsafe filesystem flags, ownership mismatch) unless an explicit policy override is set. Enforced by wiring preflight `policy_ok=false` into `apply()` refusal. See `SPEC §2.3`, `PLAN/45-preflight.md`.

- __Error Id (`ErrorId`)__ — Stable identifiers mapped to exit codes per `SPEC/error_codes.toml`: `E_POLICY`, `E_OWNERSHIP`, `E_LOCKING`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`, `E_GENERIC`. See `src/api/errors.rs`, `SPEC §6`.

- __Exit Codes__ — Stable numeric mapping for integration and CI: e.g., `lock_timeout=30`, `atomic_swap_failed=40`, `smoke_test_failed=80`. See `SPEC/error_codes.toml`.

- __EXDEV__ — Cross-filesystem rename error requiring degraded fallback (policy-controlled). See `SPEC §2.10`.

- __Facts Redaction__ — Policy that removes or normalizes volatile fields (timestamps, timings) and masks secrets to ensure dry-run/real-run parity and prevent leaks. Uses `TS_ZERO` for timestamps in dry-run. See `src/logging/redact.rs`, `PLAN/40-facts-logging.md`, `SPEC §13`.

- __`fsync(parent)` Bound__ — Operational requirement: `fsync` of the parent directory must occur ≤ 50ms after `rename`. See `SPEC §9`.

- __Golden Fixtures__ — Canonical JSONL outputs for plan/preflight/apply/rollback facts, compared byte-for-byte in CI with a zero‑SKIP gate. See `SPEC §12`.

- __Bronze/Silver/Gold/Platinum Tier__ — Maturity levels that apply to a mechanism’s process (e.g., golden fixtures, smoke tests, rollback). “Gold tier” is not the same as “golden fixtures”. See `DOCS/GOLDEN_FIXTURES.md` → Terminology Disambiguation.

- __Health Verification (Smoke Tests)__ — Minimal post-apply command suite; any failure triggers auto-rollback unless explicitly disabled by policy. See `SPEC §2.9`, `SPEC §11`.

- __LockGuard__ — Opaque guard returned by a `LockManager` to serialize mutations. Released on drop. See `SPEC §3.2`, `PLAN/50-locking-concurrency.md`.

- __LockManager__ — Adapter that enforces serialized mutation with bounded wait and timeout. Required in production; omission allowed only in dev/test (emits WARN). See `SPEC §2.5`, `§14`.

- __`no_lock_manager` Fact__ — A `apply.attempt` fact warning emitted when no `LockManager` is provided (allowed in dev/test only). See `src/api/apply.rs`.

- __FileLockManager__ — Reference `LockManager` implementation using file locks (`fs2`). Intended for dev/test; production may supply another implementation. See `src/adapters/lock_file.rs`.

- __OwnershipOracle__ — Adapter that provides provenance/ownership of targets (`uid`, `gid`, and optional `pkg` origin) for strict ownership gating and facts enrichment. See `src/adapters/ownership.rs`, `src/adapters/ownership_default.rs`.

- __Partial Restoration__ — State where rollback could not fully restore previous topology; must be captured in facts with guidance. See `SPEC §2.2`.

- __PathResolver__ — Adapter that resolves binaries/paths (e.g., to discover providers). See `SPEC §3.2`.

- __Plan__ — Ordered set of actions derived from `PlanInput`, with deterministic `plan_id` and per-action `action_id`. See `SPEC §3.1`, `PLAN/70-pseudocode.md`.

- __Plan Input (`PlanInput`)__ — User input describing targets and providers plus policy flags; normalized to ensure determinism. See `PLAN/10-types-traits.md`.

- __Plan/Action IDs (`plan_id`, `action_id`)__ — UUIDv5 identifiers computed from normalized inputs and stable namespaces. See `SPEC §2.7`, `PLAN/35-determinism.md`.

- __Preflight__ — Safety and policy checks performed prior to mutations, producing a structured diff. Fail-closed on critical violations unless overridden by explicit policy. See `SPEC §4`, `§2.3`, `PLAN/45-preflight.md`.

- __Preflight Diff (YAML)__ — Deterministically ordered rows with keys: `action_id`, `path`, `current_kind`, `planned_kind`, `policy_ok`, `provenance.{uid,gid,pkg}`, `notes`. See `SPEC/preflight.yaml`.

- __`policy_ok`__ — Boolean field in each preflight row summarizing whether policy gates passed for that action. When `false`, `apply()` must refuse to proceed unless a policy override is set. See `src/api/preflight.rs`, `PLAN/45-preflight.md`.

- __Override Preflight (`override_preflight`)__ — Policy switch that allows proceeding despite `policy_ok=false` (for controlled contexts). Its use must be explicit and logged. See `PLAN/45-preflight.md`, `src/policy/config.rs`.

- __Gating (Policy / Preflight Gating)__ — The act of evaluating preconditions (filesystem flags, ownership, preservation capabilities, allowed/forbidden paths) and blocking execution when violations are detected (fail‑closed), unless explicitly overridden. See `PLAN/45-preflight.md`, `src/api/preflight.rs`.

- __Preservation Capabilities__ — Filesystem support for preserving owner, mode, timestamps, xattrs, ACLs, capabilities; probed in preflight and gated by policy. See `SPEC §2.3`.

- __Provenance__ — Origin metadata for changes: `origin` (`repo|aur|manual`), `helper`, `uid`, `gid`, `pkg`, `env_sanitized`. Emitted in facts and subject to masking. See `audit_event.schema.json`, `SPEC §2.4`.

- __Rescue Profile__ — Always-available backup symlink set and verified fallback toolset (GNU or BusyBox) present on `PATH` to recover from failures. See `SPEC §2.6`.

- __Rollback__ — Automatic reverse-order restoration of prior state on failure; idempotent; facts must record partial restoration if any step fails. See `SPEC §2.2`.

- __Safe Copy + Fsync + Rename__ — Degraded replacement path used when `renameat` returns EXDEV (cross-filesystem), preserving atomic visibility within policy constraints. See `SPEC §2.10`.

- __SafePath__ — Path type with invariants preventing traversal/escape and enabling TOCTOU-safe operations via parent directory handles. All mutating APIs require `SafePath`. See `SPEC §3.3`, `src/types/safepath.rs`.

- __Schema Version (`schema_version`)__ — Version tag in each fact; current is `1`. Changes require migration and dual-emit periods. See `SPEC §5`, `§13`.

- __Secret Masking__ — Redaction policy to prevent sensitive data exposure in facts across all sinks. See `SPEC §2.4`, `§13`, `src/logging/redact.rs`.

- __Severity / Decision / Stage__ — Core fact fields describing outcome and context of an event. `stage ∈ {plan, preflight, apply.attempt, apply.result, rollback}`, `decision ∈ {success, failure, warn}`, `severity ∈ {info, warn, error}`. See `audit_event.schema.json`.

- __SmokeTestRunner__ — Adapter that executes the minimal smoke test suite after apply; controls auto‑rollback behavior via policy. See `SPEC §3.2`, `§11`.

- __Switchyard__ — The library crate that orchestrates safe, atomic, reversible filesystem swaps with strong auditability and recovery guarantees. See `SPEC §0`.

- __Thread-safety (`Send + Sync`)__ — Core types are thread-safe; multiple threads may invoke `apply()`, but only one mutator proceeds under lock in production. See `SPEC §14`.

- __TOCTOU-safe Sequence__ — Normative syscall sequence for mutations: open parent with `O_DIRECTORY|O_NOFOLLOW` → `openat` final component/prepare staged artifact → `renameat` into place → `fsync(parent)` within bound. See `SPEC §3.3`.

- __Traceability__ — Mapping between requirements and tests/fixtures; report tracked in `SPEC/traceability.md`. See `SPEC/traceability.md`.

- __UUIDv5 Namespace__ — Stable, project-defined namespace used for deterministic `plan_id` and `action_id`. See `PLAN/35-determinism.md`, `PLAN/adr/ADR-0006-determinism-ids.md`.

- __`TS_ZERO`__ — Constant timestamp string `"1970-01-01T00:00:00Z"` used during dry-run and redaction to ensure deterministic outputs. See `src/logging/redact.rs`.

- __Silver Tier (Exit Codes)__ — Sprint‑level maturity target for exit code mapping: a curated subset of `ErrorId`s is fully mapped and tested (`E_LOCKING`, `E_POLICY`, `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_SMOKE`), while others remain provisional per ADR‑0014. See `DOCS/EXIT_CODES_TIERS.md`, `PLAN/30-errors-and-exit-codes.md`, `PLAN/adr/ADR-0014-exit-codes-deferral.md`.
