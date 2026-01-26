use std::env;

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct Config {
    pub stellar_network: StellarNetwork,
    pub horizon_url: String,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone)]
pub enum StellarNetwork {
    Testnet,
    Mainnet,
}

impl Config {
    /// Build configuration from CLI flags and environment variables.
    pub fn from_sources(cli: &Cli) -> Result<Self, String> {
        // -------- Network --------
        let network_raw = cli
            .network
            .clone()
            .or_else(|| env::var("STELLAR_NETWORK").ok())
            .ok_or("STELLAR_NETWORK is required")?;

        let stellar_network = match network_raw.as_str() {
            "testnet" => StellarNetwork::Testnet,
            "mainnet" => StellarNetwork::Mainnet,
            other => return Err(format!("Invalid STELLAR_NETWORK: {}", other)),
        };

        // -------- Horizon URL --------
        let horizon_url = cli
            .horizon_url
            .clone()
            .or_else(|| env::var("HORIZON_URL").ok())
            .ok_or("HORIZON_URL is required")?;

        // -------- Poll Interval --------
        let poll_interval_seconds = cli
            .poll_interval
            .or_else(|| env::var("POLL_INTERVAL_SECONDS").ok()?.parse().ok())
            .ok_or("POLL_INTERVAL_SECONDS is required and must be a number")?;

        Ok(Self {
            stellar_network,
            horizon_url,
            poll_interval_seconds,
        })
    }
}