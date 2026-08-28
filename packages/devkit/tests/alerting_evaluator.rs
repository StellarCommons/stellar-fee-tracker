//! Unit tests for the alert rule evaluator.
//!
//! - #638: FeeAbove condition (fires above threshold, not at/below, cooldown).
//! - #639: FeeBelow, P95Above, SpikeCountExceeds, CapacityUsageAbove each fire.

use std::time::{Duration, Instant};

use stellar_devkit::alerting::evaluator::{FeeSnapshot, RuleEvaluator};
use stellar_devkit::alerting::rule::{AlertCondition, AlertRule};

fn rule(id: &str, condition: AlertCondition, threshold: u64) -> AlertRule {
    AlertRule::new(id, id, condition, threshold).with_cooldown(300)
}

// ---------------------------------------------------------------------------
// #638 — FeeAbove
// ---------------------------------------------------------------------------

#[test]
fn fee_above_fires_when_fee_exceeds_threshold() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::FeeAbove, 100)];
    let snap = FeeSnapshot {
        base_fee: 150,
        ..Default::default()
    };
    let fired = ev.evaluate(&rules, &snap);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].current_value, 150);
    assert_eq!(fired[0].threshold, 100);
}

#[test]
fn fee_above_does_not_fire_at_or_below_threshold() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::FeeAbove, 100)];
    assert!(ev
        .evaluate(&rules, &FeeSnapshot { base_fee: 100, ..Default::default() })
        .is_empty());
    assert!(ev
        .evaluate(&rules, &FeeSnapshot { base_fee: 99, ..Default::default() })
        .is_empty());
}

#[test]
fn fee_above_respects_cooldown_window() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::FeeAbove, 100)];
    let snap = FeeSnapshot {
        base_fee: 150,
        ..Default::default()
    };
    let t0 = Instant::now();

    // First fires.
    assert_eq!(ev.evaluate_at(&rules, &snap, t0).len(), 1);
    // Second within the 300s cooldown is suppressed.
    let within = t0 + Duration::from_secs(120);
    assert!(ev.evaluate_at(&rules, &snap, within).is_empty());
    // After the cooldown expires it fires again.
    let after = t0 + Duration::from_secs(301);
    assert_eq!(ev.evaluate_at(&rules, &snap, after).len(), 1);
}

#[test]
fn disabled_rule_never_fires() {
    let ev = RuleEvaluator::new();
    let mut r = rule("r", AlertCondition::FeeAbove, 100);
    r.enabled = false;
    let snap = FeeSnapshot {
        base_fee: 999,
        ..Default::default()
    };
    assert!(ev.evaluate(&[r], &snap).is_empty());
}

// ---------------------------------------------------------------------------
// #639 — all other conditions
// ---------------------------------------------------------------------------

#[test]
fn fee_below_fires_when_fee_under_threshold() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::FeeBelow, 50)];
    assert_eq!(
        ev.evaluate(&rules, &FeeSnapshot { base_fee: 40, ..Default::default() })
            .len(),
        1
    );
    assert!(ev
        .evaluate(&rules, &FeeSnapshot { base_fee: 50, ..Default::default() })
        .is_empty());
}

#[test]
fn p95_above_fires_on_p95_metric() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::P95Above, 100_000)];
    let fired = ev.evaluate(
        &rules,
        &FeeSnapshot {
            p95: 219_192,
            ..Default::default()
        },
    );
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].current_value, 219_192);
}

#[test]
fn spike_count_exceeds_fires_on_spike_metric() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::SpikeCountExceeds, 3)];
    assert_eq!(
        ev.evaluate(&rules, &FeeSnapshot { spike_count: 5, ..Default::default() })
            .len(),
        1
    );
    assert!(ev
        .evaluate(&rules, &FeeSnapshot { spike_count: 3, ..Default::default() })
        .is_empty());
}

#[test]
fn capacity_usage_above_fires_on_capacity_metric() {
    let ev = RuleEvaluator::new();
    let rules = [rule("r", AlertCondition::CapacityUsageAbove, 80)];
    assert_eq!(
        ev.evaluate(
            &rules,
            &FeeSnapshot {
                capacity_usage_pct: 95,
                ..Default::default()
            }
        )
        .len(),
        1
    );
    assert!(ev
        .evaluate(
            &rules,
            &FeeSnapshot {
                capacity_usage_pct: 80,
                ..Default::default()
            }
        )
        .is_empty());
}

#[test]
fn each_condition_reads_its_own_metric_independently() {
    let ev = RuleEvaluator::new();
    // Only the p95 metric is high; a FeeAbove rule on base_fee must not fire.
    let rules = [
        rule("fee", AlertCondition::FeeAbove, 100),
        rule("p95", AlertCondition::P95Above, 100),
    ];
    let fired = ev.evaluate(
        &rules,
        &FeeSnapshot {
            base_fee: 10,
            p95: 500,
            ..Default::default()
        },
    );
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].rule_id, "p95");
}
