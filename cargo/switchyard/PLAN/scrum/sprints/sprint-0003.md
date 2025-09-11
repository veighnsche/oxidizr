## Sprint 03 — Active

### Theme

- Gherkin alignment + Rescue & Provenance + CI Traceability + Smoke Suite Expansion + Policy/Docs Sync

### Objectives

- Align all Gherkin features with current implementation (introduce @xfail where appropriate); prepare for immediate execution.
- Implement rescue verification and wire policy gating; complete provenance fields.
- Emit `lock_wait_ms` on locking timeout and broaden error/exit_code coverage in facts.
- Produce CI traceability artifacts and integrate with PRs/releases.
- Expand smoke runner toward SPEC §11 command set (deterministic subset).
- Complete SPEC_UPDATE/ADR notes for policy flags and error taxonomy boundaries.

### Stories & Tasks (parallelizable lanes)

0) Gherkin Alignment & Updates (Owner: Dev-A)
   - Update `SPEC/features/*.feature` to reflect current code:
     - `atomic_swap.feature` EXDEV scenario: clarify degraded symlink fallback (unlink+symlink) and `degraded=true` when allowed.
     - `locking_rescue.feature`: mark rescue scenario `@xfail` until implemented; refine locking timeout to allow missing `lock_wait_ms` or add `@xfail` note; include `error_id=E_LOCKING` + `exit_code=30` expectation.
     - `observability.feature`: mark provenance completeness scenario `@xfail` (until origin/helper/pkg complete); keep secret masking scenario; note redaction parity for comparisons.
     - `determinism_attestation.feature`: assert attestation attached to `apply.result` summary and includes `signature`, `bundle_hash`, `public_key_id`.
     - `operational_bounds.feature`: assert timing recorded and `severity=warn` when >50ms, rather than strictly enforcing ≤50ms.
     - `conservatism_ci.feature`: mark CI-gate scenario `@xfail` (runs upstream), keep dry-run default and fail-closed scenarios.
   - Ensure `steps-contract.yaml` remains consistent; no changes expected.

1) Rescue Verification (Owner: Dev-C)
   - Implement `src/rescue.rs::verify_rescue_tools()` to check rescue symlink set and GNU/BusyBox presence via a `PathResolver`.
   - Add policy flag enforcement `require_rescue` in preflight; emit notes and STOP when unmet unless overridden.
   - Tests: unit tests with mock resolver; update preflight rows and facts accordingly.

2) Locking Metrics & Error Emission (Owner: Dev-B)
   - In `src/api/apply.rs`, on lock timeout failure path, capture and emit `lock_wait_ms` alongside `E_LOCKING` with `exit_code=30`.
   - Add golden fragment/test asserting presence; update facts schema validation where needed.

3) Provenance Completeness (Owner: Dev-A)
   - Extend apply results to include provenance `{uid,gid,pkg}` when oracle present; populate `origin`/`helper` where adapters provide it; set `env_sanitized=true`.
   - Expand redaction policy to mask configured secret fields; add unit tests.

4) Traceability CI Integration (Owner: Dev-D)
   - Add CI job to run `python SPEC/tools/traceability.py` and upload artifact.
   - Gate (non-blocking) pass/fail on script health; link report in PR summary.

5) Smoke Runner Command Coverage (Owner: Dev-E)
   - Implement deterministic subset of commands (e.g., `stat`, `readlink`, `sha256sum`) with fixed args and environment.
   - Add tests for command invocation and failure classification; ensure auto-rollback path preserves invariants.

6) Policy & SPEC Doc-Sync (Owner: Dev-C)
   - SPEC_UPDATE for `override_preflight`, `require_preservation`, degraded symlink semantics, and preflight row clarifications.
   - ADR addendum on Silver-tier scope and deferrals for error taxonomy and CI gates.

7) Exit Code Coverage Report (Owner: Dev-B)
   - Generate a simple summary (by test) of covered `ErrorId`→`exit_code` in CI; attach as artifact.

### Risks / Constraints

- Keep smoke deterministic (no time or locale dependence); avoid fragile file system expectations.
- CI environment limitations; ensure jobs run without privileged operations.

### References

- `SPEC/tools/traceability.py`, `.github/workflows/ci.yml`
- `src/adapters/smoke.rs`, `src/api/apply.rs`

### Acceptance Criteria (DoD)

- Updated `.feature` files merged; scenarios pass locally where applicable; `@xfail` marks applied where implementation is pending.
- Lock timeout path emits `E_LOCKING` with `exit_code=30` and records `lock_wait_ms` when measurable.
- Rescue verification integrated in preflight with `require_rescue` gating and tests proving STOP behavior.
- Apply results include extended provenance where adapters are present; redaction policy and unit tests updated.
- Traceability artifact produced in CI; linked in PR summary; non-blocking gate green.
- Deterministic smoke runner subset implemented with unit tests.
