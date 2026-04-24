use stellar_devkit::simulation::congestion_predictor::{CongestionLevel, CongestionPredictor};

#[test]
fn low_tx_low_fee_is_low() {
    assert_eq!(CongestionPredictor::predict(50, 100), CongestionLevel::Low);
}

#[test]
fn moderate_tx_is_moderate() {
    assert_eq!(
        CongestionPredictor::predict(300, 100),
        CongestionLevel::Moderate
    );
}

#[test]
fn moderate_fee_is_moderate() {
    assert_eq!(
        CongestionPredictor::predict(50, 400),
        CongestionLevel::Moderate
    );
}

#[test]
fn high_tx_is_high() {
    assert_eq!(
        CongestionPredictor::predict(600, 100),
        CongestionLevel::High
    );
}

#[test]
fn high_fee_is_high() {
    assert_eq!(
        CongestionPredictor::predict(50, 2_000),
        CongestionLevel::High
    );
}

#[test]
fn critical_tx_is_critical() {
    assert_eq!(
        CongestionPredictor::predict(900, 100),
        CongestionLevel::Critical
    );
}

#[test]
fn critical_fee_is_critical() {
    assert_eq!(
        CongestionPredictor::predict(50, 10_000),
        CongestionLevel::Critical
    );
}
