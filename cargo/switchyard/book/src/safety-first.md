# Safety First: Switchyard Invariants That Protect You

Switchyard is designed with a safety-first mindset. This page distills the safety model from the SPEC into practical guarantees, operator checklists, and what-to-do guidance.

> Note: Source of truth for normative requirements is `cargo/switchyard/SPEC/SPEC.md`.

## Atomicity

- What it guarantees
  - No user-visible broken or missing path during swaps.
  - Crash-safe: either the old target or the new link is visible; never an intermediate state.
- How it’s implemented
  - TOCTOU-safe sequence: open parent O_DIRECTORY|O_NOFOLLOW → openat → renameat → fsync(parent) within a bounded interval.
- Operator checklist
  - Ensure the target’s parent mount is writable and not `noexec`.
  - Monitor `apply.result` facts for `perf.swap_ms` and atomicity errors.

## Rollback & Recovery

- What it guarantees
  - Automatic reverse-order rollback starts on the first apply error.
  - Idempotent restore paths: running rollback twice yields the same state.
- What can still go wrong
  - If rollback fails (e.g., missing backup payload), you may end up in partial restoration.
- Operator checklist
  - Inspect `rollback.summary` for `summary_error_ids`.
  - Use the Recovery Playbook to complete restoration if needed.

## SafePath Everywhere

- Why it matters
  - Avoids path traversal and TOCTOU hazards; ensures all mutating paths remain under approved roots.
- Guarantees
  - All mutating public APIs take `SafePath`, constructed via `SafePath::from_rooted(root, candidate)`.
- Operator checklist
  - Build `SafePath` at boundaries (especially in CLIs or adapters).
  - Reject any inputs that cannot be converted.

## Determinism

- What it guarantees
  - Stable `plan_id` and `action_id` (UUIDv5) from normalized inputs.
  - Dry-run facts match Commit facts after redaction (timestamps zeroed).
- Why you care
  - Enables golden fixtures, reproducibility, and byte-identical diffs across environments.

## Locking & Concurrency

- Guarantees
  - Only one mutator proceeds at a time in production.
  - Bounded wait → `E_LOCKING`; facts record `lock_wait_ms` and approximate `lock_attempts`.
- Operator checklist
  - Provide a `LockManager` in Commit mode.
  - Tune timeouts and observe `apply.attempt` facts.

## Rescue Profile

- Guarantees
  - A fallback toolset is available (BusyBox or a deterministic GNU subset) so break-glass recovery is possible.
- Operator checklist
  - In preflight, require rescue (`require_rescue=true`) for production deployments.
  - Confirm presence of a functional fallback on PATH.

## Observability & Audit

- Guarantees
  - Every stage emits structured JSON facts (schema v2) with before/after hashes.
  - Attestation bundles can be signed; secret masking enforced.
- Operator checklist
  - Configure durable sinks (e.g., JSONL file) and secure retention.
  - Verify facts against `/SPEC/audit_event.v2.schema.json` when integrating.

## Health Verification (Smoke)

- Guarantees
  - Minimal smoke suite must run post-apply in Commit; failures trigger auto-rollback unless explicitly disabled by policy.
- Operator checklist
  - Provide a `SmokeTestRunner` adapter in production.
  - Review smoke failures and associated auto-rollback facts.

## Filesystems & Degraded Mode (EXDEV)

- Behavior
  - Cross-filesystem swap uses safe copy+sync+rename when applicable; symlink replacement may degrade to unlink+symlink under policy.
- Operator checklist
  - Decide policy for `allow_degraded_fs`; monitor `degraded` flags and reasons.

## Where to Go Next

- Architecture Overview: how modules enforce these invariants.
- Recovery Playbook: step-by-step guidance when things go sideways.
- Policy Knobs: tailor safety and durability for your environment.
