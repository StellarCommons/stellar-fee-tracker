pub fn mean(fees: &[f64]) -> f64 {
    if fees.is_empty() {
        return 0.0;
    }
    fees.iter().sum::<f64>() / fees.len() as f64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeModelError {
    InvalidFeeRange,
    BaseFeeOutOfRange,
    NegativeVolatility,
    ZeroSeed,
}

#[derive(Debug, Clone, Copy)]
pub struct FeeModelConfig {
    pub base_fee: f64,
    pub volatility: f64,
    pub trend_per_step: f64,
    pub min_fee: f64,
    pub max_fee: f64,
    pub seed: u64,
}

impl Default for FeeModelConfig {
    fn default() -> Self {
        Self {
            base_fee: 100.0,
            volatility: 0.1,
            trend_per_step: 0.0,
            min_fee: 0.0,
            max_fee: 1_000.0,
            seed: 0x5EED_1234_ABCD_0001,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeeModel {
    config: FeeModelConfig,
    state: f64,
    rng: u64,
    generated: u64,
}

impl FeeModel {
    pub fn new(config: FeeModelConfig) -> Result<Self, FeeModelError> {
        if config.min_fee > config.max_fee {
            return Err(FeeModelError::InvalidFeeRange);
        }
        if config.base_fee < config.min_fee || config.base_fee > config.max_fee {
            return Err(FeeModelError::BaseFeeOutOfRange);
        }
        if config.volatility < 0.0 {
            return Err(FeeModelError::NegativeVolatility);
        }
        if config.seed == 0 {
            return Err(FeeModelError::ZeroSeed);
        }
        Ok(Self {
            state: config.base_fee,
            rng: config.seed,
            config,
            generated: 0,
        })
    }

    pub fn config(&self) -> &FeeModelConfig {
        &self.config
    }

    pub fn current_fee(&self) -> f64 {
        self.state
    }

    pub fn generated(&self) -> u64 {
        self.generated
    }

    pub fn next_fee(&mut self) -> f64 {
        let amplitude = self.config.volatility * self.config.base_fee;
        let noise = (self.next_unit() * 2.0 - 1.0) * amplitude;
        self.state = (self.state + self.config.trend_per_step + noise)
            .clamp(self.config.min_fee, self.config.max_fee);
        self.generated += 1;
        self.state
    }

    pub fn generate(&mut self, count: usize) -> Vec<f64> {
        let mut fees = Vec::with_capacity(count);
        for _ in 0..count {
            fees.push(self.next_fee());
        }
        fees
    }

    pub fn reset(&mut self) {
        self.state = self.config.base_fee;
        self.rng = self.config.seed;
        self.generated = 0;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> FeeModelConfig {
        FeeModelConfig {
            base_fee: 100.0,
            volatility: 0.2,
            trend_per_step: 0.0,
            min_fee: 50.0,
            max_fee: 150.0,
            seed: 42,
        }
    }

    #[test]
    fn a_new_model_starts_at_the_base_fee() {
        let model = FeeModel::new(config()).expect("model");
        assert_eq!(model.current_fee(), 100.0);
        assert_eq!(model.generated(), 0);
    }

    #[test]
    fn an_inverted_range_is_rejected() {
        let mut config = config();
        config.min_fee = 200.0;
        config.max_fee = 100.0;
        assert_eq!(
            FeeModel::new(config).err(),
            Some(FeeModelError::InvalidFeeRange)
        );
    }

    #[test]
    fn a_base_fee_outside_the_range_is_rejected() {
        let mut config = config();
        config.base_fee = 500.0;
        assert_eq!(
            FeeModel::new(config).err(),
            Some(FeeModelError::BaseFeeOutOfRange)
        );
    }

    #[test]
    fn negative_volatility_is_rejected() {
        let mut config = config();
        config.volatility = -0.1;
        assert_eq!(
            FeeModel::new(config).err(),
            Some(FeeModelError::NegativeVolatility)
        );
    }

    #[test]
    fn a_zero_seed_is_rejected() {
        let mut config = config();
        config.seed = 0;
        assert_eq!(FeeModel::new(config).err(), Some(FeeModelError::ZeroSeed));
    }

    #[test]
    fn output_stays_within_the_configured_range() {
        let mut model = FeeModel::new(config()).expect("model");
        for fee in model.generate(500) {
            assert!((50.0..=150.0).contains(&fee), "fee out of range: {fee}");
        }
    }

    #[test]
    fn the_same_seed_replays_the_same_sequence() {
        let mut first = FeeModel::new(config()).expect("model");
        let mut second = FeeModel::new(config()).expect("model");
        assert_eq!(first.generate(50), second.generate(50));
    }

    #[test]
    fn different_seeds_diverge() {
        let mut first_config = config();
        let mut second_config = config();
        second_config.seed = 43;
        let mut first = FeeModel::new(first_config).expect("model");
        let mut second = FeeModel::new(second_config).expect("model");
        assert_ne!(first.generate(50), second.generate(50));
    }

    #[test]
    fn a_positive_trend_drifts_upward() {
        let mut trending = config();
        trending.volatility = 0.0;
        trending.trend_per_step = 1.0;
        let mut model = FeeModel::new(trending).expect("model");
        let fees = model.generate(20);
        assert!(fees[19] > fees[0], "{} !> {}", fees[19], fees[0]);
    }

    #[test]
    fn a_negative_trend_drifts_downward() {
        let mut trending = config();
        trending.volatility = 0.0;
        trending.trend_per_step = -2.0;
        let mut model = FeeModel::new(trending).expect("model");
        let fees = model.generate(20);
        assert!(fees[19] < fees[0]);
    }

    #[test]
    fn a_trend_saturates_at_the_ceiling() {
        let mut trending = config();
        trending.volatility = 0.0;
        trending.trend_per_step = 25.0;
        let mut model = FeeModel::new(trending).expect("model");
        let fees = model.generate(20);
        assert_eq!(fees[19], 150.0);
    }

    #[test]
    fn a_trend_saturates_at_the_floor() {
        let mut trending = config();
        trending.volatility = 0.0;
        trending.trend_per_step = -25.0;
        let mut model = FeeModel::new(trending).expect("model");
        let fees = model.generate(20);
        assert_eq!(fees[19], 50.0);
    }

    #[test]
    fn zero_volatility_is_flat() {
        let mut flat = config();
        flat.volatility = 0.0;
        let mut model = FeeModel::new(flat).expect("model");
        let fees = model.generate(10);
        assert!(fees.iter().all(|fee| (*fee - 100.0).abs() < f64::EPSILON));
    }

    #[test]
    fn high_volatility_still_respects_the_ceiling() {
        let mut wild = config();
        wild.volatility = 5.0;
        let mut model = FeeModel::new(wild).expect("model");
        for fee in model.generate(200) {
            assert!(fee <= 150.0);
        }
    }

    #[test]
    fn generate_returns_the_requested_count() {
        let mut model = FeeModel::new(config()).expect("model");
        assert_eq!(model.generate(17).len(), 17);
        assert_eq!(model.generated(), 17);
    }

    #[test]
    fn generate_handles_an_empty_request() {
        let mut model = FeeModel::new(config()).expect("model");
        assert!(model.generate(0).is_empty());
    }

    #[test]
    fn reset_restores_the_starting_point() {
        let mut model = FeeModel::new(config()).expect("model");
        let first_run = model.generate(25);
        model.reset();
        assert_eq!(model.current_fee(), 100.0);
        assert_eq!(model.generated(), 0);
        assert_eq!(model.generate(25), first_run);
    }

    #[test]
    fn the_mean_summarises_a_sequence() {
        let mut model = FeeModel::new(config()).expect("model");
        let fees = model.generate(100);
        let mean = mean(&fees);
        assert!(mean > 50.0 && mean < 150.0);
    }

    #[test]
    fn the_mean_of_nothing_is_zero() {
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn the_default_configuration_is_valid() {
        let model = FeeModel::new(FeeModelConfig::default()).expect("default");
        assert_eq!(model.current_fee(), FeeModelConfig::default().base_fee);
    }
}
