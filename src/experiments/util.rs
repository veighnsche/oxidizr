use crate::error::{Error, Result};
use crate::system::Worker;
use std::path::{Path, PathBuf};

/// Resolve a target path under /usr/bin
pub fn resolve_usrbin(filename: &str) -> PathBuf {
    Path::new("/usr/bin").join(filename)
}

/// Create symlinks for (filename -> src) applets, using a target resolver.
/// Adds detailed logs and wraps errors with src/target context.
pub fn create_symlinks<F>(worker: &Worker, applets: &[(String, PathBuf)], resolve: F) -> Result<()>
where
    F: Fn(&str) -> PathBuf,
{
    for (filename, src) in applets {
        let target = resolve(filename);
        log::info!("Symlinking {} -> {}", src.display(), target.display());
        if let Err(e) = worker.replace_file_with_symlink(src, &target) {
            log::error!(
                "❌ Failed to create symlink: src={} -> target={}: {}",
                src.display(),
                target.display(),
                e
            );
            return Err(Error::ExecutionFailed(format!(
                "failed to symlink {} -> {}: {}",
                src.display(),
                target.display(),
                e
            )));
        }
    }
    Ok(())
}

/// Restore a list of targets, logging each and surfacing errors with context.
pub fn restore_targets(worker: &Worker, targets: &[PathBuf]) -> Result<()> {
    for target in targets {
        log::info!("[disable] Restoring {} (if backup exists)", target.display());
        if let Err(e) = worker.restore_file(target) {
            log::error!(
                "❌ Failed to restore {}: {}",
                target.display(),
                e
            );
            return Err(Error::ExecutionFailed(format!(
                "failed to restore {}: {}",
                target.display(),
                e
            )));
        }
    }
    Ok(())
}

/// Log a short summary of the first `max_items` applets to be linked.
pub fn log_applets_summary(prefix: &str, applets: &[(String, PathBuf)], max_items: usize) {
    log::info!(
        "Preparing to link {} applet(s) for {}",
        applets.len(),
        prefix
    );
    for (i, (filename, src)) in applets.iter().enumerate() {
        if i >= max_items {
            log::info!("  (…truncated)");
            break;
        }
        let target = resolve_usrbin(filename);
        log::info!("  [{}] {} -> {}", i + 1, src.display(), target.display());
    }
}

/// Verify a package is installed, emitting explicit logs.
pub fn verify_installed(worker: &Worker, package: &str) -> Result<()> {
    if worker.check_installed(package)? {
        log::info!("✅ Expected: '{}' installed, Received: present", package);
        Ok(())
    } else {
        log::error!("❌ Expected: '{}' installed, Received: absent", package);
        Err(Error::ExecutionFailed(format!(
            "package '{}' not installed after operation",
            package
        )))
    }
}

/// Verify a package is removed, emitting explicit logs.
pub fn verify_removed(worker: &Worker, package: &str) -> Result<()> {
    if worker.check_installed(package)? {
        log::error!("❌ Expected: '{}' absent after removal, Received: present", package);
        Err(Error::ExecutionFailed(format!(
            "package '{}' still installed after removal",
            package
        )))
    } else {
        log::info!("✅ Expected: '{}' absent after removal, Received: absent", package);
        Ok(())
    }
}
