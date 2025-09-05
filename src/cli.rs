use crate::error::Result;
use crate::experiment::UutilsExperiment;
use crate::worker::{System, Worker};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(author, version, about = "oxidizr-arch style coreutils switching (scaffold)")]
pub struct Cli {
    /// Skip confirmation prompts (dangerous; intended for automation/tests)
    #[arg(long)]
    pub assume_yes: bool,

    /// Do not run apt-get update before actions
    #[arg(long)]
    pub no_update: bool,

    /// Select which experiment to operate on (currently only 'coreutils' scaffold)
    #[arg(long, default_value = "coreutils")]
    pub experiment: String,

    /// AUR helper to use for package operations (scaffold default)
    #[arg(long, default_value = "paru")]
    pub aur_helper: String,

    /// Override package name (Arch/AUR). Defaults depend on experiment.
    #[arg(long)]
    pub package: Option<String>,

    /// Override bin directory containing replacement binaries
    #[arg(long)]
    pub bin_dir: Option<PathBuf>,

    /// Optional unified dispatch binary path (e.g., /usr/bin/coreutils)
    #[arg(long)]
    pub unified_binary: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

fn findutils_scaffold(package: Option<String>, bin_dir: Option<PathBuf>) -> UutilsExperiment {
    let default_pkg = "uutils-findutils".to_string();
    let default_bin = PathBuf::from("/usr/lib/uutils/findutils");
    UutilsExperiment {
        name: "findutils".to_string(),
        package: package.unwrap_or(default_pkg),
        supported_releases: vec!["rolling".into()],
        unified_binary: None,
        bin_directory: bin_dir.unwrap_or(default_bin),
    }
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

fn coreutils_scaffold(package: Option<String>, bin_dir: Option<PathBuf>, unified: Option<PathBuf>) -> UutilsExperiment {
    // Arch/AUR defaults for uutils coreutils
    let default_pkg = "uutils-coreutils".to_string();
    let default_bin = PathBuf::from("/usr/lib/uutils/coreutils");
    let default_unified = PathBuf::from("/usr/bin/coreutils");

    UutilsExperiment {
        name: "coreutils".to_string(),
        package: package.unwrap_or(default_pkg),
        supported_releases: vec!["rolling".into()],
        unified_binary: Some(unified.unwrap_or(default_unified)),
        bin_directory: bin_dir.unwrap_or(default_bin),
    }
}

pub fn handle_cli() -> Result<()> {
    let cli = Cli::parse();
    let worker = System { aur_helper: cli.aur_helper.clone() };
    let update_lists = !cli.no_update;

    let exp = match cli.experiment.as_str() {
        "coreutils" => coreutils_scaffold(cli.package.clone(), cli.bin_dir.clone(), cli.unified_binary.clone()),
        "findutils" => findutils_scaffold(cli.package.clone(), cli.bin_dir.clone()),
        other => UutilsExperiment {
            name: other.to_string(),
            package: cli.package.clone().unwrap_or_else(|| format!("uutils-{}", other)),
            supported_releases: vec!["rolling".into()],
            unified_binary: cli.unified_binary.clone(),
            bin_directory: cli.bin_dir.clone().unwrap_or_else(|| PathBuf::from("/usr/lib/uutils")),
        },
    };

    match cli.command {
        Commands::Enable => {
            enforce_root()?;
            if !cli.assume_yes && !confirm("Enable and switch to Rust replacements?")? { return Ok(()); }
            exp.enable(&worker, cli.assume_yes, update_lists)?;
            println!("Enabled experiment: {}", exp.name);
        }
        Commands::Disable => {
            enforce_root()?;
            if !cli.assume_yes && !confirm("Disable and restore system-provided tools?")? { return Ok(()); }
            exp.disable(&worker, update_lists)?;
            println!("Disabled experiment: {}", exp.name);
        }
        Commands::Check => {
            let ok = exp.check_compatible(&worker)?;
            println!("Compatible: {}", ok);
        }
        Commands::ListTargets => {
            for t in exp.list_targets(&worker)? {
                println!("{}", t.display());
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
