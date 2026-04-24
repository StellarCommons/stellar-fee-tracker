/// Computes percentile statistics over fee samples.
pub struct Percentile;

impl Percentile {
    /// Returns the nearest-rank percentile of a sorted slice.
    /// `p` must be in 1..=100. Returns 0 for empty slices.
    pub fn nearest_rank(sorted: &[u64], p: usize) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let idx = ((p as f64 / 100.0) * sorted.len() as f64).ceil() as usize;
        sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
    }

    /// Returns the linear-interpolation percentile of a sorted slice.
    /// `p` must be in 0..=100. Returns 0 for empty slices.
    pub fn linear_interpolation(sorted: &[u64], p: usize) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        if sorted.len() == 1 {
            return sorted[0];
        }
        let rank = (p as f64 / 100.0) * (sorted.len() - 1) as f64;
        let lo = rank.floor() as usize;
        let hi = rank.ceil() as usize;
        let frac = rank - lo as f64;
        (sorted[lo] as f64 + frac * (sorted[hi] as f64 - sorted[lo] as f64)).round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_basic() {
        let data = [10, 20, 30, 40, 50];
        assert_eq!(Percentile::nearest_rank(&data, 50), 30);
        assert_eq!(Percentile::nearest_rank(&data, 100), 50);
        assert_eq!(Percentile::nearest_rank(&data, 1), 10);
    }

    #[test]
    fn nearest_rank_empty() {
        assert_eq!(Percentile::nearest_rank(&[], 50), 0);
    }

    #[test]
    fn linear_interpolation_basic() {
        let data = [10, 20, 30, 40, 50];
        assert_eq!(Percentile::linear_interpolation(&data, 0), 10);
        assert_eq!(Percentile::linear_interpolation(&data, 100), 50);
        assert_eq!(Percentile::linear_interpolation(&data, 50), 30);
    }

    #[test]
    fn linear_interpolation_empty() {
        assert_eq!(Percentile::linear_interpolation(&[], 50), 0);
    }
}
