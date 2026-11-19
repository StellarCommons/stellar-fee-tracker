//! Integration test against a mocked Horizon fee_stats endpoint.
use std::time::Duration;

use stellar_devkit::horizon_client::HorizonClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn fetches_fee_stats_from_mocked_horizon() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fee_stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "last_ledger": "100",
            "p50_accepted_fee": "100"
        })))
        .mount(&mock_server)
        .await;

    let client = HorizonClient::new(mock_server.uri(), Duration::from_secs(5))
        .expect("client should build");
    let body = client.fee_stats_raw().await.expect("request should succeed");
    let stats: serde_json::Value = serde_json::from_str(&body).expect("body should be JSON");

    assert_eq!(stats["last_ledger"], "100");
    assert_eq!(stats["p50_accepted_fee"], "100");
}
