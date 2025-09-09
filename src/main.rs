use clap::Parser;

fn main() {
    // Initialize structured logging (tracing) with VERBOSE 0..3 mapping.
    oxidizr_arch::logging::init_logging();

    // Parse CLI arguments
    let cli = oxidizr_arch::cli::Cli::parse();

    // Handle command and execute
    if let Err(e) = oxidizr_arch::cli::handle_cli(cli) {
        use oxidizr_arch::Error;
        let code = match &e {
            Error::Incompatible(_) => 10,
            Error::NothingToLink(_) => 20,
            Error::RestoreBackupMissing(_) => 30,
            Error::RepoGateFailed { .. } => 40,
            _ => 1,
        };
        tracing::error!(error=%e, exit_code=code, "fatal_error");
        std::process::exit(code);
    }
}
