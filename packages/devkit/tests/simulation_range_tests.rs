use stellar_devkit::simulation::fee_model::{FeeModel, FeeModelConfig};
use stellar_devkit::simulation::network_load::{mean, NetworkLoad, NetworkLoadConfig};

const SAMPLES: usize = 500;

fn fee_config(seed: u64) -> FeeModelConfig {
    FeeModelConfig {
        base_fee: 100.0,
        volatility: 0.35,
        trend_per_step: 0.1,
        min_fee: 40.0,
        max_fee: 400.0,
        seed,
    }
}

fn load_config(seed: u64) -> NetworkLoadConfig {
    NetworkLoadConfig {
        min_load: 0.05,
        max_load: 0.95,
        correlation: 0.85,
        noise: 0.1,
        seed,
    }
}

#[test]
fn fee_model_samples_stay_inside_the_configured_fee_range() {
    for seed in [1, 2, 3, 4, 5] {
        let mut model = FeeModel::new(fee_config(seed)).expect("fee model");
        for fee in model.generate(SAMPLES) {
            assert!(
                (40.0..=400.0).contains(&fee),
                "seed {seed} produced {fee} outside 40..=400"
            );
        }
    }
}

#[test]
fn network_load_samples_stay_inside_the_configured_load_range() {
    let mut model = FeeModel::new(fee_config(9)).expect("fee model");
    let mut load = NetworkLoad::new(load_config(9)).expect("network load");
    for fee in model.generate(SAMPLES) {
        let value = load.from_fee(fee, 40.0, 400.0);
        assert!(
            (0.05..=0.95).contains(&value),
            "load {value} outside 0.05..=0.95 for fee {fee}"
        );
    }
}

#[test]
fn saturated_fees_saturate_the_load_band() {
    let mut flat = fee_config(11);
    flat.volatility = 0.0;
    flat.trend_per_step = 50.0;
    let mut model = FeeModel::new(flat).expect("fee model");
    let mut load = NetworkLoad::new(load_config(11)).expect("network load");
    let values = load.generate(&model.generate(SAMPLES), 40.0, 400.0);
    assert!(values.iter().all(|value| *value <= 0.95));
    assert!(mean(&values) > 0.5, "mean was {}", mean(&values));
}

#[test]
fn a_trending_fee_series_drives_the_load_series_upward() {
    let mut trending = fee_config(13);
    trending.volatility = 0.0;
    trending.trend_per_step = 0.5;
    let mut model = FeeModel::new(trending).expect("fee model");
    let mut load = NetworkLoad::new(load_config(13)).expect("network load");
    let fees = model.generate(SAMPLES);
    let loads = load.generate(&fees, 40.0, 400.0);
    let (early, late) = loads.split_at(fees.len() / 2);
    assert!(
        mean(late) > mean(early),
        "load did not follow the fee trend"
    );
}

#[test]
fn the_two_simulators_stay_in_range_across_extreme_configurations() {
    let cases = [
        (0.0, 0.0, 40.0, 400.0),
        (2.0, -25.0, 40.0, 400.0),
        (0.0, 25.0, 40.0, 400.0),
        (1.0, 0.0, 100.0, 100.0),
    ];
    for (index, (volatility, trend, min_fee, max_fee)) in cases.into_iter().enumerate() {
        let config = FeeModelConfig {
            base_fee: (min_fee + max_fee) / 2.0,
            volatility,
            trend_per_step: trend,
            min_fee,
            max_fee,
            seed: 20 + index as u64,
        };
        let mut model = FeeModel::new(config).expect("fee model");
        let mut load = NetworkLoad::new(load_config(30 + index as u64)).expect("network load");
        for fee in model.generate(200) {
            assert!((min_fee..=max_fee).contains(&fee), "fee {fee} escaped");
            let value = load.from_fee(fee, min_fee, max_fee);
            assert!((0.05..=0.95).contains(&value), "load {value} escaped");
        }
    }
}
