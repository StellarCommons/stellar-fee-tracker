//! Flags fee values that are statistically far from their local
//! neighborhood (a z-score-style check against nearby values).
//!
//! Implemented as a standalone module so it can be used on its own; the
//! `is_outlier` function is the check that [`crate::validator`] (and other
//! data-quality code) can call.
//!
//! Closes #765.

pub struct OutlierDetector {
    /// A value more than this many neighborhood standard deviations from
    /// the neighborhood mean is flagged.
    threshold_std_devs: f64,
}

impl OutlierDetector {
    pub fn new(threshold_std_devs: f64) -> Self {
        Self { threshold_std_devs }
    }

    pub fn is_outlier(&self, neighbors: &[f64], value: f64) -> bool {
        if neighbors.is_empty() {
            return false;
        }
        let mean = neighbors.iter().sum::<f64>() / neighbors.len() as f64;
        let variance =
            neighbors.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / neighbors.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return value != mean;
        }
        (value - mean).abs() / std_dev > self.threshold_std_devs
    }
}
