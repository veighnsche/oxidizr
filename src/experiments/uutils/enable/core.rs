use crate::config::packages;
use crate::error::{CoreutilsError, Result};
use crate::experiments::uutils::model::UutilsExperiment;
use crate::utils::audit::AUDIT;
use crate::utils::worker::Worker;
use std::io::{self, Write};

impl UutilsExperiment {
    /// Enables the uutils experiment by installing the package and setting up symlinks.
    pub fn enable<W: Worker>(
        &self,
        worker: &W,
        assume_yes: bool,
        update_lists: bool,
    ) -> Result<()> {
        if !self.check_compatible(worker)? {
            return Err(CoreutilsError::Incompatible(
                "Unsupported Arch release".into(),
            ));
        }
        if update_lists {
            log::info!("Updating package lists...");
            worker.update_packages(assume_yes)?;
        }
        // Repo capability checks and availability gating
        let extra_available = worker.extra_repo_available()?;
        let aur_helper = worker.aur_helper_name()?;
        let aur_available = aur_helper.is_some();
        let _ = AUDIT.log_provenance(
            "uutils.enable",
            "repo_capabilities",
            "observed",
            &format!(
                "extra_available={}, aur_available={}, helper={:?}",
                extra_available, aur_available, aur_helper
            ),
            "",
            None,
        );

        if !extra_available && !aur_available {
            return Err(CoreutilsError::ExecutionFailed(
                "You do not have access to extra or AUR repositories.".into(),
            ));
        }

        if self.package_name == packages::UUTILS_COREUTILS {
            if !extra_available {
                return Err(CoreutilsError::ExecutionFailed(
                    "Cannot download because the extra repository is not available.".into(),
                ));
            }
        } else if self.package_name == packages::UUTILS_FINDUTILS {
            if !aur_available {
                return Err(CoreutilsError::ExecutionFailed(
                    "Cannot download uutils-findutils because no AUR helper is installed.".into(),
                ));
            }
        }

        // Already-installed detection and prompt to reuse
        if worker.check_installed(&self.package_name)? {
            let mut reuse = true;
            if !assume_yes {
                print!(
                    "Detected {} installed. Use existing instead of downloading? [Y/n]: ",
                    self.package_name
                );
                io::stdout().flush().ok();
                let mut s = String::new();
                if io::stdin().read_line(&mut s).is_ok() {
                    let ans = s.trim().to_ascii_lowercase();
                    reuse = ans.is_empty() || ans == "y" || ans == "yes";
                }
            }
            let _ = AUDIT.log_provenance(
                "uutils.enable",
                "already_installed",
                if reuse { "reuse" } else { "reinstall_requested" },
                &self.package_name,
                "",
                None,
            );
            if reuse {
                log::info!(
                    "Using existing installation of '{}' (no download)",
                    self.package_name
                );
            } else {
                log::info!(
                    "Reinstall requested for '{}' (will attempt package install)",
                    self.package_name
                );
            }
            // Proceed to install call either way; worker.install_package will be a no-op if present
        }

        log::info!("Installing package: {}", self.package_name);
        worker.install_package(&self.package_name, assume_yes)?;

        let applets = if self.name == "coreutils" {
            self.handle_coreutils_applets(worker)?
        } else {
            self.handle_non_coreutils_applets(worker)?
        };

        if applets.is_empty() {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "No applets selected for family '{}' (bin_directory: {}). This usually means the package did not install binaries in expected locations. \
                 Hints: ensure '{}' is installed; verify presence under {} or cargo-style /usr/lib/cargo/bin/<family>/.",
                self.name,
                self.bin_directory.display(),
                self.package_name,
                self.bin_directory.display()
            )));
        }

        self.log_applets_summary(&applets);
        self.create_symlinks(worker, &applets)
    }
}
