//! Fills small timestamp gaps in a fee-value time series via linear
//! interpolation.
//!
//! Closes #764.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimestampedValue {
    pub unix_time: u64,
    pub value: f64,
}

pub struct RepairPipeline {
    /// Gaps up to this many seconds are interpolated; larger gaps are left
    /// as-is rather than guessed at.
    max_gap_secs: u64,
}

impl RepairPipeline {
    pub fn new(max_gap_secs: u64) -> Self {
        Self { max_gap_secs }
    }

    /// Interpolates a missing point directly between two known points when
    /// their gap is small enough, returning `None` (leaving the series
    /// untouched) otherwise. `series` must be sorted by `unix_time`
    /// ascending.
    pub fn repair(
        &self,
        series: &[TimestampedValue],
        missing_at: u64,
    ) -> Option<TimestampedValue> {
        let before = series.iter().rev().find(|p| p.unix_time < missing_at)?;
        let after = series.iter().find(|p| p.unix_time > missing_at)?;

        let gap = after.unix_time.saturating_sub(before.unix_time);
        if gap == 0 || gap > self.max_gap_secs {
            return None;
        }

        let fraction = (missing_at - before.unix_time) as f64 / gap as f64;
        let value = before.value + (after.value - before.value) * fraction;
        Some(TimestampedValue { unix_time: missing_at, value })
    }
}
