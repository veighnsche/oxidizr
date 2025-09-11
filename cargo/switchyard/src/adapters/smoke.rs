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
                let want = std::fs::canonicalize(source.as_path()).unwrap_or_else(|_| source.as_path().to_path_buf());
                let got = std::fs::canonicalize(&resolved).unwrap_or(resolved);
                if want != got {
                    return Err(SmokeFailure);
                }
            }
        }
        Ok(())
    }
}
