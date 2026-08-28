//! Unit tests for the alert cooldown tracker (#640).

use std::time::{Duration, Instant};

use stellar_devkit::alerting::evaluator::CooldownTracker;

#[test]
fn second_alert_within_window_is_suppressed() {
    let ct = CooldownTracker::new();
    let t0 = Instant::now();

    assert!(ct.should_fire_at("r", 300, t0), "first fire allowed");
    ct.record_at("r", t0);
    assert!(
        !ct.should_fire_at("r", 300, t0 + Duration::from_secs(120)),
        "within the cooldown window must be suppressed"
    );
}

#[test]
fn alert_fires_after_cooldown_expires() {
    let ct = CooldownTracker::new();
    let t0 = Instant::now();
    ct.record_at("r", t0);

    assert!(!ct.should_fire_at("r", 300, t0 + Duration::from_secs(299)));
    // At exactly the boundary (elapsed == cooldown) the rule may fire again.
    assert!(ct.should_fire_at("r", 300, t0 + Duration::from_secs(300)));
    assert!(ct.should_fire_at("r", 300, t0 + Duration::from_secs(301)));
}

#[test]
fn reset_cooldown_allows_immediate_fire() {
    let ct = CooldownTracker::new();
    let t0 = Instant::now();
    ct.record_at("r", t0);
    assert!(!ct.should_fire_at("r", 300, t0));

    ct.reset_cooldown("r");
    assert!(ct.should_fire_at("r", 300, t0), "reset clears the cooldown");
}

#[test]
fn unknown_rule_is_always_allowed() {
    let ct = CooldownTracker::new();
    assert!(ct.should_fire("never-recorded", 300));
}

#[test]
fn cooldowns_are_tracked_per_rule() {
    let ct = CooldownTracker::new();
    let t0 = Instant::now();
    ct.record_at("a", t0);

    assert!(!ct.should_fire_at("a", 300, t0), "a is on cooldown");
    assert!(
        ct.should_fire_at("b", 300, t0),
        "b has an independent cooldown"
    );
}
