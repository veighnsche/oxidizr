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
        // Minimal placeholder: always succeeds.
        Ok(())
    }
}
