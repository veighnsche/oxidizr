# High-Level Pseudocode (Planning Only; No Rust Implementation)

This document sketches the algorithmic flow for Switchyard core functions. It uses Rust-like terminology but is not compilable code.

## Orchestration Entrypoints (api.rs)

```rust
fn plan(input: PlanInput) -> Plan
  assert input.targets not empty
  normalize = normalize_input(input)                 // canonicalize paths, policy
  plan_id = uuidv5(NAMESPACE, serialize(normalize)) // deterministic
  actions = []
  for t in normalize.targets:
    current = inspect_target(t)                      // file/dir/symlink/missing
    desired = compute_desired(t, normalize.providers)
    kind = decide_action_kind(current, desired)
    action_id = uuidv5(plan_id, t)
    actions.push(Action{ action_id, path: t, kind, metadata })
  return Plan{ plan_id, actions }

fn preflight(plan: &Plan) -> PreflightReport
  rows = []
  for a in plan.actions:
    policy_ok = check_policy(a)                      // ownership, preservation, fs flags
    provenance = capture_provenance(a)
    notes = []
    rows.push(PreflightRow{ action_id:a.action_id, path:a.path, current_kind, planned_kind:a.kind, policy_ok, provenance, notes })
  ensure_rows_deterministic(rows)
  return PreflightReport{ rows }

fn apply(plan: &Plan, mode: ApplyMode, adapters: &Adapters) -> ApplyReport
  if production and no LockManager: emit WARN fact; UNSUPPORTED concurrent apply
  guard = maybe_acquire_lock(adapters.lock_manager, timeout)
  emit_fact(stage="apply.attempt", plan_id=plan.plan_id)
  backups = []
  for a in plan.actions:
    emit_fact(stage="apply.attempt", action_id=a.action_id, path=a.path)
    if mode == DryRun:
      emit_fact(stage="apply.result", decision="success", dry_run=true)
      continue
    result = match a.kind:
      ReplaceSymlink => replace_symlink(a, &mut backups)
      RestoreFromBackup => restore_backup(a)
      Skip => Ok
    if result is Err:
      emit_fact(decision="failure")
      rollback(backups)                              // best-effort reverse order
      return ApplyReport{ decision: Failure, partial_restoration_state }
    else:
      emit_fact(decision="success")
  run_smoke = adapters.smoke_runner.run(plan)
  if run_smoke is Err and not policy.disable_auto_rollback:
    emit_fact(E_SMOKE)
    rollback(backups)
    return ApplyReport{ decision: Failure, cause: Smoke }
  emit_fact(stage="apply.result", decision="success")
  return ApplyReport{ decision: Success }

fn plan_rollback_of(report: &ApplyReport) -> Plan
  // Use recorded backups and action order to construct reverse plan
  actions = derive_rollback_actions(report)
  rid = uuidv5(NAMESPACE, serialize(report))
  return Plan{ plan_id: rid, actions }
```

## Filesystem Primitives (fs/*)

```rust
fn replace_symlink(a: &Action, backups: &mut Vec<Backup>) -> Result<(), Error>
  // SafePath invariant holds; TOCTOU-safe sequence enforced
  parent = open_parent_dir_no_follow(a.path)
  staged = stage_new_symlink(a)
  backup = backup_existing_if_any(a.path)
  atomic_rename(parent, staged, a.path)             // renameat
  fsync_parent(parent) within 50ms
  record_hashes(before, after)
  backups.push(backup)
  Ok

fn restore_backup(a: &Action) -> Result<(), Error>
  parent = open_parent_dir_no_follow(a.path)
  src = locate_backup(a)
  atomic_rename(parent, src, a.path)
  fsync_parent(parent)
  Ok

fn atomic_rename(parent_dir, staged, target) -> Result<(), Error>
  if same_filesystem(staged, target):
    renameat(staged, target)
  else:
    if not policy.allow_degraded_fs: return Err(E_EXDEV)
    copy_file(staged, target_tmp)
    fsync(target_tmp)
    renameat(target_tmp, target)
  fsync_parent(parent_dir)
  Ok
```

