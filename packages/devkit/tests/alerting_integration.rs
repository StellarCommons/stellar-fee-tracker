//! Integration test for the full alert pipeline (#642).
//!
//! simulation source → spike transformer → alerting engine.
//! Asserts an alert is fired and recorded in history.

use stellar_devkit::alerting::{
    AlertCondition, AlertEngine, AlertRegistry, AlertRule, FeeSnapshot, StdoutDispatcher,
};
use stellar_devkit::streaming::transformer::FeeEvent as StreamFeeEvent;
use stellar_devkit::streaming::{FeeRecord, SpikeDetectionTransformer, SpikeTransformerEvent};

/// Deterministic simulation source: every 5th record is a large fee (spike).
fn simulation_source(n: u64) -> Vec<FeeRecord> {
    (0..n)
        .map(|i| FeeRecord {
            fee_amount: if i % 5 == 0 { 900 } else { 200 },
            ledger_sequence: i,
            timestamp_ms: 1_700_000_000_000 + i as i64,
            transaction_hash: None,
            is_spike: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        })
        .collect()
}

#[tokio::test]
async fn full_pipeline_fires_and_records_alert() {
    // 1. Source
    let source = simulation_source(20);

    // 2. Spike transformer (baseline 200, 2× threshold → >400 is a spike)
    let transformer = SpikeDetectionTransformer::new(200, 2.0);
    let spike_count = source
        .iter()
        .filter(|record| {
            matches!(
                transformer.transform(StreamFeeEvent::NewFeeRecord((*record).clone())),
                Some(SpikeTransformerEvent::SpikeDetected(_))
            )
        })
        .count() as u64;
    assert!(spike_count > 0, "the source must produce spikes");

    // 3. Alerting engine with a SpikeCountExceeds rule
    let mut registry = AlertRegistry::new();
    registry.add(
        AlertRule::new("spike", "spike alert", AlertCondition::SpikeCountExceeds, 2)
            .with_cooldown(0),
    );
    let mut engine = AlertEngine::with_registry(registry);
    engine.add_dispatcher(Box::new(StdoutDispatcher::new()));

    let snapshot = FeeSnapshot {
        base_fee: 900,
        spike_count,
        ..Default::default()
    };

    let fired = engine.process(&snapshot).await;

    // 4. Assertions
    assert!(!fired.is_empty(), "an alert should have fired");
    assert_eq!(fired[0].rule_id, "spike");
    assert_eq!(fired[0].current_value, spike_count);
    assert_eq!(
        engine.history().len(),
        fired.len(),
        "fired events are recorded in history"
    );
    assert_eq!(engine.history().all()[0].rule_id, "spike");
}
