//! Unit tests for the fee data anonymiser (`toolchain::anonymiser`).
//! Kept standalone until the `toolchain` module is wired into the crate.

use super::anonymiser::{anonymise, AnonymiseMode, FeeRecord};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<FeeRecord> {
        vec![FeeRecord {
            ledger_sequence: 1,
            transaction_hash: Some("abc123".into()),
            fee: 100.0,
        }]
    }

    #[test]
    fn remove_mode_clears_transaction_hash() {
        let out = anonymise(&sample(), AnonymiseMode::Remove);
        assert!(out[0].transaction_hash.is_none());
    }

    #[test]
    fn hash_mode_replaces_transaction_hash() {
        let out = anonymise(&sample(), AnonymiseMode::Hash);
        assert_ne!(out[0].transaction_hash, Some("abc123".to_string()));
    }

    #[test]
    fn fee_amount_is_unchanged() {
        let out = anonymise(&sample(), AnonymiseMode::Remove);
        assert_eq!(out[0].fee, 100.0);
    }
}
