pub mod coreutils;
pub mod findutils;
pub mod sudors;
pub mod util;

use crate::checks::Distribution;
use crate::error::{Error, Result};
use crate::logging::PROVENANCE;
use crate::system::Worker;
use std::io::{self, Write};
use std::path::PathBuf;

/// Package constants
pub const UUTILS_COREUTILS: &str = "uutils-coreutils";
pub const UUTILS_FINDUTILS: &str = "uutils-findutils-bin";
pub const SUDO_RS: &str = "sudo-rs";

/// Experiment trait for common operations
pub trait ExperimentOps {
    fn name(&self) -> &str;
    fn package_name(&self) -> &str;
    fn check_compatible(&self, distro: &Distribution) -> Result<bool>;
    fn enable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()>;
    fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()>;
    fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()>;
    fn list_targets(&self) -> Vec<PathBuf>;
}

/// Unified experiment enum
pub enum Experiment {
    Coreutils(coreutils::CoreutilsExperiment),
    Findutils(findutils::FindutilsExperiment),
    SudoRs(sudors::SudoRsExperiment),
}

impl Experiment {
    pub fn name(&self) -> &str {
        match self {
            Experiment::Coreutils(e) => e.name(),
            Experiment::Findutils(e) => e.name(),
            Experiment::SudoRs(e) => e.name(),
        }
    }

    pub fn enable(
        &self,
        worker: &Worker,
        assume_yes: bool,
        update_lists: bool,
        skip_compat_check: bool,
    ) -> Result<()> {
        let distro = worker.distribution()?;
        
        // Check compatibility unless overridden
        if !skip_compat_check {
            let compatible = match self {
                Experiment::Coreutils(e) => e.check_compatible(&distro)?,
                Experiment::Findutils(e) => e.check_compatible(&distro)?,
                Experiment::SudoRs(e) => e.check_compatible(&distro)?,
            };
            
            if !compatible {
                return Err(Error::Incompatible(format!(
                    "Unsupported distro '{}'. Supported: {:?}. Pass --skip-compatibility-check to override.",
                    distro.id,
                    crate::checks::SUPPORTED_DISTROS
                )));
            }
        }
        
        match self {
            Experiment::Coreutils(e) => e.enable(worker, assume_yes, update_lists),
            Experiment::Findutils(e) => e.enable(worker, assume_yes, update_lists),
            Experiment::SudoRs(e) => e.enable(worker, assume_yes, update_lists),
        }
    }

    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Coreutils(e) => e.disable(worker, assume_yes, update_lists),
            Experiment::Findutils(e) => e.disable(worker, assume_yes, update_lists),
            Experiment::SudoRs(e) => e.disable(worker, assume_yes, update_lists),
        }
    }

    pub fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Coreutils(e) => e.remove(worker, assume_yes, update_lists),
            Experiment::Findutils(e) => e.remove(worker, assume_yes, update_lists),
            Experiment::SudoRs(e) => e.remove(worker, assume_yes, update_lists),
        }
    }

    pub fn check_compatible(&self, distro: &Distribution) -> Result<bool> {
        match self {
            Experiment::Coreutils(e) => e.check_compatible(distro),
            Experiment::Findutils(e) => e.check_compatible(distro),
            Experiment::SudoRs(e) => e.check_compatible(distro),
        }
    }

    pub fn list_targets(&self) -> Vec<PathBuf> {
        match self {
            Experiment::Coreutils(e) => e.list_targets(),
            Experiment::Findutils(e) => e.list_targets(),
            Experiment::SudoRs(e) => e.list_targets(),
        }
    }
}

/// Get all available experiments
pub fn all_experiments() -> Vec<Experiment> {
    vec![
        Experiment::Coreutils(coreutils::CoreutilsExperiment::new()),
        Experiment::Findutils(findutils::FindutilsExperiment::new()),
        Experiment::SudoRs(sudors::SudoRsExperiment::new()),
    ]
}

/// Common download flow implementation with repo gating and prompts
pub fn check_download_prerequisites(
    worker: &Worker,
    package: &str,
    assume_yes: bool,
) -> Result<()> {
    // Check repo capabilities
    let extra_available = worker.extra_repo_available()?;
    let aur_helper = worker.aur_helper_name()?;
    let aur_available = aur_helper.is_some();
    
    let _ = PROVENANCE.log(
        "experiments",
        "repo_capabilities",
        "observed",
        &format!(
            "extra_available={}, aur_available={}, helper={:?}",
            extra_available, aur_available, aur_helper
        ),
        "",
        None,
    );

    // Gate on repo availability
    if !extra_available && !aur_available {
        log::error!(
            "❌ Expected: access to 'extra' repo or an AUR helper; Received: extra_available={}, aur_available={}",
            extra_available,
            aur_available
        );
        return Err(Error::ExecutionFailed(
            format!(
                "❌ Expected: access to 'extra' repo or AUR helper; Received: extra_available={}, aur_available={}",
                extra_available, aur_available
            )
            .into(),
        ));
    }

    // Per-package repo requirements
    match package {
        UUTILS_COREUTILS | SUDO_RS => {
            if !extra_available {
                log::error!(
                    "❌ Expected: extra repo available for '{}'; Received: extra_available=false",
                    package
                );
                return Err(Error::ExecutionFailed(
                    format!(
                        "❌ Expected: extra repo available for '{}'; Received: extra_available=false",
                        package
                    )
                    .into(),
                ));
            }
            // Gate on actual package presence in the repo to avoid ambiguous 'not found' failures
            match worker.repo_has_package(package) {
                Ok(true) => {
                    log::info!("✅ Package '{}' present in repositories (pacman -Si)", package);
                }
                Ok(false) => {
                    log::error!(
                        "❌ Package '{}' not found in repositories (pacman -Si). Mirrors may be out of sync or the repo set is incomplete.",
                        package
                    );
                    return Err(Error::ExecutionFailed(format!(
                        "package '{}' not found in repositories (pacman -Si). Try 'pacman -Syy' to refresh, switch mirrors, or rerun later.",
                        package
                    )));
                }
                Err(e) => {
                    log::warn!("Warning: failed to probe repo for '{}': {}", package, e);
                }
            }
        }
        UUTILS_FINDUTILS => {
            if !aur_available {
                log::error!(
                    "❌ Expected: an AUR helper present for '{}'; Received: none",
                    package
                );
                return Err(Error::ExecutionFailed(
                    format!(
                        "❌ Expected: an AUR helper present for '{}'; Received: none",
                        package
                    )
                    .into(),
                ));
            }
        }
        _ => {}
    }

    log::info!(
        "✅ Repository gating satisfied for '{}': extra_available={}, aur_available={}",
        package, extra_available, aur_available
    );

    // Check if already installed and prompt for reuse
    if worker.check_installed(package)? {
        let mut reuse = true;
        if !assume_yes {
            print!(
                "Detected {} installed. Use existing instead of downloading? [Y/n]: ",
                package
            );
            io::stdout().flush().ok();
            let mut s = String::new();
            if io::stdin().read_line(&mut s).is_ok() {
                let ans = s.trim().to_ascii_lowercase();
                reuse = ans.is_empty() || ans == "y" || ans == "yes";
            }
        }
        
        let _ = PROVENANCE.log(
            "experiments",
            "already_installed",
            if reuse { "reuse" } else { "reinstall_requested" },
            package,
            "",
            None,
        );
        
        if reuse {
            log::info!(
                "Using existing installation of '{}' (no download)",
                package
            );
        } else {
            log::info!(
                "Reinstall requested for '{}' (will attempt package install)",
                package
            );
        }
    }
    
    Ok(())
}
