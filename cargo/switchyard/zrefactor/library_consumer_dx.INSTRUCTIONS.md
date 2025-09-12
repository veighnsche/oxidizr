# Library Consumer DX Overhaul — Actionable Instructions (additive, non-breaking)

Scope: Only improvements visible to external consumers of the `switchyard` crate. No internal reorganizations required to adopt. All steps are additive and should be SemVer-minor changes.

## Deliverables (consumer-facing)

- A small `prelude` for one-line imports of common types
- An ergonomic `ApiBuilder` for constructing `Switchyard`
- A fluent `PlanBuilder` DSL to create `Plan` without manual structs
- Optional `config` loader (serde TOML/YAML) to build Policy + API from files
- Strongly-typed, serde-friendly reports and IDs with convenience (to_json / from_json)
- Better error ergonomics: typed `ApiError` + `ErrorId` with stable mappings
- Logging DX: macros and a `test-utils` feature for consumers to test their integrations
- Discoverable features and crate docs with runnable examples

---

## 1) Add a curated prelude (one import for common use)

- File: `src/prelude.rs`
  - Re-export the commonly-used public surface:

    ```rust
    pub use crate::api::{Switchyard, ApiBuilder};
    pub use crate::policy::Policy; // plus profiles when available
    pub use crate::types::{Plan, PlanInput, ApplyMode};
    pub use crate::adapters::{FileLockManager, FsOwnershipOracle};
    ```

- File: `src/lib.rs`
  - `pub mod prelude;`
- Acceptance:
  - Example compiles:

    ```rust
    use switchyard::prelude::*;
    let _ = ApplyMode::DryRun;
    ```

## 2) Introduce `ApiBuilder` (consumer ergonomics)

- File: `src/api/mod.rs`
  - Add an additive builder:

    ```rust
    pub struct ApiBuilder<E: FactsEmitter, A: AuditSink> {
        facts: E,
        audit: A,
        policy: Policy,
        lock: Option<Box<dyn LockManager>>,
        owner: Option<Box<dyn OwnershipOracle>>,
        attest: Option<Box<dyn Attestor>>,
        smoke: Option<Box<dyn SmokeTestRunner>>,
        lock_timeout_ms: u64,
    }
    impl<E: FactsEmitter, A: AuditSink> ApiBuilder<E, A> {
        pub fn new(facts: E, audit: A, policy: Policy) -> Self { /* defaults */ }
        pub fn with_lock_manager(mut self, lm: Box<dyn LockManager>) -> Self { /* ... */ }
        pub fn with_ownership_oracle(mut self, o: Box<dyn OwnershipOracle>) -> Self { /* ... */ }
        pub fn with_attestor(mut self, a: Box<dyn Attestor>) -> Self { /* ... */ }
        pub fn with_smoke_runner(mut self, s: Box<dyn SmokeTestRunner>) -> Self { /* ... */ }
        pub fn with_lock_timeout_ms(mut self, ms: u64) -> Self { /* ... */ }
        pub fn build(self) -> Switchyard<E, A> { /* ... */ }
    }
    ```

  - Keep `Switchyard::new(...)` intact; implement it via the builder defaults.
- Acceptance:
  - Example compiles:

    ```rust
    use switchyard::prelude::*;
    let api = ApiBuilder::new(facts, audit, Policy::default())
        .with_lock_timeout_ms(1500)
        .build();
    ```

## 3) Provide a fluent `PlanBuilder`

- File: `src/types/plan_builder.rs` (new)
  - Fluent DSL that produces `Plan`:

    ```rust
    pub struct PlanBuilder { link: Vec<LinkRequest>, restore: Vec<RestoreRequest> }
    impl PlanBuilder {
        pub fn new() -> Self { /* ... */ }
        pub fn link(mut self, source: SafePath, target: SafePath) -> Self { /* push */ }
        pub fn restore_latest(mut self, target: SafePath, tag: impl Into<String>) -> Self { /* ... */ }
        pub fn finish(self) -> Plan { /* ... */ }
    }
    ```

- File: `src/types/mod.rs`
  - `pub mod plan_builder;`
- Acceptance:
  - Example compiles:

    ```rust
    use switchyard::types::plan_builder::PlanBuilder;
    let plan = PlanBuilder::new()
        .link(src, tgt)
        .restore_latest(tgt2, "coreutils")
        .finish();
    ```

## 4) Config loader (opt-in, feature-gated)

- Files: `src/config/{mod.rs,policy.rs,api.rs}` (new), gated behind `features = ["config"]`
  - `fn load_policy_from_toml(path: &Path) -> Result<Policy, ConfigError>`
  - `fn build_api_from_toml<E, A>(facts: E, audit: A, path: &Path) -> Result<Switchyard<E, A>, ConfigError>`
