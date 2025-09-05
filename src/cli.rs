#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum AurHelperArg {
    /// Auto-detect installed helper (prefer paru, then yay, then trizen, then pamac)
    Auto,
    /// Do not use any AUR helper (pacman only)
    None,
    Paru,
    Yay,
    Trizen,
    Pamac,
}

impl AurHelperArg {
    fn as_helper_str(&self) -> &'static str {
        match self {
            AurHelperArg::Auto => "auto",
            AurHelperArg::None => "none",
            AurHelperArg::Paru => "paru",
            AurHelperArg::Yay => "yay",
            AurHelperArg::Trizen => "trizen",
            AurHelperArg::Pamac => "pamac",
        }
    }
}
use crate::error::Result;
use crate::experiments::{all_experiments, Experiment};
use crate::utils::worker::System;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about = "oxidizr-arch style coreutils switching (scaffold)")]
pub struct Cli {
    /// Skip confirmation prompts (dangerous; intended for automation/tests)
    #[arg(long)]
    pub assume_yes: bool,

    /// Do not run pacman -Sy before actions
    #[arg(long)]
    pub no_update: bool,

    /// Select all known experiments from the registry
    #[arg(long, short = 'a')]
    pub all: bool,

    /// Select which experiments to operate on (comma separated or repeatable)
    #[arg(long, value_delimiter = ',')]
    pub experiments: Vec<String>,

    /// Backward compatibility: single experiment selection (deprecated)
    #[arg(long)]
    pub experiment: Option<String>,

    /// Skip compatibility checks (dangerous)
    #[arg(long)]
    pub no_compatibility_check: bool,

    /// AUR helper to use for package operations (auto-detect by default)
    #[arg(long, value_enum, default_value_t = AurHelperArg::Auto)]
    pub aur_helper: AurHelperArg,

    /// Force a specific package manager (AUR helper) instead of auto-detect (e.g., paru, yay, trizen, pamac)
    #[arg(long)]
    pub package_manager: Option<String>,

    /// Override package name (Arch/AUR). Defaults depend on experiment.
    #[arg(long)]
    pub package: Option<String>,

    /// Override bin directory containing replacement binaries
    #[arg(long)]
    pub bin_dir: Option<PathBuf>,

    /// Optional unified dispatch binary path (e.g., /usr/bin/coreutils)
    #[arg(long)]
    pub unified_binary: Option<PathBuf>,

    /// Dry-run: print planned actions without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Wait for pacman database lock to clear, in seconds (polling). If unset, fail fast when lock is present.
    #[arg(long)]
    pub wait_lock: Option<u64>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Enable the Rust replacement utilities (install + symlink swap-in)
    Enable,
    /// Disable the Rust replacement utilities (restore + remove)
    Disable,
    /// Check distro compatibility for this experiment
    Check,
    /// List computed target paths that would be affected
    ListTargets,
}

// (legacy scaffolds removed; use experiments registry instead)

pub fn handle_cli() -> Result<()> {
    let cli = Cli::parse();
    // Prefer --package-manager when provided; else fallback to --aur-helper
    let effective_helper = cli
        .package_manager
        .clone()
        .unwrap_or_else(|| cli.aur_helper.as_helper_str().to_string());
    let worker = System { aur_helper: effective_helper, dry_run: cli.dry_run, wait_lock_secs: cli.wait_lock };
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

    let mut exps: Vec<Experiment> = all_experiments(&worker);
    if !cli.all {
        // Filter by provided names
        exps = exps.into_iter().filter(|e| selection.contains(&e.name())).collect();
    }

    if exps.is_empty() {
        eprintln!("No experiments selected. Use --all or --experiments=<names>");
        return Ok(());
    }

    match cli.command {
        Commands::Enable => {
            if !cli.dry_run { enforce_root()?; }
            if !cli.assume_yes && !confirm("Enable and switch to Rust replacements?")? { return Ok(()); }
            for e in &exps {
                e.enable(&worker, cli.assume_yes, update_lists, cli.no_compatibility_check)?;
                println!("Enabled experiment: {}", e.name());
            }
        }
        Commands::Disable => {
            if !cli.dry_run { enforce_root()?; }
            if !cli.assume_yes && !confirm("Disable and restore system-provided tools?")? { return Ok(()); }
            for e in &exps {
                e.disable(&worker, update_lists)?;
                println!("Disabled experiment: {}", e.name());
            }
        }
        Commands::Check => {
            for e in &exps {
                let ok = e.check_compatible(&worker)?;
                println!("{}\tCompatible: {}", e.name(), ok);
            }
        }
        Commands::ListTargets => {
            for e in &exps {
                for t in e.list_targets(&worker)? {
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
            return Err(crate::error::CoreutilsError::Other("This command must be run as root".into()));
        }
    }
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut s = String::new();
    if io::stdin().read_line(&mut s).is_err() { return Ok(false); }
    let ans = s.trim().to_ascii_lowercase();
    Ok(ans == "y" || ans == "yes")
}

fn default_experiments() -> Vec<String> {
    let mut v = vec!["coreutils".to_string(), "sudo-rs".to_string()];
    v.sort();
    v
}
