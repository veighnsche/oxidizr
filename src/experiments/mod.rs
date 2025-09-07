pub mod sudors;
pub mod uutils;

pub use uutils::UutilsExperiment;
pub use uutils::enable::*;

use crate::error::Result;
use crate::utils::worker::Worker;
use std::path::PathBuf;

pub use sudors::SudoRsExperiment;

pub enum Experiment<'a> {
    Uutils(UutilsExperiment),
    SudoRs(SudoRsExperiment<'a>),
}

impl<'a> Experiment<'a> {
    pub fn name(&self) -> String {
        match self {
            Experiment::Uutils(u) => u.name.clone(),
            Experiment::SudoRs(_) => "sudo-rs".to_string(),
        }
    }

    pub fn enable<W: Worker>(
        &self,
        worker: &W,
        assume_yes: bool,
        update_lists: bool,
        no_compatibility_check: bool,
    ) -> Result<()> {
        match self {
            Experiment::Uutils(u) => {
                if no_compatibility_check || u.check_compatible(worker)? {
                    u.enable(worker, assume_yes, update_lists)
                } else {
                    Ok(())
                }
            }
            Experiment::SudoRs(s) => {
                if no_compatibility_check || s.check_compatible(worker)? {
                    s.enable(worker, assume_yes, update_lists)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn disable<W: Worker>(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Uutils(u) => u.disable(worker, assume_yes, update_lists),
            Experiment::SudoRs(s) => s.disable(worker, assume_yes, update_lists),
        }
    }

    pub fn check_compatible<W: Worker>(&self, worker: &W) -> Result<bool> {
        match self {
            Experiment::Uutils(u) => u.check_compatible(worker),
            Experiment::SudoRs(s) => s.check_compatible(worker),
        }
    }

    pub fn list_targets<W: Worker>(&self, worker: &W) -> Result<Vec<std::path::PathBuf>> {
        match self {
            Experiment::Uutils(u) => u.list_targets(worker),
            Experiment::SudoRs(s) => s.list_targets(worker),
        }
    }
}

pub fn all_experiments<'a, W: Worker>(worker: &'a W) -> Vec<Experiment<'a>> {
    // Arch-oriented defaults; CLI may still construct custom instances.
    let coreutils = UutilsExperiment {
        name: "coreutils".into(),
        package: "uutils-coreutils".into(),
        unified_binary: Some(PathBuf::from("/usr/bin/coreutils")),
        bin_directory: PathBuf::from("/usr/lib/uutils/coreutils"),
    };
    let findutils = UutilsExperiment {
        name: "findutils".into(),
        package: "uutils-findutils".into(),
        unified_binary: None,
        bin_directory: PathBuf::from("/usr/lib/cargo/bin/findutils"),
    };
    let sudo = SudoRsExperiment { system: worker };
    vec![
        Experiment::Uutils(coreutils),
        Experiment::Uutils(findutils),
        Experiment::SudoRs(sudo),
    ]
}
