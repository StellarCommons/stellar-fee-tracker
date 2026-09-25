//! Exponential-backoff delay calculator with a configurable base delay,
//! multiplier, and max delay cap.
//!
//! Closes #793.

#[derive(Debug, Clone, Copy)]
pub struct BackoffConfig {
    pub base_delay_ms: u64,
    pub multiplier: f64,
    pub max_delay_ms: u64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self { base_delay_ms: 100, multiplier: 2.0, max_delay_ms: 30_000 }
    }
}

pub struct BackoffCalculator {
    config: BackoffConfig,
}

impl BackoffCalculator {
    pub fn new(config: BackoffConfig) -> Self {
        Self { config }
    }

    /// Delay in milliseconds before retry attempt `attempt` (0-indexed).
    pub fn delay_for(&self, attempt: u32) -> u64 {
        let scaled =
            self.config.base_delay_ms as f64 * self.config.multiplier.powi(attempt as i32);
        (scaled as u64).min(self.config.max_delay_ms)
    }
}
