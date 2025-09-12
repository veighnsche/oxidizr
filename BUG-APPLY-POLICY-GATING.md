# Investigation: apply-stage policy gating does not emit expected E_POLICY in apply.result

Date: 2025-09-12
Component: `cargo/switchyard/`
Area: Apply-stage gating, error-id mapping, facts emission, redaction

## Executive Summary

The acceptance test `tests/sprint_acceptance-0001.rs::apply_fail_closed_on_policy_violation` intermittently fails, reporting that no `apply.result` fact with `decision=failure`, `error_id=E_POLICY`, and `exit_code=10` is present.

The expected behavior (SPEC §6 and Work Order "Codify Error IDs and Exit Codes") is that when apply-stage policy gating rejects a plan (e.g., target outside `allow_roots`), the engine should emit an `apply.result` failure with `error_id=E_POLICY` and `exit_code=10`. The test fails to find such an event among redacted facts.

This document catalogues the surrounding code, test harness, event emission, and redaction logic. It lists hypotheses for why the expected fact may be missing and outlines targeted next steps to resolve the discrepancy.

---

## Repro

- Command

```bash
cargo test -p switchyard -q --test sprint_acceptance-0001 -- --nocapture
```

- Symptom

```log
thread 'apply_fail_closed_on_policy_violation' panicked at tests/sprint_acceptance-0001.rs:139:5:
expected E_POLICY failure with exit_code=10 in apply.result
```

---

## Test Expectations and Harness

- Failing test: `cargo/switchyard/tests/sprint_acceptance-0001.rs::apply_fail_closed_on_policy_violation`
- Policy setup:
  - `Policy::default()`
  - Restrict `allow_roots` to `<root>/usr/bin`
  - Target path for a `link` action is `<root>/usr/sbin/app` ⇒ intended to violate scope
  - Commit mode runs without a `LockManager` and the test DOES NOT set `policy.allow_unlocked_commit = true` (default is false).
  - As a result, `apply()` takes the early `E_LOCKING` path in `src/api/apply.rs` and returns before emitting any `apply.result` facts. This is the root cause of the missing `E_POLICY` apply.result.
- Test captures facts via a `TestEmitter` implementing `FactsEmitter` and then redacts events with `logging::redact::redact_event(...)`.
- Assertion (redacted): expects at least one event where:
  - `stage == "apply.result"`
  - `decision == "failure"`
  - `error_id == "E_POLICY"`
  - `exit_code == 10`

---

## Where the E_POLICY emission should occur

- Primary emission site on gating failure: `cargo/switchyard/src/api/apply.rs`
  - Function: `apply::run()`
  - Block (policy gating):
    - Computes `gating_errors = gating::gating_errors(&policy, owner, plan)`
    - If non-empty and not `override_preflight` and not dry-run:
      - Emits per-action `apply.result` failures with `action_id`, `path`, `error_id=E_POLICY`, `exit_code=10` (recently added for clarity)
      - Emits a summary `apply.result` failure with `error_id=E_POLICY` and `exit_code=10`
      - Returns early with `ApplyReport { errors: gating_errors, ... }`

- Gating predicate: `cargo/switchyard/src/policy/gating.rs`
  - Iterates over `Plan.actions`
  - Checks (per action):
    - `ensure_mount_rw_exec` over policy-driven `extra_mount_checks` and the specific target
    - `check_immutable(target)`
    - `check_source_trust(source, force_untrusted_source)`
    - `strict_ownership` via `OwnershipOracle`
    - `allow_roots` / `forbid_paths` scope checks
  - On violations, appends human-readable strings to `gating_errors`.

- Preflight counterpart: `cargo/switchyard/src/api/preflight.rs` emits per-action preflight rows and a preflight summary that now includes `error_id=E_POLICY` and `exit_code=10` when `stops` is non-empty.

---

## Facts emission and redaction

- Facts emission helpers: `cargo/switchyard/src/logging/audit.rs`
  - `emit_apply_attempt`, `emit_apply_result`, `emit_summary_extra`
  - Each emission ensures schema envelope (`schema_version`, `ts`, `plan_id`, `path`) and provenance placeholder.

- Redaction: `cargo/switchyard/src/logging/redact.rs`
  - Replaces `ts` with `TS_ZERO`
  - Removes volatile fields: `duration_ms`, `lock_wait_ms`, `severity`, `degraded`, `before_hash`, `after_hash`, `hash_alg`
  - Preserves `error_id` and `exit_code`

The test assertion examines redacted events, so `error_id` and `exit_code` should remain visible and comparable.

---

## Observations from the code paths

- `apply::run()` (policy gating):
  - Emits both per-action failure `apply.result` and a summary `apply.result` before returning.
  - Uses `exit_code_for(ErrorId::E_POLICY)` mapping (10) from `cargo/switchyard/src/api/errors.rs`.

