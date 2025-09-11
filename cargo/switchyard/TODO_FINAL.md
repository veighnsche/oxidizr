# Switchyard TODO_FINAL — Release‑Blocking Tasks Only

Scope: The smallest set of tasks required to ship a solid Silver‑tier library release with a clear sense of done. Everything else is documented as a deferral.

Status baseline:

- Core engine, preflight gating (incl. require_preservation/require_rescue), degraded EXDEV semantics, locking with E_LOCKING and lock_wait_ms, minimal smoke subset, determinism/redaction, tests and traceability job are in place.

---

## Release Criteria (must all pass)

- Facts schema validation passes for all tests.
- Curated golden fragments remain stable (no unexpected diffs).
- Rescue gating behaves deterministically (PATH‑driven + override) and does not block non‑rescue scenarios.
- Lock timeout emits `E_LOCKING` with `exit_code=30`; canon excludes volatile fields after redaction.
- README and docs reflect implemented behavior; SPEC/ADR entries merged.

---

## Completed in this iteration

- Provenance completeness minimal pass (env_sanitized present across plan, preflight, apply.attempt, apply.result, rollback; tests added)
- Error/exit code coverage tests for: E_BACKUP_MISSING, E_RESTORE_FAILED, E_ATOMIC_SWAP, E_EXDEV, E_POLICY, E_SMOKE
- Preflight YAML export helper `preflight::to_yaml(&PreflightReport)` and golden-style test
- Docs accepted: SPEC_UPDATE_0002 and ADR-0017
- Minimal rustdocs added for `preflight` module (crate docs already present)

---

## Tasks (Release‑Blocking)

None — all release‑blocking tasks are complete.

---

## Nice‑to‑Have (Non‑blocking for this release)

- Property tests: `IdempotentRollback` and `AtomicReplace` invariants.
- Adapter‑based rescue verifier and richer preflight notes for rescue toolset.
- Feature‑flag external smoke checks (`sha256sum -c` on tiny fixture) with output redaction.
- CI: flip curated golden diff gate to blocking once scenarios stabilize.
- Versioning & changelog (0.2.0) with `CHANGELOG.md` — deferred per instruction (no bump required now)

---

## Explicit Deferrals (documented)

- Rescue profile maintenance of a backup symlink set (RC1).
- Cross‑filesystem acceptance matrix (ext4/xfs/btrfs/tmpfs).
- Full secret masking policy beyond current list.
- Zero‑SKIP enforcement at this crate’s CI layer (documented policy; enforced upstream orchestration).
