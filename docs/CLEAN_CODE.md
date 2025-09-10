# Clean Code Principles (Rust, Safety-Oriented)

## 1) Clarity Over Cleverness

* Prefer straightforward code to “smart” tricks.
* Name things by **intent + effect**.
* Keep functions short; one reason to change.

> If a new engineer can’t follow it in one read, it’s not clean.

---

## 2) Explicit > Implicit

* Make side effects and dependencies visible in function signatures.
* Avoid hidden globals; inject handles (traits or structs).
* Prefer `#[must_use]` on important return values.

---

## 3) Types Encode Invariants

* Use **newtypes** for units/paths/IDs; don’t pass raw `String`/`PathBuf`.
* Model states with **enums** and typed state machines.
* Prefer `Option`/`Result` to sentinel values; avoid `bool` parameters—use enums.

---

## 4) Immutability by Default

* Keep data immutable; mutate via narrow, controlled scopes.
* Favor builders that validate and then return an **invariant-holding** type.

---

## 5) Honest Error Handling

* No `unwrap`/`expect` outside tests and `main`.
* Create a **categorical error type** (`thiserror`) with rich context (path, syscall, expected vs actual).
* Return early; avoid nested error pyramids.

---

## 6) Side-Effect Isolation

* Separate **pure logic** from I/O and syscalls.
* Gate all side effects behind **small interfaces/traits** (`Fs`, `Clock`, `CmdRunner`), easy to mock.
* Keep “edges” (I/O) thin and centralized.

---

## 7) Determinism & Idempotence

* Same inputs → same outputs; document any nondeterminism.
* Design operations to be **safe to retry**.
* Stabilize environment in tests (locale, TZ, umask).

---

## 8) Atomicity & Consistency

* Prefer **single atomic action** (e.g., `rename`) over many tiny mutations.
* Validate preconditions, apply change, verify postconditions.
* Commit/log only after success.

---

## 9) Observability Is a Feature

* Use `tracing` with **structured fields** (not free-text).
* One root span per operation with a unique ID.
* Logs should let someone reconstruct: **what, where, when, why, result**.

---

## 10) Small, Composable Modules

* Organize by responsibility (not by technology).
* Public APIs are **minimal**; private helpers do the heavy lifting.
* Keep cyclomatic complexity low; extract intentful functions.

---

## 11) Safe Concurrency

* Prefer message passing/ownership to shared mutability.
* Bound parallelism; avoid global executors as implicit deps.
* Document Send/Sync assumptions; don’t guess—prove.

---

## 12) Dependency Discipline

* Bring in crates only when they reduce total risk/complexity.
* Pin features; avoid optional features you don’t use.
* Periodically audit (`cargo audit`, `cargo deny`).

---

## 13) Document Invariants, Not Trivia

* Top of each module: **purpose, invariants, failure modes**.
* Public items need a **what/why/when**; private code can be lighter.
* Keep examples runnable (`doctest`) when helpful.

---

## 14) Intent-Revealing Names

* Functions are **verbs** (“prepare\_manifest”, “apply\_change”).
* Types are **nouns** with domain meaning (“OperationId”, “Plan”, “Manifest”).
* Avoid abbreviations unless industry-standard.

---

## 15) Cohesive APIs, Stable Contracts

* Design for **least surprise**; avoid hidden work (network, disk) in getters.
* Provide **total functions** where possible (cover every case).
* Version error categories and exit codes; treat them as API.

---

## 16) `unsafe` With Proof Obligations

* Default to `#![forbid(unsafe_code)]`.
* If needed, isolate `unsafe` in a small module:

  * Document **safety preconditions**.
  * Wrap in a safe API.
  * Test those preconditions explicitly.

---

## 17) Testing for Behavior, Not Lines

* Unit tests for pure logic; integration tests for edges.
* Property tests for invariants (idempotence, round-trip).
* “Table tests” for tricky behavior; capture regressions with fixtures.

---

## 18) Configuration Is Data, Once

* Load config once; pass a typed config object.
* Validate on load; reject ambiguous or partial configs.
* Record effective config (minus secrets) for audit.

---

## 19) Fail Closed, Loudly

* Ambiguity → error, not guess.
* Provide actionable messages with context and remediation.
* Don’t proceed on partial success; roll back or bail.

---

## 20) Formatting, Lints, and CI Are Non-Negotiable

* Consistent style: `rustfmt` and `clippy --deny warnings`.
* Keep builds reproducible; lockfile checked in for binaries.
* CI runs tests, lints, audits, and docs/links (doctests).

---

## 21) Make the Happy Path Obvious

* Guard clauses for error/edge cases first.
* Keep the main flow linear and readable.
* Prefer composition to deep nesting.

---

## 22) Comments Explain *Why*, Code Shows *How*

* Avoid narrating the code; explain intent, trade-offs, and constraints.
* Link to specs or decisions (ADR) if relevant.

---

## 23) Time, Locale, and Filesystems Are Adversaries

* Treat clocks, locales, and FS semantics as **inputs**.
* Normalize or pin them where possible; document assumptions.

---

## 24) Minimal Public Surface

* Expose the smallest API that solves the problem.
* Use `pub(crate)` liberally; keep evolution space.
* Add `#[non_exhaustive]` to enums you expect to grow.

---

## 25) Design for Recovery

* For any action that changes state, know the inverse and log enough to perform it.
* Prefer storing **evidence** (hashes, manifests) over trusting memory.

---

### Tiny Rust Patterns (Illustrative)

```rust
// 2) Explicit deps, 6) Side-effect isolation
pub trait Fs {
    fn read(&self, p: &Path) -> Result<Vec<u8>, FsError>;
    fn atomic_replace(&self, from: &Path, to: &Path) -> Result<(), FsError>;
}

// 3) Types encode invariants
#[derive(Clone, Debug)]
pub struct OperationId(Uuid);

#[derive(Debug)]
pub enum Phase { Prepared, Applied, Verified }

// 5) Honest error handling
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("filesystem: {path:?}: {source}")]
    Fs { #[source] source: std::io::Error, path: PathBuf },
    #[error("invariant violated: {0}")]
    Invariant(String),
}

// 9) Observability
#[tracing::instrument(name = "apply_plan", skip(fs), fields(op_id=%op_id.0))]
pub fn apply_plan(fs: &impl Fs, plan: &Plan, op_id: OperationId) -> Result<(), Error> {
    // guard clauses …
    // happy path …
    Ok(())
}
```
