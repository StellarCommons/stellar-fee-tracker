use stellar_devkit::analysis::spike_classifier::{SpikeClassifier, SpikeSeverity};

// ── classify ──────────────────────────────────────────────────────────────────

#[test]
fn classify_below_threshold_returns_none() {
    assert_eq!(SpikeClassifier::classify(199, 100), None);
}

#[test]
fn classify_low_severity() {
    assert_eq!(SpikeClassifier::classify(200, 100), Some(SpikeSeverity::Low));
    assert_eq!(SpikeClassifier::classify(499, 100), Some(SpikeSeverity::Low));
}

#[test]
fn classify_medium_severity() {
    assert_eq!(SpikeClassifier::classify(500, 100), Some(SpikeSeverity::Medium));
    assert_eq!(SpikeClassifier::classify(999, 100), Some(SpikeSeverity::Medium));
}

#[test]
fn classify_high_severity() {
    assert_eq!(SpikeClassifier::classify(1_000, 100), Some(SpikeSeverity::High));
    assert_eq!(SpikeClassifier::classify(4_999, 100), Some(SpikeSeverity::High));
}

#[test]
fn classify_critical_severity() {
    assert_eq!(SpikeClassifier::classify(5_001, 100), Some(SpikeSeverity::Critical));
}

#[test]
fn classify_zero_baseline_returns_none() {
    assert_eq!(SpikeClassifier::classify(1_000, 0), None);
}

// ── detect ────────────────────────────────────────────────────────────────────

#[test]
fn detect_no_spikes_in_flat_sequence() {
    let fees = vec![100u64; 10];
    assert!(SpikeClassifier::detect(&fees, 100).is_empty());
}

#[test]
fn detect_single_spike_correct_count_and_severity() {
    // one Low spike surrounded by baseline
    let fees = vec![100, 100, 300, 100, 100];
    let events = SpikeClassifier::detect(&fees, 100);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, SpikeSeverity::Low);
    assert_eq!(events[0].duration_ledgers, 1);
}

#[test]
fn detect_consecutive_spike_duration() {
    // three consecutive ledgers at 6× baseline → one Medium event of duration 3
    let fees = vec![100, 600, 600, 600, 100];
    let events = SpikeClassifier::detect(&fees, 100);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, SpikeSeverity::Medium);
    assert_eq!(events[0].duration_ledgers, 3);
}

#[test]
fn detect_multiple_separate_spikes() {
    let fees = vec![100, 300, 100, 1_500, 100];
    let events = SpikeClassifier::detect(&fees, 100);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].severity, SpikeSeverity::Low);
    assert_eq!(events[1].severity, SpikeSeverity::High);
}

#[test]
fn detect_escalating_spike_uses_max_severity() {
    // run goes Low → Critical; event should be Critical
    let fees = vec![100, 300, 6_000, 100];
    let events = SpikeClassifier::detect(&fees, 100);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, SpikeSeverity::Critical);
    assert_eq!(events[0].duration_ledgers, 2);
}

#[test]
fn detect_empty_slice_returns_empty() {
    assert!(SpikeClassifier::detect(&[], 100).is_empty());
}
