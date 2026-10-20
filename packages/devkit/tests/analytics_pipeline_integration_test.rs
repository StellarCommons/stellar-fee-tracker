//! Integration test for the "analyze current fee regime and forecast"
//! analytics pipeline: trend detection (#56) feeds regime classification
//! (#59), while volatility (#57) feeds both the regime classifier and the
//! confidence bounds around the forecast (#60, #61).
//!
//! The trend and regime detectors and the confidence-bound wrapper are
//! implemented locally as minimal stand-ins until those modules land in the
//! crate, matching this crate's existing test convention (see
//! tests/fee_analysis_pipeline_integration_test.rs). The forecaster,
//! volatility calculator, and rolling window are imported from the crate.
//!
//! Closes #792.

use stellar_devkit::analysis::rolling_window::RollingWindow;
use stellar_devkit::analytics::forecaster::Forecaster;
use stellar_devkit::analytics::volatility::{VolatilityCalculator, VolatilityStats};

/// Tolerance for floating-point comparisons.
const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrendDirection {
    Rising,
    Falling,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Trend {
    direction: TrendDirection,
    /// Least-squares slope in fee units per sample.
    slope: f64,
    /// Difference between the last and first sample in the window.
    change: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regime {
    Calm,
    Normal,
    Congested,
}

#[derive(Debug, Clone, PartialEq)]
struct Forecast {
    points: Vec<f64>,
    lower: Vec<f64>,
    upper: Vec<f64>,
    /// Confidence level the bounds were built for (for example `0.95`).
    confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct AnalysisReport {
    trend: Trend,
    volatility: VolatilityStats,
    regime: Regime,
    forecast: Forecast,
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}

/// Classifies a window as rising, falling, or flat.
///
/// A change of at most one fee unit is treated as flat so that small
/// integer-rounding noise does not flip the direction.
fn detect_trend(values: &[f64]) -> Trend {
    if values.len() < 2 {
        return Trend { direction: TrendDirection::Flat, slope: 0.0, change: 0.0 };
    }

    let count = values.len() as f64;
    let x_mean = (count - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / count;

    let numerator: f64 = values
        .iter()
        .enumerate()
        .map(|(index, value)| (index as f64 - x_mean) * (value - y_mean))
        .sum();
    let denominator: f64 = (0..values.len())
        .map(|index| (index as f64 - x_mean).powi(2))
        .sum();
    let slope = if denominator == 0.0 { 0.0 } else { numerator / denominator };

    let change = values[values.len() - 1] - values[0];
    let direction = if change > 1.0 {
        TrendDirection::Rising
    } else if change < -1.0 {
        TrendDirection::Falling
    } else {
        TrendDirection::Flat
    };

    Trend { direction, slope, change }
}

/// Classifies the window as calm, normal, or congested from its trend and
/// volatility. A large move or a large standard deviation is congested; a
/// small move combined with low variance is calm; everything else is normal.
fn classify_regime(trend: &Trend, volatility: &VolatilityStats) -> Regime {
    if volatility.std_dev >= 25.0 || trend.change.abs() >= 50.0 {
        Regime::Congested
    } else if volatility.std_dev <= 5.0 && trend.change.abs() <= 5.0 {
        Regime::Calm
    } else {
        Regime::Normal
    }
}

/// Maps a confidence level to a normal-distribution z-score. Only the three
/// levels used by the fee tracker are supported; anything lower falls back
/// to one standard deviation.
fn z_score(confidence: f64) -> f64 {
    if confidence >= 0.99 {
        2.576
    } else if confidence >= 0.95 {
        1.96
    } else if confidence >= 0.90 {
        1.645
    } else {
        1.0
    }
}

/// Extends the linear-extrapolation forecaster with confidence bounds that
/// widen with the square root of the forecast horizon.
fn forecast_with_bounds(history: &[f64], steps_ahead: usize, confidence: f64) -> Forecast {
    let points = Forecaster::forecast(history, steps_ahead);
    let std_dev = VolatilityCalculator::compute(history).std_dev;
    let z = z_score(confidence);

    let mut lower = Vec::with_capacity(steps_ahead);
    let mut upper = Vec::with_capacity(steps_ahead);
    for (step, point) in points.iter().enumerate() {
        let margin = z * std_dev * ((step + 1) as f64).sqrt();
        lower.push(point - margin);
        upper.push(point + margin);
    }

    Forecast { points, lower, upper, confidence }
}

/// Runs the full pipeline over a fee-value window: trend -> volatility ->
/// regime -> forecast with confidence bounds.
fn analyze_fee_regime(values: &[f64], steps_ahead: usize, confidence: f64) -> AnalysisReport {
    let trend = detect_trend(values);
    let volatility = VolatilityCalculator::compute(values);
    let regime = classify_regime(&trend, &volatility);
    let forecast = forecast_with_bounds(values, steps_ahead, confidence);
    AnalysisReport { trend, volatility, regime, forecast }
}

#[test]
fn a_steady_low_variance_window_is_calm_and_flat() {
    let history = [100.0, 100.4, 99.8, 100.2, 100.0];
    let report = analyze_fee_regime(&history, 3, 0.95);

    assert_eq!(report.trend.direction, TrendDirection::Flat);
    assert_eq!(report.regime, Regime::Calm);
    assert!(report.trend.slope.abs() < 1.0);
    assert!(report.volatility.std_dev < 1.0);

    for point in &report.forecast.points {
        assert!((point - 100.0).abs() < 1.0);
    }
}

#[test]
fn a_linear_rise_is_forecast_upward() {
    let history = [100.0, 110.0, 120.0, 130.0, 140.0];
    let report = analyze_fee_regime(&history, 2, 0.95);

    assert_eq!(report.trend.direction, TrendDirection::Rising);
    assert!(report.trend.slope > 9.9 && report.trend.slope < 10.1);
    assert!(report.trend.change > 39.0);
    assert_eq!(report.regime, Regime::Normal);
    assert!(approx_eq(report.forecast.points[0], 150.0));
    assert!(approx_eq(report.forecast.points[1], 160.0));
}

#[test]
fn a_linear_fall_is_forecast_downward() {
    let history = [200.0, 180.0, 160.0, 140.0, 120.0];
    let report = analyze_fee_regime(&history, 1, 0.90);

    assert_eq!(report.trend.direction, TrendDirection::Falling);
    assert!(report.trend.slope < -19.9);
    assert!(report.trend.change < -79.0);
    assert_eq!(report.regime, Regime::Congested);
    assert!(approx_eq(report.forecast.points[0], 100.0));
}

#[test]
fn confidence_bounds_nest_the_forecast_and_widen_with_the_horizon() {
    let history = [100.0, 110.0, 120.0, 130.0, 140.0];
    let report = analyze_fee_regime(&history, 4, 0.95);

    for (index, point) in report.forecast.points.iter().enumerate() {
        assert!(report.forecast.lower[index] <= *point + EPSILON);
        assert!(report.forecast.upper[index] >= *point - EPSILON);
    }

    let first_width = report.forecast.upper[0] - report.forecast.lower[0];
    let last_width = report.forecast.upper[3] - report.forecast.lower[3];
    assert!(last_width > first_width);
    assert!(approx_eq(first_width, 2.0 * 1.96 * report.volatility.std_dev));
    assert!(approx_eq(report.forecast.confidence, 0.95));
}

#[test]
fn higher_volatility_produces_wider_bounds() {
    let calm = [100.0, 100.1, 99.9, 100.0, 100.0];
    let volatile = [50.0, 200.0, 60.0, 190.0, 70.0];

    let calm_report = analyze_fee_regime(&calm, 2, 0.95);
    let volatile_report = analyze_fee_regime(&volatile, 2, 0.95);

    let calm_width = calm_report.forecast.upper[0] - calm_report.forecast.lower[0];
    let volatile_width = volatile_report.forecast.upper[0] - volatile_report.forecast.lower[0];

    assert_eq!(calm_report.regime, Regime::Calm);
    assert_eq!(volatile_report.regime, Regime::Congested);
    assert!(volatile_width > calm_width);
}

#[test]
fn empty_and_single_sample_windows_do_not_panic() {
    let empty = analyze_fee_regime(&[], 2, 0.95);
    assert_eq!(empty.trend.direction, TrendDirection::Flat);
    assert_eq!(empty.regime, Regime::Calm);
    assert!(approx_eq(empty.volatility.std_dev, 0.0));
    assert_eq!(empty.forecast.points.len(), 2);
    assert!(approx_eq(empty.forecast.points[0], 0.0));
    assert!(approx_eq(empty.forecast.lower[0], 0.0));
    assert!(approx_eq(empty.forecast.upper[0], 0.0));

    let single = analyze_fee_regime(&[123.0], 2, 0.95);
    assert!(approx_eq(single.forecast.points[0], 123.0));
    assert!(approx_eq(single.forecast.lower[1], 123.0));
    assert!(approx_eq(single.forecast.upper[1], 123.0));
}

#[test]
fn the_pipeline_runs_after_every_live_window_update() {
    let mut window = RollingWindow::new(4);
    for value in [100.0, 101.0, 100.0, 101.0] {
        window.push(value);
    }

    let steady = analyze_fee_regime(&window.snapshot(), 2, 0.95);
    assert_eq!(steady.trend.direction, TrendDirection::Flat);
    assert_eq!(steady.regime, Regime::Calm);

    window.push(500.0);
    assert_eq!(window.snapshot(), vec![101.0, 100.0, 101.0, 500.0]);

    let shocked = analyze_fee_regime(&window.snapshot(), 2, 0.95);
    assert_eq!(shocked.trend.direction, TrendDirection::Rising);
    assert_eq!(shocked.regime, Regime::Congested);
}

#[test]
fn the_full_report_is_internally_consistent() {
    let history = [90.0, 95.0, 100.0, 105.0, 110.0, 115.0];
    let report = analyze_fee_regime(&history, 3, 0.99);

    assert_eq!(report.trend.direction, TrendDirection::Rising);
    assert!(report.trend.slope > 0.0);
    assert!(report.trend.change > 0.0);
    assert!(report.volatility.std_dev > 0.0);
    assert!(report.volatility.coefficient_of_variation > 0.0);
    assert_eq!(report.regime, Regime::Normal);
    assert_eq!(report.forecast.points.len(), 3);
    assert!(report.forecast.points[2] > report.forecast.points[0]);
    assert!(approx_eq(report.forecast.confidence, 0.99));
}
