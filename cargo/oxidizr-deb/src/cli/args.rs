use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum Package {
    Coreutils,
    Findutils,
    Sudo,
}

#[derive(Debug, Parser)]
#[command(name = "oxidizr-deb", version, about = "Debian-family CLI to swap GNU coreutils with uutils-coreutils and sudo with sudo-rs using Switchyard")] 
pub struct Cli {
    /// Root of the filesystem tree to operate on (default "/")
    #[arg(long, global = true, default_value = "/")]
    pub root: PathBuf,

    /// Commit changes to disk (default is dry-run)
    #[arg(long, global = true, default_value_t = false)]
    pub commit: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Rustify a package (fetch/verify + safe swap)
    Use {
        /// Which package to rustify
        #[arg(value_enum)]
        package: Package,
        /// Offline mode: use a local artifact instead of fetching
        #[arg(long, default_value_t = false)]
        offline: bool,
        /// Local artifact path when --offline (still validated)
        #[arg(long, value_name = "PATH")]
        use_local: Option<PathBuf>,
    },
    /// Restore GNU/stock tools for a package (or all)
    Restore {
        /// Package to restore; when omitted, restores all known packages
        #[arg(value_enum)]
        package: Option<Package>,
    },
    /// Report current rustified state
    Status,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum, default_value_t = Shell::Bash)]
        shell: Shell,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}
