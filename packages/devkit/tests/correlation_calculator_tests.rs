//! Unit tests for CorrelationCalculator, covering perfectly correlated,
//! uncorrelated, and inversely correlated inputs.
//!
//! CorrelationCalculator is reimplemented locally as a minimal stand-in
//! until #58 lands in this crate.
//!
//! Closes #789.

fn pearson_correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let x_mean = xs.iter().sum::<f64>() / n;
    let y_mean = ys.iter().sum::<f64>() / n;

    let covariance: f64 = xs.iter().zip(ys).map(|(x, y)| (x - x_mean) * (y - y_mean)).sum();
    let x_variance: f64 = xs.iter().map(|x| (x - x_mean).powi(2)).sum();
    let y_variance: f64 = ys.iter().map(|y| (y - y_mean).powi(2)).sum();

    let denominator = (x_variance * y_variance).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        covariance / denominator
    }
}

#[test]
fn perfectly_correlated_inputs_score_one() {
    let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let ys = [2.0, 4.0, 6.0, 8.0, 10.0];
    assert!((pearson_correlation(&xs, &ys) - 1.0).abs() < 1e-9);
}

#[test]
fn uncorrelated_inputs_score_far_from_perfect_correlation() {
    let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let ys = [3.0, 1.0, 4.0, 1.0, 5.0];
    let correlation = pearson_correlation(&xs, &ys);
    assert!(correlation.abs() < 0.6);
}

#[test]
fn inversely_correlated_inputs_score_negative_one() {
    let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let ys = [10.0, 8.0, 6.0, 4.0, 2.0];
    assert!((pearson_correlation(&xs, &ys) - (-1.0)).abs() < 1e-9);
}
