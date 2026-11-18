//! In-memory Storage implementation for tests and local runs.
use crate::storage::{FeeRecordLike, QueryParams, Storage};

#[derive(Default)]
pub struct MemoryStorage {
    records: Vec<FeeRecordLike>,
}

impl Storage for MemoryStorage {
    fn insert(&mut self, record: FeeRecordLike) -> Result<(), String> {
        self.records.push(record);
        Ok(())
    }

    fn query(&self, params: QueryParams) -> Vec<FeeRecordLike> {
        self.records
            .iter()
            .filter(|r| params.from_ledger.is_none_or(|f| r.ledger >= f))
            .filter(|r| params.to_ledger.is_none_or(|t| r.ledger <= t))
            .map(|r| FeeRecordLike { ledger: r.ledger, fee_charged: r.fee_charged })
            .take(params.limit.unwrap_or(usize::MAX))
            .collect()
    }
}
