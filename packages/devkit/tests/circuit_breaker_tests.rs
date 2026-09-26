use std::time::{Duration, Instant};

use stellar_devkit::resilience::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

fn breaker(failure_threshold: u32, success_threshold: u32, cooldown: Duration) -> CircuitBreaker {
    CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold,
        success_threshold,
        cooldown,
    })
}

#[test]
fn a_breaker_starts_closed_and_admits_traffic() {
    let mut breaker = breaker(3, 2, Duration::from_secs(1));
    assert_eq!(breaker.state(), CircuitState::Closed);
    for _ in 0..10 {
        assert!(breaker.is_call_permitted());
        breaker.record_success();
    }
    assert_eq!(breaker.state(), CircuitState::Closed);
}

#[test]
fn a_sustained_failure_run_opens_the_circuit() {
    let mut breaker = breaker(4, 2, Duration::from_secs(1));
    for _ in 0..3 {
        breaker.record_failure();
        assert!(breaker.is_call_permitted());
    }
    assert_eq!(breaker.state(), CircuitState::Closed);
    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Open);
}

#[test]
fn an_open_circuit_blocks_traffic_and_counts_rejections() {
    let mut breaker = breaker(1, 1, Duration::from_secs(30));
    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Open);
    for _ in 0..5 {
        assert!(!breaker.is_call_permitted());
    }
    assert_eq!(breaker.rejected_calls(), 5);
}

#[test]
fn the_circuit_half_opens_once_the_cooldown_elapses() {
    let mut breaker = breaker(1, 1, Duration::from_millis(500));
    let start = Instant::now();
    breaker.record_failure_at(start);
    assert!(!breaker.is_call_permitted_at(start + Duration::from_millis(499)));
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(500)));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
}

#[test]
fn a_half_open_circuit_closes_after_the_success_threshold() {
    let mut breaker = breaker(1, 3, Duration::from_millis(100));
    let start = Instant::now();
    breaker.record_failure_at(start);
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(100)));
    breaker.record_success_at(start + Duration::from_millis(110));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    breaker.record_success_at(start + Duration::from_millis(120));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    breaker.record_success_at(start + Duration::from_millis(130));
    assert_eq!(breaker.state(), CircuitState::Closed);
}

#[test]
fn a_single_probe_failure_reopens_the_circuit() {
    let mut breaker = breaker(1, 3, Duration::from_millis(100));
    let start = Instant::now();
    breaker.record_failure_at(start);
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(100)));
    breaker.record_success_at(start + Duration::from_millis(110));
    breaker.record_failure_at(start + Duration::from_millis(120));
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(!breaker.is_call_permitted_at(start + Duration::from_millis(219)));
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(220)));
}

#[test]
fn every_transition_is_recorded_in_order() {
    let mut breaker = breaker(1, 1, Duration::from_millis(100));
    let start = Instant::now();
    breaker.record_failure_at(start);
    let opened = breaker.last_transition().expect("opened");
    assert_eq!(
        (opened.from, opened.to),
        (CircuitState::Closed, CircuitState::Open)
    );
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(100)));
    let half_open = breaker.last_transition().expect("half open");
    assert_eq!(
        (half_open.from, half_open.to),
        (CircuitState::Open, CircuitState::HalfOpen)
    );
    breaker.record_success_at(start + Duration::from_millis(110));
    let closed = breaker.last_transition().expect("closed");
    assert_eq!(
        (closed.from, closed.to),
        (CircuitState::HalfOpen, CircuitState::Closed)
    );
}

#[test]
fn an_intermittent_failure_never_opens_the_circuit() {
    let mut breaker = breaker(3, 2, Duration::from_secs(1));
    for _ in 0..50 {
        breaker.record_failure();
        breaker.record_success();
    }
    assert_eq!(breaker.state(), CircuitState::Closed);
    assert_eq!(breaker.rejected_calls(), 0);
}

#[test]
fn a_recovered_circuit_opens_again_on_a_new_failure_run() {
    let mut breaker = breaker(2, 1, Duration::from_millis(100));
    let start = Instant::now();
    breaker.record_failure_at(start);
    breaker.record_failure_at(start);
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(breaker.is_call_permitted_at(start + Duration::from_millis(100)));
    breaker.record_success_at(start + Duration::from_millis(110));
    assert_eq!(breaker.state(), CircuitState::Closed);
    breaker.record_failure_at(start + Duration::from_millis(120));
    breaker.record_failure_at(start + Duration::from_millis(130));
    assert_eq!(breaker.state(), CircuitState::Open);
}

#[test]
fn the_default_breaker_tolerates_five_failures() {
    let mut breaker = CircuitBreaker::default();
    for _ in 0..5 {
        breaker.record_failure();
    }
    assert_eq!(breaker.state(), CircuitState::Open);
    breaker.reset();
    assert_eq!(breaker.state(), CircuitState::Closed);
}
