//! Computes percentiles (p10/p50/p90/p99) over a fixed-size rolling window
//! of fee values, combining windowing and percentile computation into one
//! type.
//!
//! The window is a [`RollingWindow`] and the percentile maths is
//! [`crate::utilities::percentile_table::percentile`], so this type only adds
//! the combination of the two.
//!
//! Closes #770.

use crate::analysis::rolling_window::RollingWindow;

pub use crate::utilities::percentile_table::percentile;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentiles {
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
}

pub struct WindowedPercentile {
    window: RollingWindow,
}

impl WindowedPercentile {
    pub fn new(capacity: usize) -> Self {
        Self { window: RollingWindow::new(capacity) }
    }

    pub fn push(&mut self, value: f64) {
        self.window.push(value);
    }

    /// Percentiles over the values currently in the window.
    pub fn percentiles(&self) -> Percentiles {
        let mut sorted = self.window.snapshot();
        sorted.sort_by(f64::total_cmp);
        Percentiles {
            p10: percentile(&sorted, 10.0),
            p50: percentile(&sorted, 50.0),
            p90: percentile(&sorted, 90.0),
            p99: percentile(&sorted, 99.0),
        }
    }
}
