use crate::core::{create_core_util, CoreUtilsImpl};
use crate::error::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Use uutils instead of GNU coreutils
    #[arg(short = 'u', long = "uutils")]
    pub use_uutils: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a core utility
    Exec {
        /// The utility to execute (e.g., ls, cat, grep)
        utility: String,
        
        /// Arguments to pass to the utility
        args: Vec<String>,
    },
    
    /// List available utilities
    List,
}

pub fn handle_cli() -> Result<()> {
    let cli = Cli::parse();
    
    let implementation = if cli.use_uutils {
        CoreUtilsImpl::Uutils
    } else {
        CoreUtilsImpl::Gnu
    };
    
    match &cli.command {
        Commands::Exec { utility, args } => {
            let util = create_core_util(utility, implementation);
            println!("Using {} implementation for {}", util.name(), utility);
            util.execute(args)
        }
        Commands::List => {
            println!("Available utilities:");
            println!("- ls");
            println!("- cat");
            println!("- grep");
            println!("- echo");
            println!("\nUse --help for more information");
            Ok(())
        }
    }
}
