pub mod checksums;
pub mod constants;
pub mod coreutils;
pub mod findutils;
pub mod sudors;
pub mod util;

use crate::checks::Distribution;
use crate::error::{Error, Result};
use crate::logging::audit_event;
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
    Checksums(checksums::ChecksumsExperiment),
}

impl Experiment {
    pub fn name(&self) -> &str {
        match self {
            Experiment::Coreutils(e) => e.name(),
            Experiment::Findutils(e) => e.name(),
            Experiment::SudoRs(e) => e.name(),
            Experiment::Checksums(e) => e.name(),
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
        let _span = tracing::info_span!(
            "experiment_enable",
            experiment = %self.name(),
            distro = %distro.id,
            skip_compat_check
        )
        .entered();

        // Check compatibility unless overridden
        if !skip_compat_check {
            let compatible = match self {
                Experiment::Coreutils(e) => e.check_compatible(&distro)?,
                Experiment::Findutils(e) => e.check_compatible(&distro)?,
                Experiment::SudoRs(e) => e.check_compatible(&distro)?,
                Experiment::Checksums(e) => e.check_compatible(&distro)?,
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
            Experiment::Checksums(e) => e.enable(worker, assume_yes, update_lists),
        }
    }

    pub fn disable(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Coreutils(e) => e.disable(worker, assume_yes, update_lists),
            Experiment::Findutils(e) => e.disable(worker, assume_yes, update_lists),
            Experiment::SudoRs(e) => e.disable(worker, assume_yes, update_lists),
            Experiment::Checksums(e) => e.disable(worker, assume_yes, update_lists),
        }
    }

    pub fn remove(&self, worker: &Worker, assume_yes: bool, update_lists: bool) -> Result<()> {
        match self {
            Experiment::Coreutils(e) => e.remove(worker, assume_yes, update_lists),
            Experiment::Findutils(e) => e.remove(worker, assume_yes, update_lists),
            Experiment::SudoRs(e) => e.remove(worker, assume_yes, update_lists),
            Experiment::Checksums(e) => e.remove(worker, assume_yes, update_lists),
        }
    }

    pub fn check_compatible(&self, distro: &Distribution) -> Result<bool> {
        match self {
            Experiment::Coreutils(e) => e.check_compatible(distro),
            Experiment::Findutils(e) => e.check_compatible(distro),
            Experiment::SudoRs(e) => e.check_compatible(distro),
            Experiment::Checksums(e) => e.check_compatible(distro),
        }
    }

    pub fn list_targets(&self) -> Vec<PathBuf> {
        match self {
            Experiment::Coreutils(e) => e.list_targets(),
            Experiment::Findutils(e) => e.list_targets(),
            Experiment::SudoRs(e) => e.list_targets(),
            Experiment::Checksums(e) => e.list_targets(),
        }
    }
}

/// Get all available experiments
pub fn all_experiments() -> Vec<Experiment> {
    // Order matters for --all: install AUR packages (findutils) before flipping checksums,
    // so that makepkg still has access to GNU checksum tools during builds. Flip checksums
    // last for safety after core utils are active.
    vec![
        Experiment::Findutils(findutils::FindutilsExperiment::new()),
        Experiment::Coreutils(coreutils::CoreutilsExperiment::new()),
        Experiment::SudoRs(sudors::SudoRsExperiment::new()),
        Experiment::Checksums(checksums::ChecksumsExperiment::new()),
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
    // Probe whether the package exists in official repos (helps clarify AUR-only cases)
    let official_has = worker.repo_has_package(package).unwrap_or(false);

    let _ = audit_event(
        "experiments",
        "repo_capabilities",
        "observed",
        &format!(
            "extra_available={}, aur_available={}, official_has={}, helper={:?}",
            extra_available, aur_available, official_has, aur_helper
        ),
        "",
        None,
    );

    // Gate on repo availability
    if !extra_available && !aur_available {
        let details = format!(
            "no 'extra' repo and no AUR helper available (extra_available={}, aur_available={})",
            extra_available, aur_available
        );
        let _ = audit_event(
            "experiments",
            "repo_gate_failed",
            "missing_repo_and_helper",
            &details,
            "",
            None,
        );
        return Err(Error::RepoGateFailed {
            package: package.into(),
            details,
        });
    }

    // Per-package repo requirements
    match package {
        UUTILS_COREUTILS | SUDO_RS => {
            if !extra_available {
                let details = "extra repo unavailable".to_string();
                let _ = audit_event(
                    "experiments",
                    "repo_gate_failed",
                    "extra_missing",
                    &details,
                    package,
                    None,
                );
                return Err(Error::RepoGateFailed {
                    package: package.into(),
                    details,
                });
            }
            // Gate on actual package presence in the repo to avoid ambiguous 'not found' failures
            match worker.repo_has_package(package) {
                Ok(true) => {
                    tracing::info!(
                        "✅ Package '{}' present in repositories (pacman -Si)",
                        package
                    );
                }
                Ok(false) => {
                    let details = "package not present in repos (pacman -Si)".to_string();
                    let _ = audit_event(
                        "experiments",
                        "repo_gate_failed",
                        "package_absent",
                        &details,
                        package,
                        None,
                    );
                    return Err(Error::RepoGateFailed {
                        package: package.into(),
                        details,
                    });
                }
                Err(e) => {
                    tracing::warn!("Warning: failed to probe repo for '{}': {}", package, e);
                }
            }
        }
        UUTILS_FINDUTILS => {
            if !aur_available {
                let details = "no AUR helper available".to_string();
                let _ = audit_event(
                    "experiments",
                    "repo_gate_failed",
                    "aur_helper_missing",
                    &details,
                    package,
                    None,
                );
                return Err(Error::RepoGateFailed {
                    package: package.into(),
                    details,
                });
            }
        }
        _ => {}
    }

    tracing::info!(
        "✅ Repository gating satisfied for '{}': extra_available={}, aur_available={}",
        package,
        extra_available,
        aur_available
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

        let _ = audit_event(
            "experiments",
            "already_installed",
            if reuse {
                "reuse"
            } else {
                "reinstall_requested"
            },
            package,
            "",
            None,
        );

        if reuse {
            tracing::info!("Using existing installation of '{}' (no download)", package);
        } else {
            tracing::info!(
                "Reinstall requested for '{}' (will attempt package install)",
                package
            );
        }
    }

    Ok(())
}
