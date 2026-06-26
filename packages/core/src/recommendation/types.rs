"use chrono::{DateTime, Utc};\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub enum Urgency {\n    Low,\n    Medium,\n    High,\n    Urgent,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendRequest {\n    pub target_ledgers: Option<u32>,\n    pub urgency: Option<Urgency>,\n    pub max_fee: Option<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FeeAlternative {\n    pub fee: String,\n    pub estimated_wait_ledgers: u32,\n    pub confidence: f64,\n    pub label: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendResponse {\n    pub recommended_fee: String,\n    pub fee_in_stroops: u64,\n    pub estimated_wait_ledgers: u32,\n    pub confidence: f64,\n    pub network_condition: String,\n    pub alternatives: Vec<FeeAlternative>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendHistoryEntry {\n    pub id: i64,\n    pub requested_at: DateTime<Utc>,\n    pub target_ledgers: u32,\n    pub urgency: String,\n    pub recommended_fee: u64,\n    pub actual_confirmed: Option<bool>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendHistoryResponse {\n    pub entries: Vec<RecommendHistoryEntry>,\n}\n"
#[derive(Debug, Clone)]
pub struct RecommendationConfig {
    pub default_confidence: f64,
    pub default_ledgers: u8,
    pub history_window_secs: u64,
    pub cache_ttl_secs: u64,
    pub min_sample_count: usize,
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            default_confidence: 0.95,
            default_ledgers: 2,
            history_window_secs: 3600,
            cache_ttl_secs: 10,
            min_sample_count: 50,
        }
    }
}

use chrono::{DateTime, Utc};\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub enum Urgency {\n    Low,\n    Medium,\n    High,\n    Urgent,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendRequest {\n    pub target_ledgers: Option<u32>,\n    pub urgency: Option<Urgency>,\n    pub max_fee: Option<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FeeAlternative {\n    pub fee: String,\n    pub estimated_wait_ledgers: u32,\n    pub confidence: f64,\n    pub label: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendResponse {\n    pub recommended_fee: String,\n    pub fee_in_stroops: u64,\n    pub estimated_wait_ledgers: u32,\n    pub confidence: f64,\n    pub network_condition: String,\n    pub alternatives: Vec<FeeAlternative>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendHistoryEntry {\n    pub id: i64,\n    pub requested_at: DateTime<Utc>,\n    pub target_ledgers: u32,\n    pub urgency: String,\n    pub recommended_fee: u64,\n    pub actual_confirmed: Option<bool>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RecommendHistoryResponse {\n    pub entries: Vec<RecommendHistoryEntry>,\n}\n
