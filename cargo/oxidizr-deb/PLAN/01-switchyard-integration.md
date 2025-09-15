# Plan: Engine Integration for oxidizr-deb (package-level rustify)

## 1) Goal

Provide a robust, reproducible integration of the safety engine into the oxidizr-deb CLI, honoring engine SPEC invariants and the Debian/Ubuntu UX addendum, with a simple package-level `rustify/restore/status` UX.

## 2) Current State

- oxidizr-deb provides a minimal CLI executing `plan → preflight → apply` via the engine.
- Adapters injected: file lock manager, default smoke runner, filesystem ownership oracle.
- Implicit per-package policies: `coreutils` and `findutils` (strict link topology, no degraded EXDEV) and `sudo` (production hardening).
- Integrated fetch-and-verify step selects and validates replacement artifacts.
- SafePath enforced at boundaries; dry-run default; commit via `--commit`.

## 3) Gaps vs Engine SPEC

- Health verification: baseline smoke is minimal; production expects deterministic checks (§11 of engine SPEC).
- Error taxonomy/exit codes: oxidizr-deb uses 0/1; alignment with a taxonomy is planned.
- Observability: file-logging sinks optional; default sink may be no-op; production may enable JSONL files.
- Degraded mode: per-package policy disallows EXDEV degraded fallback; ensure facts surface `degraded` accurately.
- Fetch/verify: integrate SHA-256 (and signature when available) before planning.

## 4) Integration Tasks

- Adapters
  - Locking: use `<root>/var/lock/oxidizr-deb.lock`; keep override internal unless needed.
  - Smoke: keep default runner; add Debian checks (sudo owner/mode) without executing external commands.
  - Ownership: filesystem ownership oracle suffices for v0.
- Policies (implicit per-package)
  - Coreutils: strict gates; no degraded EXDEV; forbidden untrusted sources.
  - Findutils: strict gates; no degraded EXDEV; forbidden untrusted sources.
  - Sudo: production base with setuid guard; require rescue/rollback readiness.
- SafePath
  - Enforce SafePath construction for all mutating paths under `--root`.
- Fetch/Verify
  - Implement fetcher module: select artifact by arch/distro; verify SHA-256 and optional signatures; support `--offline --use-local`.
- Observability
  - Keep default sinks; optionally add a feature to enable file-backed JSONL (`file-logging`).
- Error Handling
  - Preserve 0/1 exit codes for v0; plan mapping to a taxonomy in a follow-up.

## 5) Acceptance Criteria

- Dry-run by default; `--commit` required.
- Locking errors surface as user-friendly messages; concurrent mutate attempts time out with bounded wait.
- Smoke runner executes in commit mode and failures trigger auto-rollback.
- Preflight stops on policy violations; no mutations occur on STOP.
- All mutating paths validated via SafePath; attempts outside `--root` rejected.
- Fetch-and-verify completes successfully before any mutation is planned.

## 6) Risks & Mitigations

- False negatives on smoke checks → keep the minimal, deterministic checks (link target resolution; sudo owner/mode) and avoid external exec.
- Lock path permission issues under custom roots → document lock path; make overridable if required.
- Divergence from SPEC schemas → reuse engine schemas; add CI fixtures.

## 7) Deliverables

- Verified integration with feature flags and adapters.
- Golden fixtures where applicable (plan/preflight/apply summaries).
- CLI help and README aligned with SPEC.
