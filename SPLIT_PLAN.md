# SPLIT_PLAN.md

## Current Responsibility Map

Per-module classification and where each concern currently lives:

- Safety primitives (single authorities)
  - `src/symlink/ops.rs`: atomic symlink swap using renameat, backups, restore, path safety.
  - `src/system/fs_checks.rs`: mount preflights (rw, noexec), immutable bit check, source trust checks.
  - `src/system/lock.rs`: single-instance process locking.
  - `src/state/mod.rs`: persist and update enabled experiments and managed targets; write state report.
  - `src/logging/{init.rs,audit.rs}`: human logs and structured JSONL audit events (`audit_event_fields`, `AuditFields`).

- Experiment/Distro layer
  - `src/experiments/{mod.rs,coreutils.rs,findutils.rs,checksums.rs,sudors.rs,util.rs}`: discovery, link plan, enable/disable/remove, experiment registry.
  - `src/system/worker/{packages.rs,aur.rs,distro.rs,fs_ops.rs}` and `src/system/worker.rs`: package policy, AUR gating, repo/distro detection, which/path lookups, replace/restore wrappers.

- CLI/UX
  - `src/main.rs`: parse CLI, init logging, delegate to `cli::handle_cli()`.
  - `src/cli/{parser.rs,handler.rs}`: commands, flags, routing to experiments; relink-managed; install-hook.
  - `src/ui/progress.rs`: progress bar rendering and quiet mode for symlink actions.

- Tests & tooling
  - `tests/` YAML suites and Rust tests.
  - `test-orch/` host-orchestrator and container-runner for Docker E2E.

ASCII call graph from main to symlink ops/state/audit (typical enable):

```
main
 └─ cli::handle_cli()
    ├─ logging::init (structured JSONL + human logs)
    ├─ Worker::new(...flags)
    └─ experiments::all_experiments()/Experiment::enable
       ├─ check_compatible (checks::SUPPORTED_DISTROS | skip)
       ├─ check_download_prerequisites(worker)
       │   ├─ worker.extra_repo_available()/repo_has_package()/aur_helper_name()
       │   └─ audit_event_fields("experiments","repo_capabilities",...)
       ├─ worker.install_package(...)
       │   ├─ pacman -S / AUR helper exec (policy in packages.rs)
       │   └─ audit_event_fields("worker","install_package.*",...)
       ├─ discover_applets(worker)/log_applets_summary
       ├─ util::create_symlinks(worker, applets, resolve_usrbin)
       │   ├─ worker.replace_file_with_symlink(src, dst)
       │   │   ├─ fs_checks::ensure_mount_rw_exec/check_immutable/check_source_trust
       │   │   └─ symlink::replace_file_with_symlink(src, dst, dry_run)
       │   └─ audit_event_fields("symlink","create",...)
       ├─ state::set_enabled(...)
       └─ logging/audit events at each decision point
```

## Target Architecture

We split into two layers with strict boundaries while preserving today’s behavior:

- Safety Core (reusable across distros/products)
  - Filesystem & symlink primitives: atomic swap, backup/restore, path safety.
  - Mount/immutability/trust preflights.
  - Process lock management.
  - State persistence/reporting primitives.
  - Structured logging/audit sink and fields (and later, attestation buffer/signature optional feature).
  - Path resolution utility (to supplant external `which`) behind a single function.

- Experiment Layer (distro/product-specific)
  - Package policy and install/remove (repo-first, AUR opt-in).
  - Distro and repo detection.
  - Experiment registry and orchestration, discovery, plan building, linking.
  - CLI/UX orchestration and user interaction.

Public API surface (Core)

- `core::fs` (traits and helpers)
  - `trait FsOps { fn replace_symlink(&self, src: &Path, dst: &Path) -> Result<()>; fn restore_file(&self, dst: &Path) -> Result<()>; }`
  - `fn ensure_mount_rw_exec(path: &Path) -> Result<()>`
  - `fn check_immutable(path: &Path) -> Result<()>`
  - `fn is_safe_path(path: &Path) -> bool`
  - `fn rename_active_pointer(active: &Path, new_target: &Path) -> Result<()>` (atomic pointer flip; to be added)

- `core::audit`
  - `struct AuditFields { .. }`
  - `fn audit_event_fields(subsystem, event, decision, fields) -> Result<()>`
  - (Later) `struct OpBuffer`, `fn start_op()`, `fn finalize_op(signing: bool) -> Result<OpSummary>`

- `core::state`
  - `fn load_state(override_dir) -> State`
  - `fn save_state(override_dir, state, dry_run) -> Result<()>`
  - `fn set_enabled(override_dir, dry_run, experiment, enabled, managed_targets) -> Result<()>`

- `core::lock`
  - `fn acquire() -> Result<LockGuard>`

- `core::path`
  - `fn which(name: &str) -> Result<Option<PathBuf>>` (initially wraps external crate; migrates to internal search per Stream E)

Public API surface (Experiment Layer)

- `worker::packages`
  - `fn update_packages(&self, assume_yes) -> Result<()>`
  - `fn repo_has_package(&self, package) -> Result<bool>`
  - `fn check_installed(&self, package) -> Result<bool>`
  - `fn install_package(&self, package, assume_yes, reinstall) -> Result<()>` (policy gate)
  - `fn remove_package(&self, package, assume_yes) -> Result<()>`

- `worker::distro`
  - `fn distribution(&self) -> Result<checks::Distribution>`
  - `fn extra_repo_available(&self) -> Result<bool>`

