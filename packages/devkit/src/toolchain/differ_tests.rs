//! Unit tests for the fee sequence differ (`toolchain::differ`).
//! Kept standalone until the `toolchain` module is wired into the crate.

use super::differ::{diff_sequences, FeeRecord};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_sequences_produce_empty_diff() {
        let seq = vec![FeeRecord { ledger_sequence: 1, fee: 100.0 }];
        let result = diff_sequences(&seq, &seq);
        assert!(result.added.is_empty() && result.removed.is_empty() && result.changed.is_empty());
    }

    #[test]
    fn added_and_removed_records_detected() {
        let before = vec![FeeRecord { ledger_sequence: 1, fee: 100.0 }];
        let after = vec![FeeRecord { ledger_sequence: 2, fee: 100.0 }];
        let result = diff_sequences(&before, &after);
        assert_eq!(result.added, vec![2]);
        assert_eq!(result.removed, vec![1]);
    }

    #[test]
    fn changed_fee_detected() {
        let before = vec![FeeRecord { ledger_sequence: 1, fee: 100.0 }];
        let after = vec![FeeRecord { ledger_sequence: 1, fee: 200.0 }];
        let result = diff_sequences(&before, &after);
        assert_eq!(result.changed, vec![(1, 100.0, 200.0)]);
    }
}
