use crate::error::Result;
use crate::utils::worker::Worker;
use crate::experiments::uutils::constants::SYSTEM_BIN_DIR;
use crate::experiments::uutils::model::UutilsExperiment;
use crate::experiments::uutils::utils::resolve_target;
use std::path::PathBuf;

impl UutilsExperiment {
    /// Logs a summary of the applets to be linked.
    pub fn log_applets_summary(&self, applets: &[(String, PathBuf)]) {
        log::info!(
            "Preparing to link {} applet(s) for '{}' (package: {})",
            applets.len(), self.name, self.package
        );
        for (i, (filename, src)) in applets.iter().enumerate().take(8) {
            let target = PathBuf::from(if cfg!(test) { "bin" } else { SYSTEM_BIN_DIR }).join(filename);
            log::info!("  [{}] {} -> {}{}", i + 1, src.display(), target.display(), if i + 1 == 8 && applets.len() > 8 { " (…truncated)" } else { "" });
        }
    }

    /// Creates symlinks for the selected applets.
    pub fn create_symlinks<W: Worker>(&self, worker: &W, applets: &[(String, PathBuf)]) -> Result<()> {
        for (filename, src) in applets {
            let target = resolve_target(worker, filename);
            let src_exists = src.exists();
            let tgt_exists = target.exists();
            log::info!(
                "Symlinking {} -> {} (src_exists={}, target_exists={})",
                src.display(), target.display(), src_exists, tgt_exists
            );
            worker.replace_file_with_symlink(src, &target)?;
        }
        Ok(())
    }
}
