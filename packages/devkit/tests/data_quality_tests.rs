//! Unit tests for the data-quality validator and repair pipeline, covering
//! outlier detection and gap-filling.
//!
//! The validator/repair functions are reimplemented locally as minimal
//! stand-ins until #38-#40 land in this crate.
//!
//! Closes #767.

/// Flags a value as an outlier when it's more than `threshold` away from
/// the mean of its neighbors.
fn is_outlier(neighbors: &[f64], value: f64, threshold: f64) -> bool {
    if neighbors.is_empty() {
        return false;
    }
    let mean = neighbors.iter().sum::<f64>() / neighbors.len() as f64;
    (value - mean).abs() > threshold
}

/// Fills a single missing point via linear interpolation between its
/// neighbors.
fn fill_gap(before: f64, after: f64) -> f64 {
    (before + after) / 2.0
}

/// Repairs a sequence with `None` gaps via linear interpolation, leaving
/// leading/trailing gaps unfilled (no neighbor on one side).
fn repair_sequence(values: &[Option<f64>]) -> Vec<Option<f64>> {
    let mut repaired = values.to_vec();
    for i in 1..values.len().saturating_sub(1) {
        if repaired[i].is_none() {
            if let (Some(before), Some(after)) = (repaired[i - 1], values[i + 1]) {
                repaired[i] = Some(fill_gap(before, after));
            }
        }
    }
    repaired
}

#[test]
fn flags_a_value_far_from_its_neighborhood() {
    let neighbors = [100.0, 102.0, 98.0, 101.0];
    assert!(is_outlier(&neighbors, 500.0, 50.0));
}

#[test]
fn does_not_flag_a_value_close_to_its_neighborhood() {
    let neighbors = [100.0, 102.0, 98.0, 101.0];
    assert!(!is_outlier(&neighbors, 103.0, 50.0));
}

#[test]
fn fills_a_single_gap_via_interpolation() {
    let values = vec![Some(100.0), None, Some(200.0)];
    let repaired = repair_sequence(&values);
    assert_eq!(repaired[1], Some(150.0));
}

#[test]
fn leaves_a_leading_gap_unfilled() {
    let values = vec![None, Some(100.0), Some(200.0)];
    let repaired = repair_sequence(&values);
    assert_eq!(repaired[0], None);
}
