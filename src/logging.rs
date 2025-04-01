use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

pub fn init_logging() -> Result<()> {
    // Default log level based on build configuration
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    // Extract the crate name from the module path
    let crate_name = module_path!().split("::").next().unwrap_or("");

    // Create an EnvFilter that respects RUST_LOG environment variable
    // If RUST_LOG is not set, use our default configuration
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("error,{}={}", crate_name, default_log_level)));
    println!("Log level: {}", filter);

    // Configure stdout layer with EnvFilter
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_filter(filter);

    // Initialize the subscriber
    Registry::default().with(stdout_layer).init();

    Ok(())
}
