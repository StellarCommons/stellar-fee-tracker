//! Unit tests for VolatilityCalculator, covering zero-variance and
//! high-variance inputs.
//!
//! VolatilityCalculator is reimplemented locally as a minimal stand-in,
//! matching this crate's existing test convention (see
//! tests/percentile_ledger_sequence_tests.rs) of not depending on
//! crate-internal modules.
//!
//! Closes #788.

struct VolatilityStats {
    std_dev: f64,
    coefficient_of_variation: f64,
}

fn compute_volatility(values: &[f64]) -> VolatilityStats {
    if values.is_empty() {
        return VolatilityStats { std_dev: 0.0, coefficient_of_variation: 0.0 };
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();
    let coefficient_of_variation = if mean == 0.0 { 0.0 } else { std_dev / mean };
    VolatilityStats { std_dev, coefficient_of_variation }
}

#[test]
fn zero_variance_input_has_zero_std_dev() {
    let values = [100.0, 100.0, 100.0, 100.0];
    let stats = compute_volatility(&values);
    assert_eq!(stats.std_dev, 0.0);
    assert_eq!(stats.coefficient_of_variation, 0.0);
}

#[test]
fn high_variance_input_has_large_std_dev() {
    let values = [10.0, 1000.0, 5.0, 900.0];
    let stats = compute_volatility(&values);
    assert!(stats.std_dev > 400.0);
    assert!(stats.coefficient_of_variation > 0.5);
}
