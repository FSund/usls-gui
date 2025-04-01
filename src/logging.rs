use anyhow::Result;
use tracing_subscriber::{filter::Targets, fmt, prelude::*, Registry};

pub fn init_logging() -> Result<()> {
    let log_level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let target_name = env!("CARGO_PKG_NAME");
    let stdout_filter = Targets::new()
        .with_default(tracing::Level::ERROR)
        .with_target(target_name, log_level);

    // Configure stdout layer
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_filter(stdout_filter);

    // Combine both layers
    Registry::default().with(stdout_layer).init();

    Ok(())
}
