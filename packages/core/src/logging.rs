use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize structured logging for the application.
///
/// This must be called once at startup (in main.rs).
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!("Logging initialized");
}
