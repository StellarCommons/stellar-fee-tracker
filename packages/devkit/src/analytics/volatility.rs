//! Computes standard deviation and coefficient of variation over a window
//! of fee values.
//!
//! Closes #782.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolatilityStats {
    pub std_dev: f64,
    pub coefficient_of_variation: f64,
}

pub struct VolatilityCalculator;

impl VolatilityCalculator {
    pub fn compute(values: &[f64]) -> VolatilityStats {
        if values.is_empty() {
            return VolatilityStats { std_dev: 0.0, coefficient_of_variation: 0.0 };
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean == 0.0 { 0.0 } else { std_dev / mean };
        VolatilityStats { std_dev, coefficient_of_variation }
    }
}
