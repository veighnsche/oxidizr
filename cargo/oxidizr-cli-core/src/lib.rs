//! Common helpers for oxidizr-* CLIs (API builder, prompts, cross-distro core)

pub mod prompts {
    use std::io::{self, Write};

    // Only prompt on TTY; in non-tty contexts (CI), do not block and proceed.
    pub fn should_proceed(assume_yes: bool, _root: &std::path::Path) -> bool {
        if assume_yes {
            return true;
        }

        if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
            eprintln!("This will modify the target root. Proceed? [y/N]: ");
            let _ = io::stderr().flush();
            let mut buf = String::new();
            if io::stdin().read_line(&mut buf).is_ok() {
                let s = buf.trim().to_lowercase();
                return s == "y" || s == "yes";
            }
            false
        } else {
            // Non-interactive: treat --commit as explicit consent already.
            true
        }
    }
}

pub mod api {
    use std::path::PathBuf;

    use switchyard::adapters::{DefaultSmokeRunner, FileLockManager, FsOwnershipOracle};
    use switchyard::logging::JsonlSink;
    use switchyard::policy::Policy;
    use switchyard::Switchyard;

    pub fn build_api(policy: Policy, lock_path: PathBuf) -> Switchyard<JsonlSink, JsonlSink> {
        Switchyard::builder(JsonlSink::default(), JsonlSink::default(), policy)
            .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
            .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
            .with_ownership_oracle(Box::new(FsOwnershipOracle::default()))
            .build()
    }
}

// Cross-distro core modules
pub mod packages;
pub mod adapter;
pub mod coverage;

// Re-exports for convenience
pub use adapter::DistroAdapter;
pub use coverage::{coverage_check, coverage_preflight, discover_applets_with_allow, intersect_distro_with_replacement, resolve_applets_for_use};
pub use packages::{dest_dir_path, static_fallback_applets, PackageKind, DEST_DIR};
