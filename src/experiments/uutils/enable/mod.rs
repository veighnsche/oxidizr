pub mod core;
pub mod coreutils;
pub mod non_coreutils;
pub mod utils;

// Directly expose the enable function as a wrapper to the method on UutilsExperiment
pub fn enable<W: crate::utils::worker::Worker>(
    experiment: &crate::experiments::uutils::model::UutilsExperiment,
    worker: &W,
    assume_yes: bool,
    update_lists: bool,
) -> crate::error::Result<()> {
    experiment.enable(worker, assume_yes, update_lists)
}
