//! SpikeClassifier end-to-end classification throughput benchmark —
//! Criterion harness.
//!
//! Reimplements a minimal SpikeClassifier locally, matching this crate's
//! existing benchmark convention of not depending on crate-internal
//! modules (see benches/sqlite_insert.rs).
//!
//! Closes #777.
//!
//! Run with: `cargo bench --bench spike_classifier -p stellar-devkit`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct SpikeClassifier {
    multiplier: f64,
}

impl SpikeClassifier {
    fn is_spike(&self, value: f64, p90: f64) -> bool {
        value > p90 * self.multiplier
    }
}

fn make_dataset(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + (i % 500) as f64).collect()
}

fn bench_classification_throughput(c: &mut Criterion) {
    let dataset = make_dataset(10_000);
    let classifier = SpikeClassifier { multiplier: 1.5 };
    let p90 = 450.0;

    c.bench_function("spike_classifier_classify_10k", |b| {
        b.iter(|| {
            let flagged =
                dataset.iter().filter(|&&v| classifier.is_spike(black_box(v), p90)).count();
            black_box(flagged)
        })
    });
}

criterion_group!(benches, bench_classification_throughput);
criterion_main!(benches);
