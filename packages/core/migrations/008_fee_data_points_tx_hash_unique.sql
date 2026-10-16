-- Migration 008: Unique transaction_hash for fee_data_points (Issue #911).
--
-- transaction_hash is the natural key but was neither indexed nor unique, so
-- overlapping polls could insert the same transaction repeatedly. Remove
-- existing duplicates (keeping the earliest row) and enforce uniqueness.

DELETE FROM fee_data_points
WHERE id NOT IN (
    SELECT MIN(id) FROM fee_data_points GROUP BY transaction_hash
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_fee_data_points_tx_hash
    ON fee_data_points (transaction_hash);
