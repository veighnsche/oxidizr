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

## Tasks (Release‑Blocking)

1) Provenance completeness (minimal, consistent)

- What: Ensure minimal provenance appears on all facts (`provenance` object exists) and expand presence where available to include `origin`, `helper`, and `env_sanitized=true`. Keep uid/gid/pkg enrichment where `OwnershipOracle` exists.
- Deliverables:
  - Unit tests asserting presence of `provenance.env_sanitized=true` across `plan`, `preflight`, `apply.attempt`, `apply.result`, and `rollback` after redaction.
  - If adapters supply `origin`/`helper`, ensure they propagate; otherwise default to empty string/masked value and keep deterministic redaction.
- Acceptance: New tests pass; redaction tests remain green.

2) Error/exit code coverage audit (tests only)

- What: Verify `error_id` and `exit_code` emission at per‑action and/or summary for the curated Silver set.
- Cases to cover: `E_BACKUP_MISSING`, `E_RESTORE_FAILED`, `E_ATOMIC_SWAP`, `E_EXDEV`, `E_POLICY`, `E_SMOKE`.
- Deliverables:
  - Focused unit tests that trigger each failure and assert the redacted canon contains the corresponding `error_id` and `exit_code`.
- Acceptance: All new tests green; existing tests unaffected.

3) Preflight YAML export helper + golden

- What: Provide a pure helper `preflight::to_yaml(&PreflightReport) -> String` matching `SPEC/preflight.yaml` shape for fixtures.
- Deliverables:
  - Helper function and one test writing a golden YAML for a two‑action plan (positive+negative rows).
- Acceptance: Golden stable under `GOLDEN_OUT_DIR`; schema validation remains green.

4) Docs: finalize SPEC_UPDATE_0002 and ADR‑0015

- What: Move `SPEC/SPEC_UPDATE_0002.md` (rescue/degraded/preflight rows) and `PLAN/adr/ADR-0015-exit-codes-silver-and-ci-gates.md` to Accepted.
- Deliverables:
  - Update status headers; link from README.
- Acceptance: Docs merged; README references remain accurate.

5) Minimal rustdocs

- What: Add crate‑level and key module rustdocs summarizing guarantees and adapter contracts (`api`, `preflight`, `fs`, `policy`, `adapters`, `logging`).
- Deliverables:
  - One‑paragraph module‑level docs; no deep narrative.
- Acceptance: `cargo doc -p switchyard` builds; clippy/missing_docs allowances remain off for now.

6) Versioning & changelog

- What: Bump crate version to 0.2.0 (first Silver) and add a short `CHANGELOG.md` entry.
- Deliverables:
  - `Cargo.toml` version bump; `CHANGELOG.md` with highlights (rescue gating, E_LOCKING telemetry, degraded semantics clarification, tests/goldens, traceability job).
- Acceptance: Commit builds and tests pass.

---

## Nice‑to‑Have (Non‑blocking for this release)

- Property tests: `IdempotentRollback` and `AtomicReplace` invariants.
- Adapter‑based rescue verifier and richer preflight notes for rescue toolset.
- Feature‑flag external smoke checks (`sha256sum -c` on tiny fixture) with output redaction.
- CI: flip curated golden diff gate to blocking once scenarios stabilize.

---

## Explicit Deferrals (documented)

- Rescue profile maintenance of a backup symlink set (RC1).
- Cross‑filesystem acceptance matrix (ext4/xfs/btrfs/tmpfs).
- Full secret masking policy beyond current list.
- Zero‑SKIP enforcement at this crate’s CI layer (documented policy; enforced upstream orchestration).
