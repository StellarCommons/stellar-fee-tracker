//! Unit tests for the alert history store (#641).

use chrono::{TimeZone, Utc};

use stellar_devkit::alerting::history::{AlertHistory, HistoryQuery};
use stellar_devkit::alerting::rule::{AlertEvent, AlertSeverity};

fn event(rule_id: &str, severity: AlertSeverity, ts_secs: i64, value: u64) -> AlertEvent {
    AlertEvent {
        rule_id: rule_id.to_string(),
        rule_name: rule_id.to_string(),
        severity,
        triggered_at: Utc.timestamp_opt(ts_secs, 0).unwrap(),
        current_value: value,
        threshold: 0,
        message: String::new(),
    }
}

#[test]
fn ring_buffer_caps_at_1000_and_drops_oldest() {
    let mut history = AlertHistory::new();
    for i in 0..1_500i64 {
        history.record(event(&format!("r{i}"), AlertSeverity::Info, i, i as u64));
    }
    assert_eq!(history.len(), 1_000, "capped at 1,000");

    let all = history.all();
    // The first 500 (r0..r499) must have been evicted, oldest-first.
    assert_eq!(all.first().unwrap().rule_id, "r500");
    assert_eq!(all.last().unwrap().rule_id, "r1499");
}

#[test]
fn query_filters_by_rule_id() {
    let mut history = AlertHistory::new();
    history.record(event("a", AlertSeverity::Info, 1, 1));
    history.record(event("b", AlertSeverity::Info, 2, 2));
    history.record(event("a", AlertSeverity::Warning, 3, 3));

    let a = history.by_rule("a");
    assert_eq!(a.len(), 2);
    assert!(a.iter().all(|e| e.rule_id == "a"));
}

#[test]
fn query_filters_by_severity() {
    let mut history = AlertHistory::new();
    history.record(event("a", AlertSeverity::Info, 1, 1));
    history.record(event("b", AlertSeverity::Critical, 2, 2));
    history.record(event("c", AlertSeverity::Critical, 3, 3));

    let crit = history.by_severity(AlertSeverity::Critical);
    assert_eq!(crit.len(), 2);
}

#[test]
fn query_filters_by_time_range() {
    let mut history = AlertHistory::new();
    for i in 0..10i64 {
        history.record(event(&format!("r{i}"), AlertSeverity::Info, 100 + i, 0));
    }
    let result = history.query(&HistoryQuery {
        from: Some(Utc.timestamp_opt(103, 0).unwrap()),
        to: Some(Utc.timestamp_opt(106, 0).unwrap()),
        ..Default::default()
    });
    // inclusive 103..=106 → 4 events
    assert_eq!(result.len(), 4);
}

#[test]
fn combined_filters_are_anded() {
    let mut history = AlertHistory::new();
    history.record(event("a", AlertSeverity::Critical, 10, 0));
    history.record(event("a", AlertSeverity::Info, 11, 0));
    history.record(event("b", AlertSeverity::Critical, 12, 0));

    let result = history.query(&HistoryQuery {
        rule_id: Some("a".to_string()),
        severity: Some(AlertSeverity::Critical),
        ..Default::default()
    });
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].triggered_at.timestamp(), 10);
}

#[test]
fn export_json_round_trips() {
    let mut history = AlertHistory::new();
    history.record(event("a", AlertSeverity::Warning, 1, 42));
    let json = history.export_json().unwrap();
    let parsed: Vec<AlertEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].current_value, 42);
}
