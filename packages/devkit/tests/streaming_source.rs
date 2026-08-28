use axum::{routing::get, Json, Router};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stellar_devkit::streaming::{FileReplaySource, PollingConfig, PollingSource, Source};
use tempfile::NamedTempFile;

#[tokio::test]
async fn polling_source_emits_one_record_per_poll() {
    let requests = Arc::new(AtomicUsize::new(0));
    let route_requests = requests.clone();
    let app = Router::new().route(
        "/fee_stats",
        get(move || {
            let requests = route_requests.clone();
            async move {
                let sequence = requests.fetch_add(1, Ordering::Relaxed) as u64 + 1;
                Json(serde_json::json!({
                    "last_ledger_base_fee": sequence * 100,
                    "ledger_capacity_usage": 0.1,
                    "p10": 100
                }))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let config = PollingConfig {
        endpoint: format!("http://{address}/fee_stats"),
        interval: std::time::Duration::ZERO,
        max_polls: Some(3),
    };
    let mut source = PollingSource::new(config);
    let mut records = Vec::new();
    while let Some(record) = source.next().await.unwrap() {
        records.push(record);
    }

    assert_eq!(records.len(), 3);
    assert_eq!(
        records
            .iter()
            .map(|record| record.fee_stroops)
            .collect::<Vec<_>>(),
        [100, 200, 300]
    );
    assert_eq!(requests.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn file_replay_source_preserves_order_and_timestamps() {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(
        file.path(),
        "timestamp_ms,fee_stroops,sequence\n1000,12,7\n2000,24,8\n3000,36,9\n",
    )
    .unwrap();

    let mut source = FileReplaySource::new(file.path()).unwrap();
    let mut records = Vec::new();
    while let Some(record) = source.next().await.unwrap() {
        records.push(record);
    }

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].timestamp_ms, 1000);
    assert_eq!(records[1].fee_stroops, 24);
    assert_eq!(records[2].sequence, 9);
}