- `worker::aur`
  - `fn aur_helper_name(&self) -> Result<Option<String>>`
  - `fn ensure_aur_preflight(&self, assume_yes) -> Result<()>`

- Experiments
  - `Experiment::enable/disable/remove(list_targets)`
  - `util::{create_symlinks, restore_targets, log_applets_summary, resolve_usrbin}`

Error taxonomy

- Maintain `crate::error::{Error, Result}` as single source.
- Core-specific variants (e.g., `Error::FsImmutable`, `Error::MountNotWritable`, `Error::AuditIo`) are internal details; mapped to existing product-level codes:
  - 10 Incompatible distro; 20 NothingToLink; 30 RestoreMissing; 40 RepoGateFailed; etc.
- Preserve existing `Result` type and error mapping at layer boundaries.

Event types & schema

- Keep JSONL envelope fields (`ts, component="product", subsystem, level, run_id, container_id, distro, event, decision, ...`).
- Only `AuditFields` carries structured extras (cmd, rc, artifacts, target, source).
- Any new Core events must use the same schema; Experiment Layer continues to log via `audit_event_fields`.

## Monorepo Option

- Extract Safety Core into `oxidizr-core` (separate crate/repo) with features:
  - `attestation` (op buffer + signing), `selinux`, `xattr`, `caps`, `internal-which`.
- `oxidizr-arch` (current product) depends on `oxidizr-core` with a pinned semver range, consuming only public APIs.
- Stability policy
  - `oxidizr-core` follows semver; minor releases may add fields/APIs but do not break existing consumers.
  - Audit JSONL schema stability guaranteed; additions are additive and optional.
- Versioning & CI
  - Tag `oxidizr-core` releases; product CI pulls tagged releases.
  - Docs for migration between versions (changelog, ADRs).

## No-Spaghetti Rules

- Core MUST NOT import from `experiments::*` or `system/worker::{packages,aur,distro}`.
- Experiment Layer MUST NOT perform atomic swap/backup/restore directly; it MUST call Core APIs.
- Single authorities enforced:
  - One symlink/backup/restore implementation (Core).
  - One logging sink (Core audit).
  - One PATH lookup (`core::path::which()` via Worker adapter).
  - One package policy path (Worker::install_package).
- Imports allowed:
  - Experiments -> Worker (package ops) and Core (fs ops, audit, state).
  - Worker -> Core (fs checks, audit, lock, path).

## Compatibility Guarantees

- CLI surface unchanged; any new flags come later via Streams (B, D) according to plans.
- Exit codes preserved (0,1,10,20,30,40) and mapped from the same error variants.
- Audit JSONL schema preserved; existing dashboards/consumers unaffected.
- State JSON schema preserved; relink-managed and pacman hook behavior identical.
- Backward-compatible shims:
  - `audit_event` remains available but implemented in terms of `audit_event_fields`.
  - `Worker.which()` forwards to Core path search; when the external crate is removed, call sites unchanged.

## Testing Impact

- Unit tests move to match module boundaries but remain functionally identical.
- Integration/E2E tests (Docker orchestrators, YAML suites) unchanged; they validate behavior remains identical post-split.
- Additional tests added for Core APIs (atomic rename invariants, mount/immutable/trust preflights) and for error mapping.

## Risk Register & Mitigations

- Logging drift: mitigate with snapshot tests of JSONL lines for key flows; keep a schema contract doc.
- Partial restore risk: add integration tests to verify idempotent restore and alias cleanup (sudors).
- PATH lookup divergence: keep dual-impl (feature-gated) while validating equivalence tests vs external crate.
- API leakage across layers: enforce module visibility and add `#[deny(unreachable_pub)]` and `#[deny(unused_pub)]` in Core.
- Circular deps: prevent by keeping Core free of Experiment/Worker imports; Worker depends on Core.
- Repo-policy regressions: rely on Stream D test matrix and `install_package` provenance audit checks.

## Decision Log (ADRs)

- ADR-001: Safety Core vs Experiment Layer boundary selected to isolate safety-critical operations from distro policy.
- ADR-002: `audit_event_fields` is the canonical logging API; `audit_event` is a compatibility wrapper.
- ADR-003: PATH lookup centralized; feature-gated migration to internal search per Stream E.
- ADR-004: Pointer flip added to Core (`rename_active_pointer`) for Stream A profiles.
- ADR-005: Package policy remains in product Worker to avoid pulling distro-specific code into Core.

---

## Appendix: External-Dependency Reconnaissance (non-binding)

- Linux capabilities
  - `caps` (pure Rust): Active maintenance; suitable to read/set caps without shelling out. License: MIT/Apache-2.0.
  - `capabilities`/`libcap` bindings: Closer to kernel ABI; maintenance varies. Prefer pure-Rust where possible.
- Extended attributes (xattrs)
  - `xattr`: Widely used for get/set/list; appears maintained. Suitable for label and metadata probes if needed.
- Atomic file ops / openat/renameat helpers
  - `openat`: Provides dirfd/openat helpers. Consider for parent `O_DIRECTORY|O_NOFOLLOW` semantics.
  - `atomic_write_file`: Focused on data file atomic writes; less relevant to symlink swap but informative.
  - Rust std rename caveats: ensure fsync of parent dir after `renameat` to persist directory updates.
- SELinux labels
  - `selinux` / `selinux-sys`: Bindings exist; maintenance mixed. Consider feature-gating; fallback to detection-only.
- Upstream coreutils context
  - `uutils/coreutils`: Active project; informs experiment rationale and dispatch/unified-binary behaviors.
  - Distro notes (Ubuntu/OSNews): migration lessons; ensure we keep checksum applets gated as we already do.
