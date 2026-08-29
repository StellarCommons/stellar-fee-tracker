use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// Custom deserializer for u64 that accepts either integer numbers or string-encoded numbers.
fn deserialize_u64_lenient<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64OrString {
        U64(u64),
        I64(i64),
        Str(String),
        Float(f64),
    }

    match U64OrString::deserialize(deserializer)? {
        U64OrString::U64(v) => Ok(v),
        U64OrString::I64(v) => {
            if v >= 0 {
                Ok(v as u64)
            } else {
                Err(de::Error::custom("expected non-negative integer"))
            }
        }
        U64OrString::Str(s) => s
            .trim()
            .parse::<u64>()
            .map_err(|e| de::Error::custom(format!("invalid u64 string: {e}"))),
        U64OrString::Float(f) => {
            if f >= 0.0 {
                Ok(f as u64)
            } else {
                Err(de::Error::custom("expected non-negative float for u64"))
            }
        }
    }
}

/// Custom deserializer for Option<u64> that accepts either integer numbers or string-encoded numbers.
fn deserialize_opt_u64_lenient<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OptU64OrString {
        U64(u64),
        I64(i64),
        Str(String),
        Float(f64),
        Null,
    }

    match OptU64OrString::deserialize(deserializer)? {
        OptU64OrString::U64(v) => Ok(Some(v)),
        OptU64OrString::I64(v) => {
            if v >= 0 {
                Ok(Some(v as u64))
            } else {
                Ok(None)
            }
        }
        OptU64OrString::Str(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<u64>()
                    .map(Some)
                    .map_err(|e| de::Error::custom(format!("invalid u64 string: {e}")))
            }
        }
        OptU64OrString::Float(f) => {
            if f >= 0.0 {
                Ok(Some(f as u64))
            } else {
                Ok(None)
            }
        }
        OptU64OrString::Null => Ok(None),
    }
}

/// Custom deserializer for f64 that accepts either float numbers or string-encoded numbers.
fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum F64OrString {
        F64(f64),
        I64(i64),
        U64(u64),
        Str(String),
    }

    match F64OrString::deserialize(deserializer)? {
        F64OrString::F64(v) => Ok(v),
        F64OrString::I64(v) => Ok(v as f64),
        F64OrString::U64(v) => Ok(v as f64),
        F64OrString::Str(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|e| de::Error::custom(format!("invalid f64 string: {e}"))),
    }
}

/// Statistical percentile distribution of Stellar transaction fees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeeDistribution {
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub min: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub max: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub mode: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p10: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p20: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p30: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p40: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p50: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p60: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p70: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p80: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p90: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p95: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub p99: u64,
}

/// Type alias for backward compatibility.
pub type FeeLevel = FeeDistribution;

/// Typed representation of the full Horizon `/fee_stats` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HorizonFeeStats {
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub last_ledger: u64,
    #[serde(default, deserialize_with = "deserialize_u64_lenient")]
    pub last_ledger_base_fee: u64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub ledger_capacity_usage: f64,
    #[serde(default)]
    pub fee_charged: FeeDistribution,
    #[serde(default)]
    pub max_fee: FeeDistribution,

    // Legacy/auxiliary top-level fields returned by Horizon
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub min: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub mode: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub max: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub p10: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub p20: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub p30: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub min_accepted_fee: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub max_accepted_fee: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_opt_u64_lenient")]
    pub transaction_count_estimate: Option<u64>,
}
