//! Predicts network congestion based on simulated load and fee models.

#[derive(Debug, PartialEq)]
pub enum CongestionLevel {
    Low,
    Moderate,
    High,
    Critical,
}

/// Predicts network congestion based on simulated load and fee models.
pub struct CongestionPredictor;

impl CongestionPredictor {
    /// Classify congestion given `tx_count` transactions and `fee` in stroops.
    pub fn predict(tx_count: u64, fee: u64) -> CongestionLevel {
        match (tx_count, fee) {
            (t, f) if t >= 800 || f >= 5_000 => CongestionLevel::Critical,
            (t, f) if t >= 500 || f >= 1_000 => CongestionLevel::High,
            (t, f) if t >= 200 || f >= 300 => CongestionLevel::Moderate,
            _ => CongestionLevel::Low,
        }
    }
}
