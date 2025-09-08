use clap::Parser;

fn main() {
    // Initialize logger
    let _ = env_logger::builder().is_test(false).try_init();
    
    // Parse CLI arguments
    let cli = oxidizr_arch::cli::Cli::parse();
    
    // Handle command and execute
    if let Err(e) = oxidizr_arch::cli::handle_cli(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
