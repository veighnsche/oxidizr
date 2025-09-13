# Connecting Gherkin Features to the Switchyard Library

This guide explains how to execute the Gherkin `.feature` files under `cargo/switchyard/SPEC/features/` against the Switchyard Rust library API using a thin BDD adapter. It provides two practical options (Rust-native and file-based) and shows how to capture facts (Audit v2) and assert oracles in step definitions.

- Library under test: `cargo/switchyard/src/` (public facade in `src/api/mod.rs` → `Switchyard`)
- Features: `cargo/switchyard/SPEC/features/*.feature`
- Requirements index: `cargo/switchyard/SPEC/requirements.yaml`

Use the builder as the standard construction path for the API (`Switchyard::builder(...)`).

---

## Option A: Rust-Native Adapter (cucumber-rs)

This approach runs `.feature` files directly from Rust tests using [cucumber-rs], calling Switchyard APIs inside step definitions.

[cucumber-rs]: https://github.com/cucumber-rs/cucumber

### 1) Add dev-dependencies

In `cargo/switchyard/Cargo.toml`:

```toml
[dev-dependencies]
cucumber = "0.20"         # or current version
async-trait = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```

If you prefer to validate facts by writing JSONL to disk, also enable the optional file sink in tests:

```toml
[features]
file-logging = []
```

Run with `--features file-logging` when you want the file-backed sink.

### 2) Create a test-only Facts/Audit sinks

Default `JsonlSink` is a no-op. For test assertions, provide either an in-memory collector or the file-backed sink (behind `file-logging`).

```rust
// tests/bdd_support.rs
use std::sync::{Arc, Mutex};
use log::Level;
use serde_json::Value;
use switchyard::logging::{FactsEmitter, AuditSink};

#[derive(Clone, Default)]
pub struct CollectingEmitter(pub Arc<Mutex<Vec<Value>>>);

impl FactsEmitter for CollectingEmitter {
    fn emit(&self, _subsystem: &str, _event: &str, _decision: &str, fields: Value) {
        self.0.lock().unwrap().push(fields);
    }
}

#[derive(Clone, Default)]
pub struct CollectingAudit(pub Arc<Mutex<Vec<(Level, String)>>>);

impl AuditSink for CollectingAudit {
    fn log(&self, level: Level, msg: &str) {
        self.0.lock().unwrap().push((level, msg.to_string()));
    }
}
```

Alternatively (disk-based, feature-gated):

```rust
// Use when running with: cargo test -p switchyard --features file-logging
use switchyard::logging::facts::FileJsonlSink;
let facts = FileJsonlSink::new("/tmp/switchyard-bdd/facts.jsonl");
let audit = FileJsonlSink::new("/tmp/switchyard-bdd/audit.jsonl");
```

### 3) Define the BDD World and map Given/When/Then

Create `tests/bdd.rs` (or `tests/bdd/main.rs`). The World holds a `Switchyard`, any constructed `Plan`, last `ApplyReport`, and collected facts.

```rust
// tests/bdd.rs
use async_trait::async_trait;
use cucumber::{given, when, then, World as CukeWorld, WorldInit};
use switchyard::api::Switchyard;
use switchyard::policy::Policy;
use switchyard::types::{ApplyMode, Plan, PlanInput, PreflightReport, safepath::SafePath};

mod bdd_support;
use bdd_support::{CollectingEmitter, CollectingAudit};

#[derive(Debug, Default, WorldInit)]
pub struct World {
    root: Option<std::path::PathBuf>,
    api: Option<Switchyard<CollectingEmitter, CollectingAudit>>, 
    plan: Option<Plan>,
    preflight: Option<PreflightReport>,
    apply_report: Option<switchyard::types::ApplyReport>,
    facts: CollectingEmitter,
    audit: CollectingAudit,
}

#[async_trait]
impl CukeWorld for World {
    type Error = std::convert::Infallible;
    async fn new() -> Result<Self, Self::Error> { Ok(Self::default()) }
}

#[given(regex = "^a plan with a single symlink replacement action$")]
async fn given_single_symlink(world: &mut World) {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Construct API via the builder (standard construction path)
    let policy = Policy::default();
    let api = Switchyard::builder(world.facts.clone(), world.audit.clone(), policy).build();
    world.api = Some(api);
    world.root = Some(root.clone());

    // Build SafePaths under the temp root (never use system paths in tests)
    let link = SafePath::from_rooted(&root, "bin/ls").unwrap();
    let target_a = SafePath::from_rooted(&root, "providers/A/ls").unwrap();

    // Populate filesystem topology as needed for the scenario (elided)
    // std::fs::create_dir_all(root.join("providers/A"))?; std::os::unix::fs::symlink(...)

    // Build a PlanInput and call builder API to produce a Plan
    // (Consult src/types/plan.rs for exact constructors.)
    let input = PlanInput::default(); // placeholder; fill with EnsureSymlink(link, target_a)
    world.plan = Some(world.api.as_ref().unwrap().plan(input));
}

#[when(regex = "^I apply the plan in Commit mode$")]
async fn when_apply_commit(world: &mut World) {
    let api = world.api.as_ref().unwrap();
    let plan = world.plan.as_ref().unwrap();
    let report = api.apply(plan, ApplyMode::Commit).unwrap();
    world.apply_report = Some(report);
}

#[then(regex = "^the target path resolves to providerB/ls without any intermediate missing path visible$")]
async fn then_visibility_ok(world: &mut World) {
    // Perform FS assertions under world.root; check that the link resolves to providerB/ls
    // and there was no window of missing path (AtomicReplace property). See SPEC §2.1.
}

#[tokio::main]
async fn main() { 
    // Point at the repo’s feature files
    World::run("cargo/switchyard/SPEC/features").await;
}
```

