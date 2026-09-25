//! Diffs two fee-value sequences and reports where they diverge beyond a
//! tolerance.
//!
//! Closes #772.

#[derive(Debug, Clone, PartialEq)]
pub struct Divergence {
    pub index: usize,
    pub left: f64,
    pub right: f64,
}

pub struct Comparator {
    tolerance: f64,
}

impl Comparator {
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// Compares sequences index-by-index up to the shorter length.
    pub fn compare(&self, left: &[f64], right: &[f64]) -> Vec<Divergence> {
        left.iter()
            .zip(right.iter())
            .enumerate()
            .filter_map(|(index, (&l, &r))| {
                if (l - r).abs() > self.tolerance {
                    Some(Divergence { index, left: l, right: r })
                } else {
                    None
                }
            })
            .collect()
    }
}
