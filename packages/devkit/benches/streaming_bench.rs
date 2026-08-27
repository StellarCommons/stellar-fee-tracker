//! Throughput benchmark for the streaming pipeline — issue #625.
//!
//! Pipeline: simulation source → spike transformer → memory sink.
//! Target: > 10,000 events/sec.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use stellar_devkit::streaming::sink::MemorySink;
use stellar_devkit::streaming::transformer::FeeEvent;
use stellar_devkit::streaming::{FeeRecord, SpikeDetectionTransformer, SpikeTransformerEvent};

/// Deterministic synthetic source producing `n` fee records, roughly 1-in-10 of
/// which are spikes above the transformer baseline.
fn simulation_source(n: usize) -> Vec<FeeRecord> {
    (0..n)
        .map(|i| {
            let base = 200u64;
            let fee = if i % 10 == 0 { base * 3 } else { base + (i as u64 % 50) };
            FeeRecord {
                fee_amount: fee,
                ledger_sequence: i as u64,
                timestamp_ms: 1_700_000_000_000 + i as i64,
                transaction_hash: None,
                is_spike: false,
                created_at: "2026-01-01T00:00:00Z".to_string(),
            }
        })
        .collect()
}

fn bench_pipeline_throughput(c: &mut Criterion) {
    const N: usize = 100_000;
    let source = simulation_source(N);
    let transformer = SpikeDetectionTransformer::new(200, 2.0);

    let mut group = c.benchmark_group("streaming_pipeline");
    group.throughput(Throughput::Elements(N as u64));
    group.sample_size(10);

    group.bench_function("source_transform_sink", |b| {
        b.iter(|| {
            let sink: MemorySink<SpikeTransformerEvent> = MemorySink::new();
            for record in &source {
                if let Some(event) =
                    transformer.transform(FeeEvent::NewFeeRecord(record.clone()))
                {
                    sink.emit(&event).unwrap();
                }
            }
            sink.len()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_pipeline_throughput);
criterion_main!(benches);
