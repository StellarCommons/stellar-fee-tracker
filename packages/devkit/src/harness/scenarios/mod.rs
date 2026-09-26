use serde::Deserialize;

use crate::types::FeeRecord;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScenarioFixture {
    pub name: String,
    pub description: String,
    pub records: Vec<FeeRecord>,
}

pub const SCENARIO_NAMES: [&str; 5] = ["baseline", "congestion", "idle", "spike", "surge"];

pub fn scenario_source(name: &str) -> Option<&'static str> {
    match name {
        "baseline" => Some(include_str!("baseline.json")),
        "congestion" => Some(include_str!("congestion.json")),
        "idle" => Some(include_str!("idle.json")),
        "spike" => Some(include_str!("spike.json")),
        "surge" => Some(include_str!("surge.json")),
        _ => None,
    }
}

impl ScenarioFixture {
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn ledger_range(&self) -> Option<(u32, u32)> {
        let first = self.records.first()?;
        let last = self.records.last()?;
        Some((first.ledger, last.ledger))
    }

    pub fn min_fee_charged(&self) -> i64 {
        self.records
            .iter()
            .map(|record| record.fee_charged)
            .min()
            .unwrap_or(0)
    }

    pub fn max_fee_charged(&self) -> i64 {
        self.records
            .iter()
            .map(|record| record.fee_charged)
            .max()
            .unwrap_or(0)
    }

    pub fn mean_fee_charged(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: i64 = self.records.iter().map(|record| record.fee_charged).sum();
        total as f64 / self.records.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> ScenarioFixture {
        let source = scenario_source(name).expect("scenario source");
        serde_json::from_str(source).expect("scenario fixture")
    }

    #[test]
    fn every_advertised_scenario_has_a_source() {
        for name in SCENARIO_NAMES {
            assert!(scenario_source(name).is_some(), "{name} has no source");
        }
    }

    #[test]
    fn an_unknown_scenario_has_no_source() {
        assert!(scenario_source("meltdown").is_none());
    }

    #[test]
    fn each_fixture_declares_its_own_name() {
        for name in SCENARIO_NAMES {
            assert_eq!(fixture(name).name, name);
        }
    }

    #[test]
    fn every_fixture_carries_records() {
        for name in SCENARIO_NAMES {
            let scenario = fixture(name);
            assert!(!scenario.records.is_empty(), "{name} has no records");
            assert!(!scenario.description.is_empty());
        }
    }

    #[test]
    fn ledgers_are_strictly_increasing() {
        for name in SCENARIO_NAMES {
            let scenario = fixture(name);
            for pair in scenario.records.windows(2) {
                assert!(pair[0].ledger < pair[1].ledger, "{name} ledgers repeat");
            }
        }
    }

    #[test]
    fn the_fee_charged_never_exceeds_the_max_fee() {
        for name in SCENARIO_NAMES {
            for record in fixture(name).records {
                assert!(record.fee_charged <= record.max_fee, "{name} overpays");
            }
        }
    }

    #[test]
    fn the_baseline_scenario_is_flat() {
        let scenario = fixture("baseline");
        assert_eq!(scenario.min_fee_charged(), 100);
        assert_eq!(scenario.max_fee_charged(), 100);
        assert_eq!(scenario.record_count(), 8);
    }

    #[test]
    fn the_spike_scenario_has_a_single_outlier() {
        let scenario = fixture("spike");
        assert_eq!(scenario.max_fee_charged(), 100000);
        let outliers = scenario
            .records
            .iter()
            .filter(|record| record.fee_charged > 1000)
            .count();
        assert_eq!(outliers, 1);
    }

    #[test]
    fn the_congestion_scenario_climbs_then_relaxes() {
        let scenario = fixture("congestion");
        assert!(scenario.mean_fee_charged() > 100.0);
        let (first, last) = scenario.ledger_range().expect("range");
        assert!(first < last);
    }

    #[test]
    fn the_idle_scenario_advertises_a_high_max_fee() {
        let scenario = fixture("idle");
        let advertised = scenario
            .records
            .iter()
            .filter(|record| record.max_fee > 1000)
            .count();
        assert_eq!(advertised, 3);
    }

    #[test]
    fn the_surge_scenario_peaks_above_its_ceiling_median() {
        let scenario = fixture("surge");
        assert!(scenario.max_fee_charged() > scenario.min_fee_charged());
        assert!(scenario.mean_fee_charged() > 100.0);
    }

    #[test]
    fn an_empty_fixture_reports_neutral_values() {
        let scenario = ScenarioFixture {
            name: "empty".to_string(),
            description: String::new(),
            records: Vec::new(),
        };
        assert_eq!(scenario.record_count(), 0);
        assert_eq!(scenario.ledger_range(), None);
        assert_eq!(scenario.min_fee_charged(), 0);
        assert_eq!(scenario.max_fee_charged(), 0);
        assert_eq!(scenario.mean_fee_charged(), 0.0);
    }
}
