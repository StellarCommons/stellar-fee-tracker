//! Alert dispatchers.
//!
//! Delivers fired [`crate::alerting::rule::AlertEvent`]s to a destination:
//! stdout, a webhook endpoint, or an append-only JSONL file.
//!
//! Scaffolded by issue #627; concrete dispatchers are added by issues
//! #632/#633/#634.
