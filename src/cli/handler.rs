use crate::cli::parser::{Cli, Commands};
use crate::error::Result;
use crate::experiments::{all_experiments, Experiment};
use crate::logging::audit_event;
use crate::system::Worker;
use std::io::{self, Write};

/// Main CLI handler - preserves backward compatibility with original
pub fn handle_cli(cli: Cli) -> Result<()> {
    // Top-level CLI span for context
    let _cli_span = tracing::info_span!(
        "cli",
        command = ?cli.command,
        all = cli.all,
        exp_count = cli.experiments.len(),
        dry_run = cli.dry_run,
        assume_yes = cli.assume_yes
    )
    .entered();
    // Prefer --package-manager when provided; else fallback to --aur-helper
    let effective_helper = cli
        .package_manager
        .clone()
        .unwrap_or_else(|| cli.aur_helper.as_helper_str().to_string());
        
    let worker = Worker::new(
        effective_helper,
        cli.dry_run,
        cli.wait_lock,
        cli.package.clone(),
        cli.bin_dir.clone(),
        cli.unified_binary.clone(),
    );
    
    let update_lists = !cli.no_update;

    // Build experiment selection
    let selection: Vec<String> = if cli.all {
        // Will be replaced by all experiments below
        Vec::new()
    } else if !cli.experiments.is_empty() {
        cli.experiments.clone()
    } else if let Some(single) = &cli.experiment {
        vec![single.clone()]
    } else {
        default_experiments()
    };

    let all_exps = all_experiments();
    let exps: Vec<Experiment> = if cli.all {
        all_exps
    } else {
        all_exps
            .into_iter()
            .filter(|e| selection.contains(&e.name().to_string()))
            .collect()
    };

    // Orchestration visibility: when both findutils and coreutils are selected, we enable
    // findutils first so GNU checksum tools remain available for AUR builds before any flipping.
    let names: Vec<String> = exps.iter().map(|e| e.name().to_string()).collect();
    let has_core = names.iter().any(|n| n == "coreutils");
    let has_find = names.iter().any(|n| n == "findutils");
    if has_core && has_find {
        tracing::info!(step = "orchestration", "enable findutils before coreutils");
    }

    if exps.is_empty() {
        tracing::warn!("No experiments selected. Use --all or --experiments=<names>");
        return Ok(());
    }

    match cli.command {
        Commands::Enable => {
            if !cli.dry_run {
                enforce_root()?;
            }
            if !cli.assume_yes && !confirm("Enable and switch to Rust replacements?")? {
                return Ok(());
            }
            for (idx, e) in exps.iter().enumerate() {
                tracing::info!(step = "enable_sequence", idx = idx + 1, total = exps.len(), experiment = %e.name());
                e.enable(
                    &worker,
                    cli.assume_yes,
                    update_lists,
                    cli.no_compatibility_check,
                )?;
                tracing::info!(event = "enabled", experiment = %e.name());
            }
        }
        Commands::Disable => {
            if !cli.dry_run {
                enforce_root()?;
            }
            
            // Ask whether to Disable (restore only) or Remove (uninstall package + restore)
            // In non-interactive/assume-yes mode, default to Remove to ensure clean state for tests.
            let do_remove = if cli.assume_yes {
                true
            } else {
                print!(
                    "Disable (swap back to GNU, keep package installed) or Remove (uninstall package and restore GNU)? [disable/Remove]: "
                );
                io::stdout().flush().ok();
                let mut s = String::new();
                let _ = io::stdin().read_line(&mut s);
                let ans = s.trim().to_ascii_lowercase();
                ans == "remove" || ans == "r"
            };
            
            let _ = audit_event(
                "cli",
                "disable_choice",
                if do_remove { "remove" } else { "disable" },
                "",
                "",
                None,
            );
            
            if do_remove {
                for e in &exps {
                    e.remove(&worker, cli.assume_yes, update_lists)?;
                    tracing::info!(event = "removed_and_restored", experiment = %e.name());
                }
            } else {
                for e in &exps {
                    e.disable(&worker, cli.assume_yes, update_lists)?;
                    tracing::info!(event = "disabled", experiment = %e.name());
                }
            }
        }
        Commands::Check => {
            let distro = worker.distribution()?;
            for e in &exps {
                let ok = e.check_compatible(&distro)?;
                println!("{}\tCompatible: {}", e.name(), ok);
            }
        }
        Commands::ListTargets => {
            for e in &exps {
                for t in e.list_targets() {
                    println!("{}\t{}", e.name(), t.display());
                }
            }
        }
    }
    
    Ok(())
}

fn enforce_root() -> Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::Uid;
        if !Uid::effective().is_root() {
            return Err(crate::Error::Other(
                "This command must be run as root".into(),
            ));
        }
    }
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut s = String::new();
    if io::stdin().read_line(&mut s).is_err() {
        return Ok(false);
    }
    let ans = s.trim().to_ascii_lowercase();
    Ok(ans == "y" || ans == "yes")
}

fn default_experiments() -> Vec<String> {
    let mut v = vec!["coreutils".to_string(), "sudo-rs".to_string()];
    v.sort();
    v
}
