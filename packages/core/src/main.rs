// These modules contain scaffolding that will be wired up in subsequent issues.
// Suppress dead-code warnings until then rather than deleting valid future code.
#![allow(dead_code)]

mod api;
mod cli;
mod config;
mod error;
mod insights;
mod logging;
mod services;
mod scheduler;
mod store;

use std::sync::Arc;

use axum::{routing::get, Router};
use clap::Parser;
use dotenvy::dotenv;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;
use crate::insights::{FeeInsightsEngine, InsightsConfig, HorizonFeeDataProvider};
use crate::logging::init_logging;
use crate::scheduler::run_fee_polling;
use crate::services::horizon::HorizonClient;
use crate::store::{FeeHistoryStore, DEFAULT_CAPACITY};

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

    // ---- Shared state ----
    let horizon_client = Arc::new(HorizonClient::new(config.horizon_url.clone()));
    tracing::info!("Horizon client initialized: {}", horizon_client.base_url());

    let fee_store = Arc::new(RwLock::new(FeeHistoryStore::new(DEFAULT_CAPACITY)));

    let insights_engine = Arc::new(RwLock::new(
        FeeInsightsEngine::new(InsightsConfig::default()),
    ));

    let horizon_provider = Arc::new(HorizonFeeDataProvider::new(
        (*horizon_client).clone(),
    ));

    // ---- Axum router ----
    // Both sub-routers must share the same state type (()).
    // HorizonClient is injected via Extension so we avoid the Router<S>
    // type mismatch that occurs when merging routers with different state.
    let app = Router::new()
        .route("/health", get(api::health::health))
        .route("/fees/current", get(api::fees::current_fees))
        .merge(api::insights::create_insights_router(insights_engine.clone()))
        .layer(axum::Extension((*horizon_client).clone()))
        .layer(CorsLayer::permissive());

    // ---- TCP listener ----
    let addr = format!("0.0.0.0:{}", config.api_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Failed to bind to {}: {}", addr, err);
            std::process::exit(1);
        });

    tracing::info!("API server listening on {}", addr);

    // ---- Run server + scheduler concurrently ----
    tokio::join!(
        async {
            axum::serve(listener, app)
                .await
                .unwrap_or_else(|err| tracing::error!("Server error: {}", err));
        },
        run_fee_polling(
            horizon_provider,
            fee_store,
            insights_engine,
            config.poll_interval_seconds,
        ),
    );

    tracing::info!("Application shut down cleanly");
}