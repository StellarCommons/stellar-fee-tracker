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
pub struct FeeModel {
    config: FeeModelConfig,
    rng: SmallRng,
}

impl FeeModel {
    /// Create a fee model. Panics if `config` is invalid — prefer `new_validated` for
    /// user-supplied configs.
    pub fn new(config: FeeModelConfig) -> Self {
        let rng = match config.seed {
            Some(s) => SmallRng::seed_from_u64(s),
            None => SmallRng::from_entropy(),
        };
        Self { config, rng }
    }

    /// Create a fee model, returning an error if the config fails validation.
    pub fn new_validated(config: FeeModelConfig) -> Result<Self, DevkitError> {
        config.validate()?;
        Ok(Self::new(config))
    }

    /// Generate `count` fee points starting from `start_timestamp`.
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
    pub fn run(config: &FeeModelConfig) -> Vec<FeePoint> {
        FeeModel::new(config.clone()).generate(config.ledger_count as usize, 0)
    }

    /// Generate `count` baseline fee values at the Stellar minimum (100 stroops).
    pub fn baseline(count: usize) -> Vec<f64> {
        vec![100.0; count]
    }

    /// Run multiple scenarios sequentially and return combined output.
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

    /// Generate `count` `FeePoint`s centred on `base_fee` with gaussian noise scaled by
    /// `noise_factor`. Timestamps are set to zero; call `apply_timestamps` to space them.
    /// Resolves #238.
    pub fn generate_baseline(&mut self, count: usize) -> Vec<FeePoint> {
        (0..count)
            .map(|i| {
                let noise = if self.config.noise_factor > 0.0 {
                    gaussian_noise(&mut self.rng)
                        * self.config.noise_factor
                        * self.config.base_fee as f64
                } else {
                    0.0
                };
                let fee = (self.config.base_fee as f64 + noise).max(1.0) as u64;
                FeePoint {
                    timestamp: 0,
                    fee,
                    ledger: i as u64 + 1,
                    is_spike: false,
                }
            })
            .collect()
    }

    /// Roll `spike_probability` for every point; multiply fee by `spike_multiplier` and
    /// set `is_spike = true` for those that fire. Resolves #239.
    pub fn inject_spikes(&mut self, points: &mut [FeePoint]) {
        for point in points.iter_mut() {
            if self.rng.gen::<f64>() < self.config.spike_probability {
                point.fee = point.fee.saturating_mul(self.config.spike_multiplier);
                point.is_spike = true;
            }
        }
    }
}

/// Box-Muller transform to generate a standard-normal sample using the RNG.
fn gaussian_noise(rng: &mut SmallRng) -> f64 {
    let u1: f64 = rng.gen::<f64>().max(f64::EPSILON);
    let u2: f64 = rng.gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Return the most-frequent value in a sorted slice (first mode when tied).
fn mode_fee(sorted: &[u64]) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let (mut best, mut best_count, mut cur, mut cur_count) = (sorted[0], 1usize, sorted[0], 1usize);
    for &v in &sorted[1..] {
        if v == cur {
            cur_count += 1;
            if cur_count > best_count {
                best = cur;
                best_count = cur_count;
            }
        } else {
            cur = v;
            cur_count = 1;
        }
    }
    best
}

/// Generate `count` evenly-spaced timestamps in milliseconds starting from `epoch_ms`.
/// Resolves #240.
pub fn generate_timestamps(count: usize, epoch_ms: u64, interval_ms: u64) -> Vec<u64> {
    (0..count)
        .map(|i| epoch_ms + (i as u64) * interval_ms)
        .collect()
}

