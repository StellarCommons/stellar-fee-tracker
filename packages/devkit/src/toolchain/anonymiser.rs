pub struct FeeRecord {
    pub ledger_sequence: u64,
    pub transaction_hash: Option<String>,
    pub fee: f64,
}

/// Anonymisation strategy for the `transaction_hash` field.
pub enum AnonymiseMode {
    Remove,
    Hash,
}

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{hash:x}")
}

/// Removes or hashes `transaction_hash` fields so records can be shared
/// safely, while leaving fee amounts untouched.
pub fn anonymise(records: &[FeeRecord], mode: AnonymiseMode) -> Vec<FeeRecord> {
    records
        .iter()
        .map(|r| FeeRecord {
            ledger_sequence: r.ledger_sequence,
            fee: r.fee,
            transaction_hash: match (&mode, &r.transaction_hash) {
                (AnonymiseMode::Remove, _) => None,
                (AnonymiseMode::Hash, Some(h)) => Some(simple_hash(h)),
                (AnonymiseMode::Hash, None) => None,
            },
        })
        .collect()
}
