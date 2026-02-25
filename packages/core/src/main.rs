// These modules contain scaffolding that will be wired up in subsequent issues.
// Suppress dead-code warnings until then rather than deleting valid future code.
#![allow(dead_code)]

mod alerts;
mod api;
mod cache;
mod metrics;
mod cli;
mod config;
mod db;
mod error;
mod insights;
mod logging;
mod repository;
mod services;
mod scheduler;
mod store;

use std::sync::Arc;

use axum::{routing::get, Router};
use axum::http::{HeaderName, Method};
use clap::Parser;
use dotenvy::dotenv;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::cache::ResponseCache;
use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;
use crate::insights::{FeeInsightsEngine, InsightsConfig, HorizonFeeDataProvider};
use crate::logging::init_logging;
use crate::metrics::AppMetrics;
use crate::repository::FeeRepository;
use crate::scheduler::run_fee_polling_with_retry;
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

    // ---- Database ----
    let db_pool = db::create_pool(&config.database_url)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Failed to initialise database: {}", err);
            std::process::exit(1);
        });
    tracing::info!("Database initialised: {}", config.database_url);

    // ---- Metrics ----
    let app_metrics = Arc::new(
        AppMetrics::new().unwrap_or_else(|err| {
            tracing::error!("Failed to initialise Prometheus metrics: {}", err);
            std::process::exit(1);
        }),
    );

    let repository = Arc::new(FeeRepository::new(db_pool));

    // ---- Shared state ----
    let horizon_client = Arc::new(HorizonClient::new(config.horizon_url.clone()));
    tracing::info!("Horizon client initialized: {}", horizon_client.base_url());

    let fee_store = Arc::new(RwLock::new(FeeHistoryStore::new(DEFAULT_CAPACITY)));

    let insights_engine = Arc::new(RwLock::new(
        FeeInsightsEngine::new(InsightsConfig::default()),
    ));
    let current_fees_cache = Arc::new(Mutex::new(ResponseCache::new(Duration::from_secs(
        config.cache_ttl_seconds,
    ))));

    // ---- Startup rehydration ----
    let rehydration_window = chrono::Utc::now() - chrono::Duration::hours(24);
    match repository.fetch_since(rehydration_window).await {
        Ok(points) if !points.is_empty() => {
            let count = points.len();
            {
                let mut store = fee_store.write().await;
                for point in &points {
                    store.push(point.clone());
                }
            }
            {
                let mut engine = insights_engine.write().await;
                if let Err(err) = engine.process_fee_data(&points).await {
                    tracing::warn!("Insights engine error during rehydration: {}", err);
                }
            }
            tracing::info!("Restored {} fee data points from database", count);
        }
        Ok(_) => tracing::info!("No historical fee data found — starting cold"),
        Err(err) => tracing::warn!("Failed to rehydrate store from database: {}", err),
    }
    let horizon_provider = Arc::new(HorizonFeeDataProvider::new(
        (*horizon_client).clone(),
    ));
    let fee_stats_provider: Arc<dyn api::fees::FeeStatsProvider + Send + Sync> =
        horizon_client.clone();

    // ---- CORS policy ----
    let origins: Vec<axum::http::HeaderValue> = config
        .allowed_origins
        .iter()
        .map(|o| o.parse().expect("Invalid origin in ALLOWED_ORIGINS"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-api-key"),
        ])
        .expose_headers([
            HeaderName::from_static("etag"),
            HeaderName::from_static("cache-control"),
            HeaderName::from_static("last-modified"),
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderName::from_static("x-ratelimit-remaining"),
            HeaderName::from_static("x-ratelimit-reset"),
            HeaderName::from_static("retry-after"),
        ])
        .max_age(Duration::from_secs(3600));

    // ---- Axum router ----
    // fees routes get shared state (Horizon client, store, insights engine)
    // insights routes get Arc<RwLock<FeeInsightsEngine>> as their own state
    // Both sub-routers are Router<()> after with_state, so merge works fine
    let fees_router = Router::new()
        .route("/fees/current", get(api::fees::current_fees))
        .route("/fees/history", get(api::fees::fee_history))
        .route("/fees/trend", get(api::fees::fee_trend))
        .with_state(Arc::new(api::fees::FeesApiState {
            fee_stats_provider: Some(fee_stats_provider),
            fee_cache: current_fees_cache,
            fee_store: fee_store.clone(),
            insights_engine: Some(insights_engine.clone()),
        }));

    // Clone for metrics endpoint closure
    let metrics_for_handler = app_metrics.clone();

    let app = Router::new()
        .route("/health", get(api::health::health))
        .route(
            "/metrics",
            get(move || {
                let m = metrics_for_handler.clone();
                async move {
                    match m.render() {
                        Ok(body) => axum::response::Response::builder()
                            .status(200)
                            .header(
                                axum::http::header::CONTENT_TYPE,
                                "text/plain; version=0.0.4",
                            )
                            .body(axum::body::Body::from(body))
                            .unwrap(),
                        Err(err) => {
                            tracing::error!("Failed to render metrics: {}", err);
                            axum::response::Response::builder()
                                .status(500)
                                .body(axum::body::Body::from("metrics error"))
                                .unwrap()
                        }
                    }
                }
            }),
        )
        .merge(fees_router)
        .merge(api::insights::create_insights_router(insights_engine.clone()))
        .merge(
            Router::new()
                .route("/alerts/config", axum::routing::post(api::alerts::create_alert))
                .route("/alerts/config", axum::routing::get(api::alerts::list_alerts))
                .route("/alerts/config/:id", axum::routing::patch(api::alerts::update_alert))
                .route("/alerts/config/:id", axum::routing::delete(api::alerts::delete_alert))
                .route("/alerts/history", axum::routing::get(api::alerts::get_alert_history))
                .with_state(repository.clone()),
        )
        .layer(cors);

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
        run_fee_polling_with_retry(
            horizon_provider,
            fee_store,
            insights_engine,
            config.poll_interval_seconds,
            config.retry_attempts,
            config.base_retry_delay_ms,
            Some(repository),
            config.storage_retention_days,
            Some(app_metrics),
        ),
    );

    tracing::info!("Application shut down cleanly");
}
