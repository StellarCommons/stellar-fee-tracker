use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{Mutex, RwLock};

use crate::cache::ResponseCache;
use crate::error::AppError;
use crate::insights::{FeeDataPoint, FeeInsightsEngine, TrendIndicator, TrendStrength};
use crate::services::horizon::{HorizonClient, HorizonFeeStats};
use crate::store::FeeHistoryStore;

/// Shared state type for the fees route.
pub type FeesState = Arc<FeesApiState>;

#[derive(Clone)]
pub struct FeesApiState {
    pub horizon_client: Option<Arc<HorizonClient>>,
    pub fee_stats_provider: Option<Arc<dyn FeeStatsProvider>>,
    pub fee_store: Arc<RwLock<FeeHistoryStore>>,
    pub insights_engine: Option<Arc<RwLock<FeeInsightsEngine>>>,
    pub current_fee_cache: Option<Arc<Mutex<ResponseCache<CurrentFeeResponse>>>>,
}

#[async_trait]
pub trait FeeStatsProvider: Send + Sync {
    async fn fetch_fee_stats(&self) -> Result<HorizonFeeStats, AppError>;
}

#[async_trait]
impl FeeStatsProvider for HorizonClient {
    async fn fetch_fee_stats(&self) -> Result<HorizonFeeStats, AppError> {
        HorizonClient::fetch_fee_stats(self).await
    }
}

#[derive(Clone, Serialize)]
pub struct PercentileFees {
    pub p10: String,
    pub p25: String,
    pub p50: String,
    pub p75: String,
    pub p90: String,
    pub p95: String,
}

