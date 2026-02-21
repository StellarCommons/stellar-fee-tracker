use axum::{Json, extract::Extension};
use serde::Serialize;

use crate::services::horizon::HorizonClient;
use crate::error::AppError;


#[derive(Serialize)]
pub struct CurrentFeeResponse {
    pub base_fee: String,
    pub min_fee: String,
    pub max_fee: String,
    pub avg_fee: String,
}


pub async fn current_fees(
    Extension(horizon_client): Extension<HorizonClient>,
) -> Result<Json<CurrentFeeResponse>, AppError> {
    let stats = horizon_client.fetch_fee_stats().await?;

    Ok(Json(CurrentFeeResponse {
        base_fee: stats.last_ledger_base_fee,
        min_fee: stats.fee_charged.min,
        max_fee: stats.fee_charged.max,
        avg_fee: stats.fee_charged.avg,
    }))
}