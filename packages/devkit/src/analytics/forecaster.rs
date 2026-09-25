//! Produces a simple linear-extrapolation forecast from recent fee-value
//! trend data.
//!
//! Takes the trend data as a plain `&[f64]` rather than a concrete Trend
//! type (#56), which doesn't exist in this crate yet.
//!
//! Closes #785.

pub struct Forecaster;

impl Forecaster {
    /// Fits a simple linear regression (index -> value) over `history` and
    /// extrapolates `steps_ahead` points past the end of it.
    pub fn forecast(history: &[f64], steps_ahead: usize) -> Vec<f64> {
        if history.len() < 2 {
            return vec![history.last().copied().unwrap_or(0.0); steps_ahead];
        }

        let n = history.len() as f64;
        let xs: Vec<f64> = (0..history.len()).map(|i| i as f64).collect();
        let x_mean = xs.iter().sum::<f64>() / n;
        let y_mean = history.iter().sum::<f64>() / n;

        let numerator: f64 =
            xs.iter().zip(history).map(|(x, y)| (x - x_mean) * (y - y_mean)).sum();
        let denominator: f64 = xs.iter().map(|x| (x - x_mean).powi(2)).sum();
        let slope = if denominator == 0.0 { 0.0 } else { numerator / denominator };
        let intercept = y_mean - slope * x_mean;

        (0..steps_ahead)
            .map(|i| {
                let x = history.len() as f64 + i as f64;
                intercept + slope * x
            })
            .collect()
    }
}