/// Overwrite the `timestamp` field of each `FeePoint` with `epoch_ms + i * interval_ms`.
/// Resolves #240.
pub fn apply_timestamps(points: &mut [FeePoint], epoch_ms: u64, interval_ms: u64) {
    for (i, point) in points.iter_mut().enumerate() {
        point.timestamp = epoch_ms + (i as u64) * interval_ms;
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

    /// Build a `FeeCurve` from a slice of `FeePoint`s by computing percentiles over the
    /// fee distribution. Matches the Horizon `/fee_stats` JSON shape. Resolves #241.
    pub fn from_fee_points(points: &[FeePoint], base_fee: u64) -> Self {
        let last_ledger = points.last().map(|p| p.ledger).unwrap_or(0);

        let mut fees: Vec<u64> = points.iter().map(|p| p.fee).collect();
        fees.sort_unstable();

        let spike_count = points.iter().filter(|p| p.is_spike).count();
        let capacity = if points.is_empty() {
            0.0_f64
        } else {
            spike_count as f64 / points.len() as f64
        };

        let pct = |p: f64| -> String {
            if fees.is_empty() {
                return base_fee.to_string();
            }
            let idx = ((p / 100.0) * (fees.len() as f64 - 1.0)).round() as usize;
            fees[idx.min(fees.len() - 1)].to_string()
        };

        let max_val = fees.last().copied().unwrap_or(base_fee).to_string();
        let min_val = fees.first().copied().unwrap_or(base_fee).to_string();
        let mode_val = mode_fee(&fees).to_string();

        let percentiles = FeePercentiles {
            max: max_val,
            min: min_val,
            mode: mode_val,
            p10: pct(10.0),
            p20: pct(20.0),
            p30: pct(30.0),
            p40: pct(40.0),
            p50: pct(50.0),
            p60: pct(60.0),
            p70: pct(70.0),
            p80: pct(80.0),
            p90: pct(90.0),
            p95: pct(95.0),
            p99: pct(99.0),
        };

        FeeCurve {
            last_ledger: last_ledger.to_string(),
            last_ledger_base_fee: base_fee.to_string(),
            ledger_capacity_usage: format!("{:.2}", capacity),
            fee_charged: percentiles.clone(),
            max_fee: percentiles,
        }
    }

    /// Serialise a slice of `FeePoint`s directly to a Horizon `/fee_stats` JSON string.
    /// Resolves #241.
    pub fn fee_points_to_json(
        points: &[FeePoint],
        base_fee: u64,
    ) -> Result<String, serde_json::Error> {
        FeeCurve::from_fee_points(points, base_fee).to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #238

    #[test]
    fn generate_baseline_returns_correct_count() {
        let cfg = FeeModelConfig {
            base_fee: 200,
            noise_factor: 0.0,
            seed: Some(1),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        assert_eq!(model.generate_baseline(10).len(), 10);
    }

    #[test]
    fn generate_baseline_no_noise_exact_base_fee() {
        let cfg = FeeModelConfig {
            base_fee: 500,
            noise_factor: 0.0,
            seed: Some(2),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        for pt in model.generate_baseline(20) {
            assert_eq!(pt.fee, 500);
            assert!(!pt.is_spike);
            assert_eq!(pt.timestamp, 0);
        }
    }

    #[test]
    fn generate_baseline_noise_shifts_some_fees() {
        let cfg = FeeModelConfig {
            base_fee: 1_000,
            noise_factor: 0.2,
            seed: Some(42),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let fees: Vec<u64> = model
            .generate_baseline(100)
            .into_iter()
            .map(|p| p.fee)
            .collect();
        assert!(
            fees.iter().any(|&f| f != 1_000),
            "noise should shift at least one fee"
        );
    }

    // #239

    #[test]
    fn inject_spikes_all_marked_at_probability_one() {
        let cfg = FeeModelConfig {
            base_fee: 100,
            spike_probability: 1.0,
            spike_multiplier: 5,
            noise_factor: 0.0,
            seed: Some(7),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let mut pts = model.generate_baseline(5);
        model.inject_spikes(&mut pts);
        for pt in &pts {
            assert!(pt.is_spike);
            assert_eq!(pt.fee, 500);
        }
    }

    #[test]
    fn inject_spikes_none_marked_at_probability_zero() {
        let cfg = FeeModelConfig {
            base_fee: 100,
            spike_probability: 0.0,
            spike_multiplier: 10,
            noise_factor: 0.0,
            seed: Some(8),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let mut pts = model.generate_baseline(10);
        model.inject_spikes(&mut pts);
        assert!(pts.iter().all(|p| !p.is_spike));
    }

    // #240

    #[test]
    fn generate_timestamps_correct_spacing() {
        let ts = generate_timestamps(5, 1_000, 200);
        assert_eq!(ts, vec![1_000, 1_200, 1_400, 1_600, 1_800]);
    }

    #[test]
    fn apply_timestamps_overwrites_points() {
        let cfg = FeeModelConfig {
            seed: Some(1),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let mut pts = model.generate_baseline(3);
        apply_timestamps(&mut pts, 5_000, 100);
        assert_eq!(pts[0].timestamp, 5_000);
        assert_eq!(pts[1].timestamp, 5_100);
        assert_eq!(pts[2].timestamp, 5_200);
    }

    // #241

    #[test]
    fn fee_points_to_json_contains_required_keys() {
        let cfg = FeeModelConfig {
            base_fee: 100,
            noise_factor: 0.1,
            seed: Some(9),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let pts = model.generate_baseline(10);
        let json = FeeCurve::fee_points_to_json(&pts, 100).unwrap();
        assert!(json.contains("last_ledger"));
        assert!(json.contains("fee_charged"));
        assert!(json.contains("max_fee"));
        assert!(json.contains("ledger_capacity_usage"));
    }

    #[test]
    fn fee_curve_from_fee_points_last_ledger_and_base_fee() {
        let cfg = FeeModelConfig {
            base_fee: 100,
            noise_factor: 0.0,
            seed: Some(3),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let pts = model.generate_baseline(7);
        let curve = FeeCurve::from_fee_points(&pts, 100);
        assert_eq!(curve.last_ledger, "7");
        assert_eq!(curve.last_ledger_base_fee, "100");
    }

    #[test]
    fn fee_curve_capacity_usage_all_spikes() {
        let cfg = FeeModelConfig {
            base_fee: 100,
            spike_probability: 1.0,
            spike_multiplier: 2,
            noise_factor: 0.0,
            seed: Some(5),
            ..Default::default()
        };
        let mut model = FeeModel::new(cfg);
        let mut pts = model.generate_baseline(10);
        model.inject_spikes(&mut pts);
        let curve = FeeCurve::from_fee_points(&pts, 100);
        assert_eq!(curve.ledger_capacity_usage, "1.00");
    }
}
