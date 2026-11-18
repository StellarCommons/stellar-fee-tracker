//! `GET /metrics` — Prometheus text exposition.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
};

use crate::metrics::AppMetrics;

/// Render the Prometheus registry. Shared by `main.rs` and the tests so the
/// production handler is the one under test.
pub async fn metrics_handler(State(metrics): State<Arc<AppMetrics>>) -> Response {
    match metrics.render() {
        Ok(body) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/plain; version=0.0.4")],
            body,
        )
            .into_response(),
        Err(err) => {
            tracing::error!("Failed to render metrics: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "metrics error").into_response()
        }
    }
}
