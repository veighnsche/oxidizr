# Shared Temp Roots & Filesystem State Contention (Parallel Apply/Smoke)

## 1) Summary

Multiple tests manipulate the same synthetic paths under a shared temporary root (e.g., `<tmp>/usr/bin/app`) while running in parallel and without a lock manager. Concurrent apply/smoke operations contend on those paths, creating transient symlink states that trip assertions (including unexpected rollback or missing fields). All affected tests pass in isolation, indicating the product code is sound; the instability arises from shared FS state across parallel tests.

## 2) Failing tests explained by this cause

- apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path — Concurrency on backup/restore test paths can produce unexpected intermediate states in parallel runs.
- apply::sidecar_integrity_disabled::e2e_apply_020_sidecar_integrity_disabled_tolerates_tamper — Sidecar/backup artifacts created under a shared tmp root contend across tests, perturbing success expectations.
- apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback — Parallel symlink replacement and smoke validation on the same target causes a brief mismatch that (in flakes) triggers rollback.

## 3) Evidence (quotes + snippets with paths/lines)

- Plain text (from dossiers)

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_1_ENOSPC_backup_restore.md:83-84
H2: Transient FS state from other tests (long path or temp dirs) affected smoke or symlink checks.

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_4_Sidecar_integrity_disabled.md:86-87
H3: Transient filesystem state (temp dirs) affecting backup/sidecar creation; not reproduced single-threaded.

/home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_5_Smoke_OK_no_rollback.md:95-97
H2: Transient timing window during symlink replacement and smoke check when run concurrently with other tests.
```

- Relevant SPEC/requirements (concurrency/locking)

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/SPEC/requirements.yaml:490-497 (REQ-T2)
"apply() MAY be called from multiple threads, but only one mutator proceeds at a time under the LockManager. Without a LockManager, concurrent apply is unsupported."

/home/vince/Projects/oxidizr-arch/cargo/switchyard/SPEC/SPEC.md:64-69 (REQ-L1–L5 summary)
Locking required in production; WARN when no lock manager; facts include lock_wait_ms and (optionally) lock_attempts.
```

- Blockers corroborating lock WARN pattern

```text
/home/vince/Projects/oxidizr-arch/cargo/switchyard/RELEASE_BLOCKER_2.md:62-70
Current implementation emits an apply.attempt WARN with lock_backend="none", no_lock_manager=true, and lock_attempts=0 when policy allows unlocked commit.
```

- Shared code path that is sensitive to concurrent FS state (smoke runner)

```rust
// /home/vince/Projects/oxidizr-arch/cargo/switchyard/FAILING_TESTS/TEST_5_Smoke_OK_no_rollback.md:41-58
impl SmokeTestRunner for DefaultSmokeRunner {
    fn run(&self, plan: &Plan) -> Result<(), SmokeFailure> {
        for act in &plan.actions {
            if let crate::types::Action::EnsureSymlink { source, target } = act {
                let md = std::fs::symlink_metadata(target.as_path()).map_err(|_| SmokeFailure)?;
                if !md.file_type().is_symlink() { return Err(SmokeFailure); }
                let link = std::fs::read_link(target.as_path()).map_err(|_| SmokeFailure)?;
                let resolved = if link.is_relative() {
                    match target.as_path().parent() { Some(p) => p.join(link), None => link }
                } else { link };
                let want = std::fs::canonicalize(source.as_path()).unwrap_or_else(|_| source.as_path().clone());
                let got = std::fs::canonicalize(&resolved).unwrap_or(resolved);
                if want != got { return Err(SmokeFailure); }
            }
        }
        Ok(())
    }
}
```

## 4) Mechanism of failure (step-by-step)

1) Tests create and mutate the same canonicalized paths under a shared temp root (e.g., `<tmp>/usr/bin/app`).
2) Without a LockManager (optional in tests), multiple apply() executions interleave across threads.
3) One test briefly unlinks/replaces the symlink while another’s smoke runner or assertion reads it.
4) The second test observes a non-symlink or a symlink pointing to the previous/next source and reports failure; on retries or single-threaded runs, the window disappears and the test passes.

## 5) Minimal manual fix options (you won’t implement them)

- Option A (preferred, low risk): Per-test unique temp roots
  - Introduce a `TestRoot` helper that creates an isolated `tempfile::TempDir` per test and threads it into `Switchyard::new(...)` and plan builders.
  - Ensure each test resolves `source/target` under its own root (e.g., `<tmp>/T_<uuid>/usr/bin/app`).
  - Blast radius: low; test-only.

- Option B (alternate, medium risk): Enable a test LockManager
  - Provide a lightweight, in-process `LockManager` for tests and set policy to `Optional` but present the manager so only one mutator proceeds.
  - Add a short bounded timeout to avoid deadlocks; assert WARN absence when manager is present.
  - Blast radius: medium; affects test timings and attempt facts.

## 6) Acceptance criteria (verifiable, testable)

- Zero flakes across 10 full-suite runs with normal threading.
- No `rollback.*` events on `smoke_ok` success; `ApplyReport.rolled_back=false` consistently.
- Sidecar/backup-related tests (`sidecar_integrity_disabled`) show stable `apply.result` success without dependency on run order.
- Locking WARN (`apply.attempt` with `decision=warn`, `lock_backend="none"`) appears only when no manager is configured per policy.

## 7) Collateral impact (what else this touches)

- Tests likely to have updated fixtures/expectations: `apply::smoke_ok::*`, `apply::sidecar_integrity_disabled::*`, any other tests sharing `<tmp>/usr/bin/*`.
- De-risks SPEC clauses: REQ-T2 (single mutator under lock), REQ-H1/H2/H3 (smoke semantics remain intact once contention is removed).

## TL;DR

- Parallel tests share FS roots and contend on the same symlink/backup paths.
- This creates transient states that cause flakes (including unexpected rollback) in smoke and sidecar tests.
- Isolate each test’s temp root or add a test LockManager to eliminate contention.
- Product code paths are correct; this is a test harness isolation issue.
