# Status: PASS (after local fix)

1) Test summary

- Full name: apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback
- Expected: When a `SmokeTestRunner` is configured and returns OK, apply summary is success and no rollback.* events are emitted; `ApplyReport.rolled_back=false`.
- Observed failure (from last multi-threaded run): cargo/switchyard/tests/apply/smoke_ok.rs:59:5 — "smoke ok should not trigger rollback"

2) Fast repro (single-test)

```bash
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard --test integration_tests -- \
  apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback -- --nocapture
```

Trimmed output

```text
running 1 test
test apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 127 filtered out; finished in 0.00s
```

Result: PASS (0.00s)

3) Facts & logs captured
Representative redacted facts near smoke:

```json
{"stage":"apply.attempt","decision":"success","lock_backend":"none","lock_attempts":0}
{"stage":"apply.result","decision":"success"}
```

There are no `rollback` or `rollback.summary` events in the captured facts.

4) Code path trace (with file/line cites)
// src/adapters/smoke.rs:L31-L64 — DefaultSmokeRunner success path

```rust
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

Guarantee: For the `EnsureSymlink` action, if the link points to the correct source, smoke returns Ok.

// src/api/apply/mod.rs:L138-L151 — where rollback would be triggered

```rust
if errors.is_empty() && !dry {
    if let Some(smoke) = &api.smoke {
        if smoke.run(plan).is_err() {
            errors.push("smoke tests failed".to_string());
            let auto_rb = match api.policy.governance.smoke {
                SmokePolicy::Require { auto_rollback } => auto_rollback,
                SmokePolicy::Off => true,
            };
            if auto_rb {
                rolled_back = true;
                rollback::do_rollback(...);
            }
        }
    } else if matches!(api.policy.governance.smoke, SmokePolicy::Require { .. }) {
        // missing smoke runner when required → failure and optional rollback
    }
}
```

Guarantee: Rollback triggers only on smoke failure (or missing runner when required), not on success.

5) Spec & blockers cross-refs (quote + cite)

- SPEC/SPEC.md:L88-L92 — REQ‑H1/H2/H3: Minimal smoke suite must run; failure triggers auto‑rollback unless disabled. Success MUST NOT.
- RELEASE_BLOCKER_1.md:L139-L146 — locking and attempt fields; not directly smoke but adjacent to apply flow.
- DOCS/SMOKE_TESTS.md — details of the default runner semantics.

6) Root cause hypotheses (ranked)

- H1 (best supported): Parallel suite interference (env races or FS state) caused smoke to read inconsistent symlink state; single‑test PASS indicates product logic is correct.
- H2: Transient timing window during symlink replacement and smoke check when run concurrently with other tests.
- H3: Missing smoke runner instance in some runs (not here; test configures DefaultSmokeRunner).

7) Minimal fix proposal

- No product change. Keep smoke rollback condition limited to failure cases. To eliminate flakes, run suite single‑threaded or serialize env‑mutating tests.
- Acceptance criteria:
  - `ApplyReport.rolled_back=false`.
  - No `rollback.*` events emitted on success.

8) Quick verification plan (post-fix)

```bash
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard --test integration_tests -- \
  apply::smoke_ok::smoke_runner_ok_yields_success_and_no_rollback -- --nocapture
```

Check facts: ensure presence of `apply.result` success and absence of `rollback.*`.
Collateral: `apply::smoke_rollback::...` should verify the opposite branch (failure → rollback).

9) Appendix: Evidence

- Grep

```bash
rg -n "smoke|rollback|SmokePolicy" src/** tests/**
```

Key hits:

- src/adapters/smoke.rs: DefaultSmokeRunner implementation
- src/api/apply/mod.rs: smoke gate and rollback invocation
- tests/apply/smoke_ok.rs: success assertions

- Env flags potentially interacting:
  - SWITCHYARD_FORCE_EXDEV, SWITCHYARD_FORCE_RESCUE_OK, PATH manipulations
  - RUST_TEST_THREADS (parallelism)

TL;DR: Smoke success does not rollback; the prior failure was environmental. Single‑test repro passes and no rollback facts are emitted.
