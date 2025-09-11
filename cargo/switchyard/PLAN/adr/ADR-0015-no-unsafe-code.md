# ADR-0015: Unsafe Rust Is Forbidden

Date: 2025-09-11
Status: Accepted
Decision Makers: Switchyard Team

## Context

Rust provides an `unsafe` escape hatch that bypasses the compiler’s safety guarantees. While sometimes necessary for FFI or low-level primitives, `unsafe` introduces risks that are inconsistent with Switchyard’s goals of determinism, auditability, and reliability as captured in SPEC v1.1 (transactionality, recovery, and conservative defaults).

## Decision

- We forbid the use of `unsafe` code in the Switchyard codebase.
- Enforcement: the crate root will include `#![forbid(unsafe_code)]` so any introduction of `unsafe` fails compilation.
- CI will run an additional detector to guard against transitive `unsafe` usage creeping into our own code (e.g., `cargo geiger` report is advisory; the compile-time `forbid` is authoritative for our sources).
- Any future need for `unsafe` must be proposed via a new ADR and will remain disallowed until explicitly accepted.

## Scope

- Applies to all code in `cargo/switchyard/` (library, binaries, tests, and benches).
- Dependencies may internally use `unsafe`; we accept that risk when widely used audited crates are involved, but our own sources must remain `unsafe`-free.

## Rationale

- Aligns with safety and audit requirements; eliminates a class of memory-safety and UB bugs.
- Reduces surface area for formal and empirical verification of behavior (e.g., deterministic planning, fail-closed semantics).
- Simplifies code review and compliance checks; any deviation requires explicit governance (an ADR).

## Consequences

- Some low-level optimizations or direct syscalls must rely on safe wrappers and well-maintained crates.
- If an unavoidable `unsafe` use-case arises (e.g., FFI adapter), it will be blocked until a dedicated ADR is authored and accepted with strict containment, tests, and auditing requirements.

## Compliance & Tooling

- Add `#![forbid(unsafe_code)]` to the crate root (e.g., `src/lib.rs` and any `main.rs`).
- CI: keep an advisory `cargo geiger` report to track transitive `unsafe` in third-party dependencies; treat increases as signals for review.
- Code Review Checklist: reject any PR introducing `unsafe` and require an ADR for exceptions.

## References

- Rust Reference: Unsafe Code Guidelines
- Switchyard SPEC v1.1 (Determinism, Transactionality, Conservative Defaults)
- ADR-0014 (project-wide deferral governance pattern)
