//! Fee polling scheduler.
//!
//! Drives the main polling loop: each tick fetches fee data from the
//! Horizon provider, pushes it into the history store, and runs the
//! insights engine — so the API layer always has fresh computed data.

use std::sync::Arc;
use std::time::Duration;

use tokio::signal;
use tokio::sync::RwLock;
use tokio::time;

use crate::insights::{
    FeeDataProvider, FeeInsightsEngine,
};
use crate::store::FeeHistoryStore;

/// Run the fee polling loop.
///
/// On each tick:
/// 1. Fetch the latest `FeeDataPoint` values from `horizon_provider`
/// 2. Push each point into `history_store`
/// 3. Run `insights_engine.process_fee_data()` with the new points
/// 4. Log a summary of the update
///
/// Errors from the provider are logged and the loop continues — a single
/// failed poll should never take down the scheduler.
///
/// Runs until `Ctrl+C` (SIGINT) is received.
pub async fn run_fee_polling(
    horizon_provider: Arc<dyn FeeDataProvider + Send + Sync>,
    history_store: Arc<RwLock<FeeHistoryStore>>,
    insights_engine: Arc<RwLock<FeeInsightsEngine>>,
    poll_interval_seconds: u64,
) {
    let mut interval = time::interval(Duration::from_secs(poll_interval_seconds));

    tracing::info!(
        "Fee polling started (interval: {}s)",
        poll_interval_seconds
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                poll_once(
                    &horizon_provider,
                    &history_store,
                    &insights_engine,
                ).await;
            }

            _ = signal::ctrl_c() => {
                tracing::info!("Shutdown signal received. Stopping polling.");
                break;
            }
        }
    }

    tracing::info!("Fee polling stopped cleanly");
}

/// Execute a single poll cycle. Extracted for testability.
async fn poll_once(
    horizon_provider: &Arc<dyn FeeDataProvider + Send + Sync>,
    history_store: &Arc<RwLock<FeeHistoryStore>>,
    insights_engine: &Arc<RwLock<FeeInsightsEngine>>,
) {
    // 1. Fetch latest fee data points from provider
    let points = match horizon_provider.fetch_latest_fees().await {
        Ok(p) => p,
        Err(err) => {
            tracing::error!("Fee polling error — skipping tick: {}", err);
            return;
        }
    };

    if points.is_empty() {
        tracing::warn!("Provider returned no fee data points this tick");
        return;
    }

    // 2. Push all points into the history store
    {
        let mut store = history_store.write().await;
        for point in &points {
            store.push(point.clone());
        }
        tracing::debug!("Store now holds {} data points", store.len());
    }

    // 3. Run the insights engine
    {
        let mut engine = insights_engine.write().await;
        match engine.process_fee_data(&points).await {
            Ok(update) => {
                tracing::info!(
                    "Insights updated — {} points processed, short-term avg: {:.1} stroops",
                    update.data_points_processed,
                    update.insights.rolling_averages.short_term.value,
                );
            }
            Err(err) => {
                tracing::error!("Insights engine error: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::insights::{FeeInsightsEngine, InsightsConfig};
    use crate::insights::error::ProviderError;
    use crate::insights::types::FeeDataPoint;
    use crate::services::mock_horizon::MockHorizonClient;
    use crate::store::{FeeHistoryStore, DEFAULT_CAPACITY};

    fn make_point(fee_amount: u64) -> FeeDataPoint {
        FeeDataPoint {
            fee_amount,
            timestamp: Utc::now(),
            transaction_hash: format!("hash_{}", fee_amount),
            ledger_sequence: 1,
        }
    }

    fn make_shared_store() -> Arc<RwLock<FeeHistoryStore>> {
        Arc::new(RwLock::new(FeeHistoryStore::new(DEFAULT_CAPACITY)))
    }

    fn make_shared_engine() -> Arc<RwLock<FeeInsightsEngine>> {
        Arc::new(RwLock::new(FeeInsightsEngine::new(InsightsConfig::default())))
    }

    #[tokio::test]
    async fn poll_once_pushes_points_into_store() {
        let points = vec![make_point(100), make_point(200), make_point(300)];
        let provider: Arc<dyn FeeDataProvider + Send + Sync> =
            Arc::new(MockHorizonClient::new().with_fees(points));
        let store = make_shared_store();
        let engine = make_shared_engine();

        poll_once(&provider, &store, &engine).await;

        assert_eq!(store.read().await.len(), 3);
    }

    #[tokio::test]
    async fn poll_once_runs_insights_engine() {
        let points = vec![make_point(100), make_point(150), make_point(200)];
        let provider: Arc<dyn FeeDataProvider + Send + Sync> =
            Arc::new(MockHorizonClient::new().with_fees(points));
        let store = make_shared_store();
        let engine = make_shared_engine();

        poll_once(&provider, &store, &engine).await;

        assert!(engine.read().await.get_last_update().is_some());
    }

    #[tokio::test]
    async fn poll_once_on_provider_error_does_not_push_to_store() {
        let provider: Arc<dyn FeeDataProvider + Send + Sync> = Arc::new(
            MockHorizonClient::new().with_error(ProviderError::ServiceUnavailable),
        );
        let store = make_shared_store();
        let engine = make_shared_engine();

        poll_once(&provider, &store, &engine).await;

        assert!(store.read().await.is_empty());
    }

    #[tokio::test]
    async fn two_poll_cycles_accumulate_data_in_store() {
        let points = vec![make_point(100), make_point(200)];
        let provider: Arc<dyn FeeDataProvider + Send + Sync> =
            Arc::new(MockHorizonClient::new().with_fees(points));
        let store = make_shared_store();
        let engine = make_shared_engine();

        poll_once(&provider, &store, &engine).await;
        poll_once(&provider, &store, &engine).await;

        assert_eq!(store.read().await.len(), 4);
    }

    #[tokio::test]
    async fn poll_once_with_empty_provider_response_leaves_store_unchanged() {
        let provider: Arc<dyn FeeDataProvider + Send + Sync> =
            Arc::new(MockHorizonClient::new());
        let store = make_shared_store();
        let engine = make_shared_engine();

        poll_once(&provider, &store, &engine).await;

        assert!(store.read().await.is_empty());
    }
}