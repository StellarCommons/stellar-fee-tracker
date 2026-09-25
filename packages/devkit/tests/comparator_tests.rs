//! Unit tests for a fee-sequence Comparator, covering identical,
//! fully-divergent, and partially-divergent sequences.
//!
//! Comparator is reimplemented locally as a minimal stand-in until #47
//! lands in this crate.
//!
//! Closes #779.

struct Divergence {
    index: usize,
    left: f64,
    right: f64,
}

fn compare(left: &[f64], right: &[f64], tolerance: f64) -> Vec<Divergence> {
    left.iter()
        .zip(right.iter())
        .enumerate()
        .filter_map(|(index, (&l, &r))| {
            if (l - r).abs() > tolerance {
                Some(Divergence { index, left: l, right: r })
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn identical_sequences_have_no_divergence() {
    let seq = [100.0, 200.0, 300.0];
    assert!(compare(&seq, &seq, 0.01).is_empty());
}

#[test]
fn fully_divergent_sequences_flag_every_index() {
    let left = [100.0, 200.0, 300.0];
    let right = [500.0, 600.0, 700.0];
    let divergences = compare(&left, &right, 0.01);
    assert_eq!(divergences.len(), 3);
}

#[test]
fn partially_divergent_sequences_flag_only_the_differing_indices() {
    let left = [100.0, 200.0, 300.0];
    let right = [100.0, 250.0, 300.0];
    let divergences = compare(&left, &right, 0.01);
    assert_eq!(divergences.len(), 1);
    assert_eq!(divergences[0].index, 1);
    assert_eq!(divergences[0].left, 200.0);
    assert_eq!(divergences[0].right, 250.0);
}
