//! Fee Data Provider Interface
//!
//! Provides abstraction layer for different fee data sources

use crate::insights::{error::ProviderError, types::FeeDataPoint};
use async_trait::async_trait;

/// Trait for fee data providers to ensure data source independence
#[async_trait]
pub trait FeeDataProvider {
    /// Fetch the latest fee data from the provider
    async fn fetch_latest_fees(&self) -> Result<Vec<FeeDataPoint>, ProviderError>;

    /// Get the name of this provider for logging/debugging
    fn provider_name(&self) -> &str;

    /// Check if the provider is currently available
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Default implementation - just try to fetch data
        self.fetch_latest_fees().await.map(|_| ())
    }

    /// Get provider-specific configuration or metadata
    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata::default()
    }
}

/// Metadata about a fee data provider
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    pub supports_historical: bool,
    pub max_batch_size: usize,
    pub rate_limit_per_minute: Option<u32>,
    pub data_freshness_seconds: u32,
}

impl Default for ProviderMetadata {
    fn default() -> Self {
        Self {
            supports_historical: false,
            max_batch_size: 100,
            rate_limit_per_minute: None,
            data_freshness_seconds: 60,
        }
    }
}

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;
