use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::insights::FeeDataPoint;
use crate::services::horizon::HorizonClient;
use crate::store::FeeHistoryStore;

/// Shared state type for the fees route.
pub type FeesState = Arc<FeesApiState>;

#[derive(Clone)]
pub struct FeesApiState {
    pub horizon_client: Option<Arc<HorizonClient>>,
    pub fee_store: Arc<RwLock<FeeHistoryStore>>,
}

#[derive(Serialize)]
pub struct PercentileFees {
    pub p10: String,
    pub p25: String,
    pub p50: String,
    pub p75: String,
    pub p90: String,
    pub p95: String,
}

#[derive(Serialize)]
pub struct CurrentFeeResponse {
    pub base_fee: String,
    pub min_fee: String,
    pub max_fee: String,
    pub avg_fee: String,
    pub percentiles: PercentileFees,
}

pub async fn current_fees(
    State(state): State<FeesState>,
) -> Result<Json<CurrentFeeResponse>, AppError> {
    let client = state
        .horizon_client
        .as_ref()
        .ok_or_else(|| AppError::Config("Horizon client missing from fees state".to_string()))?;
    let stats = client.fetch_fee_stats().await?;

    Ok(Json(CurrentFeeResponse {
        base_fee: stats.last_ledger_base_fee,
        min_fee: stats.fee_charged.min,
        max_fee: stats.fee_charged.max,
        avg_fee: stats.fee_charged.avg,
        percentiles: PercentileFees {
            p10: stats.fee_charged.p10,
            p25: stats.fee_charged.p25,
            p50: stats.fee_charged.p50,
            p75: stats.fee_charged.p75,
            p90: stats.fee_charged.p90,
            p95: stats.fee_charged.p95,
        },
    }))
}

#[derive(Debug, Deserialize)]
pub struct FeeHistoryQuery {
    pub window: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeSummary {
    pub min: u64,
    pub max: u64,
    pub avg: f64,
    pub p50: u64,
    pub p95: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeHistoryResponse {
    pub window: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub data_points: usize,
    pub fees: Vec<FeeDataPoint>,
    pub summary: FeeSummary,
}

pub async fn fee_history(
    State(state): State<FeesState>,
    Query(params): Query<FeeHistoryQuery>,
) -> Result<Json<FeeHistoryResponse>, (StatusCode, Json<Value>)> {
    let window = params.window.unwrap_or_else(|| "1h".to_string());
    let duration = parse_window(&window).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Unsupported window value: {}", window) })),
        )
    })?;

    let to = Utc::now();
    let from = to - duration;
    let fees = {
        let store = state.fee_store.read().await;
        store.get_since(from)
    };
    let summary = compute_summary(&fees);

    Ok(Json(FeeHistoryResponse {
        window,
        from,
        to,
        data_points: fees.len(),
        fees,
        summary,
    }))
}

fn parse_window(value: &str) -> Option<Duration> {
    match value {
        "1h" => Some(Duration::hours(1)),
        "6h" => Some(Duration::hours(6)),
        "24h" => Some(Duration::hours(24)),
        _ => None,
    }
}

fn compute_summary(fees: &[FeeDataPoint]) -> FeeSummary {
    if fees.is_empty() {
        return FeeSummary {
            min: 0,
            max: 0,
            avg: 0.0,
            p50: 0,
            p95: 0,
        };
    }

    let mut values: Vec<u64> = fees.iter().map(|f| f.fee_amount).collect();
    values.sort_unstable();
    let sum: u64 = values.iter().sum();
    let len = values.len();

    FeeSummary {
        min: values[0],
        max: values[len - 1],
        avg: sum as f64 / len as f64,
        p50: percentile_nearest_rank(&values, 50),
        p95: percentile_nearest_rank(&values, 95),
    }
}

fn percentile_nearest_rank(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let n = sorted.len();
    let rank = ((percentile * n).saturating_add(99) / 100).max(1);
    sorted[rank - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use chrono::Duration;
    use tower::ServiceExt;

    fn make_fee_state_with_points(points: Vec<FeeDataPoint>) -> FeesState {
        let mut store = FeeHistoryStore::new(100);
        for point in points {
            store.push(point);
        }

        Arc::new(FeesApiState {
            horizon_client: None,
            fee_store: Arc::new(RwLock::new(store)),
        })
    }

    fn test_points(count: usize, minutes_ago_start: i64) -> Vec<FeeDataPoint> {
        (0..count)
            .map(|idx| FeeDataPoint {
                fee_amount: 100 + (idx as u64 * 100),
                timestamp: Utc::now() - Duration::minutes(minutes_ago_start - idx as i64),
                transaction_hash: format!("tx-{}", idx),
                ledger_sequence: 50_000_000 + idx as u64,
            })
            .collect()
    }

    #[test]
    fn current_fee_response_serialises_with_percentiles() {
        let response = CurrentFeeResponse {
            base_fee: "100".into(),
            min_fee: "100".into(),
            max_fee: "5000".into(),
            avg_fee: "213".into(),
            percentiles: PercentileFees {
                p10: "100".into(),
                p25: "100".into(),
                p50: "150".into(),
                p75: "300".into(),
                p90: "500".into(),
                p95: "800".into(),
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["base_fee"], "100");
        assert_eq!(json["percentiles"]["p10"], "100");
        assert_eq!(json["percentiles"]["p50"], "150");
        assert_eq!(json["percentiles"]["p95"], "800");
    }

    #[test]
    fn percentile_fees_has_all_six_fields() {
        let p = PercentileFees {
            p10: "100".into(),
            p25: "100".into(),
            p50: "150".into(),
            p75: "300".into(),
            p90: "500".into(),
            p95: "800".into(),
        };
        let json = serde_json::to_value(&p).unwrap();
        for field in &["p10", "p25", "p50", "p75", "p90", "p95"] {
            assert!(json.get(field).is_some(), "missing field: {}", field);
            assert!(!json[field].as_str().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn fee_history_returns_data_points_and_summary_for_supported_windows() {
        for window in ["1h", "6h", "24h"] {
            let state = make_fee_state_with_points(test_points(10, 10));
            let app = Router::new()
                .route("/fees/history", get(fee_history))
                .with_state(state);

            let response = app
                .oneshot(
                    Request::builder()
                        .uri(format!("/fees/history?window={}", window))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let payload: FeeHistoryResponse = serde_json::from_slice(&body).unwrap();

            assert_eq!(payload.window, window);
            assert_eq!(payload.data_points, 10);
            assert_eq!(payload.summary.min, 100);
            assert_eq!(payload.summary.max, 1000);
        }
    }

    #[tokio::test]
    async fn fee_history_invalid_window_returns_400() {
        let state = make_fee_state_with_points(test_points(10, 10));
        let app = Router::new()
            .route("/fees/history", get(fee_history))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/fees/history?window=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
