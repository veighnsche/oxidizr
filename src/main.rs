use oxidizr_arch::Result;
use oxidizr_arch::cli::handle_cli;

fn main() -> Result<()> {
    let _ = env_logger::builder().is_test(false).try_init();
    // Handle command line arguments and execute the appropriate action
    if let Err(e) = handle_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
