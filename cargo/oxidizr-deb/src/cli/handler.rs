use std::path::PathBuf;

use clap::Parser;
use switchyard::adapters::{DefaultSmokeRunner, FileLockManager, FsOwnershipOracle};
use switchyard::logging::JsonlSink;
use switchyard::policy::Policy;
use switchyard::Switchyard;
use switchyard::types::ApplyMode;

use crate::cli::args::{Cli, Commands, Package, Shell};
use crate::commands::{restore, status, r#use};

pub fn dispatch(cli: Cli) -> Result<(), String> {
    // Default policy: conservative, disallow degraded EXDEV for built-ins
    let mut policy = Policy::coreutils_switch_preset();

    // Narrow scope to requested root
    policy.scope.allow_roots.push(cli.root.clone());

    let lock_path = cli.root.join("var/lock/oxidizr-deb.lock");
    let api: Switchyard<JsonlSink, JsonlSink> = Switchyard::builder(
        JsonlSink::default(),
        JsonlSink::default(),
        policy,
    )
    .with_lock_manager(Box::new(FileLockManager::new(lock_path)))
    .with_smoke_runner(Box::new(DefaultSmokeRunner::default()))
    .with_ownership_oracle(Box::new(FsOwnershipOracle::default()))
    .build();

    let apply_mode = if cli.commit { ApplyMode::Commit } else { ApplyMode::DryRun };

    match cli.command {
        Commands::Use { package, offline, use_local } => {
            r#use::exec(&api, &cli.root, package, offline, use_local, apply_mode)
        }
        Commands::Restore { package } => {
            restore::exec(&api, &cli.root, package, apply_mode)
        }
        Commands::Status => status::exec(&cli.root),
        Commands::Completions { shell } => crate::cli::completions::emit(shell),
    }
}
