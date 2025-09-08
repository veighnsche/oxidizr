use clap::Parser;
use log::LevelFilter;

fn main() {
    // Initialize logger with 4-level VERBOSE mapping.
    // If VERBOSE is not set, honor standard env_logger environment (e.g., RUST_LOG).
    if let Ok(vstr) = std::env::var("VERBOSE") {
        let v = vstr.parse::<u8>().unwrap_or(1);
        let level = match v {
            0 => LevelFilter::Error,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
        let mut builder = env_logger::Builder::new();
        builder.filter_level(level);
        let _ = builder.is_test(false).try_init();
    } else {
        let _ = env_logger::Builder::from_env(env_logger::Env::default())
            .is_test(false)
            .try_init();
    }

    // Parse CLI arguments
    let cli = oxidizr_arch::cli::Cli::parse();
    
    // Handle command and execute
    if let Err(e) = oxidizr_arch::cli::handle_cli(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
