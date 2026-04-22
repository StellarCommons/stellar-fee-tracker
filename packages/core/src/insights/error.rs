//! Error types for fee insights operations

use thiserror::Error;

/// Errors that can occur during insights processing
#[derive(Error, Debug)]
pub enum InsightsError {
    #[error("Invalid fee data: {message}")]
    InvalidData { message: String },

    #[error("Calculation error: {message}")]
    CalculationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Data provider error: {source}")]
    ProviderError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Insufficient data for calculation: {operation}")]
    InsufficientData { operation: String },

    #[error("Numerical overflow in calculation: {operation}")]
    NumericalOverflow { operation: String },
}

/// Errors from fee data providers
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Data format error: {message}")]
    FormatError { message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl InsightsError {
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData {
            message: message.into(),
        }
    }

    pub fn calculation_error(message: impl Into<String>) -> Self {
        Self::CalculationError {
            message: message.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::StorageError {
            message: message.into(),
        }
    }

    pub fn insufficient_data(operation: impl Into<String>) -> Self {
        Self::InsufficientData {
            operation: operation.into(),
        }
    }

    pub fn numerical_overflow(operation: impl Into<String>) -> Self {
        Self::NumericalOverflow {
            operation: operation.into(),
        }
    }
}
