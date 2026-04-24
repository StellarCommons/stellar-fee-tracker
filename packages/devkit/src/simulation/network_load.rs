//! Generates synthetic network load profiles for simulation.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Configuration for the seeded random network load generator.
pub struct NetworkLoadConfig {
    /// Minimum transactions per ledger.
    pub min_tx: u64,
    /// Maximum transactions per ledger.
    pub max_tx: u64,
    /// Maximum transactions the network can process per ledger (capacity limit).
    pub ledger_capacity: u64,
    /// Time between ledger closes in milliseconds.
    pub ledger_interval_ms: u64,
    /// Optional RNG seed for reproducibility.
    pub seed: Option<u64>,
}

impl Default for NetworkLoadConfig {
    fn default() -> Self {
        Self {
            min_tx: 10,
            max_tx: 1000,
            ledger_capacity: 1000,
            ledger_interval_ms: 5000,
            seed: None,
        }
    }
}

impl NetworkLoadConfig {
    /// Returns the average capacity pressure ratio based on midpoint tx count.
    pub fn avg_pressure(&self) -> f64 {
        let mid = (self.min_tx + self.max_tx) as f64 / 2.0;
        (mid / self.ledger_capacity as f64).clamp(0.0, 1.0)
    }
}

/// A single simulated ledger produced by the throughput simulator.
#[derive(Debug, Clone)]
pub struct SimulatedLedger {
    pub ledger_seq: u64,
    pub tx_count: u64,
    /// Capacity pressure in [0.0, 1.0]: tx_count / ledger_capacity.
    pub pressure: f64,
}

/// Stateful network load simulator with seeded RNG for reproducible output.
pub struct NetworkLoad {
    config: NetworkLoadConfig,
    rng: SmallRng,
}

impl NetworkLoad {
    pub fn new(config: NetworkLoadConfig) -> Self {
        let rng = match config.seed {
            Some(s) => SmallRng::seed_from_u64(s),
            None => SmallRng::from_entropy(),
        };
        Self { config, rng }
    }

    /// Generate `count` random tx-count samples within [min_tx, max_tx].
    pub fn generate(&mut self, count: usize) -> Vec<u64> {
        (0..count)
            .map(|_| self.rng.gen_range(self.config.min_tx..=self.config.max_tx))
            .collect()
    }

    /// Simulate `count` ledger closes, returning full `SimulatedLedger` records.
    pub fn simulate(&mut self, count: usize) -> Vec<SimulatedLedger> {
        self.generate(count)
            .into_iter()
            .enumerate()
            .map(|(i, tx_count)| {
                let pressure = (tx_count as f64 / self.config.ledger_capacity as f64).clamp(0.0, 1.0);
                SimulatedLedger {
                    ledger_seq: i as u64 + 1,
                    tx_count,
                    pressure,
                }
            })
            .collect()
    }
}
