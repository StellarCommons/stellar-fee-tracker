//! Estimates current ledger sequence from a known ledger + avg close time.
const AVG_CLOSE_TIME_SECS: u64 = 5;

pub struct LedgerEstimator {
    known_sequence: u32,
    known_unix_time: u64,
}

impl LedgerEstimator {
    pub fn new(known_sequence: u32, known_unix_time: u64) -> Self {
        Self { known_sequence, known_unix_time }
    }

    pub fn estimate_at(&self, now_unix_time: u64) -> u32 {
        let elapsed = now_unix_time.saturating_sub(self.known_unix_time);
        let ledgers_passed = u32::try_from(elapsed / AVG_CLOSE_TIME_SECS).unwrap_or(u32::MAX);
        self.known_sequence.saturating_add(ledgers_passed)
    }
}
