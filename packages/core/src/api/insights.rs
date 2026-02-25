//! Insights API endpoints

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{Json, Response},
    routing::get,
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::insights::{CongestionTrends, FeeExtremes, FeeInsightsEngine, RollingAverages};
use super::headers::{cache_control, compute_etag, if_none_match_matches, last_modified};

/// Shared state for the insights API
pub type InsightsState = Arc<RwLock<FeeInsightsEngine>>;

/// Create the insights API router
pub fn create_insights_router(insights_engine: InsightsState) -> Router {
    Router::new()
        .route("/insights", get(get_current_insights))
        .route("/insights/averages", get(get_rolling_averages))
        .route("/insights/extremes", get(get_extremes))
        .route("/insights/congestion", get(get_congestion_trends))
        .route("/insights/health", get(get_insights_health))
        .with_state(insights_engine)
}

/// Get current insights
async fn get_current_insights(
    State(engine): State<InsightsState>,
    request_headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let engine = engine.read().await;
    let insights = engine.get_current_insights();
    let body = serde_json::to_vec(&insights).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to serialize insights: {}", err) })),
        )
    })?;
    let etag = compute_etag(&body);
    let last_modified_value = last_modified(insights.last_updated);

    if if_none_match_matches(&request_headers, &etag) {
        return Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::CACHE_CONTROL, cache_control(10, 20))
            .header(header::ETAG, etag.as_str())
            .header(header::LAST_MODIFIED, last_modified_value)
            .body(Body::empty())
            .expect("304 insights response should be valid"));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, cache_control(10, 20))
        .header(header::ETAG, etag.as_str())
        .header(header::LAST_MODIFIED, last_modified_value)
        .body(Body::from(body))
        .expect("insights response should be valid"))
}

/// Get rolling averages
async fn get_rolling_averages(
    State(engine): State<InsightsState>,
) -> Result<Json<RollingAverages>, (StatusCode, Json<Value>)> {
    let engine = engine.read().await;
    let averages = engine.get_rolling_averages();
    Ok(Json(averages))
}

/// Get fee extremes
async fn get_extremes(
    State(engine): State<InsightsState>,
) -> Result<Json<FeeExtremes>, (StatusCode, Json<Value>)> {
    let engine = engine.read().await;
    let extremes = engine.get_extremes();
    Ok(Json(extremes))
}

/// Get congestion trends
async fn get_congestion_trends(
    State(engine): State<InsightsState>,
) -> Result<Json<CongestionTrends>, (StatusCode, Json<Value>)> {
    let engine = engine.read().await;
    let trends = engine.get_congestion_trends();
    Ok(Json(trends))
}

/// Get insights engine health status
async fn get_insights_health(
    State(engine): State<InsightsState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let engine = engine.read().await;
    
    let health_info = serde_json::json!({
        "status": "healthy",
        "last_update": engine.get_last_update(),
        "config": {
            "polling_interval_seconds": engine.get_config().polling_interval.num_seconds(),
            "time_windows": engine.get_config().time_windows.len(),
            "spike_threshold": engine.get_config().spike_detection.threshold_multiplier
        }
    });
    
    Ok(Json(health_info))
}
