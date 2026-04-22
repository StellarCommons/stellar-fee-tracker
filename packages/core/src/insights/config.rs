//! Configuration for fee insights system

use crate::insights::types::TimeWindow;
use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for the fee insights engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsConfig {
    pub polling_interval: Duration,
    pub time_windows: Vec<TimeWindow>,
    pub spike_detection: SpikeConfig,
    pub storage_retention: Duration,
}

/// Configuration for spike detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpikeConfig {
    pub threshold_multiplier: f64,
    pub minimum_spike_duration: Duration,
    pub congestion_window: Duration,
}

/// Configuration for rolling averages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AverageConfig {
    pub max_buffer_size: usize,
    pub min_samples_for_calculation: usize,
}

/// Configuration for extremes tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremesConfig {
    pub tracking_period: Duration,
    pub historical_periods_to_keep: usize,
}

impl Default for InsightsConfig {
    fn default() -> Self {
        Self {
            polling_interval: Duration::minutes(1),
            time_windows: vec![
                TimeWindow {
                    name: "short_term".to_string(),
                    duration: Duration::hours(1),
                    min_samples: 10,
                },
                TimeWindow {
                    name: "medium_term".to_string(),
                    duration: Duration::hours(6),
                    min_samples: 30,
                },
                TimeWindow {
                    name: "long_term".to_string(),
                    duration: Duration::hours(24),
                    min_samples: 100,
                },
            ],
            spike_detection: SpikeConfig::default(),
            storage_retention: Duration::days(7),
        }
    }
}

impl Default for SpikeConfig {
    fn default() -> Self {
        Self {
            threshold_multiplier: 2.0,
            minimum_spike_duration: Duration::minutes(5),
            congestion_window: Duration::hours(1),
        }
    }
}

impl Default for AverageConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10000,
            min_samples_for_calculation: 5,
        }
    }
}

impl Default for ExtremesConfig {
    fn default() -> Self {
        Self {
            tracking_period: Duration::hours(24),
            historical_periods_to_keep: 30,
        }
    }
}
