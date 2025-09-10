# Planned Types, Enums, and Traits (Planning Only)

Rust terminology and tentative signatures for review. No implementation code.

## Core Data Types

```rust
// Planning-only: conceptual types
pub struct SafePath { /* invariant: rooted; no ".." after normalization */ }

pub struct PlanInput {
    pub targets: Vec<SafePath>,           // paths to mutate (symlinks/links)
    pub providers: Vec<SafePath>,         // candidate sources
    pub policy: PolicyFlags,              // conservative defaults
}

pub struct Plan {
    pub plan_id: uuid::Uuid,              // UUIDv5
    pub actions: Vec<Action>,             // ordered, rollback-aware
}

pub enum ActionKind { ReplaceSymlink, RestoreFromBackup, Skip }

pub struct Action {
    pub action_id: uuid::Uuid,            // UUIDv5 per action
    pub path: SafePath,
    pub kind: ActionKind,
    pub metadata: ActionMeta,
}

pub struct ActionMeta {
    pub current_kind: String,             // file/dir/symlink/missing
    pub planned_kind: String,             // symlink/restore_from_backup/skip
}

pub enum ApplyMode { DryRun, RealRun }

pub struct PreflightRow { /* matches SPEC/preflight.yaml */ }

pub struct PreflightReport { pub rows: Vec<PreflightRow> }

pub struct ApplyReport { /* overall summary + per-step facts */ }

pub struct PolicyFlags {
    pub allow_degraded_fs: bool,
    pub strict_ownership: bool,
    pub disable_auto_rollback: bool,
}
```

## Error Taxonomy (maps to SPEC/error_codes.toml)

```rust
pub enum ErrorKind {
    E_POLICY,
    E_OWNERSHIP,
    E_LOCKING,
    E_ATOMIC_SWAP,
    E_EXDEV,
    E_BACKUP_MISSING,
    E_RESTORE_FAILED,
    E_SMOKE,
}

pub struct Error { pub kind: ErrorKind, pub msg: String }
```

## Adapter Traits (see also `20-adapters.md`)

```rust
pub trait OwnershipOracle {
    fn owner_of(&self, path: &SafePath) -> Result<OwnershipInfo, Error>;
}

pub trait LockManager {
    fn acquire_process_lock(&self) -> Result<LockGuard, Error>;
}

pub trait PathResolver {
    fn resolve(&self, bin: &str) -> Result<SafePath, Error>;
}

pub trait Attestor {
    fn sign(&self, bundle: &[u8]) -> Result<Signature, Error>;
}

pub trait SmokeTestRunner {
    fn run(&self, plan: &Plan) -> Result<(), Error>; // E_SMOKE on failure
}
```

## Additional Planning Types

```rust
// Planning-only pseudocode; not actual code

// Reports (summaries used by API)
pub struct ApplySummary {
    pub decision: String,                 // "success" | "failure"
    pub degraded: bool,                   // true if EXDEV fallback used
    pub exit_code: i32,                   // maps from ErrorKind when failure
}

pub struct ApplyReport {
    pub plan_id: uuid::Uuid,
    pub summary: ApplySummary,
    pub per_step_facts_path: SafePath,    // where JSONL facts were written
    pub partial_restoration: Option<Vec<SafePath>>, // on rollback failure
}

// Adapters bundle (see also impl/15-policy-and-adapters.md)
pub struct Adapters {
    pub ownership: Box<dyn OwnershipOracle + Send + Sync>,
    pub lock: Option<Box<dyn LockManager + Send + Sync>>,   // None in dev/test
    pub path: Box<dyn PathResolver + Send + Sync>,
    pub attest: Box<dyn Attestor + Send + Sync>,
    pub smoke: Box<dyn SmokeTestRunner + Send + Sync>,
}

// Policy defaults (conservative)
impl Default for PolicyFlags {
    fn default() -> Self {
        PolicyFlags {
            allow_degraded_fs: false,
            strict_ownership: true,
            disable_auto_rollback: false,
        }
    }
}
```

## Builders (Planning Sketch)

```rust
pub struct PlanBuilder { /* inputs and normalization state */ }
impl PlanBuilder {
    fn new() -> Self { /* ... */ }
    fn target(mut self, t: SafePath) -> Self { /* ... */ }
    fn provider(mut self, p: SafePath) -> Self { /* ... */ }
    fn policy(mut self, pol: PolicyFlags) -> Self { /* ... */ }
    fn build(self) -> PlanInput { /* normalized, sorted */ }
}