Notes:

- Keep all filesystem operations under a temporary root. Construct `SafePath` via `SafePath::from_rooted` (see `src/types/safepath.rs`).
- Use the `Switchyard::builder(...)` to construct the API and attach adapters (e.g., `with_lock_manager`, `with_smoke_runner`) as scenarios require.
- For preflight YAML parity (REQ-PF1), call the YAML exporter in `preflight/yaml.rs::to_yaml()` and compare bytes between DryRun and Commit.

### 4) Run BDD

- Run all features:

```bash
cargo test -p switchyard --test bdd -- --nocapture
```

- Run with file sink (write JSONL), then validate lines:

```bash
cargo test -p switchyard --test bdd --features file-logging -- --nocapture
```

You can filter by tags using cucumber-rs’s runtime options or by splitting feature sets.

---

## Option B: File-Backed Facts Adapter (no custom emitter)

If you prefer not to implement an in-memory emitter, enable the `file-logging` feature and create a `FileJsonlSink` for `facts` and `audit`. Step definitions can read and parse JSONL files to assert Audit v2 envelope and scenario-specific fields.

```rust
use switchyard::logging::facts::FileJsonlSink;
use switchyard::api::Switchyard;
use switchyard::policy::Policy;

let facts = FileJsonlSink::new("/tmp/switchyard-bdd/facts.jsonl");
let audit = FileJsonlSink::new("/tmp/switchyard-bdd/audit.jsonl");
let api = Switchyard::builder(facts, audit, Policy::default()).build();
```

From your Then steps, load `/tmp/switchyard-bdd/facts.jsonl` lines, `serde_json::from_str` them, and validate fields against `/SPEC/audit_event.v2.schema.json` using your preferred JSON Schema validator.

---

## Mapping to `steps-contract.yaml`

`cargo/switchyard/SPEC/features/steps-contract.yaml` enumerates canonical phrases and expected effects. Keep your step regex/text aligned with those phrases to maintain traceability.

- Example: "Given a plan with a single symlink replacement action" → constructs a `Plan` from a `PlanInput` with one `EnsureSymlink` action.
- Example: "Then apply.attempt includes lock_wait_ms" → parse collected facts and assert presence of `lock_wait_ms` in `apply.attempt` events.

---

## Coverage, Tags, and Traceability

- Scenario tags (`@REQ-…`) in `.feature` files map to entries in `SPEC/requirements.yaml`.
- Keep scenario names in sync with `verify.bdd` entries in the requirements index to enable automated coverage checks (see `SPEC/tools/traceability.py`).

---

## CI Integration (optional)

- For golden fixtures, use `test_ci_runner.py --golden <scenario>` which runs named Rust tests that write canon JSON under `$GOLDEN_OUT_DIR` and diffs against `cargo/switchyard/tests/golden/...`.
- You can call BDD scenarios from a small Rust binary or wrapper test. For example, group smoke/EXDEV BDD into a `#[test]` that invokes the cucumber runner and asserts a successful exit.

---

## Gotchas and Best Practices

- Always operate under temporary directories in tests; never mutate system paths. Use `SafePath::from_rooted` everywhere.
- Use the builder (`Switchyard::builder`) as the default way to construct the API and attach adapters per scenario.
- DryRun vs Commit: prefer DryRun for deterministic fact comparisons; Commit when validating locking, EXDEV, restore/rollback, and smoke.
- If using the file sink, remember to clean up or isolate output paths per scenario to avoid cross-test interference.

---

## Minimal Checklist to Get Started

1. Add `cucumber`, `tokio`, `async-trait`, and `serde_json` as dev-dependencies.
2. Create `tests/bdd_support.rs` with `CollectingEmitter` and `CollectingAudit`.
3. Create `tests/bdd.rs`:
   - Define a `World` holding `Switchyard`, `Plan`, reports, and sinks.
   - Implement Given/When/Then mapping to Switchyard APIs.
   - Point the runner at `cargo/switchyard/SPEC/features/`.
4. Run: `cargo test -p switchyard --test bdd -- --nocapture` (and optionally `--features file-logging`).

That’s it. You can now execute the Gherkin features against the Switchyard library code and assert Audit v2 facts, preflight YAML parity, and error/exit-code classifications.
