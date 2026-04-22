//! Mock Horizon client for testing
//!
//! Implements `FeeDataProvider` with configurable responses so tests can
//! exercise the scheduler, insights engine, and API handlers without a
//! live Horizon node.
//!
//! Gated behind `#[cfg(test)]` — never compiled into production builds.

use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::insights::{
    error::ProviderError,
    provider::{FeeDataProvider, ProviderMetadata},
    types::FeeDataPoint,
};

/// A configurable mock implementation of `FeeDataProvider`.
///
/// # Example
/// ```rust
/// let mock = MockHorizonClient::new()
///     .with_fees(vec![fee_point])
///     .with_healthy(true);
/// ```
pub struct MockHorizonClient {
    /// Pre-configured fee data points to return on `fetch_latest_fees`.
    responses: Vec<FeeDataPoint>,
    /// When `Some`, `fetch_latest_fees` returns this error instead of `responses`.
    error: Option<ProviderError>,
    /// Tracks total number of `fetch_latest_fees` calls.
    pub call_count: Arc<AtomicUsize>,
    /// Controls whether `health_check` succeeds or returns `ServiceUnavailable`.
    healthy: bool,
}

impl MockHorizonClient {
    /// Create a new mock with no responses and a healthy status.
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
            error: None,
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: true,
        }
    }

    /// Set the fee data points returned by `fetch_latest_fees`.
    pub fn with_fees(mut self, fees: Vec<FeeDataPoint>) -> Self {
        self.responses = fees;
        self
    }

    /// Set the error returned by `fetch_latest_fees` (overrides `with_fees`).
    pub fn with_error(mut self, error: ProviderError) -> Self {
        self.error = Some(error);
        self
    }

    /// Control whether `health_check` succeeds (`true`) or fails (`false`).
    pub fn with_healthy(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }

    /// Returns the current call count without consuming the mock.
    pub fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl Default for MockHorizonClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeeDataProvider for MockHorizonClient {
    async fn fetch_latest_fees(&self) -> Result<Vec<FeeDataPoint>, ProviderError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if let Some(ref err) = self.error {
            // Clone the error into a new matching variant — ProviderError is not Clone
            return Err(match err {
                ProviderError::NetworkError { message } => ProviderError::NetworkError {
                    message: message.clone(),
                },
                ProviderError::FormatError { message } => ProviderError::FormatError {
                    message: message.clone(),
                },
                ProviderError::AuthError { message } => ProviderError::AuthError {
                    message: message.clone(),
                },
                ProviderError::RateLimitExceeded => ProviderError::RateLimitExceeded,
                ProviderError::ServiceUnavailable => ProviderError::ServiceUnavailable,
            });
        }

        Ok(self.responses.clone())
    }

    fn provider_name(&self) -> &str {
        "MockHorizon"
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        if self.healthy {
            Ok(())
        } else {
            Err(ProviderError::ServiceUnavailable)
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            supports_historical: false,
            max_batch_size: 100,
            rate_limit_per_minute: None,
            data_freshness_seconds: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_fee_point(fee_amount: u64) -> FeeDataPoint {
        FeeDataPoint {
            fee_amount,
            timestamp: Utc::now(),
            transaction_hash: format!("hash_{}", fee_amount),
            ledger_sequence: 1,
        }
    }

    #[tokio::test]
    async fn returns_configured_fee_points() {
        let points = vec![make_fee_point(100), make_fee_point(200)];
        let mock = MockHorizonClient::new().with_fees(points.clone());

        let result = mock.fetch_latest_fees().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].fee_amount, 100);
        assert_eq!(result[1].fee_amount, 200);
    }

    #[tokio::test]
    async fn returns_empty_vec_by_default() {
        let mock = MockHorizonClient::new();
        let result = mock.fetch_latest_fees().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn returns_configured_error() {
        let mock = MockHorizonClient::new().with_error(ProviderError::NetworkError {
            message: "simulated timeout".into(),
        });

        let result = mock.fetch_latest_fees().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::NetworkError { .. }
        ));
    }

    #[tokio::test]
    async fn call_counter_increments_on_each_fetch() {
        let mock = MockHorizonClient::new();
        assert_eq!(mock.calls(), 0);

        mock.fetch_latest_fees().await.unwrap();
        assert_eq!(mock.calls(), 1);

        mock.fetch_latest_fees().await.unwrap();
        assert_eq!(mock.calls(), 2);
    }

    #[tokio::test]
    async fn call_counter_increments_even_on_error() {
        let mock = MockHorizonClient::new().with_error(ProviderError::ServiceUnavailable);

        let _ = mock.fetch_latest_fees().await;
        assert_eq!(mock.calls(), 1);
    }

    #[tokio::test]
    async fn health_check_succeeds_when_healthy() {
        let mock = MockHorizonClient::new().with_healthy(true);
        assert!(mock.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn health_check_fails_when_unhealthy() {
        let mock = MockHorizonClient::new().with_healthy(false);
        let result = mock.health_check().await;
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::ServiceUnavailable
        ));
    }

    #[tokio::test]
    async fn error_does_not_affect_health_check() {
        // fetch_latest_fees error and health_check are independent
        let mock = MockHorizonClient::new()
            .with_error(ProviderError::NetworkError {
                message: "down".into(),
            })
            .with_healthy(true);

        assert!(mock.health_check().await.is_ok());
        assert!(mock.fetch_latest_fees().await.is_err());
    }

    #[test]
    fn provider_name_is_mock_horizon() {
        let mock = MockHorizonClient::new();
        assert_eq!(mock.provider_name(), "MockHorizon");
    }
}
