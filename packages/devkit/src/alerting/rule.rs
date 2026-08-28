//! Alert rule and event data model.
//!
//! Defines the configurable [`AlertRule`] threshold (#628), the [`AlertEvent`]
//! fired when a rule matches (#629), and the [`AlertCondition`] /
//! [`AlertSeverity`] enums.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level attached to a rule and the events it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// The condition a rule evaluates against current fee data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Fires when the base fee is strictly above the threshold.
    FeeAbove,
    /// Fires when the base fee is strictly below the threshold.
    FeeBelow,
    /// Fires when the p95 fee is strictly above the threshold.
    P95Above,
    /// Fires when the observed spike count is strictly above the threshold.
    SpikeCountExceeds,
    /// Fires when ledger capacity usage (percent) is strictly above the threshold.
    CapacityUsageAbove,
}

/// A configurable threshold rule for fee-based alerting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertRule {
    /// Stable unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Condition evaluated against current fee data.
    pub condition: AlertCondition,
    /// Threshold value compared against the condition's metric.
    pub threshold: u64,
    /// Evaluation window in seconds (metric aggregation horizon).
    pub window_secs: u64,
    /// Minimum seconds between two alerts for this rule.
    pub cooldown_secs: u64,
    /// Severity attached to fired events.
    pub severity: AlertSeverity,
    /// Whether the rule is active.
    pub enabled: bool,
}

impl AlertRule {
    /// Construct a rule with sensible defaults (`window_secs` 60, `cooldown_secs`
    /// 300, `severity` Warning, `enabled` true).
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        condition: AlertCondition,
        threshold: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            condition,
            threshold,
            window_secs: 60,
            cooldown_secs: 300,
            severity: AlertSeverity::Warning,
            enabled: true,
        }
    }

    /// Builder-style override for the cooldown window.
    pub fn with_cooldown(mut self, cooldown_secs: u64) -> Self {
        self.cooldown_secs = cooldown_secs;
        self
    }

    /// Builder-style override for the severity.
    pub fn with_severity(mut self, severity: AlertSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// An event fired when a rule condition is met.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertEvent {
    /// Id of the rule that fired.
    pub rule_id: String,
    /// Name of the rule that fired.
    pub rule_name: String,
    /// Severity of the fired rule.
    pub severity: AlertSeverity,
    /// When the event was triggered (UTC).
    pub triggered_at: DateTime<Utc>,
    /// The metric value that triggered the rule.
    pub current_value: u64,
    /// The rule threshold that was crossed.
    pub threshold: u64,
    /// Human-readable summary message.
    pub message: String,
}
