//! Rule evaluation and cooldown tracking.
//!
//! Evaluates a set of [`crate::alerting::rule::AlertRule`]s against current fee
//! data and emits events, honouring each rule's cooldown window.
//!
//! Scaffolded by issue #627; the evaluator and cooldown tracker are added by
//! issues #630/#631.
