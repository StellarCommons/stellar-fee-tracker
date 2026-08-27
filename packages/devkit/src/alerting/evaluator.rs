//! Rule evaluation (#630) and cooldown tracking (#631).
//!
//! [`RuleEvaluator`] checks a set of [`AlertRule`]s against a [`FeeSnapshot`] of
//! current fee metrics and emits an [`AlertEvent`] for each triggered rule,
//! suppressing repeats within a rule's cooldown window via [`CooldownTracker`].

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::Utc;

use super::rule::{AlertCondition, AlertEvent, AlertRule};

/// Snapshot of the current fee metrics a rule is evaluated against.
///
/// Kept deliberately small and `Copy` so it can be cheaply threaded through the
/// evaluator; a bridge from the richer `protocol::HorizonFeeStats` can populate
/// it upstream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeeSnapshot {
    /// Current base fee (stroops).
    pub base_fee: u64,
    /// Current p95 fee (stroops).
    pub p95: u64,
    /// Number of spikes observed in the current window.
    pub spike_count: u64,
    /// Ledger capacity usage as an integer percentage (0–100).
    pub capacity_usage_pct: u64,
}

/// Tracks the last time each rule fired, to enforce `cooldown_secs`.
///
/// Thread-safe via an internal `Mutex`. `Instant`-based times are injectable at
/// the call sites (`*_at` methods) so cooldown behaviour is deterministically
/// testable without sleeping.
#[derive(Debug, Default)]
pub struct CooldownTracker {
    last_triggered: Mutex<HashMap<String, Instant>>,
}

impl CooldownTracker {
    /// Creates an empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `rule_id` may fire at `now` given its `cooldown_secs`.
    pub fn should_fire_at(&self, rule_id: &str, cooldown_secs: u64, now: Instant) -> bool {
        let map = self.last_triggered.lock().expect("cooldown mutex poisoned");
        match map.get(rule_id) {
            Some(&last) => now.duration_since(last) >= Duration::from_secs(cooldown_secs),
            None => true,
        }
    }

    /// Convenience wrapper using the real clock.
    pub fn should_fire(&self, rule_id: &str, cooldown_secs: u64) -> bool {
        self.should_fire_at(rule_id, cooldown_secs, Instant::now())
    }

    /// Records that `rule_id` fired at `now`.
    pub fn record_at(&self, rule_id: &str, now: Instant) {
        self.last_triggered
            .lock()
            .expect("cooldown mutex poisoned")
            .insert(rule_id.to_string(), now);
    }

    /// Convenience wrapper using the real clock.
    pub fn record(&self, rule_id: &str) {
        self.record_at(rule_id, Instant::now());
    }

    /// Clears the cooldown for a rule so it may fire immediately. Exposed for
    /// testing per issue #631.
    pub fn reset_cooldown(&self, rule_id: &str) {
        self.last_triggered
            .lock()
            .expect("cooldown mutex poisoned")
            .remove(rule_id);
    }
}

/// Evaluates alert rules against fee data, emitting events while honouring
/// per-rule cooldowns.
#[derive(Debug, Default)]
pub struct RuleEvaluator {
    cooldown: CooldownTracker,
}

impl RuleEvaluator {
    /// Creates a new evaluator with an empty cooldown tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Access the underlying cooldown tracker (e.g. to reset in tests).
    pub fn cooldown(&self) -> &CooldownTracker {
        &self.cooldown
    }

    /// The metric value a condition reads from the snapshot.
    fn current_value(condition: AlertCondition, s: &FeeSnapshot) -> u64 {
        match condition {
            AlertCondition::FeeAbove | AlertCondition::FeeBelow => s.base_fee,
            AlertCondition::P95Above => s.p95,
            AlertCondition::SpikeCountExceeds => s.spike_count,
            AlertCondition::CapacityUsageAbove => s.capacity_usage_pct,
        }
    }

    /// Whether `value` crosses `threshold` for the given condition.
    fn is_triggered(condition: AlertCondition, value: u64, threshold: u64) -> bool {
        match condition {
            AlertCondition::FeeBelow => value < threshold,
            AlertCondition::FeeAbove
            | AlertCondition::P95Above
            | AlertCondition::SpikeCountExceeds
            | AlertCondition::CapacityUsageAbove => value > threshold,
        }
    }

    /// Evaluate `rules` against `snapshot`, returning the events that fired.
    /// Uses the real clock; see [`RuleEvaluator::evaluate_at`] for tests.
    pub fn evaluate(&self, rules: &[AlertRule], snapshot: &FeeSnapshot) -> Vec<AlertEvent> {
        self.evaluate_at(rules, snapshot, Instant::now())
    }

    /// Evaluate `rules` against `snapshot` as of `now`, recording fires so
    /// cooldown windows are honoured deterministically.
    pub fn evaluate_at(
        &self,
        rules: &[AlertRule],
        snapshot: &FeeSnapshot,
        now: Instant,
    ) -> Vec<AlertEvent> {
        let mut events = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let value = Self::current_value(rule.condition, snapshot);
            if !Self::is_triggered(rule.condition, value, rule.threshold) {
                continue;
            }
            if !self.cooldown.should_fire_at(&rule.id, rule.cooldown_secs, now) {
                continue;
            }
            self.cooldown.record_at(&rule.id, now);
            events.push(AlertEvent {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                severity: rule.severity,
                triggered_at: Utc::now(),
                current_value: value,
                threshold: rule.threshold,
                message: format!(
                    "{} ({:?}): value {} vs threshold {}",
                    rule.name, rule.condition, value, rule.threshold
                ),
            });
        }
        events
    }
}
