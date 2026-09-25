//! PercentileTable computation benchmark over a large dataset — Criterion
//! harness.
//!
//! PercentileTable is reimplemented locally as a minimal stand-in until
//! #43 lands in this crate, matching this crate's existing benchmark
//! convention (see benches/sqlite_insert.rs).
//!
//! Closes #776.
//!
//! Run with: `cargo bench --bench percentile_table -p stellar-devkit`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct PercentileTable {
    p10: f64,
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
            p10: percentile(&sorted, 10.0),
            p50: percentile(&sorted, 50.0),
            p90: percentile(&sorted, 90.0),
            p99: percentile(&sorted, 99.0),
        }
    }
}

fn make_dataset(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + (i % 1000) as f64).collect()
}

fn bench_percentile_computation(c: &mut Criterion) {
    let dataset = make_dataset(100_000);
    c.bench_function("percentile_table_compute_100k", |b| {
        b.iter(|| {
            let table = PercentileTable::compute(black_box(&dataset));
            black_box((table.p10, table.p50, table.p90, table.p99))
        })
    });
}

criterion_group!(benches, bench_percentile_computation);
criterion_main!(benches);
