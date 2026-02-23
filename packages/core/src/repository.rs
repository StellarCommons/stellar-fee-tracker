//! Database repository for fee data persistence.
//!
//! All SQLite read/write logic lives here. The scheduler calls
//! [`FeeRepository::insert_fee_points`] after each poll tick and
//! [`FeeRepository::prune_older_than`] to keep the database bounded.
//!
//! On startup, [`FeeRepository::fetch_since`] rehydrates the in-memory
//! [`FeeHistoryStore`] from the last 24 hours of persisted data.

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::insights::types::FeeDataPoint;
use crate::services::horizon::HorizonFeeStats;

/// Repository for reading and writing fee data to SQLite.
pub struct FeeRepository {
    pool: SqlitePool,
}

impl FeeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Bulk-insert fee data points in a single transaction.
    /// Timestamps are stored as RFC 3339 strings.
    pub async fn insert_fee_points(&self, points: &[FeeDataPoint]) -> Result<(), sqlx::Error> {
        if points.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for point in points {
            let timestamp = point.timestamp.to_rfc3339();
            let fee_amount = point.fee_amount as i64;
            let ledger_sequence = point.ledger_sequence as i64;

            sqlx::query(
                "INSERT INTO fee_data_points
                 (fee_amount, timestamp, transaction_hash, ledger_sequence)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(fee_amount)
            .bind(&timestamp)
            .bind(&point.transaction_hash)
            .bind(ledger_sequence)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Fetch all fee data points with timestamp >= `since`, ordered ascending.
    pub async fn fetch_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<FeeDataPoint>, sqlx::Error> {
        let since_str = since.to_rfc3339();

        let rows = sqlx::query(
            "SELECT fee_amount, timestamp, transaction_hash, ledger_sequence
             FROM fee_data_points
             WHERE timestamp >= ?
             ORDER BY timestamp ASC",
        )
        .bind(&since_str)
        .fetch_all(&self.pool)
        .await?;

        let points = rows
            .into_iter()
            .filter_map(|row| {
                use sqlx::Row;
                let fee_amount: i64 = row.try_get("fee_amount").ok()?;
                let timestamp_str: String = row.try_get("timestamp").ok()?;
                let transaction_hash: String = row.try_get("transaction_hash").ok()?;
                let ledger_sequence: i64 = row.try_get("ledger_sequence").ok()?;

                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .ok()?
                    .with_timezone(&Utc);

                Some(FeeDataPoint {
                    fee_amount: fee_amount as u64,
                    timestamp,
                    transaction_hash,
                    ledger_sequence: ledger_sequence as u64,
                })
            })
            .collect();

        Ok(points)
    }

    /// Insert a fee snapshot (point-in-time Horizon fee_stats capture).
    pub async fn insert_snapshot(&self, snapshot: &HorizonFeeStats) -> Result<(), sqlx::Error> {
        let captured_at = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO fee_snapshots (base_fee, min_fee, max_fee, avg_fee, captured_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&snapshot.last_ledger_base_fee)
        .bind(&snapshot.fee_charged.min)
        .bind(&snapshot.fee_charged.max)
        .bind(&snapshot.fee_charged.avg)
        .bind(&captured_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all fee_data_points with timestamp older than `cutoff`.
    /// Returns the number of rows deleted.
    pub async fn prune_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, sqlx::Error> {
        let cutoff_str = cutoff.to_rfc3339();

        let result = sqlx::query(
            "DELETE FROM fee_data_points WHERE timestamp < ?",
        )
        .bind(&cutoff_str)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    use crate::db::create_pool;
    use crate::services::horizon::FeeCharged;

    async fn make_repo() -> FeeRepository {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        FeeRepository::new(pool)
    }

    fn make_point(fee_amount: u64, seconds_ago: i64) -> FeeDataPoint {
        FeeDataPoint {
            fee_amount,
            timestamp: Utc::now() - Duration::seconds(seconds_ago),
            transaction_hash: format!("hash_{}", fee_amount),
            ledger_sequence: 1,
        }
    }

    #[tokio::test]
    async fn insert_and_fetch_roundtrip() {
        let repo = make_repo().await;
        let points = vec![
            make_point(100, 300),
            make_point(200, 200),
            make_point(300, 100),
        ];

        repo.insert_fee_points(&points).await.unwrap();

        let since = Utc::now() - Duration::seconds(400);
        let fetched = repo.fetch_since(since).await.unwrap();

        assert_eq!(fetched.len(), 3);
        assert_eq!(fetched[0].fee_amount, 100);
        assert_eq!(fetched[1].fee_amount, 200);
        assert_eq!(fetched[2].fee_amount, 300);
    }

    #[tokio::test]
    async fn fetch_since_filters_old_points() {
        let repo = make_repo().await;
        let points = vec![
            make_point(100, 7200), // 2 hours ago — outside window
            make_point(200, 1800), // 30 min ago — inside window
            make_point(300, 600),  // 10 min ago — inside window
        ];

        repo.insert_fee_points(&points).await.unwrap();

        let since = Utc::now() - Duration::hours(1);
        let fetched = repo.fetch_since(since).await.unwrap();

        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].fee_amount, 200);
        assert_eq!(fetched[1].fee_amount, 300);
    }

    #[tokio::test]
    async fn insert_empty_slice_is_ok() {
        let repo = make_repo().await;
        let result = repo.insert_fee_points(&[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn prune_older_than_removes_old_rows() {
        let repo = make_repo().await;
        let points = vec![
            make_point(100, 7200), // 2 hours ago — outside window, should be pruned
            make_point(200, 1800), // 30 min ago — clearly inside window, kept
            make_point(300, 600),  // 10 min ago — inside window, kept
        ];

        repo.insert_fee_points(&points).await.unwrap();

        let cutoff = Utc::now() - Duration::hours(1);
        let deleted = repo.prune_older_than(cutoff).await.unwrap();

        assert_eq!(deleted, 1);

        let remaining = repo.fetch_since(Utc::now() - Duration::days(1)).await.unwrap();
        assert_eq!(remaining.len(), 2);
    }

    #[tokio::test]
    async fn prune_older_than_returns_zero_when_nothing_to_prune() {
        let repo = make_repo().await;
        let points = vec![make_point(100, 60)]; // 1 min ago

        repo.insert_fee_points(&points).await.unwrap();

        let cutoff = Utc::now() - Duration::hours(1);
        let deleted = repo.prune_older_than(cutoff).await.unwrap();

        assert_eq!(deleted, 0);
    }

    #[tokio::test]
    async fn insert_snapshot_succeeds() {
        let repo = make_repo().await;
        let stats = HorizonFeeStats {
            last_ledger_base_fee: "100".into(),
            fee_charged: FeeCharged {
                min: "100".into(),
                max: "5000".into(),
                avg: "213".into(),
                p10: "100".into(),
                p25: "100".into(),
                p50: "150".into(),
                p75: "300".into(),
                p90: "500".into(),
                p95: "800".into(),
            },
        };

        let result = repo.insert_snapshot(&stats).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn fetch_since_returns_empty_when_no_data() {
        let repo = make_repo().await;
        let fetched = repo.fetch_since(Utc::now() - Duration::hours(24)).await.unwrap();
        assert!(fetched.is_empty());
    }
}