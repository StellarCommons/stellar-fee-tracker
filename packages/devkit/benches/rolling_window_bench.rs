use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use stellar_devkit::analysis::rolling_window::RollingWindow;

fn bench_rolling_window_algorithms(c: &mut Criterion) {
    let data: Vec<u64> = (1u64..=100_000).collect();
    let window = 50;

    let mut group = c.benchmark_group("rolling_window_100k");

    group.bench_with_input(BenchmarkId::new("sma", window), &data, |b, d| {
        b.iter(|| RollingWindow::sma(d, window))
    });

    group.bench_with_input(BenchmarkId::new("ema", "alpha=0.1"), &data, |b, d| {
        b.iter(|| RollingWindow::ema(d, 0.1))
    });

    group.bench_with_input(BenchmarkId::new("wma", window), &data, |b, d| {
        b.iter(|| RollingWindow::wma(d, window))
    });

    group.finish();
}

criterion_group!(benches, bench_rolling_window_algorithms);
criterion_main!(benches);