- `Cargo.toml`: add `features = ["config"]` and `serde` dependencies under that feature.
- Acceptance:
  - Example compiles (behind feature):

    ```rust
    // cargo test --features config
    let api = switchyard::config::build_api_from_toml(facts, audit, "switchyard.toml")?;
    ```

## 5) Typed reports + (de)serialization helpers

- File: `src/types/report.rs`
  - Ensure `Plan`, `PreflightReport`, `ApplyReport` derive `Serialize`/`Deserialize` (or provide stable serialization helpers if internal invariants conflict).
  - Add helpers:

    ```rust
    impl ApplyReport { pub fn to_json(&self) -> serde_json::Value { /* ... */ } }
    impl TryFrom<serde_json::Value> for ApplyReport { type Error = ReportError; /* ... */ }
    ```

- Acceptance:
  - Example compiles:

    ```rust
    let v = report.to_json();
    let r = ApplyReport::try_from(v).unwrap();
    ```

## 6) Error ergonomics for consumers

- Files: `src/api/errors/mod.rs`, `src/types/error_id.rs`
  - Introduce `pub enum ErrorId { E_LOCKING, E_SMOKE, E_BACKUP_MISSING, /* ... */ }` with `serde` and `Display`.
  - Ensure `ApiError` contains a stable `ErrorId` and `Display` impl.
  - Provide `From` conversions from common lower-level errors.
- Acceptance:
  - Example compiles:

    ```rust
    match api.apply(&plan, ApplyMode::Commit) {
        Ok(r) => { /* ... */ }
        Err(e) => eprintln!("{} {}", e.id(), e),
    }
    ```

## 7) Logging DX for consumers

- Files: `src/logging/macros.rs` and `src/logging/test_utils.rs` (behind `features = ["test-utils"]`)
  - Macros like `audit_summary!(slog, stage, decision, { key: value, .. })` for common patterns.
  - `TestEmitter`, `TestAudit` helpers for consumer integration tests.
- Acceptance:
  - Example (test-only) compiles with `--features test-utils`.

## 8) Feature flags and doc cfg

- `Cargo.toml`
  - Add features: `prelude` (on by default), `config`, `serde-reports`, `test-utils`.
  - Gate modules with `#[cfg(feature = "...")]` and add `#![cfg_attr(docsrs, feature(doc_cfg))]`.
- Docs: annotate feature-gated items with `#[cfg_attr(feature = "...", doc(cfg(feature = "...")))]`.
- Acceptance: `cargo doc --all-features` shows feature badges, examples compile with `--doc`.

## 9) Crate-level docs and examples

- File: `src/lib.rs`
  - Add a crate-level quickstart with `prelude`, `ApiBuilder`, `Policy::production()` and a full plan → preflight → apply flow.
- Directory: `examples/`
  - Add `examples/quickstart.rs` and `examples/cookbook.rs` with small, runnable snippets.
- Acceptance:
  - `cargo run --example quickstart` runs (with mocked adapters where needed or behind features).

## 10) Curated re-exports (stable entry points)

- File: `src/lib.rs`
  - Provide stable namespaces:

    ```rust
    pub mod api;     // re-export Switchyard, ApiBuilder
    pub mod policy;  // re-export Policy and profiles
    pub mod types;   // re-export Plan, PlanInput, ApplyMode, IDs, Reports
    pub mod logging; // re-export logging facade/macros (where applicable)
    pub mod prelude; // curated one-liner
    ```

- Acceptance: common imports work from the top-level without deep paths.

---

## PR plan (additive)

- PR1: Prelude + ApiBuilder (no breaking changes).
- PR2: PlanBuilder + examples (quickstart, cookbook).
- PR3: Config loader (feature-gated) + docs.
- PR4: Typed reports and IDs + serde helpers.
- PR5: Error ergonomics (`ErrorId`, `ApiError` docs) + conversions.
- PR6: Logging macros + test-utils feature.
- PR7: Feature flags polish, doc cfg, crate-level quickstart.

## Acceptance gates

- Examples compile: `cargo run --example quickstart`.
- Docs compile: `cargo doc --all-features` with rendered feature badges.
- No API breaks: all existing public items remain; additions only.
- New items have rustdoc examples that compile with `cargo test --doc`.

---

## Related

- API DX/DW Overhaul (breaking): see `zrefactor/api_refactor.INSTRUCTIONS.md`
- Logging/Audit facade (breaking): see `zrefactor/logging_audit_refactor.INSTRUCTIONS.md`
- Policy refactor and gating evaluator (breaking): see `zrefactor/policy_refactor.INSTRUCTIONS.md`
- Preflight orchestration using policy-owned gating: see `zrefactor/preflight_gating_refactor.INSTRUCTIONS.md`
- Cohesion and target layout: see `zrefactor/responsibility_cohesion_report.md`
- Workspace index and guardrails: see `zrefactor/README.md` and `zrefactor/refactor_rulebook.INSTRUCTIONS.md`
