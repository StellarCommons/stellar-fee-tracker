use serde::Deserialize;
use reqwest::Client;

use crate::error::AppError;


#[derive(Clone)]
pub struct HorizonClient {
    base_url: String,
    http: Client,
}

impl HorizonClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http: Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}


#[derive(Debug, Deserialize)]
pub struct HorizonTransaction {
    pub hash: String,
    pub successful: bool,
    pub fee_charged: String,
}

#[derive(Debug, Deserialize)]
pub struct HorizonOperation {
    #[serde(rename = "type")]
    pub op_type: String,

    pub from: Option<String>,
    pub to: Option<String>,

    pub asset_type: Option<String>,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,

    pub amount: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HorizonFeeStats {
    pub last_ledger_base_fee: String,
    pub fee_charged: FeeCharged,
}

#[derive(Debug, Deserialize)]
pub struct FeeCharged {
    pub min: String,
    pub max: String,
    pub avg: String,
}


impl HorizonClient {
    pub async fn fetch_fee_stats(&self) -> Result<HorizonFeeStats, AppError> {
        let url = format!("{}/fee_stats", self.base_url);

        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|err| AppError::Network(err.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "Horizon returned HTTP {}",
                response.status()
            )));
        }

        let stats = response
            .json::<HorizonFeeStats>()
            .await
            .map_err(|err| AppError::Parse(err.to_string()))?;

        Ok(stats)
    }
}


impl HorizonClient {
    pub async fn fetch_latest_transaction(
        &self,
    ) -> Result<HorizonTransaction, AppError> {
        Err(AppError::Network(
            "fetch_latest_transaction not implemented yet".into(),
        ))
    }

    pub async fn fetch_operations(
        &self,
        _tx_hash: &str,
    ) -> Result<Vec<HorizonOperation>, AppError> {
        Err(AppError::Network(
            "fetch_operations not implemented yet".into(),
        ))
    }
}