#[derive(Clone, Serialize)]
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
    if let Some(cache) = &state.current_fee_cache {
        let cached = {
            let cache_guard = cache.lock().await;
            cache_guard.get()
        };
        if let Some(response) = cached {
            return Ok(Json(response));
        }
    }

    let stats = if let Some(provider) = state.fee_stats_provider.as_ref() {
        provider.fetch_fee_stats().await?
    } else {
        let client = state
            .horizon_client
            .as_ref()
            .ok_or_else(|| AppError::Config("Horizon client missing from fees state".to_string()))?;
        client.fetch_fee_stats().await?
    };

    let response = CurrentFeeResponse {
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
    };

    if let Some(cache) = &state.current_fee_cache {
        let mut cache_guard = cache.lock().await;
        cache_guard.set(response.clone());
    }

    Ok(Json(response))
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendChanges {
    #[serde(rename = "1h_pct")]
    pub one_h_pct: Option<f64>,
    #[serde(rename = "6h_pct")]
    pub six_h_pct: Option<f64>,
    #[serde(rename = "24h_pct")]
    pub twenty_four_h_pct: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeTrendResponse {
    pub status: String,
    pub trend_strength: String,
    pub changes: TrendChanges,
    pub recent_spike_count: usize,
    pub predicted_congestion_minutes: Option<i64>,
    pub last_updated: DateTime<Utc>,
}

pub async fn fee_trend(
    State(state): State<FeesState>,
) -> Result<Json<FeeTrendResponse>, AppError> {
    let engine = state
        .insights_engine
        .as_ref()
        .ok_or_else(|| AppError::Config("Insights engine missing from fees state".to_string()))?;
    let insights = engine.read().await.get_current_insights();
    let averages = &insights.rolling_averages;
    let current_avg = averages.short_term.value;

    let changes = TrendChanges {
        one_h_pct: percent_change(current_avg, &averages.short_term),
        six_h_pct: percent_change(current_avg, &averages.medium_term),
        twenty_four_h_pct: percent_change(current_avg, &averages.long_term),
    };

    Ok(Json(FeeTrendResponse {
        status: trend_indicator_to_string(&insights.congestion_trends.current_trend),
        trend_strength: trend_strength_to_string(&insights.congestion_trends.trend_strength),
        changes,
        recent_spike_count: insights.congestion_trends.recent_spikes.len(),
        predicted_congestion_minutes: insights
            .congestion_trends
            .predicted_duration
            .map(|d| d.num_minutes()),
        last_updated: insights.last_updated,
    }))
}

fn percent_change(current_avg: f64, window_avg: &crate::insights::AverageResult) -> Option<f64> {
    if window_avg.is_partial || window_avg.value <= 0.0 {
        return None;
    }
    Some(((current_avg - window_avg.value) / window_avg.value) * 100.0)
}

fn trend_indicator_to_string(indicator: &TrendIndicator) -> String {
    match indicator {
        TrendIndicator::Normal => "Normal",
        TrendIndicator::Rising => "Rising",
        TrendIndicator::Congested => "Congested",
        TrendIndicator::Declining => "Declining",
    }
    .to_string()
}

fn trend_strength_to_string(strength: &TrendStrength) -> String {
    match strength {
        TrendStrength::Weak => "Weak",
        TrendStrength::Moderate => "Moderate",
        TrendStrength::Strong => "Strong",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use async_trait::async_trait;
    use crate::insights::InsightsConfig;
    use crate::services::horizon::FeeCharged;
    use chrono::Duration;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct MockFeeStatsProvider {
        calls: Arc<AtomicUsize>,
        response: HorizonFeeStats,
    }

    impl MockFeeStatsProvider {
        fn new(response: HorizonFeeStats) -> Self {
            Self {
                calls: Arc::new(AtomicUsize::new(0)),
                response,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl FeeStatsProvider for MockFeeStatsProvider {
        async fn fetch_fee_stats(&self) -> Result<HorizonFeeStats, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.response.clone())
        }
    }

    fn sample_horizon_stats(base_fee: &str) -> HorizonFeeStats {
        HorizonFeeStats {
            last_ledger_base_fee: base_fee.to_string(),
            fee_charged: FeeCharged {
                min: "100".to_string(),
                max: "5000".to_string(),
                avg: "213".to_string(),
                p10: "100".to_string(),
                p25: "100".to_string(),
                p50: "150".to_string(),
                p75: "300".to_string(),
                p90: "500".to_string(),
                p95: "800".to_string(),
            },
        }
    }

    fn make_fee_state_with_points(points: Vec<FeeDataPoint>) -> FeesState {
        let mut store = FeeHistoryStore::new(100);
        for point in points {
            store.push(point);
        }

        Arc::new(FeesApiState {
            horizon_client: None,
            fee_stats_provider: None,
            fee_store: Arc::new(RwLock::new(store)),
            insights_engine: None,
            current_fee_cache: None,
        })
    }

    fn make_fee_state_with_engine(engine: FeeInsightsEngine) -> FeesState {
        Arc::new(FeesApiState {
            horizon_client: None,
            fee_stats_provider: None,
            fee_store: Arc::new(RwLock::new(FeeHistoryStore::new(100))),
            insights_engine: Some(Arc::new(RwLock::new(engine))),
            current_fee_cache: None,
        })
    }

    fn make_fee_state_for_current(
        provider: Arc<dyn FeeStatsProvider>,
        ttl: std::time::Duration,
    ) -> FeesState {
        Arc::new(FeesApiState {
            horizon_client: None,
            fee_stats_provider: Some(provider),
            fee_store: Arc::new(RwLock::new(FeeHistoryStore::new(100))),
            insights_engine: None,
            current_fee_cache: Some(Arc::new(Mutex::new(ResponseCache::new(ttl)))),
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

    #[tokio::test]
    async fn current_fees_uses_cache_on_second_call() {
        let provider = Arc::new(MockFeeStatsProvider::new(sample_horizon_stats("100")));
        let state = make_fee_state_for_current(provider.clone(), std::time::Duration::from_secs(60));
        let app = Router::new()
            .route("/fees/current", get(current_fees))
            .with_state(state);

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/fees/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/fees/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        assert_eq!(provider.calls(), 1);
    }

    #[tokio::test]
    async fn current_fees_fetches_fresh_data_after_ttl_expires() {
        let provider = Arc::new(MockFeeStatsProvider::new(sample_horizon_stats("100")));
        let state = make_fee_state_for_current(provider.clone(), std::time::Duration::from_millis(5));
        let app = Router::new()
            .route("/fees/current", get(current_fees))
            .with_state(state);

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/fees/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/fees/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        assert_eq!(provider.calls(), 2);
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

    fn points_with_spike(high_fee: u64) -> Vec<FeeDataPoint> {
        let now = Utc::now();
        vec![
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(60),
                transaction_hash: "tx1".to_string(),
                ledger_sequence: 1,
            },
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(50),
                transaction_hash: "tx2".to_string(),
                ledger_sequence: 2,
            },
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(40),
                transaction_hash: "tx3".to_string(),
                ledger_sequence: 3,
            },
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(30),
                transaction_hash: "tx4".to_string(),
                ledger_sequence: 4,
            },
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(20),
                transaction_hash: "tx5".to_string(),
                ledger_sequence: 5,
            },
            FeeDataPoint {
                fee_amount: high_fee,
                timestamp: now - Duration::minutes(10),
                transaction_hash: "tx6".to_string(),
                ledger_sequence: 6,
            },
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now,
                transaction_hash: "tx7".to_string(),
                ledger_sequence: 7,
            },
        ]
    }

    fn points_without_spike() -> Vec<FeeDataPoint> {
        let now = Utc::now();
        vec![
            FeeDataPoint {
                fee_amount: 100,
                timestamp: now - Duration::minutes(50),
                transaction_hash: "n1".to_string(),
                ledger_sequence: 11,
            },
            FeeDataPoint {
                fee_amount: 110,
                timestamp: now - Duration::minutes(40),
                transaction_hash: "n2".to_string(),
                ledger_sequence: 12,
            },
            FeeDataPoint {
                fee_amount: 120,
                timestamp: now - Duration::minutes(30),
                transaction_hash: "n3".to_string(),
                ledger_sequence: 13,
            },
        ]
    }

    #[tokio::test]
    async fn fee_trend_returns_rising_status() {
        let mut engine = FeeInsightsEngine::new(InsightsConfig::default());
        let rising_points = points_with_spike(500);
        engine.process_fee_data(&rising_points).await.unwrap();
        let state = make_fee_state_with_engine(engine);

        let app = Router::new()
            .route("/fees/trend", get(fee_trend))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/fees/trend")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: FeeTrendResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.status, "Rising");
    }

    #[test]
    fn trend_indicator_declining_serialises_to_human_readable_string() {
        assert_eq!(
            trend_indicator_to_string(&TrendIndicator::Declining),
            "Declining"
        );
    }

    #[tokio::test]
    async fn fee_trend_returns_normal_status() {
        let mut engine = FeeInsightsEngine::new(InsightsConfig::default());
        let normal_points = points_without_spike();
        engine.process_fee_data(&normal_points).await.unwrap();
        let state = make_fee_state_with_engine(engine);

        let app = Router::new()
            .route("/fees/trend", get(fee_trend))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/fees/trend")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: FeeTrendResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload.status, "Normal");
    }

    #[tokio::test]
    async fn fee_trend_returns_null_changes_for_partial_windows() {
        let state = make_fee_state_with_engine(FeeInsightsEngine::new(InsightsConfig::default()));
        let app = Router::new()
            .route("/fees/trend", get(fee_trend))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/fees/trend")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: FeeTrendResponse = serde_json::from_slice(&body).unwrap();
        assert!(payload.changes.one_h_pct.is_none());
        assert!(payload.changes.six_h_pct.is_none());
        assert!(payload.changes.twenty_four_h_pct.is_none());
    }
}
