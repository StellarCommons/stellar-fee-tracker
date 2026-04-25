use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use stellar_devkit::analysis::percentile::Percentile;

fn bench_percentile_algorithms(c: &mut Criterion) {
    let mut data: Vec<u64> = (1u64..=1_000_000).collect();
    data.sort_unstable();

    let mut group = c.benchmark_group("percentile_1M");

    group.bench_with_input(BenchmarkId::new("nearest_rank", "p50"), &data, |b, d| {
        b.iter(|| Percentile::nearest_rank(d, 50))
    });

    group.bench_with_input(BenchmarkId::new("linear_interpolation", "p50"), &data, |b, d| {
        b.iter(|| Percentile::linear_interpolation(d, 50))
    });

    group.bench_with_input(BenchmarkId::new("fee_distribution_summary", "all"), &data, |b, d| {
        b.iter(|| Percentile::fee_distribution_summary(d))
    });

    group.finish();
}

criterion_group!(benches, bench_percentile_algorithms);
criterion_main!(benches);
