use crate::analysis::percentile::Percentile;

/// Classifies fee spikes in a time series.
pub struct SpikeClassifier;

impl SpikeClassifier {
    /// Returns indices of fees that fall outside 1.5 × IQR from Q1/Q3.
    /// `fees` need not be sorted; indices refer to the original slice.
    pub fn iqr_outliers(fees: &[u64]) -> Vec<usize> {
        if fees.len() < 4 {
            return vec![];
        }
        let mut sorted = fees.to_vec();
        sorted.sort_unstable();
        let q1 = Percentile::nearest_rank(&sorted, 25) as f64;
        let q3 = Percentile::nearest_rank(&sorted, 75) as f64;
        let iqr = q3 - q1;
        let lower = q1 - 1.5 * iqr;
        let upper = q3 + 1.5 * iqr;
        fees.iter()
            .enumerate()
            .filter(|(_, &v)| (v as f64) < lower || (v as f64) > upper)
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iqr_outliers_detects_spike() {
        let fees = [100u64, 102, 98, 101, 99, 500, 103, 97];
        let outliers = SpikeClassifier::iqr_outliers(&fees);
        assert!(outliers.contains(&5), "500 should be flagged");
    }

    #[test]
    fn iqr_outliers_no_outliers() {
        let fees = [10u64, 11, 12, 13, 14, 15];
        assert!(SpikeClassifier::iqr_outliers(&fees).is_empty());
    }

    #[test]
    fn iqr_outliers_too_small() {
        assert!(SpikeClassifier::iqr_outliers(&[1, 2, 3]).is_empty());
    }
}
