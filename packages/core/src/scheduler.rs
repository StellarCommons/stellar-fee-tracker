use std::time::Duration;

use tokio::signal;
use tokio::time;

use crate::services::horizon::HorizonClient;



// ============================================================
// POLLING LOOP
// ============================================================

pub async fn run_fee_polling(
    horizon_client: HorizonClient,
    poll_interval_seconds: u64,
) {
    let mut interval = time::interval(Duration::from_secs(poll_interval_seconds));

    tracing::info!(
        "Fee polling started (interval: {}s)",
        poll_interval_seconds
    );

    loop {
        tokio::select! {
            _ = interval.tick() => {
                match horizon_client.fetch_fee_stats().await {
                    Ok(stats) => {
                        tracing::info!(
                            "Polled fee stats — base: {}, min: {}, max: {}, avg: {}",
                            stats.last_ledger_base_fee,
                            stats.fee_charged.min,
                            stats.fee_charged.max,
                            stats.fee_charged.avg
                        );
                    }
                    Err(err) => {
                        tracing::error!("Fee polling error: {}", err);
                    }
                }
            }

            _ = signal::ctrl_c() => {
                tracing::info!("Shutdown signal received. Stopping polling.");
                break;
            }
        }
    }

    tracing::info!("Fee polling stopped cleanly");
}


