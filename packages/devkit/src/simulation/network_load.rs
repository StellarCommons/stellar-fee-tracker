#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkLoadError {
    InvalidRange,
    CorrelationOutOfRange,
    NegativeNoise,
    ZeroSeed,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkLoadConfig {
    pub min_load: f64,
    pub max_load: f64,
    pub correlation: f64,
    pub noise: f64,
    pub seed: u64,
}

impl Default for NetworkLoadConfig {
    fn default() -> Self {
        Self {
            min_load: 0.0,
            max_load: 1.0,
            correlation: 0.8,
            noise: 0.05,
            seed: 0x10AD_0BED_0000_0001,
        }
    }
}

pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

#[derive(Debug, Clone)]
pub struct NetworkLoad {
    config: NetworkLoadConfig,
    rng: u64,
    last: f64,
    samples: u64,
}

impl NetworkLoad {
    pub fn new(config: NetworkLoadConfig) -> Result<Self, NetworkLoadError> {
        if config.min_load < 0.0 || config.max_load > 1.0 || config.min_load > config.max_load {
            return Err(NetworkLoadError::InvalidRange);
        }
        if config.correlation < 0.0 || config.correlation > 1.0 {
            return Err(NetworkLoadError::CorrelationOutOfRange);
        }
        if config.noise < 0.0 {
            return Err(NetworkLoadError::NegativeNoise);
        }
        if config.seed == 0 {
            return Err(NetworkLoadError::ZeroSeed);
        }
        Ok(Self {
            config,
            rng: config.seed,
            last: config.min_load,
            samples: 0,
        })
    }

    pub fn config(&self) -> &NetworkLoadConfig {
        &self.config
    }

    pub fn last_load(&self) -> f64 {
        self.last
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }

    pub fn from_fee(&mut self, fee: f64, fee_floor: f64, fee_ceiling: f64) -> f64 {
        let position = normalize(fee, fee_floor, fee_ceiling);
        let span = self.config.max_load - self.config.min_load;
        let correlated = position * span * self.config.correlation;
        let jitter = (self.next_unit() * 2.0 - 1.0) * self.config.noise * span;
        self.last = (self.config.min_load + correlated + jitter)
            .clamp(self.config.min_load, self.config.max_load);
        self.samples += 1;
        self.last
    }

    pub fn generate(&mut self, fees: &[f64], fee_floor: f64, fee_ceiling: f64) -> Vec<f64> {
        let mut loads = Vec::with_capacity(fees.len());
        for fee in fees {
            loads.push(self.from_fee(*fee, fee_floor, fee_ceiling));
        }
        loads
    }

    pub fn reset(&mut self) {
        self.rng = self.config.seed;
        self.last = self.config.min_load;
        self.samples = 0;
    }

