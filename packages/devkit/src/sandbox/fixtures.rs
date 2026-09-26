use crate::harness::scenarios::ScenarioFixture;
use crate::types::FeeRecord;

pub fn flat(ledgers: usize, start: u32, fee_charged: i64, max_fee: i64) -> Vec<FeeRecord> {
    (0..ledgers as u32)
        .map(|offset| FeeRecord {
            ledger: start + offset,
            fee_charged,
            max_fee,
        })
        .collect()
}

pub fn ramp(
    ledgers: usize,
    start: u32,
    from_fee: i64,
    to_fee: i64,
    max_fee: i64,
) -> Vec<FeeRecord> {
    if ledgers == 0 {
        return Vec::new();
    }
    let steps = (ledgers - 1) as i64;
    (0..ledgers as u32)
        .map(|offset| {
            let progress = (offset as i64).min(steps);
            let step = (to_fee - from_fee) as f64 / steps as f64;
            FeeRecord {
                ledger: start + offset,
                fee_charged: from_fee + (step * progress as f64).round() as i64,
                max_fee,
            }
        })
        .collect()
}

pub fn spikes(
    ledgers: usize,
    start: u32,
    base_fee: i64,
    spike_fee: i64,
    every: usize,
) -> Vec<FeeRecord> {
    let period = every.max(1);
    (0..ledgers as u32)
        .map(|offset| FeeRecord {
            ledger: start + offset,
            fee_charged: if (offset as usize).is_multiple_of(period) {
                spike_fee
            } else {
                base_fee
            },
            max_fee: spike_fee.max(base_fee),
        })
        .collect()
}

pub fn random_walk(
    ledgers: usize,
    start: u32,
    base_fee: i64,
    spread: i64,
    seed: u64,
) -> Vec<FeeRecord> {
    if seed == 0 {
        return flat(ledgers, start, base_fee, base_fee);
    }
    let mut rng = seed;
    let mut state = base_fee;
    (0..ledgers as u32)
        .map(|offset| {
            let noise = (next_unit(&mut rng) * 2.0 - 1.0) * spread as f64;
            state = (state as f64 + noise).round() as i64;
            state = state.clamp(base_fee - spread.abs(), base_fee + spread.abs());
            FeeRecord {
                ledger: start + offset,
                fee_charged: state,
                max_fee: state,
            }
        })
        .collect()
}

pub fn burst(
    ledgers: usize,
    start: u32,
    base_fee: i64,
    burst_fee: i64,
    from: usize,
    to: usize,
) -> Vec<FeeRecord> {
    let base = flat(ledgers, start, base_fee, base_fee);
    base.into_iter()
        .enumerate()
        .map(|(offset, record)| {
            if offset >= from && offset < to {
                FeeRecord {
                    fee_charged: burst_fee,
                    max_fee: burst_fee,
                    ..record
                }
            } else {
                record
            }
        })
        .collect()
}

pub fn fixture(name: &str, description: &str, records: Vec<FeeRecord>) -> ScenarioFixture {
    ScenarioFixture {
        name: name.to_string(),
        description: description.to_string(),
        records,
    }
}

pub fn fees(records: &[FeeRecord]) -> Vec<i64> {
    records.iter().map(|record| record.fee_charged).collect()
}

