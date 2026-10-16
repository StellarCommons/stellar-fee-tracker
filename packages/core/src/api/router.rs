//! Shared assembly of the business API routes.
//!
//! Used by both `main.rs` and the integration tests so that route changes
//! cannot drift between the production router and the one under test.

use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::sync::{Mutex, RwLock};

use crate::api;
use crate::api::fees::{CurrentFeeResponse, FeeStatsProvider};
use crate::cache::ResponseCache;
use crate::insights::FeeInsightsEngine;
use crate::metrics::AppMetrics;
use crate::middleware::auth::require_api_key;
use crate::recommendation::engine::FeeRecommendationEngine;
use crate::recommendation::types::RecommendationConfig;
use crate::repository::FeeRepository;
use crate::services::horizon::HorizonClient;
use crate::store::FeeHistoryStore;

/// Everything the business routes need.
pub struct ApiRouterDeps {
    pub fee_stats_provider: Arc<dyn FeeStatsProvider + Send + Sync>,
    pub fee_cache: Arc<Mutex<ResponseCache<CurrentFeeResponse>>>,
    pub fee_store: Arc<RwLock<FeeHistoryStore>>,
    pub insights_engine: Arc<RwLock<FeeInsightsEngine>>,
    pub horizon_client: Option<Arc<HorizonClient>>,
    pub repository: Arc<FeeRepository>,
    pub metrics: Arc<AppMetrics>,
    pub recommendation_config: RecommendationConfig,
    /// When set, every route returned by this builder requires the key.
    pub api_key: Option<String>,
}

/// Build the fees, recommendation, insights and alerts routes (everything
/// except `/health` and `/metrics`, which sit outside the auth layer).
pub fn build_api_router(deps: ApiRouterDeps) -> Router {
    let fees_router = Router::new()
        .route("/fees/current", get(api::fees::current_fees))
        .route("/fees/history", get(api::fees::fee_history))
        .route("/fees/trend", get(api::fees::fee_trend))
        .route(
            "/fees/account/:account_id",
            get(api::fees::account_fee_history),
        )
        .route(
            "/fees/transaction/:hash",
            get(api::fees::transaction_fee_lookup),
        )
        .route("/fees/wait-advisory", get(api::fees::wait_advisory))
        .with_state(Arc::new(api::fees::FeesApiState {
            fee_stats_provider: Some(deps.fee_stats_provider),
            fee_cache: deps.fee_cache,
            fee_store: deps.fee_store.clone(),
            insights_engine: Some(deps.insights_engine.clone()),
            horizon_client: deps.horizon_client,
        }));

    let recommendation_engine = FeeRecommendationEngine::new(
        deps.fee_store,
        Some(deps.insights_engine.clone()),
        deps.recommendation_config,
    )
    .with_repository(deps.repository.clone());
    let recommendation_state = Arc::new(api::recommendation::RecommendationApiState {
        engine: recommendation_engine,
        metrics: Some(deps.metrics),
        repository: Some(deps.repository.clone()),
    });
    let recommendation_router = Router::new()
        .route(
            "/fees/recommend",
            get(api::recommendation::get_recommend).post(api::recommendation::recommend),
        )
        .route(
            "/fees/recommend/history",
            get(api::recommendation::recommend_history),
        )
        .with_state(recommendation_state);

    let api_routes = Router::new()
        .merge(fees_router)
        .merge(recommendation_router)
        .merge(api::insights::create_insights_router(deps.insights_engine))
        .merge(
            Router::new()
                .route(
                    "/alerts/config",
                    axum::routing::post(api::alerts::create_alert),
                )
                .route(
                    "/alerts/config",
                    axum::routing::get(api::alerts::list_alerts),
                )
                .route(
                    "/alerts/config/:id",
                    axum::routing::patch(api::alerts::update_alert),
                )
                .route(
                    "/alerts/config/:id",
                    axum::routing::delete(api::alerts::delete_alert),
                )
                .route(
                    "/alerts/history",
                    axum::routing::get(api::alerts::get_alert_history),
                )
                .with_state(deps.repository),
        );

    match deps.api_key {
        Some(expected_key) => {
            tracing::info!("API key authentication is enabled for protected routes");
            api_routes.layer(axum::middleware::from_fn_with_state(
                Some(expected_key),
                require_api_key,
            ))
        }
        None => api_routes,
    }
}
