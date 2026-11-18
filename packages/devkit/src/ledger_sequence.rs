//! Ledger-sequence newtype for bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LedgerSequence(u32);

impl LedgerSequence {
    pub fn new(seq: u32) -> Self {
        Self(seq)
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

impl std::ops::Add<u32> for LedgerSequence {
    type Output = Self;
    fn add(self, rhs: u32) -> Self {
        Self(self.0.saturating_add(rhs))
    }
}