    fn next_unit(&mut self) -> f64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        let value = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (value >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub fn normalize(value: f64, floor: f64, ceiling: f64) -> f64 {
    if ceiling <= floor {
        return 0.0;
    }
    ((value - floor) / (ceiling - floor)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> NetworkLoadConfig {
        NetworkLoadConfig {
            min_load: 0.1,
            max_load: 0.9,
            correlation: 0.8,
            noise: 0.05,
            seed: 7,
        }
    }

    #[test]
    fn a_new_load_starts_at_the_floor() {
        let load = NetworkLoad::new(config()).expect("load");
        assert_eq!(load.last_load(), 0.1);
        assert_eq!(load.samples(), 0);
    }

    #[test]
    fn an_out_of_unit_range_is_rejected() {
        let mut invalid = config();
        invalid.max_load = 1.5;
        assert_eq!(
            NetworkLoad::new(invalid).err(),
            Some(NetworkLoadError::InvalidRange)
        );
    }

    #[test]
    fn an_inverted_range_is_rejected() {
        let mut invalid = config();
        invalid.min_load = 0.9;
        invalid.max_load = 0.2;
        assert_eq!(
            NetworkLoad::new(invalid).err(),
            Some(NetworkLoadError::InvalidRange)
        );
    }

    #[test]
    fn correlation_above_one_is_rejected() {
        let mut invalid = config();
        invalid.correlation = 1.2;
        assert_eq!(
            NetworkLoad::new(invalid).err(),
            Some(NetworkLoadError::CorrelationOutOfRange)
        );
    }

    #[test]
    fn negative_noise_is_rejected() {
        let mut invalid = config();
        invalid.noise = -0.1;
        assert_eq!(
            NetworkLoad::new(invalid).err(),
            Some(NetworkLoadError::NegativeNoise)
        );
    }

    #[test]
    fn a_zero_seed_is_rejected() {
        let mut invalid = config();
        invalid.seed = 0;
        assert_eq!(
            NetworkLoad::new(invalid).err(),
            Some(NetworkLoadError::ZeroSeed)
        );
    }

    #[test]
    fn output_stays_within_the_configured_range() {
        let mut load = NetworkLoad::new(config()).expect("load");
        for fee in [0.0, 25.0, 50.0, 75.0, 100.0, 1e9] {
            let value = load.from_fee(fee, 0.0, 100.0);
            assert!((0.1..=0.9).contains(&value), "load out of range: {value}");
        }
    }

    #[test]
    fn a_higher_fee_never_produces_a_lower_load_on_average() {
        let mut load = NetworkLoad::new(config()).expect("load");
        let calm = load.generate(&vec![10.0; 200], 0.0, 100.0);
        let busy = load.generate(&vec![90.0; 200], 0.0, 100.0);
        assert!(
            mean(&busy) > mean(&calm),
            "{} !> {}",
            mean(&busy),
            mean(&calm)
        );
    }

    #[test]
    fn the_ceiling_fee_lands_near_the_top_of_the_band() {
        let mut load = NetworkLoad::new(config()).expect("load");
        let values = load.generate(&vec![100.0; 400], 0.0, 100.0);
        assert!(mean(&values) > 0.7, "mean was {}", mean(&values));
    }

    #[test]
    fn the_floor_fee_lands_near_the_bottom_of_the_band() {
        let mut load = NetworkLoad::new(config()).expect("load");
        let values = load.generate(&vec![0.0; 400], 0.0, 100.0);
        assert!(mean(&values) < 0.2, "mean was {}", mean(&values));
    }

    #[test]
    fn zero_correlation_ignores_the_fee() {
        let mut uncorrelated = config();
        uncorrelated.correlation = 0.0;
        let mut load = NetworkLoad::new(uncorrelated).expect("load");
        let low = load.generate(&vec![0.0; 300], 0.0, 100.0);
        let high = load.generate(&vec![100.0; 300], 0.0, 100.0);
        assert!((mean(&low) - mean(&high)).abs() < 0.05);
    }

    #[test]
    fn the_same_seed_replays_the_same_loads() {
        let mut first = NetworkLoad::new(config()).expect("load");
        let mut second = NetworkLoad::new(config()).expect("load");
        let fees = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            first.generate(&fees, 0.0, 100.0),
            second.generate(&fees, 0.0, 100.0)
        );
    }

    #[test]
    fn different_seeds_diverge() {
        let mut other = config();
        other.seed = 8;
        let mut first = NetworkLoad::new(config()).expect("load");
        let mut second = NetworkLoad::new(other).expect("load");
        let fees = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_ne!(
            first.generate(&fees, 0.0, 100.0),
            second.generate(&fees, 0.0, 100.0)
        );
    }

    #[test]
    fn an_empty_fee_run_produces_no_loads() {
        let mut load = NetworkLoad::new(config()).expect("load");
        assert!(load.generate(&[], 0.0, 100.0).is_empty());
    }

    #[test]
    fn reset_replays_the_sequence() {
        let mut load = NetworkLoad::new(config()).expect("load");
        let first_run = load.generate(&[1.0, 2.0, 3.0], 0.0, 100.0);
        load.reset();
        assert_eq!(load.samples(), 0);
        assert_eq!(load.generate(&[1.0, 2.0, 3.0], 0.0, 100.0), first_run);
    }

    #[test]
    fn a_degenerate_fee_window_yields_unweighted_noise_only() {
        let mut load = NetworkLoad::new(config()).expect("load");
        let values = load.generate(&vec![50.0; 200], 100.0, 100.0);
        assert!(values.iter().all(|value| (0.1..=0.9).contains(value)));
        assert!(mean(&values) < 0.2, "mean was {}", mean(&values));
    }

    #[test]
    fn the_default_configuration_is_valid() {
        let load = NetworkLoad::new(NetworkLoadConfig::default()).expect("default");
        assert_eq!(load.last_load(), NetworkLoadConfig::default().min_load);
    }
}
