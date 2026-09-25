//! Computes percentiles (p10/p50/p90/p99) over a fixed-size rolling window
//! of fee values, combining windowing and percentile computation into one
//! type.
//!
//! RollingWindow (#44) and PercentileTable (#43) don't exist in this crate
//! yet, so windowing is implemented inline here rather than composed from
//! them; the public surface (`push`, `percentiles`) is what a later
//! composed version should preserve.
//!
//! Closes #770.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentiles {
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
}

pub struct WindowedPercentile {
    capacity: usize,
    values: VecDeque<f64>,
}

impl WindowedPercentile {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, values: VecDeque::with_capacity(capacity) }
    }

    pub fn push(&mut self, value: f64) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    fn percentile(sorted: &[f64], pct: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let rank = (pct / 100.0) * (sorted.len() as f64 - 1.0);
        let lower = rank.floor() as usize;
        let upper = rank.ceil() as usize;
        if lower == upper {
            sorted[lower]
        } else {
            let frac = rank - lower as f64;
            sorted[lower] + (sorted[upper] - sorted[lower]) * frac
        }
    }

    /// Percentiles over the values currently in the window.
    pub fn percentiles(&self) -> Percentiles {
        let mut sorted: Vec<f64> = self.values.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Percentiles {
            p10: Self::percentile(&sorted, 10.0),
            p50: Self::percentile(&sorted, 50.0),
            p90: Self::percentile(&sorted, 90.0),
            p99: Self::percentile(&sorted, 99.0),
        }
    }
}
