use criterion::{criterion_group, criterion_main, Criterion};
use stellar_devkit::simulation::congestion_predictor::{
    congestion_score, CongestionInput, CongestionPredictor,
};

fn bench_predict(c: &mut Criterion) {
    let mut group = c.benchmark_group("congestion_predictor");

    group.bench_function("predict_1M", |b| {
        b.iter(|| {
            for i in 0..1_000_000u64 {
                let _ = CongestionPredictor::predict(i % 1000, 100 + (i * 10) % 10_000);
            }
        })
    });

    group.bench_function("congestion_score_1M", |b| {
        b.iter(|| {
            for i in 0..1_000_000u64 {
                let input = CongestionInput {
                    recent_fee_window: 100.0 + (i % 10_000) as f64,
                    capacity_usage: (i % 101) as f64 / 100.0,
                    spike_count: (i % 11) as u32,
                };
                let _ = congestion_score(&input);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_predict);
criterion_main!(benches);
