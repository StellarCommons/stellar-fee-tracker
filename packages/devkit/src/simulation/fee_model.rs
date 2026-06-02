//! Models for simulating Stellar transaction fee behaviour.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::error::DevkitError;

/// Configuration for a fee simulation scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct FeeModelConfig {
    /// Base fee in stroops.
    pub base_fee: u64,
    /// Probability [0.0, 1.0] that any given ledger is a spike.
    pub spike_probability: f64,
    /// Multiplier applied to base_fee during a spike.
    pub spike_multiplier: u64,
    /// Ledger close interval in seconds (used for timestamp spacing).
    pub ledger_interval_secs: u64,
    /// Number of ledgers to generate in a single `run()` call.
    pub ledger_count: u64,
    /// Optional RNG seed for reproducibility.
    pub seed: Option<u64>,
    /// Standard deviation of gaussian noise as a fraction of `base_fee` (0.0 = no noise).
    pub noise_factor: f64,
}

impl Default for FeeModelConfig {
    fn default() -> Self {
        Self {
            base_fee: 100,
            spike_probability: 0.05,
            spike_multiplier: 10,
            ledger_interval_secs: 5,
            ledger_count: 100,
            seed: None,
            noise_factor: 0.0,
        }
    }
}

impl FeeModelConfig {
    /// Validates that the configuration can produce sensible fee samples.
    ///
    /// # Errors
    ///
    /// Returns [`DevkitError::Simulation`] when any field is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::FeeModelConfig;
    ///
    /// // Default config is always valid.
    /// assert!(FeeModelConfig::default().validate().is_ok());
    ///
    /// // base_fee of zero is rejected.
    /// let bad = FeeModelConfig { base_fee: 0, ..FeeModelConfig::default() };
    /// assert!(bad.validate().is_err());
    ///
    /// // spike_probability must be in [0.0, 1.0].
    /// let bad = FeeModelConfig { spike_probability: 1.5, ..FeeModelConfig::default() };
    /// assert!(bad.validate().is_err());
    ///
    /// // spike_multiplier of zero is rejected.
    /// let bad = FeeModelConfig { spike_multiplier: 0, ..FeeModelConfig::default() };
    /// assert!(bad.validate().is_err());
    ///
    /// // Negative noise_factor is rejected.
    /// let bad = FeeModelConfig { noise_factor: -0.1, ..FeeModelConfig::default() };
    /// assert!(bad.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), DevkitError> {
        if self.base_fee == 0 {
            return Err(DevkitError::Simulation(
                "base_fee must be greater than zero".to_string(),
            ));
        }
        if !self.spike_probability.is_finite() || !(0.0..=1.0).contains(&self.spike_probability) {
            return Err(DevkitError::Simulation(
                "spike_probability must be a finite value between 0.0 and 1.0".to_string(),
            ));
        }
        if self.spike_multiplier == 0 {
            return Err(DevkitError::Simulation(
                "spike_multiplier must be greater than zero".to_string(),
            ));
        }
        if !self.noise_factor.is_finite() || self.noise_factor < 0.0 {
            return Err(DevkitError::Simulation(
                "noise_factor must be a finite value >= 0.0".to_string(),
            ));
        }
        Ok(())
    }
}

/// A single simulated fee data point.
#[derive(Debug, Clone)]
pub struct FeePoint {
    /// Simulated Unix timestamp (seconds).
    pub timestamp: u64,
    /// Fee in stroops for this ledger.
    pub fee: u64,
    /// Ledger sequence number (1-based within the scenario).
    pub ledger: u64,
    /// Whether this ledger was a spike.
    pub is_spike: bool,
}

/// Stateful fee model that uses a seeded RNG for reproducible simulations.
///
/// # Reproducibility
///
/// Pass `seed: Some(n)` in [`FeeModelConfig`] to get deterministic output.
/// The same seed always produces the same sequence regardless of platform.
///
/// ```
/// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
///
/// let cfg = FeeModelConfig { seed: Some(42), ledger_count: 10, ..FeeModelConfig::default() };
/// let run_a = FeeModel::run(&cfg);
/// let run_b = FeeModel::run(&cfg);
///
/// // Identical seeds produce identical fee sequences.
/// let fees_a: Vec<u64> = run_a.iter().map(|p| p.fee).collect();
/// let fees_b: Vec<u64> = run_b.iter().map(|p| p.fee).collect();
/// assert_eq!(fees_a, fees_b);
/// ```
pub struct FeeModel {
    config: FeeModelConfig,
    rng: SmallRng,
}

impl FeeModel {
    /// Create a fee model. Panics if `config` is invalid — prefer [`new_validated`] for
    /// user-supplied configs.
    ///
    /// [`new_validated`]: FeeModel::new_validated
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
    ///
    /// // Unseeded — output differs between runs.
    /// let _model = FeeModel::new(FeeModelConfig::default());
    ///
    /// // Seeded — output is reproducible.
    /// let cfg = FeeModelConfig { seed: Some(7), ..FeeModelConfig::default() };
    /// let _model = FeeModel::new(cfg);
    /// ```
    pub fn new(config: FeeModelConfig) -> Self {
        let rng = match config.seed {
            Some(s) => SmallRng::seed_from_u64(s),
            None => SmallRng::from_entropy(),
        };
        Self { config, rng }
    }

    /// Create a fee model, returning an error if the config fails validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
    ///
    /// assert!(FeeModel::new_validated(FeeModelConfig::default()).is_ok());
    ///
    /// let bad = FeeModelConfig { base_fee: 0, ..FeeModelConfig::default() };
    /// assert!(FeeModel::new_validated(bad).is_err());
    /// ```
    pub fn new_validated(config: FeeModelConfig) -> Result<Self, DevkitError> {
        config.validate()?;
        Ok(Self::new(config))
    }