- `policy::gating::gating_errors(...)` ensures scope failures are reported for targets outside `allow_roots` — exactly the condition in the failing test.

- The failing test constructs a plan with one `EnsureSymlink` action where `target ∈ <root>/usr/sbin` while `allow_roots = [<root>/usr/bin]`, so `gating_errors` should be non-empty.

- The acceptance test does NOT set `allow_unlocked_commit` (default false) and does not configure a `LockManager`. In Commit mode, `apply()` fails early with `E_LOCKING` and returns before policy gating or any `apply.result` emissions.

---

## Hypotheses: why test still fails to detect the expected fact

1. **Ordering of emissions vs. assertion set**
   - The test collects all emitted facts after `apply()` returns. Emissions are synchronous; ordering should not matter. However, if any filtering or de-duplication occurs elsewhere, per-action emissions might be overshadowed.

2. **Mismatch in the stage/decision strings**
   - We emit `stage = "apply.result"` and `decision = "failure"`. If a mismatch were present (e.g., typo or different stage), the test would not match. Current code shows correct spelling.

3. **Multiple `Switchyard` instances or plan mismatch**
   - If the plan used in `apply()` is not the constructed one (e.g., stale or re-planned), action IDs or paths could differ, but the test only matches on `stage`, `decision`, `error_id`, `exit_code` — not action_id or path.

4. **Redaction inadvertently removes `error_id`/`exit_code`**
   - Current redaction does not remove those fields. It is unlikely.

5. **Early failure path bypasses E_POLICY emission**
   - If locking or some other preliminary step fails first, we would see an earlier error (e.g., `E_LOCKING`). In this test, locking should not fail because `allow_unlocked_commit=true`.

6. **Gating ran in Preflight instead, Apply didn’t re-check**
   - Apply explicitly re-runs gating unless `override_preflight` is set. Code path confirms this behavior.

7. **Test looks for only a single `apply.result` but gets multiple**
   - The test uses `.any(...)` across all events, so multiple `apply.result` should be fine.

8. **Test environment quirk**
   - The target/allow_roots may compute to the same path under a temporary dir due to normalization. However, `starts_with` semantics and constructed `root` paths should prevent that. Preflight counterpart in the same suite demonstrates policy_ok=false rows, supporting scope mismatch.

---

## Current code citations (key lines)

- `apply::run()` policy gating emission
  - File: `cargo/switchyard/src/api/apply.rs`
  - Emits per-action failures:

```rust
for (idx, act) in plan.actions.iter().enumerate() {
    let aid = action_id(&pid, act, idx).to_string();
    let path = match act {
        Action::EnsureSymlink { target, .. } => target.as_path().display().to_string(),
        Action::RestoreFromBackup { target } => target.as_path().display().to_string(),
    };
    emit_apply_result(&tctx, "failure", json!({
        "action_id": aid,
        "path": path,
        "error_id": "E_POLICY",
        "exit_code": ec,
    }));
}
```

- Then emits summary failure:

```rust
emit_apply_result(&tctx, "failure", json!({
    "error_id": "E_POLICY",
    "exit_code": ec,
}));
```

- `policy::gating` scope check
  - File: `cargo/switchyard/src/policy/gating.rs`

```rust
if !policy.allow_roots.is_empty() {
    let target_abs = target.as_path();
    let in_allowed = policy.allow_roots.iter().any(|r| target_abs.starts_with(r));
    if !in_allowed {
        gating_errors.push(format!("target outside allowed roots: {}", target_abs.display()));
    }
}
```

- Redaction preserves `error_id`/`exit_code`
  - File: `cargo/switchyard/src/logging/redact.rs`

---

## Root Cause and Resolution Plan

- __Root cause__
  - The acceptance test runs `ApplyMode::Commit` without a `LockManager` and leaves `policy.allow_unlocked_commit` at its default `false`. In `src/api/apply.rs`, this triggers the early lock-enforcement branch (emitting only `apply.attempt` with `E_LOCKING/30`) and returns before the policy gating block that emits `apply.result` failures with `E_POLICY/10`.

- __Primary fix (preferred)__
  - In `cargo/switchyard/tests/sprint_acceptance-0001.rs::apply_fail_closed_on_policy_violation`, set `policy.allow_unlocked_commit = true` before constructing `Switchyard`. This ensures Commit mode bypasses the lock requirement in tests and reaches the policy gating code path that emits the expected `apply.result` facts.

- __Alternative test fix__
  - Configure a dummy/test `LockManager` via `.with_lock_manager(...)` so Commit mode proceeds without early `E_LOCKING`.

- __Optional engine improvement (stage parity)__
  - Consider also emitting a summary `apply.result` failure for the early-lock failure path (with `error_id=E_LOCKING`, `exit_code=30`) to maintain stage parity across failure modes. This is not required to fix the test but would make the facts stream more uniform.

