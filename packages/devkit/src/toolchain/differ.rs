use std::collections::HashMap;

pub struct FeeRecord {
    pub ledger_sequence: u64,
    pub fee: f64,
}

pub struct DiffResult {
    pub added: Vec<u64>,
    pub removed: Vec<u64>,
    pub changed: Vec<(u64, f64, f64)>,
    pub total_delta: f64,
}

/// Compares two fee sequences by `ledger_sequence`, producing a
/// human-readable diff summary of additions, removals, and changes.
pub fn diff_sequences(before: &[FeeRecord], after: &[FeeRecord]) -> DiffResult {
    let before_map: HashMap<_, _> = before.iter().map(|r| (r.ledger_sequence, r.fee)).collect();
    let after_map: HashMap<_, _> = after.iter().map(|r| (r.ledger_sequence, r.fee)).collect();

    let mut result = DiffResult { added: vec![], removed: vec![], changed: vec![], total_delta: 0.0 };
    for (seq, fee) in &after_map {
        match before_map.get(seq) {
            None => result.added.push(*seq),
            Some(old_fee) if (old_fee - fee).abs() > f64::EPSILON => {
                result.changed.push((*seq, *old_fee, *fee));
                result.total_delta += fee - old_fee;
            }
            _ => {}
        }
    }
    for seq in before_map.keys() {
        if !after_map.contains_key(seq) {
            result.removed.push(*seq);
        }
    }
    result
}
