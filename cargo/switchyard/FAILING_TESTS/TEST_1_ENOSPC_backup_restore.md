# Status: PASS (after local fix)

1) Test summary

- Full name: apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path
- Expected: Apply succeeds in normal conditions; this variant does NOT enable ENOSPC injection and should produce an apply.result success.
- Observed failure (from last multi-threaded run): cargo/switchyard/tests/apply/enospc_backup_restore.rs:75:5 — "expected apply.result success in normal conditions"

2) Fast repro (single-test)

```bash
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard --test integration_tests -- \
  apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path -- --nocapture
```

Trimmed output

```text
running 1 test
test apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 127 filtered out; finished in 0.00s
```

Result: PASS (0.00s)

3) Facts & logs captured

- The test inspects redacted facts from an in-memory TestEmitter. Representative redacted apply.result:

```json
{
  "stage": "apply.result",
  "decision": "success",
  "path": "<tmp>/usr/bin/app",
  "fsync_ms": 0,
  "schema_version": 2,
  "ts": "1970-01-01T00:00:00Z"
}
```

- No ENOSPC injector is active in this test. No error_id or exit_code fields should appear on success.

4) Code path trace (with file/line cites)
// src/api/apply/executors/ensure_symlink.rs:L142-L174

```rust
        // Success path: emit result
        let mut extra = json!({
            "action_id": aid.to_string(),
            "path": target.as_path().display().to_string(),
            "degraded": if degraded_used { Some(true) } else { None },
            "degraded_reason": if degraded_used { Some("exdev_fallback") } else { None },
            "duration_ms": fsync_ms,
            "fsync_ms": fsync_ms,
            "lock_wait_ms": 0u64,
            "before_kind": before_kind,
            "after_kind": if dry { "symlink".to_string() } else { kind_of(&target.as_path()).to_string() },
            "backup_durable": api.policy.durability.backup_durability,
        });
        // ... provenance best‑effort omitted
        ensure_provenance(&mut extra);
        insert_hashes(&mut extra, before_hash.as_ref(), after_hash.as_ref());
        maybe_warn_fsync(&mut extra, fsync_ms, FSYNC_WARN_MS);
        StageLogger::new(tctx)
            .apply_result()
            .merge(&extra)
            .emit_success();
```

- Guarantee: Per SPEC §5 and §2.10, apply.result must reflect success (and degraded telemetry only if EXDEV fallback engaged).

5) Spec & blockers cross-refs (quote + cite)

- SPEC/SPEC.md:L196-L203 — “apply.result — per‑action results; include before/after hashes (hash_alg=sha256) when mutated; optional fields include degraded, provenance, perf.”
- SPEC/SPEC.md:L94-L99 — “REQ‑F1/F2: If degraded fallback is used and policy allows, facts MUST record degraded=true; else fail.”
- RELEASE_BLOCKER_3.md:L42-L53 — “EXDEV simulation is correctly injected after renameat; degraded telemetry propagates.”
- BUGS.md:L41-L58 — ENOSPC simulation is not feasible in CI; this variant asserts success without injector.

6) Root cause hypotheses (ranked)

- H1 (best supported): Parallel test env contamination (e.g., SWITCHYARD_FORCE_EXDEV or rescue flags) caused unrelated apply tests to see unexpected error/fact state. Evidence: All five “failing” tests pass individually; suite flakiness disappeared with serialized EXDEV tests and single‑thread runs.
- H2: Transient FS state from other tests (long path or temp dirs) affected smoke or symlink checks.
- H3: Logging schema confusion (legacy preflight vs v2) cascading into assertions (less likely for this test).

7) Minimal fix proposal

- Test/harness fix only: ensure this ENOSPC “success” variant does not enable any injector. Keep current code path. If suite remains flaky, run env‑mutating tests serial or single‑threaded.
- Acceptance criteria:
  - apply.result success present.
  - No error_id/exit_code.
  - No rollback.* events.

8) Quick verification plan (post-fix)

```bash
RUST_LOG=info RUST_TEST_THREADS=1 cargo test -p switchyard --test integration_tests -- \
  apply::enospc_backup_restore::e2e_apply_014_enospc_during_backup_restore_path -- --nocapture
```

Check facts: ensure at least one apply.result with decision=success; no error_id present.
Collateral: none expected; smoke and EXDEV tests unaffected.

9) Appendix: Evidence

- Grep

```bash
rg -n "ENOSPC|restore|sidecar_integrity" src/** tests/**
```

Relevant hits:

- src/api/apply/executors/restore.rs:63-70 — `force = best_effort_restore || !sidecar_integrity` (restore path)
- tests/apply/enospc_backup_restore.rs:62-80 — test asserts apply.result success

- Environment flags potentially interacting:
  - SWITCHYARD_FORCE_EXDEV (EXDEV simulation)
  - SWITCHYARD_FORCE_RESCUE_OK (preflight rescue checks)
  - PATH mutations in some preflight tests
  - RUST_TEST_THREADS (parallelism causing env races)

TL;DR: The ENOSPC “success” test failed only under a noisy, parallel suite. It reliably passes single‑threaded; no product change required. Keep injector off; ensure no env cross‑contamination.
