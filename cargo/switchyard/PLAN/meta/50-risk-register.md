# Risk Register (v0)

| ID | Risk | Area | Likelihood | Impact | Mitigation | Trigger | Owner |
|---|---|---|---|---|---|---|---|
| R1 | TOCTOU or SafePath gaps | FS Ops | M | H | Patterned syscalls; tests; code review | Failing `api_toctou.feature` or unit tests | Maintainers |
| R2 | Non-deterministic behavior in CI | Determinism | M | M | Deterministic mode, golden logs, UUIDv5 | Golden diff instability across runs | SRE/CI Lead |
| R3 | Audit logs incomplete or invalid | Observability | L | H | Schema validation, coverage review | Schema validation errors in CI | Maintainers |
| R4 | CI flakes / external deps | CI | M | M | Retries, isolation, cache hygiene | Intermittent test failures without code changes | SRE/CI Lead |
| R5 | Attestation key mgmt complexity | Supply Chain | M | M | Dev/test key rotation, key vault integration plan | Signing step fails or missing keys | Security/Platform |
| R6 | EXDEV semantics vary by filesystem | Filesystems | M | M | Robust fallback (copy+sync+rename), telemetry | `exdev_fallback_failed` occurrences in tests | Maintainers |
| R7 | Locking timeouts in production | Concurrency | L | H | Bounded waits, clear timeouts, metrics | Elevated `E_LOCKING` rate; `lock_wait_ms` spikes | Tech Lead |
| R8 | Steps-contract drift vs features | Spec/Test | M | M | Contract lint in CI; review changes | `traceability.py` unmatched step errors | QA/Requirements |
| R9 | Golden fixture churn | CI/Determinism | M | M | Stable ordering, redactions, review policy | Frequent fixture updates in PRs | Maintainers + QA |
| R10 | Dependency or license incompatibility | Supply Chain | L | H | Crate audits, license checks | New dep flagged by license tool | Maintainers |
| R11 | Provenance/secret masking gaps | Security | L | H | Masking policy tests; review sinks | Secret detector finds leakage | Security/Platform |
| R12 | Schedule slip | Delivery | M | M | Buffer in milestones; critical path tracking | Missed milestone checkpoints | PM/Tech Lead |

## Assumptions & Fallback Plans

- BDD runner or adapter is available and compatible with `SPEC/features/steps-contract.yaml`.
  - Fallback: implement minimal Rust `cucumber-rs` adapter or Go `godog` shim to satisfy steps.

- Dev/test attestation keys are available to CI for signing.
  - Fallback: generate ephemeral keys during CI with clear provenance; rotate per run.

- Filesystem semantics (ext4, xfs, btrfs, tmpfs) are accessible in CI for acceptance tests.
  - Fallback: prioritize tmpfs and ext4 in PR CI; run others in nightly matrix.

- Steps-contract remains the canonical vocabulary; features adhere to it.
  - Fallback: update contract and re-run `traceability.py`; treat drift as a change requiring review.

- Golden fixtures remain stable under deterministic mode and redaction rules.
  - Fallback: tighten redactions/ordering and update fixtures under controlled review policy.
