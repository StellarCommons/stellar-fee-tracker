use super::fee_stats::HorizonFeeStats;
use crate::error::DevkitError;
use serde_json::Value;

/// Parse raw Horizon /fee_stats JSON into HorizonFeeStats.
pub fn parse_fee_stats(json: &str) -> Result<HorizonFeeStats, DevkitError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| DevkitError::Protocol(format!("JSON parse error: {e}")))?;

    // If json is empty object or invalid, check required fields
    if !value.is_object() || value.as_object().map_or(true, |o| o.is_empty()) {
        return Err(DevkitError::Protocol("empty JSON object".to_string()));
    }

    if value.get("last_ledger_base_fee").is_none() {
        return Err(DevkitError::Protocol(
            "missing required field last_ledger_base_fee".to_string(),
        ));
    }

    let stats: HorizonFeeStats = serde_json::from_value(value)
        .map_err(|e| DevkitError::Protocol(format!("field deserialization error: {e}")))?;

    validate_fee_stats(&stats)?;

    Ok(stats)
}

/// Validate parsed HorizonFeeStats against known Stellar network constraints.
pub fn validate_fee_stats(stats: &HorizonFeeStats) -> Result<(), DevkitError> {
    if stats.last_ledger_base_fee < 100 {
        return Err(DevkitError::Protocol(format!(
            "last_ledger_base_fee must be >= 100, got {}",
            stats.last_ledger_base_fee
        )));
    }

    if !(0.0..=1.0).contains(&stats.ledger_capacity_usage) {
        return Err(DevkitError::Protocol(format!(
            "ledger_capacity_usage must be in [0.0, 1.0], got {}",
            stats.ledger_capacity_usage
        )));
    }

    let fl = &stats.fee_charged;
    if fl.p10 > 0 || fl.p50 > 0 || fl.p90 > 0 || fl.p99 > 0 {
        if !(fl.p10 <= fl.p50 && fl.p50 <= fl.p90 && fl.p90 <= fl.p99) {
            return Err(DevkitError::Protocol(format!(
                "fee_charged percentiles must be monotonic: p10={} p50={} p90={} p99={}",
                fl.p10, fl.p50, fl.p90, fl.p99
            )));
        }
    }
    if fl.min > 0 || fl.mode > 0 || fl.max > 0 {
        if !(fl.min <= fl.mode && fl.mode <= fl.max) {
            return Err(DevkitError::Protocol(format!(
                "fee_charged min/mode/max must be ordered: min={} mode={} max={}",
                fl.min, fl.mode, fl.max
            )));
        }
    }

    Ok(())
}
