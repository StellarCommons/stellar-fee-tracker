mod cli;
mod config;
mod error;
mod insights;
mod logging;
mod services;
mod scheduler;

use clap::Parser;
use dotenvy::dotenv;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;
use crate::logging::init_logging;
use crate::services::horizon::HorizonClient;
use crate::scheduler::run_fee_polling;

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

    // --------------------------------------------------------
    // START POLLING LOOP (Issue 8)
    // --------------------------------------------------------
    //
    // This call blocks the application until Ctrl+C (SIGINT).
    // All fee fetching now happens inside the scheduler.
    //
    run_fee_polling(
        horizon_client,
        config.poll_interval_seconds,
    )
    .await;

    tracing::info!("Application shut down cleanly");
}