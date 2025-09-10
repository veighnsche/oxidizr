# Switchyard BDD Features

This directory contains Gherkin `.feature` files that formalize the normative scenarios from `SPEC.md` in a machine-readable format.

- Features are tagged with requirement IDs (e.g., `@REQ-A1`) which correspond to entries in `../requirements.yaml`.
- Step vocabulary is defined in `steps-contract.yaml` to decouple scenarios from any specific test runner or environment.
- These files are specification artifacts first; a thin adapter can map steps to concrete actions in your existing orchestrators.

## Files

- `atomic_swap.feature` — Atomic enable/rollback, EXDEV degraded mode, smoke-triggered rollback.
- `observability.feature` — Audit schema validity, determinism (dry-run vs real-run), hashing, masking, provenance.
- `locking_rescue.feature` — Bounded locking behavior, warnings without lock manager, rescue profile and fallback.
- `steps-contract.yaml` — Canonical step phrases, parameters, and expected effects.

## Tagging Convention

- Tags mirror requirement IDs from `../requirements.yaml`.
- A scenario may have multiple tags if it verifies multiple requirements.

## How to Consume

You can implement any of the following adapters:

1. A minimal Rust adapter using `cucumber-rs` that calls into the library APIs.
2. A Go adapter using `godog` that shells out to your existing container runner.
3. A simple YAML-driven shim that translates Given/When/Then into your `tests/*/task.yaml` operations (no changes to `tests/`).

## Traceability

Use `../tools/traceability.py` to generate a coverage report that joins `requirements.yaml` with feature tags and highlights uncovered MUSTs.
