//! Gap filling and duplicate removal for fee data.
//!
//! Provides detect → plan → apply pipeline used by the `repair` subcommand.

use crate::simulation::fee_model::FeePoint;
use std::collections::BTreeSet;

/// Issues that can be detected in fee data.
#[derive(Debug, Clone, PartialEq)]
pub enum DataIssue {
    DuplicateLedger(u64),
    OutOfOrder { ledger: u64, expected: u64, actual: u64 },
    ZeroFee(u64),
    Outlier { ledger: u64, fee: u64, z_score: f64 },
    LedgerGap { from: u64, to: u64, gap_size: u64 },
}

impl std::fmt::Display for DataIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateLedger(l) => write!(f, "duplicate ledger {l}"),
            Self::OutOfOrder { ledger, expected, actual } => {
                write!(f, "out-of-order at ledger {ledger}: expected ts>={expected}, got {actual}")
            }
            Self::ZeroFee(l) => write!(f, "zero fee at ledger {l}"),
            Self::Outlier { ledger, fee, z_score } => {
                write!(f, "outlier at ledger {ledger}: fee={fee}, z={z_score:.2}")
            }
            Self::LedgerGap { from, to, gap_size } => {
                write!(f, "ledger gap {from} -> {to} ({gap_size} missing)")
            }
        }
    }
}

/// Repair actions that can be applied.
#[derive(Debug, Clone)]
pub enum RepairAction {
    RemoveDuplicate(u64),
    Reorder,
    FillZeroFee { ledger: u64, replacement: u64 },
    CapOutlier { ledger: u64, original: u64, capped: u64 },
    InterpolateGap { from: u64, to: u64, count: u64 },
}

/// Identifies and repairs common issues in fee data.
pub struct Repair;

impl Repair {
    /// Detect all issues in a slice of fee points.
    pub fn detect(points: &[FeePoint]) -> Vec<DataIssue> {
        let mut issues = Vec::new();
        let mut seen = BTreeSet::new();
        let mut prev_ts: Option<u64> = None;
        let mut prev_ledger: Option<u64> = None;

        let mean = mean_fee(points);
        let std_dev = std_dev_fee(points);

        for p in points {
            if p.fee == 0 {
                issues.push(DataIssue::ZeroFee(p.ledger));
            }
            if !seen.insert(p.ledger) {
                issues.push(DataIssue::DuplicateLedger(p.ledger));
            }
            if let Some(ts) = prev_ts {
                if p.timestamp < ts {
                    issues.push(DataIssue::OutOfOrder {
                        ledger: p.ledger,
                        expected: ts,
                        actual: p.timestamp,
                    });
                }
            }
            if let Some(pl) = prev_ledger {
                if p.ledger > pl + 1 {
                    issues.push(DataIssue::LedgerGap {
                        from: pl,
                        to: p.ledger,
                        gap_size: p.ledger - pl - 1,
                    });
                }
            }
            if std_dev > 0.0 {
                let z = (p.fee as f64 - mean) / std_dev;
                if z.abs() > 3.0 {
                    issues.push(DataIssue::Outlier { ledger: p.ledger, fee: p.fee, z_score: z });
                }
            }
            prev_ts = Some(p.timestamp);
            prev_ledger = Some(p.ledger);
        }
        issues
    }

    /// Generate repair actions for detected issues.
    pub fn plan(points: &[FeePoint]) -> Vec<RepairAction> {
        let issues = Self::detect(points);
        let mean = mean_fee(points);
        let mut actions = Vec::new();

        for issue in &issues {
            match issue {
                DataIssue::DuplicateLedger(l) => {
                    actions.push(RepairAction::RemoveDuplicate(*l));
                }
                DataIssue::OutOfOrder { .. } => {
                    if !actions.iter().any(|a| matches!(a, RepairAction::Reorder)) {
                        actions.push(RepairAction::Reorder);
                    }
                }
                DataIssue::ZeroFee(ledger) => {
                    let neighbors: Vec<u64> = points
                        .iter()
                        .filter(|p| p.ledger != *ledger && p.fee > 0)
                        .map(|p| p.fee)
                        .collect();
                    let replacement = if neighbors.is_empty() {
                        100
                    } else {
                        neighbors.iter().sum::<u64>() / neighbors.len() as u64
                    };
                    actions.push(RepairAction::FillZeroFee { ledger: *ledger, replacement });
                }
                DataIssue::Outlier { ledger, fee, .. } => {
                    actions.push(RepairAction::CapOutlier {
                        ledger: *ledger,
                        original: *fee,
                        capped: (mean * 2.0) as u64,
                    });
                }
                DataIssue::LedgerGap { from, to, gap_size } => {
                    actions.push(RepairAction::InterpolateGap { from: *from, to: *to, count: *gap_size });
                }
            }
        }
        actions
    }

