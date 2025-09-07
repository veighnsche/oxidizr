use crate::config::aur_helpers;
use crate::error::{CoreutilsError, Result};
use crate::utils::audit::AUDIT;
use std::process::Command;
use which::which;

/// Trait for package management operations
pub trait PackageManager {
    fn update(&self) -> Result<()>;
    fn install(&self, package: &str) -> Result<()>;
    fn remove(&self, package: &str) -> Result<()>;
    fn is_installed(&self, package: &str) -> Result<bool>;
}

/// Pacman-based package manager for Arch Linux
pub struct PacmanManager {
    aur_helper: String,
    dry_run: bool,
}

impl PacmanManager {
    pub fn new(aur_helper: String, dry_run: bool) -> Self {
        Self {
            aur_helper,
            dry_run,
        }
    }

    fn validate_package_name(&self, name: &str) -> Result<()> {
        if name.is_empty() || name.starts_with('-') {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "Invalid package name: {}",
                name
            )));
        }

        if name.len() > crate::config::security::MAX_PACKAGE_NAME_LENGTH {
            return Err(CoreutilsError::ExecutionFailed(
                "Package name too long".into(),
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '.')
        {
            return Err(CoreutilsError::ExecutionFailed(format!(
                "Invalid characters in package name: {}",
                name
            )));
        }

        Ok(())
    }

    fn get_aur_helper(&self) -> Option<String> {
        if !self.aur_helper.is_empty() && which(&self.aur_helper).is_ok() {
            return Some(self.aur_helper.clone());
        }

        for helper in &aur_helpers::DEFAULT_HELPERS {
            if which(helper).is_ok() {
                return Some(helper.to_string());
            }
        }

        None
    }
}

impl PackageManager for PacmanManager {
    fn update(&self) -> Result<()> {
        if self.dry_run {
            log::info!("[dry-run] pacman -Sy");
            return Ok(());
        }

        let status = Command::new("pacman").args(["-Sy"]).status()?;

        if status.success() {
            let _ = AUDIT.log_operation("PACKAGE_UPDATE", "pacman -Sy", true);
            Ok(())
        } else {
            let _ = AUDIT.log_operation("PACKAGE_UPDATE", "pacman -Sy", false);
            Err(CoreutilsError::ExecutionFailed("pacman -Sy failed".into()))
        }
    }

    fn install(&self, package: &str) -> Result<()> {
        self.validate_package_name(package)?;

        if self.dry_run {
            log::info!("[dry-run] pacman -S --noconfirm {}", package);
            return Ok(());
        }

        if self.is_installed(package)? {
            log::info!("Package '{}' already installed", package);
            return Ok(());
        }

        // Try pacman first
        let status = Command::new("pacman")
            .args(["-S", "--noconfirm", package])
            .status()?;

        if status.success() {
            let _ = AUDIT.log_operation("PACKAGE_INSTALL", package, true);
            return Ok(());
        }

        // Fall back to AUR helper
        if let Some(helper) = self.get_aur_helper() {
            let status = Command::new("sudo")
                .args([
                    "-u",
                    "builder",
                    "--",
                    &helper,
                    "-S",
                    "--noconfirm",
                    "--needed",
                    package,
                ])
                .status()?;

            if status.success() {
                let _ = AUDIT.log_operation("PACKAGE_INSTALL_AUR", package, true);
                return Ok(());
            }
        }

        let _ = AUDIT.log_operation("PACKAGE_INSTALL", package, false);
        Err(CoreutilsError::ExecutionFailed(format!(
            "Failed to install '{}'",
            package
        )))
    }

    fn remove(&self, package: &str) -> Result<()> {
        self.validate_package_name(package)?;

        if self.dry_run {
            log::info!("[dry-run] pacman -R --noconfirm {}", package);
            return Ok(());
        }

        let status = Command::new("pacman")
            .args(["-R", "--noconfirm", package])
            .status()?;

        if status.success() {
            let _ = AUDIT.log_operation("PACKAGE_REMOVE", package, true);
            Ok(())
        } else {
            let _ = AUDIT.log_operation("PACKAGE_REMOVE", package, false);
            Err(CoreutilsError::ExecutionFailed(format!(
                "Failed to remove '{}'",
                package
            )))
        }
    }

    fn is_installed(&self, package: &str) -> Result<bool> {
        self.validate_package_name(package)?;

        let status = Command::new("pacman").args(["-Qi", package]).status()?;

        Ok(status.success())
    }
}
