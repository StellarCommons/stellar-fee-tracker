//! Database connection pool and migrations.
//!
//! Call [`create_pool`] at startup. It connects to the SQLite database
//! and runs all pending migrations automatically via `sqlx::migrate!`.

use sqlx::SqlitePool;

/// Create a SQLite connection pool and run all pending migrations.
///
/// `database_url` must be a valid SQLite connection string, e.g.:
/// - `"sqlite://stellar_fees.db"` — file-based database
/// - `"sqlite::memory:"` — in-memory database (useful for tests)
///
/// Returns an error if the connection cannot be established or any
/// migration fails.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_pool_succeeds_with_in_memory_database() {
        let pool = create_pool("sqlite::memory:").await;
        assert!(pool.is_ok(), "Expected Ok but got: {:?}", pool.err());
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        // Running create_pool twice on the same DB must not fail —
        // CREATE TABLE IF NOT EXISTS ensures idempotency.
        let pool = create_pool("sqlite::memory:").await.unwrap();

        // Run migrations a second time explicitly
        let result = sqlx::migrate!("./migrations").run(&pool).await;
        assert!(
            result.is_ok(),
            "Second migration run failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn fee_data_points_table_exists_after_migration() {
        let pool = create_pool("sqlite::memory:").await.unwrap();

        // Insert a row to verify the table and columns exist
        let result = sqlx::query(
            "INSERT INTO fee_data_points
             (fee_amount, timestamp, transaction_hash, ledger_sequence)
             VALUES (100, '2024-01-01T00:00:00Z', 'testhash', 1)",
        )
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Insert failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn fee_snapshots_table_exists_after_migration() {
        let pool = create_pool("sqlite::memory:").await.unwrap();

        let result = sqlx::query(
            "INSERT INTO fee_snapshots (base_fee, min_fee, max_fee, avg_fee, captured_at)
             VALUES ('100', '100', '5000', '213', '2024-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await;

        assert!(result.is_ok(), "Insert failed: {:?}", result.err());
    }
}
