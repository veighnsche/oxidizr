use coreutils_switch::cli::handle_cli;
use coreutils_switch::Result;

fn main() -> Result<()> {
    let _ = env_logger::builder().is_test(false).try_init();
    // Handle command line arguments and execute the appropriate action
    if let Err(e) = handle_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
