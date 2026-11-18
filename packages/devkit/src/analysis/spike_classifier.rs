//! Flags a fee value as a spike when it exceeds the rolling window's p90
//! by a configurable multiplier.
//!
//! Takes the window's p90 as a plain `f64` so it can be fed from
//! [`crate::analysis::windowed_percentile::WindowedPercentile`] (built on
//! `RollingWindow` and `PercentileTable`) or from any other source.
//!
//! Closes #771.

pub struct SpikeClassifier {
    multiplier: f64,
}

impl SpikeClassifier {
    pub fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    pub fn is_spike(&self, value: f64, p90: f64) -> bool {
        value > p90 * self.multiplier
    }
}
