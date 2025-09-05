use rust_coreutils_switch::cli::handle_cli;
use rust_coreutils_switch::Result;

fn main() -> Result<()> {
    // Handle command line arguments and execute the appropriate action
    if let Err(e) = handle_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
