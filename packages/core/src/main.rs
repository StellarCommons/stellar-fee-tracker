mod cli;
mod config;
mod error;
mod logging;
mod services;

use clap::Parser;
use dotenvy::dotenv;

use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;
use crate::logging::init_logging;
use crate::services::horizon::HorizonClient;

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenv().ok();

    // Initialize structured logging
    init_logging();

    // Parse CLI flags
    let cli = Cli::parse();

    // Build configuration (CLI overrides env)
    let config = Config::from_sources(&cli)
        .map_err(AppError::Config)
        .unwrap_or_else(|err| {
            tracing::error!("{}", err);
            std::process::exit(1);
        });

    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize Horizon client
    let horizon_client = HorizonClient::new(config.horizon_url.clone());
    tracing::info!(
        "Horizon client initialized with base URL: {}",
        horizon_client.base_url()
    );

    // Fetch fee stats (Issue 5 integration)
    match horizon_client.fetch_fee_stats().await {
        Ok(stats) => {
            tracing::info!("Base fee: {}", stats.last_ledger_base_fee);
            tracing::info!(
                "Fee charged — min: {}, max: {}, avg: {}",
                stats.fee_charged.min,
                stats.fee_charged.max,
                stats.fee_charged.avg
            );
        }
        Err(err) => {
            tracing::error!("Failed to fetch fee stats: {}", err);
        }
    }
}