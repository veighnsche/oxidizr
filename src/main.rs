use clap::Parser;

fn main() {
    // Initialize structured logging (tracing) with VERBOSE 0..3 mapping.
    oxidizr_arch::logging::init_logging();

    // Parse CLI arguments
    let cli = oxidizr_arch::cli::Cli::parse();
    
    // Handle command and execute
    if let Err(e) = oxidizr_arch::cli::handle_cli(cli) {
        tracing::error!(error=%e, "fatal_error");
        std::process::exit(1);
    }
}
