//! Lenient fee_stats parsing: missing/malformed optional fields -> None.
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct LenientFeeStats {
    pub last_ledger: Option<u32>,
    pub p50_accepted_fee: Option<String>,
}

pub fn parse_lenient(raw_json: &str) -> LenientFeeStats {
    let value: Value = match serde_json::from_str(raw_json) {
        Ok(v) => v,
        Err(_) => return LenientFeeStats::default(),
    };

    LenientFeeStats {
        last_ledger: value.get("last_ledger").and_then(Value::as_u64).map(|v| v as u32),
        p50_accepted_fee: value
            .get("p50_accepted_fee")
            .and_then(Value::as_str)
            .map(String::from),
    }
}