## Preflight (preflight.rs)

```rust
fn check_policy(a: &Action) -> bool
  // Ownership, filesystem flags (ro,noexec,immutable), preservation capabilities
  owner_ok = adapters.ownership.owner_of(a.path).is_root_owned_and_not_world_writable()
  fs_ok = check_filesystem_flags(a.path)
  preservation_ok = probe_preservation_capabilities()
  return owner_ok && fs_ok && preservation_ok || explicit_policy_override
```

## Detailed Algorithms (planning)

```rust
fn normalize_input(input: PlanInput) -> Normalized
  // Ensure SafePath for every path-carrying field
  targets   = sort([ SafePath::from_rooted(root, t) for t in input.targets ])
  providers = sort([ SafePath::from_rooted(root, p) for p in input.providers ])
  policy    = canonicalize_policy(input.policy)        // explicit booleans, stable order
  return { targets, providers, policy }

fn inspect_target(path: SafePath) -> String
  // Return one of: missing|file|dir|symlink
  parent = path.open_parent_dir_no_follow()
  ent = fstatat(parent, final_component(path))
  if ent.not_found: return "missing"
  if ent.is_symlink: return "symlink"
  if ent.is_dir: return "dir"
  if ent.is_file: return "file"

fn compute_desired(t: SafePath, providers: Vec<SafePath>) -> SafePath
  // Choose provider according to policy and PathResolver
  // Stable selection: first provider whose basename matches t basename, else providers[0]
  candidates = [p for p in providers if basename(p)==basename(t)]
  return (candidates.first_or_default() ?? providers[0])

fn decide_action_kind(current: String, desired: SafePath) -> ActionKind
  if current == "symlink" and readlink(t) == desired: return Skip
  if current == "missing" or current == "symlink" or current == "file": return ReplaceSymlink
  if current == "dir": return ReplaceSymlink        // policy may reject; preflight will gate

fn maybe_acquire_lock(lock_mgr, timeout_ms) -> Option<LockGuard>
  if lock_mgr is None:
    emit_fact(stage="apply.attempt", decision="warn", severity="warn", msg="No LockManager; UNSUPPORTED")
    return None
  start = now_ms()
  guard = lock_mgr.acquire_process_lock(timeout_ms)   // bounded wait
  waited = now_ms() - start
  emit_fact(stage="apply.attempt", decision="success", lock_wait_ms=waited)
  return Some(guard)

fn record_hashes(before, after)
  // Compute sha256; store in facts per step
  emit_fact(hash_alg="sha256", before_hash=sha256(before), after_hash=sha256(after))

fn run_smoke_suite(smoke, plan, policy)
  res = smoke.run(plan)
  if res is Err and !policy.disable_auto_rollback:
    return Err(E_SMOKE)
  return Ok

fn finalize_attestation(attestor, bundle_bytes)
  sig = attestor.sign(bundle_bytes)  // ed25519
  emit_fact(stage="apply.result", decision="success", attestation=sig)

fn derive_rollback_actions(apply_report) -> Vec<Action>
  // Read per-step facts/backups and invert ReplaceSymlink into RestoreFromBackup in reverse order
  actions = []
  for b in apply_report.backups.reverse():
    actions.push(Action{ action_id=uuidv5(apply_report.plan_id, b.path.rel), path=b.path, kind=RestoreFromBackup, metadata={} })
  return actions
```

## Facts and Determinism (logging/, determinism/)

```text
fn emit_fact(fields)
  mask_secrets(fields)
  stable_order(fields)
  fields.schema_version = 1
  write_jsonl(fields)

fn normalize_input(input) -> Normalized
  // Normalize paths (SafePath), sort lists for stability, redact timestamps for dry-run
```
