//! Alert history store (#635).
//!
//! [`AlertHistory`] retains recently fired [`AlertEvent`]s in a bounded ring
//! buffer (default 1,000) for review and deduplication, with querying by rule,
//! severity, and time range, plus JSON export.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use super::rule::{AlertEvent, AlertSeverity};
use crate::error::DevkitError;

/// Default ring-buffer capacity.
pub const DEFAULT_CAPACITY: usize = 1_000;

/// Filter for [`AlertHistory::query`]. All set fields must match (AND).
#[derive(Debug, Clone, Default)]
pub struct HistoryQuery {
    /// Restrict to a single rule id.
    pub rule_id: Option<String>,
    /// Restrict to a single severity.
    pub severity: Option<AlertSeverity>,
    /// Inclusive lower bound on `triggered_at`.
    pub from: Option<DateTime<Utc>>,
    /// Inclusive upper bound on `triggered_at`.
    pub to: Option<DateTime<Utc>>,
}

/// Bounded, queryable store of fired alert events.
#[derive(Debug, Clone)]
pub struct AlertHistory {
    events: VecDeque<AlertEvent>,
    capacity: usize,
}

impl Default for AlertHistory {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl AlertHistory {
    /// New history with the default capacity (1,000).
    pub fn new() -> Self {
        Self::default()
    }

    /// New history with an explicit capacity (minimum 1).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    /// Record an event, evicting the oldest when at capacity.
    pub fn record(&mut self, event: AlertEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Number of retained events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Configured capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// All retained events, oldest first.
    pub fn all(&self) -> Vec<AlertEvent> {
        self.events.iter().cloned().collect()
    }

    /// Events matching every set field of `query`.
    pub fn query(&self, query: &HistoryQuery) -> Vec<AlertEvent> {
        self.events
            .iter()
            .filter(|e| {
                query.rule_id.as_ref().is_none_or(|id| &e.rule_id == id)
                    && query.severity.is_none_or(|s| e.severity == s)
                    && query.from.is_none_or(|f| e.triggered_at >= f)
                    && query.to.is_none_or(|t| e.triggered_at <= t)
            })
            .cloned()
            .collect()
    }

    /// Convenience: all events for a rule id.
    pub fn by_rule(&self, rule_id: &str) -> Vec<AlertEvent> {
        self.query(&HistoryQuery {
            rule_id: Some(rule_id.to_string()),
            ..Default::default()
        })
    }

    /// Convenience: all events of a severity.
    pub fn by_severity(&self, severity: AlertSeverity) -> Vec<AlertEvent> {
        self.query(&HistoryQuery {
            severity: Some(severity),
            ..Default::default()
        })
    }

    /// Export the full history as a JSON array.
    pub fn export_json(&self) -> Result<String, DevkitError> {
        serde_json::to_string(&self.all())
            .map_err(|e| DevkitError::Storage(format!("history export failed: {e}")))
    }
}
