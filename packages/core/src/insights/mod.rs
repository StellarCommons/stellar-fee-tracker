//! Fee Insights Module
//!
//! This module provides analytical insights from raw blockchain fee data,
//! including rolling averages, extremes tracking, and congestion detection.

// These re-exports will be consumed by Issues #06 and #07 when the engine
// and API server are wired up. Suppress dead-code warnings until then.
#![allow(unused_imports)]

pub mod calculator;
pub mod config;
pub mod detector;
pub mod engine;
pub mod error;
pub mod horizon_adapter;
pub mod provider;
pub mod tracker;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::InsightsConfig;
pub use engine::FeeInsightsEngine;
pub use error::InsightsError;
pub use horizon_adapter::HorizonFeeDataProvider;
pub use provider::{FeeDataProvider, ProviderMetadata};
pub use types::*;
