//! RollingWindow insert/query benchmark — Criterion harness.
//!
//! RollingWindow is reimplemented locally as a minimal stand-in until #44
//! lands in this crate, matching this crate's existing benchmark
//! convention (see benches/sqlite_insert.rs).
//!
//! Closes #775.
//!
//! Run with: `cargo bench --bench rolling_window -p stellar-devkit`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
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

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("rolling_window_insert");
    for capacity in [100usize, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(capacity), &capacity, |b, &cap| {
            b.iter(|| {
                let mut window = RollingWindow::new(cap);
                for i in 0..cap * 2 {
                    window.push(black_box(i as f64));
                }
                black_box(window.snapshot().len())
            });
        });
    }
    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let mut window = RollingWindow::new(1_000);
    for i in 0..1_000 {
        window.push(i as f64);
    }
    c.bench_function("rolling_window_snapshot_1k", |b| {
        b.iter(|| black_box(window.snapshot()))
    });
}

criterion_group!(benches, bench_insert, bench_query);
criterion_main!(benches);
