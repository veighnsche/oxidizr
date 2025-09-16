mod cli;
mod commands;
mod packages;
mod fetch;
mod adapters;
mod util;
mod errors;

use clap::Parser;

fn main() {
    let cli = crate::cli::args::Cli::parse();
    if let Err(e) = crate::cli::handler::dispatch(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
