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
//! Rule management ([`registry`]) and the running [`engine`] are layered on top
//! once their respective pieces land.

pub mod dispatcher;
pub mod evaluator;
pub mod history;
pub mod rule;