    /// Generate `count` fee points starting from `start_timestamp`.
    ///
    /// Ledger sequence numbers are 1-based and contiguous within the returned slice.
    /// Timestamps advance by `config.ledger_interval_secs` per point.
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
    ///
    /// let cfg = FeeModelConfig { seed: Some(1), ..FeeModelConfig::default() };
    /// let mut model = FeeModel::new(cfg);
    /// let points = model.generate(5, 1_700_000_000);
    ///
    /// assert_eq!(points.len(), 5);
    /// // Ledgers are 1-based.
    /// assert_eq!(points[0].ledger, 1);
    /// assert_eq!(points[4].ledger, 5);
    /// // Timestamps advance by ledger_interval_secs (default 5 s).
    /// assert_eq!(points[1].timestamp - points[0].timestamp, 5);
    /// // Every fee is at least 1 stroop.
    /// assert!(points.iter().all(|p| p.fee >= 1));
    /// ```
    pub fn generate(&mut self, count: usize, start_timestamp: u64) -> Vec<FeePoint> {
        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let is_spike = self.rng.gen::<f64>() < self.config.spike_probability;
            let base = if self.config.noise_factor > 0.0 {
                let noise = gaussian_noise(&mut self.rng)
                    * self.config.noise_factor
                    * self.config.base_fee as f64;
                (self.config.base_fee as f64 + noise).max(1.0) as u64
            } else {
                self.config.base_fee
            };
            let fee = if is_spike {
                base * self.config.spike_multiplier
            } else {
                base
            };
            points.push(FeePoint {
                timestamp: start_timestamp + (i as u64) * self.config.ledger_interval_secs,
                fee,
                ledger: i as u64 + 1,
                is_spike,
            });
        }
        points
    }

    /// Generate a single fee sample in stroops (accounts for noise and spike probability).
    pub fn sample_fee(&mut self) -> u64 {
        let base = if self.config.noise_factor > 0.0 {
            let noise = gaussian_noise(&mut self.rng)
                * self.config.noise_factor
                * self.config.base_fee as f64;
            (self.config.base_fee as f64 + noise).max(1.0) as u64
        } else {
            self.config.base_fee
        };
        let is_spike = self.rng.gen::<f64>() < self.config.spike_probability;
        if is_spike {
            base * self.config.spike_multiplier
        } else {
            base
        }
    }

    /// Generate `count` fee values in stroops (accounts for noise and spike probability).
    pub fn generate_fees(&mut self, count: usize) -> Vec<u64> {
        (0..count).map(|_| self.sample_fee()).collect()
    }

    /// Convenience batch runner: generates `config.ledger_count` points from timestamp 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
    ///
    /// let cfg = FeeModelConfig {
    ///     seed: Some(99),
    ///     ledger_count: 20,
    ///     ..FeeModelConfig::default()
    /// };
    /// let points = FeeModel::run(&cfg);
    ///
    /// assert_eq!(points.len(), 20);
    /// assert_eq!(points[0].timestamp, 0);
    /// assert!(points.iter().all(|p| p.fee >= 1));
    /// ```
    pub fn run(config: &FeeModelConfig) -> Vec<FeePoint> {
        FeeModel::new(config.clone()).generate(config.ledger_count as usize, 0)
    }

    /// Generate `count` baseline fee values at the Stellar minimum (100 stroops).
    pub fn baseline(count: usize) -> Vec<f64> {
        vec![100.0; count]
    }

    /// Run multiple scenarios sequentially and return combined output.
    ///
    /// Ledger sequence numbers and timestamps are continuous across scenario boundaries
    /// so the combined slice can be fed directly into analysis functions.
    ///
    /// # Examples
    ///
    /// ```
    /// use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
    ///
    /// let low = FeeModelConfig { seed: Some(1), ledger_count: 5, ..FeeModelConfig::default() };
    /// let high = FeeModelConfig {
    ///     seed: Some(2),
    ///     ledger_count: 5,
    ///     base_fee: 500,
    ///     ..FeeModelConfig::default()
    /// };
    ///
    /// let points = FeeModel::run_scenarios(&[low, high]);
    ///
    /// assert_eq!(points.len(), 10);
    /// // Ledger sequence is continuous across the boundary.
    /// assert_eq!(points[5].ledger, 6);
    /// // Timestamps never go backwards.
    /// for w in points.windows(2) {
    ///     assert!(w[1].timestamp >= w[0].timestamp);
    /// }
    /// ```
    pub fn run_scenarios(configs: &[FeeModelConfig]) -> Vec<FeePoint> {
        let mut all = Vec::new();
        let mut ledger_offset = 0u64;
        let mut time_offset = 0u64;
        for config in configs {
            let mut model = FeeModel::new(config.clone());
            let mut points = model.generate(config.ledger_count as usize, time_offset);
            for p in &mut points {
                p.ledger += ledger_offset;
            }
            let last = points
                .last()
                .map(|p| (p.ledger, p.timestamp))
                .unwrap_or((0, 0));
            ledger_offset = last.0;
            time_offset = last.1 + config.ledger_interval_secs;
            all.extend(points);
        }
        all
    }
}

/// Box-Muller transform to generate a standard-normal sample using the RNG.
fn gaussian_noise(rng: &mut SmallRng) -> f64 {
    let u1: f64 = rng.gen::<f64>().max(f64::EPSILON);
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
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
