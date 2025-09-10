# Risk Register (v0)

| ID | Risk | Area | Likelihood | Impact | Mitigation | Owner |
|---|---|---|---|---|---|---|
| R1 | TOCTOU or SafePath gaps | FS Ops | M | H | Patterned syscalls; tests; code review | TBD |
| R2 | Non-deterministic behavior in CI | Determinism | M | M | deterministic mode, golden logs | TBD |
| R3 | Audit logs incomplete or invalid | Observability | L | H | schema validation, coverage review | TBD |
| R4 | CI flakes / external deps | CI | M | M | retries, isolation, cache hygiene | TBD |
