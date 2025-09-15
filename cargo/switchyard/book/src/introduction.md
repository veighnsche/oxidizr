# Introduction

Switchyard is a Rust library for safe, atomic, and reversible filesystem swaps backed by evidence.

What it is:
- A library crate embedded by higher‑level CLIs.
- Guarantees: TOCTOU‑safe sequences, policy‑gated preflight, deterministic IDs, optional smoke with auto‑rollback, structured audit facts.

What it is not:
- A CLI manual. The guide focuses on the `switchyard` library API and its invariants.

Citations:
- SPEC overview: `cargo/switchyard/SPEC/SPEC.md`
- Module map: `cargo/switchyard/src/`
