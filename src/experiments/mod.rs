pub mod sudors;
pub mod uutils;

pub use uutils::UutilsExperiment;
pub use uutils::enable::*;

use crate::error::Result;
use crate::utils::worker::Worker;
use std::path::PathBuf;
use crate::config::packages;

pub use sudors::SudoRsExperiment;

pub enum Experiment<'a, W: Worker> {
    Uutils(UutilsExperiment),
    SudoRs(SudoRsExperiment<'a, W>),
}

impl<'a, W: Worker> Experiment<'a, W> {
    pub fn name(&self) -> String {
        match self {
            Experiment::Uutils(u) => u.name.clone(),
            Experiment::SudoRs(_) => "sudo-rs".to_string(),
        }
    }

    pub fn enable(
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

    pub fn disable(&self, worker: &W, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Uutils(u) => u.disable(worker, assume_yes, update_lists),
            Experiment::SudoRs(s) => s.disable(worker, assume_yes, update_lists),
        }
    }

    pub fn check_compatible(&self, worker: &W) -> Result<bool> {
        match self {
            Experiment::Uutils(u) => u.check_compatible(worker),
            Experiment::SudoRs(s) => s.check_compatible(worker),
        }
    }

    pub fn list_targets(&self, worker: &W) -> Result<Vec<std::path::PathBuf>> {
        match self {
            Experiment::Uutils(u) => u.list_targets(worker),
            Experiment::SudoRs(s) => s.list_targets(worker),
        }
    }
}

pub fn all_experiments<'a, W: Worker>(worker: &'a W) -> Vec<Experiment<'a, W>> {
    let dist = worker.distribution().unwrap();
    let id = dist.id.to_ascii_lowercase();
    let is_supported_os = matches!(id.as_str(), "arch" | "manjaro" | "cachyos" | "endeavouros");

    let coreutils_pkg = if is_supported_os { packages::UUTILS_COREUTILS } else { "coreutils" };
    // Prefer binary AUR package for findutils when supported, falls back to repo findutils otherwise
    let findutils_pkg = if is_supported_os { packages::UUTILS_FINDUTILS } else { "findutils" };
    let sudo_pkg = if is_supported_os { packages::SUDO_RS } else { "sudo" };

    let coreutils = UutilsExperiment {
        name: "coreutils".into(),
        package_name: coreutils_pkg.to_string(),
        unified_binary: if is_supported_os { Some(PathBuf::from("/usr/bin/coreutils")) } else { None },
        bin_directory: if is_supported_os { PathBuf::from("/usr/lib/uutils/coreutils") } else { PathBuf::from("/usr/bin") },
    };

    let findutils = UutilsExperiment {
        name: "findutils".into(),
        package_name: findutils_pkg.to_string(),
        unified_binary: None,
        bin_directory: if is_supported_os { PathBuf::from("/usr/lib/cargo/bin/findutils") } else { PathBuf::from("/usr/bin") },
    };

    let sudo = SudoRsExperiment { system: worker, package_name: sudo_pkg.to_string() };

    vec![
        Experiment::Uutils(coreutils),
        Experiment::Uutils(findutils),
        Experiment::SudoRs(sudo),
    ]
}
