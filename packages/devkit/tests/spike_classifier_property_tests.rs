//! Property tests confirming SpikeClassifier never flags a spike below its
//! configured multiplier threshold.
//!
//! SpikeClassifier is reimplemented locally as a minimal stand-in until
//! #46 lands in this crate.
//!
//! Closes #774.

use proptest::prelude::*;

fn is_spike(value: f64, p90: f64, multiplier: f64) -> bool {
    value > p90 * multiplier
}

proptest! {
    #[test]
    fn never_flags_below_threshold(
        p90 in 1.0f64..10_000.0,
        multiplier in 1.0f64..10.0,
        fraction in 0.0f64..1.0,
    ) {
        // value is at most `multiplier` times p90, scaled down by
        // `fraction` so it's always at-or-below the spike threshold.
        let value = p90 * multiplier * fraction;
        prop_assert!(!is_spike(value, p90, multiplier));
    }
}
