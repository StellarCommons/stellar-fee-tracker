//! Unit tests for the streaming storage (in-memory) sink — issue #624.
//!
//! Feeds a batch of records through [`MemorySink`] and asserts every record is
//! persisted, in order, to the in-memory store.

use stellar_devkit::streaming::{FeeRecord, MemorySink};

fn make_record(seq: u64) -> FeeRecord {
    FeeRecord {
        fee_amount: 100 + seq,
        ledger_sequence: seq,
        timestamp_ms: 1_700_000_000_000 + seq as i64,
        transaction_hash: None,
        is_spike: false,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn sink_persists_all_200_records() {
    let sink: MemorySink<FeeRecord> = MemorySink::new();
    for i in 0..200 {
        sink.emit(&make_record(i)).unwrap();
    }
    assert_eq!(sink.len(), 200, "all 200 records must be persisted");
    assert_eq!(sink.snapshot().len(), 200);
}

#[test]
fn sink_preserves_emission_order() {
    let sink: MemorySink<FeeRecord> = MemorySink::new();
    for i in 0..200 {
        sink.emit(&make_record(i)).unwrap();
    }
    let snap = sink.snapshot();
    assert_eq!(snap.first().unwrap().ledger_sequence, 0);
    assert_eq!(snap.last().unwrap().ledger_sequence, 199);
}

#[test]
fn sink_starts_empty() {
    let sink: MemorySink<FeeRecord> = MemorySink::new();
    assert!(sink.is_empty());
    assert_eq!(sink.len(), 0);
}
