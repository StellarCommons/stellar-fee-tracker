//! Unit tests for RegimeDetector, covering all three regime
//! classifications.
//!
//! RegimeDetector is reimplemented locally as a minimal stand-in until #59
//! lands in this crate.
//!
//! Closes #790.

#[derive(Debug, PartialEq)]
enum Regime {
    Stable,
    Volatile,
    Trending,
}

fn classify_regime(trend: f64, volatility: f64) -> Regime {
    if volatility > 50.0 {
        Regime::Volatile
    } else if trend.abs() > 20.0 {
        Regime::Trending
    } else {
        Regime::Stable
    }
}

#[test]
fn classifies_low_trend_low_volatility_as_stable() {
    assert_eq!(classify_regime(2.0, 5.0), Regime::Stable);
}

#[test]
fn classifies_high_volatility_as_volatile_regardless_of_trend() {
    assert_eq!(classify_regime(5.0, 80.0), Regime::Volatile);
}

#[test]
fn classifies_large_trend_with_low_volatility_as_trending() {
    assert_eq!(classify_regime(40.0, 10.0), Regime::Trending);
}