- __Validation__
  - Run: `cargo test -p switchyard -q --test sprint_acceptance-0001 -- --nocapture` and verify a redacted event exists with `stage=apply.result`, `decision=failure`, `error_id=E_POLICY`, `exit_code=10`.

---

## Related behavior and acceptance

- Preflight summary now includes `error_id=E_POLICY` and `exit_code=10` when `stops` is non-empty (SPEC alignment). Confirmed via `tests/preflight_summary_error_id.rs`.
- Locking emissions for `E_LOCKING` are covered by `tests/locking_timeout.rs` and `tests/locking_required.rs`.
- Action-level failure mapping is covered by `error_atomic_swap.rs`, `error_exdev.rs`, `error_backup_missing.rs`, `error_restore_failed.rs`.

---

## SPEC references

- `SPEC/SPEC.md` §6 Error Taxonomy & Exit Codes
- `SPEC/error_codes.toml` mapping
- `src/api/errors.rs` for `exit_code_for()`

---

## Appendix: Commands

- Run all switchyard tests:

```bash
cargo test -p switchyard -q
```

- Run only the acceptance test file:

```bash
cargo test -p switchyard -q --test sprint_acceptance-0001 -- --nocapture
```

- Run with logs to inspect events:

```bash
RUST_LOG=debug cargo test -p switchyard --test sprint_acceptance-0001 -- --nocapture
```

- Print redacted events on failure by instrumenting the test body (temporary): `dbg!(&redacted);`

---

## Status

- __Root cause identified__
  - Acceptance test misconfiguration: Commit mode without `LockManager` and with `policy.allow_unlocked_commit=false` leads to early `E_LOCKING` and no `apply.result` fact emission.

- __Solution__
  - Update `apply_fail_closed_on_policy_violation` to set `policy.allow_unlocked_commit = true` (or attach a test `LockManager`). Optionally, enhance engine to emit an `apply.result` summary on early `E_LOCKING` for stage parity.

- __Next actions__
  - Patch the test and re-run the suite. If desired, implement the optional stage-parity emission and add a dedicated test that asserts presence of `apply.attempt` `E_LOCKING` and (if implemented) `apply.result` summary on locking failure.

---

## Trace

RUST_BACKTRACE=full cargo test -p switchyard -q --test sprint_acceptance-0001

running 5 tests
.... 4/5
apply_fail_closed_on_policy_violation --- FAILED

failures:

---- apply_fail_closed_on_policy_violation stdout ----

