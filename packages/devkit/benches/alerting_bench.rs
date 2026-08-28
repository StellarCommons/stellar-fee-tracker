//! Throughput benchmark for the alert evaluator (#643).
//!
//! Measures rule evaluations per second with 10 active rules.
//! Target: > 100,000 evaluations/sec.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use stellar_devkit::alerting::evaluator::{FeeSnapshot, RuleEvaluator};
use stellar_devkit::alerting::rule::{AlertCondition, AlertRule};

fn ten_rules() -> Vec<AlertRule> {
    let conditions = [
        AlertCondition::FeeAbove,
        AlertCondition::FeeBelow,
        AlertCondition::P95Above,
        AlertCondition::SpikeCountExceeds,
        AlertCondition::CapacityUsageAbove,
    ];
    (0..10)
        .map(|i| {
            AlertRule::new(
                format!("rule-{i}"),
                format!("rule {i}"),
                conditions[i % conditions.len()],
                50,
            )
            // cooldown 0 so every evaluation exercises the full trigger path.
            .with_cooldown(0)
        })
        .collect()
}

fn bench_evaluator(c: &mut Criterion) {
    let rules = ten_rules();
    let snapshot = FeeSnapshot {
        base_fee: 500,
        p95: 500,
        spike_count: 500,
        capacity_usage_pct: 99,
    };

    let mut group = c.benchmark_group("alerting_evaluator");
    group.throughput(Throughput::Elements(rules.len() as u64));

    group.bench_function("evaluate_10_rules", |b| {
        let evaluator = RuleEvaluator::new();
        b.iter(|| {
            let events = evaluator.evaluate(&rules, &snapshot);
            criterion::black_box(events)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_evaluator);
criterion_main!(benches);
