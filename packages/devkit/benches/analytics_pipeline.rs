//! Full analytics pipeline (trend + volatility + regime + forecast)
//! benchmark over a realistic window size — Criterion harness.
//!
//! Each stage is reimplemented locally as a minimal stand-in until
//! #56/#57/#59/#60 land in this crate, matching this crate's existing
//! benchmark convention (see benches/sqlite_insert.rs).
//!
//! Closes #791.
//!
//! Run with: `cargo bench --bench analytics_pipeline -p stellar-devkit`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn compute_trend(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    values[values.len() - 1] - values[0]
}

fn compute_volatility(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

#[derive(Debug, PartialEq)]
enum Regime {
    Stable,
    Volatile,
    Trending,
}

fn classify_regime(trend: f64, volatility: f64) -> Regime {
    if volatility > 50.0 {
        Regime::Volatile
    } else if trend.abs() > 20.0 {
        Regime::Trending
    } else {
        Regime::Stable
    }
}

fn forecast_next(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    let xs: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
    let x_mean = xs.iter().sum::<f64>() / n;
    let y_mean = values.iter().sum::<f64>() / n;
    let numerator: f64 = xs.iter().zip(values).map(|(x, y)| (x - x_mean) * (y - y_mean)).sum();
    let denominator: f64 = xs.iter().map(|x| (x - x_mean).powi(2)).sum();
    let slope = if denominator == 0.0 { 0.0 } else { numerator / denominator };
    let intercept = y_mean - slope * x_mean;
    intercept + slope * n
}

fn make_window(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + (i as f64 * 0.5).sin() * 20.0 + i as f64 * 0.1).collect()
}

fn bench_full_pipeline(c: &mut Criterion) {
    let window = make_window(500);
    c.bench_function("analytics_pipeline_500", |b| {
        b.iter(|| {
            let trend = compute_trend(black_box(&window));
            let volatility = compute_volatility(black_box(&window));
            let regime = classify_regime(trend, volatility);
            let forecast = forecast_next(black_box(&window));
            black_box((trend, volatility, regime, forecast))
        })
    });
}

criterion_group!(benches, bench_full_pipeline);
criterion_main!(benches);
