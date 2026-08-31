//! Threshold-based fee alerting engine.
//!
//! The alerting module watches fee metrics and fires [`AlertEvent`]s when a
//! configured [`AlertRule`] condition is met. It is composed of:
//!
//! - [`rule`] — the [`AlertRule`] / [`AlertEvent`] data model and enums.
//! - [`evaluator`] — evaluates rules against current fee data with cooldown.
//! - [`dispatcher`] — delivers fired events (stdout / webhook / file).
//! - [`history`] — a bounded, queryable store of past events.
//!
//! Rule management ([`registry`]) and the running [`engine`] tie the pieces
//! together into a working alerting system.

pub mod dispatcher;
pub mod engine;
pub mod evaluator;
pub mod history;
pub mod registry;
pub mod rule;

pub use dispatcher::{AlertDispatcher, FileDispatcher, StdoutDispatcher, WebhookDispatcher};
pub use engine::AlertEngine;
pub use evaluator::{CooldownTracker, FeeSnapshot, RuleEvaluator};
pub use history::{AlertHistory, HistoryQuery};
pub use registry::AlertRegistry;
pub use rule::{AlertCondition, AlertEvent, AlertRule, AlertSeverity};
