//! Smoke test adapter used to validate post-apply health.
//!
//! Minimal expectations for integrators:
//! - Implement `SmokeTestRunner` and inject it via `Switchyard::with_smoke_runner(...)`.
//! - Ensure the runner is deterministic and safe to execute repeatedly.
//! - At minimum, validate that each `EnsureSymlink` target resolves to the intended source.
//! - Optionally perform additional invariant checks (e.g., permissions, executable bits).
//!
//! Behavior in `ApplyMode::Commit` when `policy.require_smoke_in_commit=true`:
//! - If no runner is configured, apply fails with `E_SMOKE` and auto-rollback (unless `disable_auto_rollback`).
//! - If the runner returns `Err(SmokeFailure)`, apply fails with `E_SMOKE` and auto-rollback (unless disabled).
//!
use crate::types::plan::Plan;

#[derive(Debug)]
pub struct SmokeFailure;

pub trait SmokeTestRunner: Send + Sync {
    fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
}

/// DefaultSmokeRunner implements a minimal, no-op smoke suite.
/// In Sprint 2, the adapter is made available and can be enabled by integrators.
/// Future iterations will implement the SPEC §11 command set.
#[derive(Debug, Default)]
pub struct DefaultSmokeRunner;

impl SmokeTestRunner for DefaultSmokeRunner {
    fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
        // Deterministic subset: validate that each EnsureSymlink target points to the source.
        for act in &_plan.actions {
            if let crate::types::Action::EnsureSymlink { source, target } = act {
                let md = match std::fs::symlink_metadata(target.as_path()) {
                    Ok(m) => m,
                    Err(_) => return Err(SmokeFailure),
                };
                if !md.file_type().is_symlink() {
                    return Err(SmokeFailure);
                }
                let link = match std::fs::read_link(target.as_path()) {
                    Ok(p) => p,
                    Err(_) => return Err(SmokeFailure),
                };
                // Resolve relative link against target parent
                let resolved = if link.is_relative() {
                    match target.as_path().parent() {
                        Some(parent) => parent.join(link),
                        None => link,
                    }
                } else {
                    link
                };
                // Compare canonicalized paths where possible
                let want = std::fs::canonicalize(source.as_path())
                    .unwrap_or_else(|_| source.as_path().to_path_buf());
                let got = std::fs::canonicalize(&resolved).unwrap_or(resolved);
                if want != got {
                    return Err(SmokeFailure);
                }
            }
        }
        Ok(())
    }
}
