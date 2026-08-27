//! Unit tests for the SQLite storage backend schema and CRUD operations.
//!
//! # Note on approach
//!
//! `SqliteStore` (packages/devkit/src/storage/sqlite.rs) is currently a stub
//! (empty struct, no methods) pending implementation in issue #594. Rather than
//! blocking issue #602 on that work, these tests validate the **storage contract**
//! directly against a raw `rusqlite` `:memory:` connection using the exact schema
//! that `SqliteStore` will implement.
//!
//! Once #594 lands, these tests will be migrated to exercise `SqliteStore`
//! directly, replacing the raw `rusqlite` calls with the trait API.
//!
//! ## Tests
//! 1. `test_insert_single_record`   — insert one record, query all, assert count=1 and all fields match
//! 2. `test_insert_batch_records`   — insert 10 records in a transaction, query all, assert count=10
//! 3. `test_query_by_fee_range`     — filter by min_fee/max_fee, assert only matching records returned
//! 4. `test_query_by_time_range`    — filter by timestamp_ms range, assert correct subset returned
//! 5. `test_query_limit`            — insert 20 records, query with LIMIT 5, assert exactly 5 returned
//! 6. `test_delete_before`          — insert old + new records, delete before cutoff, assert old ones gone
//! 7. `test_latest`                 — insert records at different timestamps, assert highest one returned

use rusqlite::{Connection, Result};

// ---------------------------------------------------------------------------
// Schema helpers
// ---------------------------------------------------------------------------

/// Create the `fee_records` table and its indices on the given connection.
/// This mirrors the schema that `SqliteStore` will use when #594 is implemented.
fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE fee_records (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            fee_amount       INTEGER NOT NULL,
            ledger_sequence  INTEGER NOT NULL,
            timestamp_ms     INTEGER NOT NULL,
            transaction_hash TEXT,
            is_spike         INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT    NOT NULL
        );
        CREATE INDEX idx_fee_records_timestamp ON fee_records(timestamp_ms);
        CREATE INDEX idx_fee_records_ledger    ON fee_records(ledger_sequence);
        ",
    )
}

