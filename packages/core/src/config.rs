use std::env;

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct Config {
    pub stellar_network: StellarNetwork,
    pub horizon_url: String,
    pub poll_interval_seconds: u64,
    pub api_port: u16,
}

#[derive(Debug, Clone)]
pub enum StellarNetwork {
    Testnet,
    Mainnet,
}

impl StellarNetwork {
    /// Returns the well-known public Horizon URL for this network.
    /// Used as the default when `HORIZON_URL` is not explicitly configured.
    pub fn default_horizon_url(&self) -> &'static str {
        match self {
            StellarNetwork::Testnet => "https://horizon-testnet.stellar.org",
            StellarNetwork::Mainnet => "https://horizon.stellar.org",
        }
    }
}

impl Config {
    /// Build configuration from CLI flags and environment variables.
    ///
    /// `HORIZON_URL` is optional — when omitted it defaults to the well-known
    /// public Horizon endpoint for the selected `STELLAR_NETWORK`.
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
        // Explicit config takes priority; falls back to network default.
        let horizon_url = cli
            .horizon_url
            .clone()
            .or_else(|| env::var("HORIZON_URL").ok())
            .unwrap_or_else(|| stellar_network.default_horizon_url().to_string());

        // -------- Poll Interval --------
        let poll_interval_seconds = cli
            .poll_interval
            .or_else(|| env::var("POLL_INTERVAL_SECONDS").ok()?.parse().ok())
            .ok_or("POLL_INTERVAL_SECONDS is required and must be a number")?;

        // -------- API Port --------
        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        Ok(Self {
            stellar_network,
            horizon_url,
            poll_interval_seconds,
            api_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- StellarNetwork::default_horizon_url ----

    #[test]
    fn testnet_defaults_to_testnet_horizon() {
        assert_eq!(
            StellarNetwork::Testnet.default_horizon_url(),
            "https://horizon-testnet.stellar.org"
        );
    }

    #[test]
    fn mainnet_defaults_to_mainnet_horizon() {
        assert_eq!(
            StellarNetwork::Mainnet.default_horizon_url(),
            "https://horizon.stellar.org"
        );
    }

    // ---- Config::from_sources URL resolution ----

    fn make_cli(network: &str, horizon_url: Option<&str>) -> Cli {
        Cli {
            network: Some(network.to_string()),
            horizon_url: horizon_url.map(str::to_string),
            poll_interval: Some(30),
        }
    }

    #[test]
    fn testnet_without_horizon_url_uses_default() {
        // HORIZON_URL env var must not be set for this test to be meaningful.
        // We use the CLI-only path (no env fallback) by providing all values via CLI.
        let cli = make_cli("testnet", None);
        // Temporarily clear env var to avoid interference
        let _guard = env::var("HORIZON_URL").ok();
        unsafe { env::remove_var("HORIZON_URL"); }

        let config = Config::from_sources(&cli).unwrap();
        assert_eq!(config.horizon_url, "https://horizon-testnet.stellar.org");
    }

    #[test]
    fn mainnet_without_horizon_url_uses_default() {
        let cli = make_cli("mainnet", None);
        unsafe { env::remove_var("HORIZON_URL"); }

        let config = Config::from_sources(&cli).unwrap();
        assert_eq!(config.horizon_url, "https://horizon.stellar.org");
    }

    #[test]
    fn explicit_horizon_url_overrides_default() {
        let custom = "https://my-private-horizon.example.com";
        let cli = make_cli("testnet", Some(custom));

        let config = Config::from_sources(&cli).unwrap();
        assert_eq!(config.horizon_url, custom);
    }

    #[test]
    fn invalid_network_returns_error() {
        let cli = make_cli("devnet", None);
        let result = Config::from_sources(&cli);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid STELLAR_NETWORK"));
    }

    #[test]
    fn api_port_defaults_to_8080() {
        let cli = make_cli("testnet", None);
        unsafe { env::remove_var("API_PORT"); }
        let config = Config::from_sources(&cli).unwrap();
        assert_eq!(config.api_port, 8080);
    }
}