    /// Apply repairs and return (cleaned_points, actions_taken).
    /// When `dry_run` is true the original points are returned unchanged.
    pub fn apply(points: &[FeePoint], dry_run: bool) -> (Vec<FeePoint>, Vec<RepairAction>) {
        let actions = Self::plan(points);
        if dry_run {
            return (points.to_vec(), actions);
        }

        let mut cleaned: Vec<FeePoint> = points.to_vec();
        let mut seen = BTreeSet::new();

        // Remove duplicates (keep first occurrence)
        cleaned.retain(|p| {
            if seen.contains(&p.ledger) { return false; }
            seen.insert(p.ledger);
            true
        });

        // Sort by (timestamp, ledger)
        cleaned.sort_by_key(|p| (p.timestamp, p.ledger));

        let mean = mean_fee(points);

        for action in &actions {
            match action {
                RepairAction::FillZeroFee { ledger, replacement } => {
                    if let Some(pt) = cleaned.iter_mut().find(|p| p.ledger == *ledger) {
                        pt.fee = *replacement;
                    }
                }
                RepairAction::CapOutlier { ledger, capped, .. } => {
                    if let Some(pt) = cleaned.iter_mut().find(|p| p.ledger == *ledger) {
                        if pt.fee > *capped {
                            pt.fee = *capped;
                        }
                    }
                }
                RepairAction::InterpolateGap { from, to, count } => {
                    let fee_from = cleaned.iter().find(|p| p.ledger == *from).map(|p| p.fee).unwrap_or(mean as u64);
                    let fee_to   = cleaned.iter().find(|p| p.ledger == *to).map(|p| p.fee).unwrap_or(mean as u64);
                    let ts_from  = cleaned.iter().find(|p| p.ledger == *from).map(|p| p.timestamp).unwrap_or(0);
                    let ts_to    = cleaned.iter().find(|p| p.ledger == *to).map(|p| p.timestamp).unwrap_or(0);
                    let steps = *count + 1;
                    for i in 1..=*count {
                        let frac = i as f64 / steps as f64;
                        let interp_fee = (fee_from as f64 + frac * (fee_to as f64 - fee_from as f64)).round() as u64;
                        let interp_ts  = (ts_from  as f64 + frac * (ts_to  as f64 - ts_from  as f64)).round() as u64;
                        cleaned.push(FeePoint {
                            timestamp: interp_ts,
                            fee: interp_fee,
                            ledger: from + i,
                            is_spike: false,
                        });
                    }
                    cleaned.sort_by_key(|p| (p.timestamp, p.ledger));
                }
                _ => {}
            }
        }

        (cleaned, actions)
    }

    /// Quality score [0.0, 1.0]. 1.0 = no issues.
    pub fn quality_score(points: &[FeePoint]) -> f64 {
        if points.is_empty() { return 0.0; }
        let issues = Self::detect(points);
        (1.0_f64 - issues.len() as f64 * 0.1).max(0.0)
    }
}

fn mean_fee(points: &[FeePoint]) -> f64 {
    if points.is_empty() { return 0.0; }
    points.iter().map(|p| p.fee as f64).sum::<f64>() / points.len() as f64
}

fn std_dev_fee(points: &[FeePoint]) -> f64 {
    if points.is_empty() { return 0.0; }
    let mean = mean_fee(points);
    let var = points.iter().map(|p| (p.fee as f64 - mean).powi(2)).sum::<f64>() / points.len() as f64;
    var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::fee_model::FeePoint;

    fn clean() -> Vec<FeePoint> {
        vec![
            FeePoint { timestamp: 0,   fee: 100, ledger: 1, is_spike: false },
            FeePoint { timestamp: 100, fee: 110, ledger: 2, is_spike: false },
            FeePoint { timestamp: 200, fee: 105, ledger: 3, is_spike: false },
        ]
    }

    fn dirty() -> Vec<FeePoint> {
        vec![
            FeePoint { timestamp: 0,   fee: 100, ledger: 1, is_spike: false },
            FeePoint { timestamp: 50,  fee: 0,   ledger: 1, is_spike: false }, // dup + zero
            FeePoint { timestamp: 200, fee: 110, ledger: 3, is_spike: false },
            FeePoint { timestamp: 150, fee: 150, ledger: 2, is_spike: false }, // out-of-order
        ]
    }

    #[test]
    fn clean_data_no_issues() {
        assert_eq!(Repair::detect(&clean()).len(), 0);
    }

    #[test]
    fn dirty_data_finds_issues() {
        let issues = Repair::detect(&dirty());
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| matches!(i, DataIssue::ZeroFee(_))));
        assert!(issues.iter().any(|i| matches!(i, DataIssue::DuplicateLedger(_))));
    }

    #[test]
    fn plan_generates_actions() {
        assert!(!Repair::plan(&dirty()).is_empty());
    }

    #[test]
    fn apply_removes_duplicates() {
        let (cleaned, _) = Repair::apply(&dirty(), false);
        let ledgers: Vec<u64> = cleaned.iter().map(|p| p.ledger).collect();
        let unique: BTreeSet<u64> = ledgers.clone().into_iter().collect();
        assert_eq!(ledgers.len(), unique.len());
    }

    #[test]
    fn quality_score_one_for_clean() {
        assert!((Repair::quality_score(&clean()) - 1.0).abs() < 0.001);
    }

    #[test]
    fn quality_score_reduced_for_dirty() {
        assert!(Repair::quality_score(&dirty()) < 1.0);
    }

    #[test]
    fn quality_score_zero_for_empty() {
        assert!((Repair::quality_score(&[])).abs() < f64::EPSILON);
    }

    #[test]
    fn dry_run_returns_original() {
        let (out, _) = Repair::apply(&dirty(), true);
        assert_eq!(out.len(), dirty().len());
    }

    #[test]
    fn gap_detected() {
        let data = vec![
            FeePoint { timestamp: 0, fee: 100, ledger: 1, is_spike: false },
            FeePoint { timestamp: 200, fee: 110, ledger: 5, is_spike: false },
        ];
        let issues = Repair::detect(&data);
        assert!(issues.iter().any(|i| matches!(i, DataIssue::LedgerGap { .. })));
    }
}
