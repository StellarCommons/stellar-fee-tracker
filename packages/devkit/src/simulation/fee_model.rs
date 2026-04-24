/// Models for simulating Stellar transaction fee behaviour.
pub struct FeeModel;

impl FeeModel {
    /// Generates `count` baseline fee values (in stroops) at the Stellar minimum (100).
    pub fn baseline(count: usize) -> Vec<f64> {
        vec![100.0; count]
use std::f64::consts::TAU;

use crate::error::DevkitError;

const DEFAULT_SEED: u64 = 0x5eed_f00d_dead_beef;

/// Configurable parameters for simulated fee generation.
#[derive(Clone, Debug, PartialEq)]
pub struct FeeModelConfig {
    /// Baseline fee level, in stroops, around which samples are generated.
    pub base_fee: u64,
    /// Probability in the range `[0.0, 1.0]` that a generated point becomes a spike.
    pub spike_probability: f64,
    /// Multiplier applied when a spike is injected.
    pub spike_multiplier: f64,
    /// Standard deviation of the gaussian noise as a fraction of `base_fee`.
    pub noise_factor: f64,
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

        if !self.spike_multiplier.is_finite() || self.spike_multiplier < 1.0 {
            return Err(DevkitError::Simulation(
                "spike_multiplier must be a finite value greater than or equal to 1.0".to_string(),
            ));
        }

        if !self.noise_factor.is_finite() || self.noise_factor < 0.0 {
            return Err(DevkitError::Simulation(
                "noise_factor must be a finite value greater than or equal to 0.0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for FeeModelConfig {
    fn default() -> Self {
        Self {
            base_fee: 100,
            spike_probability: 0.0,
            spike_multiplier: 10.0,
            noise_factor: 0.05,
        }
    }
}

/// Models Stellar fee behaviour using gaussian baseline noise plus optional spikes.
#[derive(Clone, Debug)]
pub struct FeeModel {
    config: FeeModelConfig,
    rng: LcgRng,
}

impl FeeModel {
    /// Creates a fee model with a deterministic default seed.
    pub fn new(config: FeeModelConfig) -> Result<Self, DevkitError> {
        Self::with_seed(config, DEFAULT_SEED)
    }

    /// Creates a fee model with an explicit seed for reproducible simulations.
    pub fn with_seed(config: FeeModelConfig, seed: u64) -> Result<Self, DevkitError> {
        config.validate()?;
        Ok(Self {
            config,
            rng: LcgRng::new(seed),
        })
    }

    /// Returns the active simulation configuration.
    pub fn config(&self) -> &FeeModelConfig {
        &self.config
    }

    /// Generates a single fee sample in stroops.
    pub fn sample_fee(&mut self) -> u64 {
        let base_fee = self.config.base_fee as f64;
        let noise = if self.config.noise_factor == 0.0 {
            0.0
        } else {
            self.rng.next_standard_normal() * self.config.noise_factor * base_fee
        };

        let mut fee = (base_fee + noise).max(1.0);
        if self.should_inject_spike() {
            fee *= self.config.spike_multiplier;
        }

        fee.round().max(1.0) as u64
    }

    /// Generates a sequence of fee samples using the current configuration.
    pub fn generate_fees(&mut self, count: usize) -> Vec<u64> {
        (0..count).map(|_| self.sample_fee()).collect()
    }

    fn should_inject_spike(&mut self) -> bool {
        self.config.spike_probability > 0.0
            && self.rng.next_unit_f64() < self.config.spike_probability
    }
}

#[derive(Clone, Debug)]
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        let seed = if seed == 0 { DEFAULT_SEED } else { seed };
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.state
    }

    fn next_unit_f64(&mut self) -> f64 {
        let raw = (self.next_u64() >> 11) as f64 / ((1_u64 << 53) as f64);
        raw.clamp(f64::EPSILON, 1.0 - f64::EPSILON)
    }

    fn next_standard_normal(&mut self) -> f64 {
        let u1 = self.next_unit_f64();
        let u2 = self.next_unit_f64();
        (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::{FeeModel, FeeModelConfig};

    #[test]
    fn config_validation_rejects_invalid_probability() {
        let err = FeeModelConfig {
            spike_probability: 1.5,
            ..FeeModelConfig::default()
        }
        .validate()
        .expect_err("config should reject invalid probability");

        assert_eq!(
            err.to_string(),
            "simulation error: spike_probability must be a finite value between 0.0 and 1.0"
        );
    }

    #[test]
    fn baseline_generator_without_noise_returns_base_fee() {
        let mut model = FeeModel::with_seed(
            FeeModelConfig {
                base_fee: 150,
                spike_probability: 0.0,
                spike_multiplier: 4.0,
                noise_factor: 0.0,
            },
            42,
        )
        .expect("valid config");

        assert_eq!(model.generate_fees(4), vec![150, 150, 150, 150]);
    }

    #[test]
    fn baseline_generator_with_noise_stays_centered_near_base_fee() {
        let mut model = FeeModel::with_seed(
            FeeModelConfig {
                base_fee: 1_000,
                spike_probability: 0.0,
                spike_multiplier: 5.0,
                noise_factor: 0.10,
            },
            7,
        )
        .expect("valid config");

        let samples = model.generate_fees(512);
        let average = samples.iter().copied().sum::<u64>() as f64 / samples.len() as f64;

        assert!(
            (average - 1_000.0).abs() < 40.0,
            "expected average near 1000, got {average}"
        );
        assert!(
            samples.windows(2).any(|pair| pair[0] != pair[1]),
            "expected gaussian noise to vary the generated samples"
        );
    }

    #[test]
    fn spike_injection_applies_multiplier() {
        let mut model = FeeModel::with_seed(
            FeeModelConfig {
                base_fee: 250,
                spike_probability: 1.0,
                spike_multiplier: 8.0,
                noise_factor: 0.0,
            },
            99,
        )
        .expect("valid config");

        assert_eq!(model.generate_fees(3), vec![2_000, 2_000, 2_000]);
    }
}