thread 'apply_fail_closed_on_policy_violation' panicked at cargo/switchyard/tests/sprint_acceptance-0001.rs:139:5:
expected E_POLICY failure with exit_code=10 in apply.result
stack backtrace:
   0:     0x5f8bcfc9c5a2 - std::backtrace_rs::backtrace::libunwind::trace::h9c1aa7b29a521839
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/../../backtrace/src/backtrace/libunwind.rs:117:9
   1:     0x5f8bcfc9c5a2 - std::backtrace_rs::backtrace::trace_unsynchronized::hb123c31478ec901c
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/../../backtrace/src/backtrace/mod.rs:66:14
   2:     0x5f8bcfc9c5a2 - std::sys::backtrace::_print_fmt::hdda75a118fd2034a
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/backtrace.rs:66:9
   3:     0x5f8bcfc9c5a2 - <std::sys::backtrace::BacktraceLock::print::DisplayBacktrace as core::fmt::Display>::fmt::hf435e8e9347709a8
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/backtrace.rs:39:26
   4:     0x5f8bcfcc5a63 - core::fmt::rt::Argument::fmt::h9802ea71fd88c728
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/fmt/rt.rs:173:76
   5:     0x5f8bcfcc5a63 - core::fmt::write::h0a51fad3804c5e7c
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/fmt/mod.rs:1465:25
   6:     0x5f8bcfc98f83 - std::io::default_write_fmt::h33ff8981097f58ea
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/io/mod.rs:639:11
   7:     0x5f8bcfc98f83 - std::io::Write::write_fmt::he54474135bb64f2f
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/io/mod.rs:1954:13
   8:     0x5f8bcfc9c3f2 - std::sys::backtrace::BacktraceLock::print::h1ec5ce5bb8ee285e
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/backtrace.rs:42:9
   9:     0x5f8bcfc9e2bc - std::panicking::default_hook::{{closure}}::h5ffefe997a3c75e4
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:300:27
  10:     0x5f8bcfc9e112 - std::panicking::default_hook::h820c77ba0601d6bb
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:324:9
  11:     0x5f8bcf43fdb4 - <alloc::boxed::Box<F,A> as core::ops::function::Fn<Args>>::call::hd2d9a835f4b8f423
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed.rs:1980:9
  12:     0x5f8bcf43fdb4 - test::test_main_with_exit_callback::{{closure}}::h32c4bc4a085dff05
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:145:21
  13:     0x5f8bcfc9ed1b - <alloc::boxed::Box<F,A> as core::ops::function::Fn<Args>>::call::h8be59125c8e59551
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed.rs:1980:9
  14:     0x5f8bcfc9ed1b - std::panicking::rust_panic_with_hook::h8b29cbe181d50030
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:841:13
  15:     0x5f8bcfc9e9d6 - std::panicking::begin_panic_handler::{{closure}}::h9f5b6f6dc6fde83e
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:699:13
  16:     0x5f8bcfc9caa9 - std::sys::backtrace::__rust_end_short_backtrace::hd7b0c344383b0b61
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/backtrace.rs:168:18
  17:     0x5f8bcfc9e69d - __rustc[5224e6b81cd82a8f]::rust_begin_unwind
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:697:5
  18:     0x5f8bcf3699f0 - core::panicking::panic_fmt::hc49fc28484033487
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/panicking.rs:75:14
  19:     0x5f8bcf36e3e3 - sprint_acceptance_0001::apply_fail_closed_on_policy_violation::hb70bd120c7e5a79a
                               at /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/sprint_acceptance-0001.rs:139:5
  20:     0x5f8bcf3ea057 - sprint_acceptance_0001::apply_fail_closed_on_policy_violation::{{closure}}::h5f2e70bcfdb32e92
                               at /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/sprint_acceptance-0001.rs:98:43
  21:     0x5f8bcf39eb96 - core::ops::function::FnOnce::call_once::hec1ae47f6cf3ab84
                               at /home/vince/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
  22:     0x5f8bcf4454bb - core::ops::function::FnOnce::call_once::h92226b62eea5e740
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/ops/function.rs:250:5
  23:     0x5f8bcf4454bb - test::__rust_begin_short_backtrace::h5403e9ff57c40dab
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:648:18
  24:     0x5f8bcf44473e - test::run_test_in_process::{{closure}}::hde5ebd764eed8d41
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:671:74
  25:     0x5f8bcf44473e - <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::h87a841e037538b15
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/panic/unwind_safe.rs:272:9
  26:     0x5f8bcf44473e - std::panicking::catch_unwind::do_call::hfce8274c0464d7ee
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:589:40
  27:     0x5f8bcf44473e - std::panicking::catch_unwind::hf1024a3b71e7559a
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:552:19
  28:     0x5f8bcf44473e - std::panic::catch_unwind::h32f944874226e619
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panic.rs:359:14
  29:     0x5f8bcf44473e - test::run_test_in_process::he51794d4da2ed405
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:671:27
  30:     0x5f8bcf44473e - test::run_test::{{closure}}::hc41d5e32018fb032
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:592:43
  31:     0x5f8bcf409514 - test::run_test::{{closure}}::hb8e8bc84d9b1bbba
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/test/src/lib.rs:622:41
  32:     0x5f8bcf409514 - std::sys::backtrace::__rust_begin_short_backtrace::h725872d9e0edd537
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/backtrace.rs:152:18
  33:     0x5f8bcf40cc9a - std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}::hc300ae8ae2205644
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/thread/mod.rs:559:17
  34:     0x5f8bcf40cc9a - <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::h98ba43e212713b1f
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/panic/unwind_safe.rs:272:9
  35:     0x5f8bcf40cc9a - std::panicking::catch_unwind::do_call::h3367b31a744e4f14
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:589:40
  36:     0x5f8bcf40cc9a - std::panicking::catch_unwind::h0a7a52dbc375b4fd
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panicking.rs:552:19
  37:     0x5f8bcf40cc9a - std::panic::catch_unwind::he34cdfe5914307ff
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/panic.rs:359:14
  38:     0x5f8bcf40cc9a - std::thread::Builder::spawn_unchecked_::{{closure}}::h73faeaeb9205adc3
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/thread/mod.rs:557:30
  39:     0x5f8bcf40cc9a - core::ops::function::FnOnce::call_once{{vtable.shim}}::h7c65d33fb6595d81
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/core/src/ops/function.rs:250:5
  40:     0x5f8bcfca25ef - <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once::h8703e59bc8145d18
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/alloc/src/boxed.rs:1966:9
  41:     0x5f8bcfca25ef - std::sys::pal::unix::thread::Thread::new::thread_start::h1ff51d6e85162efd
                               at /rustc/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/sys/pal/unix/thread.rs:107:17
  42:     0x7da9d5c969cb - <unknown>
  43:     0x7da9d5d1aa0c - <unknown>
  44:                0x0 - <unknown>


failures:
    apply_fail_closed_on_policy_violation

test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p switchyard --test sprint_acceptance-0001`
