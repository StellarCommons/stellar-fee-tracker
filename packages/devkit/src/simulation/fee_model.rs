//! Models for simulating Stellar transaction fee behaviour.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Configuration for the seeded fee simulation.
pub struct FeeModelConfig {
    /// Base fee in stroops.
    pub base_fee: u64,
    /// Probability [0.0, 1.0] that any given ledger is a spike.
    pub spike_probability: f64,
    /// Multiplier applied to base_fee during a spike.
    pub spike_multiplier: u64,
    /// Ledger close interval in seconds (used for timestamp spacing).
    pub ledger_interval_secs: u64,
    /// Optional RNG seed for reproducibility.
    pub seed: Option<u64>,
}

impl Default for FeeModelConfig {
    fn default() -> Self {
        Self {
            base_fee: 100,
            spike_probability: 0.05,
            spike_multiplier: 10,
            ledger_interval_secs: 5,
            seed: None,
        }
    }
}

/// A single simulated fee data point.
pub struct FeePoint {
    /// Simulated Unix timestamp (seconds).
    pub timestamp: u64,
    /// Fee in stroops for this ledger.
    pub fee: u64,
    /// Whether this ledger was a spike.
    pub is_spike: bool,
}

/// Stateful fee model that uses a seeded RNG for reproducible simulations.
pub struct FeeModel {
    config: FeeModelConfig,
    rng: SmallRng,
}

impl FeeModel {
    pub fn new(config: FeeModelConfig) -> Self {
        let rng = match config.seed {
            Some(s) => SmallRng::seed_from_u64(s),
            None => SmallRng::from_entropy(),
        };
        Self { config, rng }
    }

    /// Generate `count` fee points starting from `start_timestamp`.
    pub fn generate(&mut self, count: usize, start_timestamp: u64) -> Vec<FeePoint> {
        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let is_spike = self.rng.gen::<f64>() < self.config.spike_probability;
            let fee = if is_spike {
                self.config.base_fee * self.config.spike_multiplier
            } else {
                self.config.base_fee
            };
            points.push(FeePoint {
                timestamp: start_timestamp + (i as u64) * self.config.ledger_interval_secs,
                fee,
                is_spike,
            });
        }
        points
    }
}

/// A fee curve snapshot matching the Horizon `fee_stats` shape.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeeCurve {
    pub last_ledger: String,
    pub last_ledger_base_fee: String,
    pub ledger_capacity_usage: String,
    pub fee_charged: FeePercentiles,
    pub max_fee: FeePercentiles,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeePercentiles {
    pub max: String,
    pub min: String,
    pub mode: String,
    pub p10: String,
    pub p20: String,
    pub p30: String,
    pub p40: String,
    pub p50: String,
    pub p60: String,
    pub p70: String,
    pub p80: String,
    pub p90: String,
    pub p95: String,
    pub p99: String,
}

impl FeeCurve {
    /// Generate a `FeeCurve` scaled by `pressure` (0.0–1.0).
    ///
    /// At pressure 0 fees stay at `base_fee`; at pressure 1 they reach `max_fee`.
    pub fn from_pressure(base_fee: u64, max_fee: u64, pressure: f64, ledger_seq: u64) -> Self {
        let pressure = pressure.clamp(0.0, 1.0);
        let fee = |pct: f64| -> String {
            let scaled = base_fee as f64 + (max_fee - base_fee) as f64 * pressure * pct;
            (scaled as u64).to_string()
        };

        FeeCurve {
            last_ledger: ledger_seq.to_string(),
            last_ledger_base_fee: base_fee.to_string(),
            ledger_capacity_usage: format!("{:.2}", pressure),
            fee_charged: FeePercentiles {
                min: base_fee.to_string(),
                max: fee(1.0),
                mode: fee(0.6),
                p10: fee(0.1),
                p20: fee(0.2),
                p30: fee(0.3),
                p40: fee(0.4),
                p50: fee(0.5),
                p60: fee(0.6),
                p70: fee(0.7),
                p80: fee(0.8),
                p90: fee(0.9),
                p95: fee(0.95),
                p99: fee(0.99),
            },
            max_fee: FeePercentiles {
                min: base_fee.to_string(),
                max: fee(1.0),
                mode: fee(0.7),
                p10: fee(0.15),
                p20: fee(0.25),
                p30: fee(0.35),
                p40: fee(0.45),
                p50: fee(0.55),
                p60: fee(0.65),
                p70: fee(0.75),
                p80: fee(0.85),
                p90: fee(0.92),
                p95: fee(0.97),
                p99: fee(1.0),
            },
        }
    }

    /// Serialise this `FeeCurve` to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