/// Insert a single fee record row.
fn insert_record(
    conn: &Connection,
    fee_amount: u64,
    ledger_sequence: u64,
    timestamp_ms: i64,
    transaction_hash: Option<&str>,
    is_spike: bool,
    created_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO fee_records
             (fee_amount, ledger_sequence, timestamp_ms, transaction_hash, is_spike, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            fee_amount as i64,
            ledger_sequence as i64,
            timestamp_ms,
            transaction_hash,
            is_spike as i32,
            created_at,
        ],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 1 – insert a single record and verify all fields round-trip correctly
// ---------------------------------------------------------------------------

#[test]
fn test_insert_single_record() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    insert_record(
        &conn,
        1000,
        42,
        1_700_000_000_000,
        Some("abc123hash"),
        false,
        "2023-11-14T22:13:20Z",
    )?;

    // Query all rows
    let mut stmt = conn.prepare(
        "SELECT fee_amount, ledger_sequence, timestamp_ms, transaction_hash, is_spike, created_at
         FROM fee_records",
    )?;
    let rows: Vec<(i64, i64, i64, Option<String>, i32, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<_>>()?;

    assert_eq!(rows.len(), 1, "expected exactly 1 row after single insert");
    let (fee, ledger, ts, hash, spike, created) = &rows[0];
    assert_eq!(*fee, 1000_i64);
    assert_eq!(*ledger, 42_i64);
    assert_eq!(*ts, 1_700_000_000_000_i64);
    assert_eq!(hash.as_deref(), Some("abc123hash"));
    assert_eq!(*spike, 0_i32);
    assert_eq!(created.as_str(), "2023-11-14T22:13:20Z");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2 – insert 10 records in a single transaction, assert count=10
// ---------------------------------------------------------------------------

#[test]
fn test_insert_batch_records() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    {
        // Wrap in an explicit transaction to mimic insert_batch behaviour
        conn.execute("BEGIN", [])?;
        for i in 0..10_u64 {
            insert_record(
                &conn,
                100 + i,
                1000 + i,
                1_700_000_000_000 + i as i64 * 1000,
                None,
                false,
                "2023-11-14T22:13:20Z",
            )?;
        }
        conn.execute("COMMIT", [])?;
    }

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM fee_records", [], |row| row.get(0))?;
    assert_eq!(count, 10, "expected 10 rows after batch insert");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3 – filter by fee range
// ---------------------------------------------------------------------------

#[test]
fn test_query_by_fee_range() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    // Low-fee records: 50, 75
    insert_record(&conn, 50, 1, 1_000, None, false, "2023-01-01T00:00:00Z")?;
    insert_record(&conn, 75, 2, 2_000, None, false, "2023-01-01T00:00:01Z")?;
    // High-fee records: 500, 1000
    insert_record(&conn, 500, 3, 3_000, None, false, "2023-01-01T00:00:02Z")?;
    insert_record(&conn, 1000, 4, 4_000, None, false, "2023-01-01T00:00:03Z")?;

    // Query min_fee=100, max_fee=999 — should return only fee=500
    let mut stmt = conn.prepare(
        "SELECT fee_amount FROM fee_records
         WHERE fee_amount >= ?1 AND fee_amount <= ?2
         ORDER BY fee_amount ASC",
    )?;
    let fees: Vec<i64> = stmt
        .query_map(rusqlite::params![100_i64, 999_i64], |row| row.get(0))?
        .collect::<Result<_>>()?;

    assert_eq!(
        fees,
        vec![500_i64],
        "expected only the 500-fee record in range [100, 999]"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4 – filter by time range
// ---------------------------------------------------------------------------

#[test]
fn test_query_by_time_range() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    // Timestamps: 1000, 2000, 3000, 4000
    for (i, ts) in [1_000_i64, 2_000, 3_000, 4_000].iter().enumerate() {
        insert_record(
            &conn,
            100 + i as u64,
            i as u64 + 1,
            *ts,
            None,
            false,
            "2023-01-01T00:00:00Z",
        )?;
    }

    // Query from=1500, to=3500 — should capture ts=2000 and ts=3000 only
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms FROM fee_records
         WHERE timestamp_ms >= ?1 AND timestamp_ms <= ?2
         ORDER BY timestamp_ms ASC",
    )?;
    let timestamps: Vec<i64> = stmt
        .query_map(rusqlite::params![1_500_i64, 3_500_i64], |row| row.get(0))?
        .collect::<Result<_>>()?;

    assert_eq!(
        timestamps,
        vec![2_000_i64, 3_000_i64],
        "expected timestamps 2000 and 3000 in range [1500, 3500]"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5 – LIMIT clause returns exactly the requested number of rows
// ---------------------------------------------------------------------------

#[test]
fn test_query_limit() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    for i in 0..20_u64 {
        insert_record(
            &conn,
            100 + i,
            i + 1,
            1_000 + i as i64,
            None,
            false,
            "2023-01-01T00:00:00Z",
        )?;
    }

    let mut stmt = conn.prepare("SELECT fee_amount FROM fee_records ORDER BY id ASC LIMIT ?1")?;
    let rows: Vec<i64> = stmt
        .query_map(rusqlite::params![5_i64], |row| row.get(0))?
        .collect::<Result<_>>()?;

    assert_eq!(rows.len(), 5, "expected exactly 5 rows when LIMIT=5");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 6 – delete_before: old records removed, new records preserved
// ---------------------------------------------------------------------------

#[test]
fn test_delete_before() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    // Old records: timestamps 1000–4000 ms (well before cutoff)
    for i in 1..=4_u64 {
        insert_record(
            &conn,
            100,
            i,
            i as i64 * 1_000,
            None,
            false,
            "2020-01-01T00:00:00Z",
        )?;
    }
    // New records: timestamps 10_000_000 and 20_000_000 ms (after cutoff)
    insert_record(
        &conn,
        200,
        10,
        10_000_000,
        None,
        false,
        "2023-06-01T00:00:00Z",
    )?;
    insert_record(
        &conn,
        300,
        11,
        20_000_000,
        None,
        false,
        "2023-12-01T00:00:00Z",
    )?;

    // Cutoff at 5_000 ms — delete everything strictly before this timestamp
    let cutoff_ms: i64 = 5_000;
    let deleted = conn.execute(
        "DELETE FROM fee_records WHERE timestamp_ms < ?1",
        rusqlite::params![cutoff_ms],
    )?;

    assert_eq!(deleted, 4, "expected 4 old records to be deleted");

    let remaining: i64 =
        conn.query_row("SELECT COUNT(*) FROM fee_records", [], |row| row.get(0))?;
    assert_eq!(
        remaining, 2,
        "expected 2 new records to remain after delete_before"
    );

    // Verify the remaining records are the new ones
    let mut stmt =
        conn.prepare("SELECT timestamp_ms FROM fee_records ORDER BY timestamp_ms ASC")?;
    let timestamps: Vec<i64> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<_>>()?;
    assert_eq!(timestamps, vec![10_000_000_i64, 20_000_000_i64]);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 7 – latest: returns the record with the highest timestamp_ms
// ---------------------------------------------------------------------------

#[test]
fn test_latest() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_schema(&conn)?;

    // Insert records with increasing timestamps; the last is the "latest"
    let records: &[(u64, u64, i64)] = &[
        (100, 1, 1_000_000),
        (200, 2, 2_000_000),
        (300, 3, 3_000_000), // ← latest
    ];
    for &(fee, ledger, ts) in records {
        insert_record(&conn, fee, ledger, ts, None, false, "2023-01-01T00:00:00Z")?;
    }

    let (latest_fee, latest_ts): (i64, i64) = conn.query_row(
        "SELECT fee_amount, timestamp_ms FROM fee_records ORDER BY timestamp_ms DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    assert_eq!(
        latest_ts, 3_000_000_i64,
        "expected latest timestamp to be 3_000_000"
    );
    assert_eq!(latest_fee, 300_i64, "expected latest fee_amount to be 300");

    Ok(())
}
