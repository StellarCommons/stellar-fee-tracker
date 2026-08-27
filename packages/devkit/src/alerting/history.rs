//! Alert history store.
//!
//! Persists recently fired [`crate::alerting::rule::AlertEvent`]s in a bounded
//! ring buffer for review and deduplication.
//!
//! Scaffolded by issue #627; the store is added by issue #635.