pub fn mean_fee(records: &[FeeRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    fees(records).iter().sum::<i64>() as f64 / records.len() as f64
}

pub fn peak_fee(records: &[FeeRecord]) -> i64 {
    records
        .iter()
        .map(|record| record.fee_charged)
        .max()
        .unwrap_or(0)
}

fn next_unit(rng: &mut u64) -> f64 {
    let mut x = *rng;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *rng = x;
    let value = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (value >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_series_holds_one_fee() {
        let records = flat(4, 10, 100, 100);
        assert_eq!(records.len(), 4);
        assert!(records.iter().all(|record| record.fee_charged == 100));
        assert_eq!(records[0].ledger, 10);
        assert_eq!(records[3].ledger, 13);
    }

    #[test]
    fn a_flat_series_of_nothing_is_empty() {
        assert!(flat(0, 10, 100, 100).is_empty());
    }

    #[test]
    fn a_ramp_climbs_between_its_endpoints() {
        let records = ramp(5, 0, 100, 500, 1000);
        assert_eq!(records.len(), 5);
        assert_eq!(records[0].fee_charged, 100);
        assert_eq!(records[4].fee_charged, 500);
        assert!(records[3].fee_charged > records[2].fee_charged);
    }

    #[test]
    fn a_ramp_of_one_ledger_lands_on_the_start() {
        assert_eq!(ramp(1, 0, 100, 500, 1000)[0].fee_charged, 100);
    }

    #[test]
    fn a_downward_ramp_falls() {
        let records = ramp(5, 0, 500, 100, 1000);
        assert!(records[0].fee_charged > records[4].fee_charged);
    }

    #[test]
    fn an_empty_ramp_is_empty() {
        assert!(ramp(0, 0, 100, 500, 1000).is_empty());
    }

    #[test]
    fn spikes_recur_on_the_period() {
        let records = spikes(6, 0, 100, 900, 2);
        assert_eq!(fees(&records), vec![900, 100, 900, 100, 900, 100]);
    }

    #[test]
    fn a_spike_period_of_zero_is_treated_as_one() {
        let records = spikes(3, 0, 100, 900, 0);
        assert!(records.iter().all(|record| record.fee_charged == 900));
    }

    #[test]
    fn a_burst_raises_only_its_window() {
        let records = burst(6, 0, 100, 5000, 2, 4);
        assert_eq!(fees(&records), vec![100, 100, 5000, 5000, 100, 100]);
    }

    #[test]
    fn a_burst_outside_the_range_changes_nothing() {
        let records = burst(4, 0, 100, 5000, 9, 12);
        assert!(records.iter().all(|record| record.fee_charged == 100));
    }

    #[test]
    fn a_random_walk_is_deterministic() {
        assert_eq!(
            random_walk(20, 0, 100, 10, 5),
            random_walk(20, 0, 100, 10, 5)
        );
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(
            random_walk(20, 0, 100, 10, 5),
            random_walk(20, 0, 100, 10, 6)
        );
    }

    #[test]
    fn a_zero_seed_falls_back_to_a_flat_series() {
        assert_eq!(random_walk(3, 0, 100, 10, 0), flat(3, 0, 100, 100));
    }

    #[test]
    fn a_random_walk_stays_within_its_spread() {
        for record in random_walk(200, 0, 100, 25, 9) {
            assert!(
                (75..=125).contains(&record.fee_charged),
                "{} escaped",
                record.fee_charged
            );
        }
    }

    #[test]
    fn a_zero_spread_walk_is_flat() {
        assert!(random_walk(5, 0, 100, 0, 3)
            .iter()
            .all(|record| record.fee_charged == 100));
    }

    #[test]
    fn generated_records_never_exceed_their_max_fee() {
        let series = [
            flat(4, 0, 100, 100),
            ramp(4, 0, 100, 500, 1000),
            spikes(4, 0, 100, 900, 2),
            burst(4, 0, 100, 5000, 1, 3),
            random_walk(4, 0, 100, 25, 4),
        ];
        for records in series {
            for record in records {
                assert!(record.fee_charged <= record.max_fee);
            }
        }
    }

    #[test]
    fn generated_ledgers_are_consecutive() {
        for record in burst(5, 700, 100, 900, 1, 2) {
            assert!((700..705).contains(&record.ledger));
        }
    }

    #[test]
    fn a_fixture_wraps_generated_records() {
        let scenario = fixture("generated", "from code", flat(3, 1, 100, 100));
        assert_eq!(scenario.name, "generated");
        assert_eq!(scenario.record_count(), 3);
        assert_eq!(scenario.min_fee_charged(), 100);
    }

    #[test]
    fn the_summaries_report_the_generated_shape() {
        let records = spikes(6, 0, 100, 900, 2);
        assert_eq!(peak_fee(&records), 900);
        assert_eq!(mean_fee(&records), 500.0);
        assert_eq!(fees(&records).len(), 6);
    }

    #[test]
    fn the_summaries_of_nothing_are_zero() {
        assert_eq!(peak_fee(&[]), 0);
        assert_eq!(mean_fee(&[]), 0.0);
        assert!(fees(&[]).is_empty());
    }
}
