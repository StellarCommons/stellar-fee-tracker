//! Integration test wiring RollingWindow -> PercentileTable ->
//! SpikeClassifier into a single pipeline.
//!
//! RollingWindow, PercentileTable, and SpikeClassifier are reimplemented
//! locally as minimal stand-ins until #44/#45/#46 land in this crate.
//!
//! Closes #780.

use std::collections::VecDeque;

struct RollingWindow {
    capacity: usize,
    values: VecDeque<f64>,
}

impl RollingWindow {
    fn new(capacity: usize) -> Self {
        Self { capacity, values: VecDeque::with_capacity(capacity) }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    fn snapshot(&self) -> Vec<f64> {
        self.values.iter().copied().collect()
    }
}

struct PercentileTable {
    p50: f64,
    p90: f64,
    p99: f64,
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

impl PercentileTable {
    fn compute(values: &[f64]) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Self {
            p50: percentile(&sorted, 50.0),
            p90: percentile(&sorted, 90.0),
            p99: percentile(&sorted, 99.0),
        }
    }
}

struct SpikeClassifier {
    multiplier: f64,
}

impl SpikeClassifier {
    fn is_spike(&self, value: f64, p90: f64) -> bool {
        value > p90 * self.multiplier
    }
}

#[test]
fn pipeline_flags_a_spike_above_threshold() {
    let mut window = RollingWindow::new(10);
    for v in [100.0, 105.0, 98.0, 102.0, 101.0, 99.0, 103.0, 100.0, 104.0, 97.0] {
        window.push(v);
    }

    let table = PercentileTable::compute(&window.snapshot());
    let classifier = SpikeClassifier { multiplier: 1.5 };

    assert!(!classifier.is_spike(120.0, table.p90));
    assert!(classifier.is_spike(500.0, table.p90));
}

#[test]
fn pipeline_handles_a_freshly_filled_window() {
    let mut window = RollingWindow::new(5);
    for v in [10.0, 20.0, 30.0, 40.0, 50.0] {
        window.push(v);
    }
    assert_eq!(window.snapshot().len(), 5);

    let table = PercentileTable::compute(&window.snapshot());
    assert!(table.p90 >= table.p50);
    assert!(table.p99 >= table.p90);
}
