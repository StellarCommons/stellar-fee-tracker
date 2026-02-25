use std::env;

use crate::cli::Cli;

#[derive(Debug, Clone)]
pub struct Config {
    pub stellar_network: StellarNetwork,
    pub horizon_url: String,
    pub poll_interval_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub api_port: u16,
    pub allowed_origins: Vec<String>,
    pub retry_attempts: u32,
    pub base_retry_delay_ms: u64,
    pub database_url: String,
    pub storage_retention_days: u64,
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
        Self::from_sources_with_env(cli, &std::collections::HashMap::new())
    }

    /// Like `from_sources` but allows injecting env var overrides — used in tests
    /// to avoid mutating the real process environment.
    #[cfg(test)]
    pub fn from_sources_with_overrides(
        cli: &Cli,
        overrides: &std::collections::HashMap<&str, &str>,
    ) -> Result<Self, String> {
        Self::from_sources_with_env(cli, &overrides.iter().map(|(k, v)| (*k, *v)).collect())
    }

    fn from_sources_with_env(
        cli: &Cli,
        overrides: &std::collections::HashMap<&str, &str>,
    ) -> Result<Self, String> {
        let get = |key: &str| -> Option<String> {
            overrides
                .get(key)
                .map(|v| v.to_string())
                .or_else(|| env::var(key).ok())
        };

        // -------- Network --------
        let network_raw = cli
            .network
            .clone()
            .or_else(|| get("STELLAR_NETWORK"))
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
            .or_else(|| get("HORIZON_URL"))
            .unwrap_or_else(|| stellar_network.default_horizon_url().to_string());

        // -------- Poll Interval --------
        let poll_interval_seconds = cli
            .poll_interval
            .or_else(|| get("POLL_INTERVAL_SECONDS")?.parse().ok())
            .ok_or("POLL_INTERVAL_SECONDS is required and must be a number")?;

        // -------- API Port --------
        let api_port = get("API_PORT")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        // -------- Cache TTL --------
        let cache_ttl_seconds = get("CACHE_TTL_SECONDS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5);

        // -------- Allowed Origins --------
        let allowed_origins = get("ALLOWED_ORIGINS")
            .unwrap_or_else(|| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // -------- Retry config --------
        let retry_attempts = get("RETRY_ATTEMPTS")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);

        let base_retry_delay_ms = get("BASE_RETRY_DELAY_MS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1000);

        // -------- Database URL --------
        let database_url = get("DATABASE_URL")
            .unwrap_or_else(|| "sqlite://stellar_fees.db".to_string());

        // -------- Storage retention --------
        let storage_retention_days = get("STORAGE_RETENTION_DAYS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(7);

        Ok(Self {
            stellar_network,
            horizon_url,
            poll_interval_seconds,
            cache_ttl_seconds,
            api_port,
            allowed_origins,
            retry_attempts,
            base_retry_delay_ms,
            database_url,
            storage_retention_days,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_cli(network: &str, horizon_url: Option<&str>) -> Cli {
        Cli {
            network: Some(network.to_string()),
            horizon_url: horizon_url.map(str::to_string),
            poll_interval: Some(30),
        }
    }

    fn no_env<'a>() -> HashMap<&'a str, &'a str> {
        HashMap::new()
    }

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

    // ---- Config::from_sources_with_overrides ----

    #[test]
    fn testnet_without_horizon_url_uses_default() {
        let cli = make_cli("testnet", None);
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.horizon_url, "https://horizon-testnet.stellar.org");
    }

    #[test]
    fn mainnet_without_horizon_url_uses_default() {
        let cli = make_cli("mainnet", None);
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.horizon_url, "https://horizon.stellar.org");
    }

    #[test]
    fn explicit_horizon_url_overrides_default() {
        let custom = "https://my-private-horizon.example.com";
        let cli = make_cli("testnet", Some(custom));
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.horizon_url, custom);
    }

    #[test]
    fn invalid_network_returns_error() {
        let cli = make_cli("devnet", None);
        let result = Config::from_sources_with_overrides(&cli, &no_env());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid STELLAR_NETWORK"));
    }

    #[test]
    fn api_port_defaults_to_8080() {
        let cli = make_cli("testnet", None);
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.api_port, 8080);
    }

    #[test]
    fn cache_ttl_defaults_to_five_seconds() {
        let cli = make_cli("testnet", None);
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.cache_ttl_seconds, 5);
    }

    #[test]
    fn cache_ttl_uses_env_override() {
        let cli = make_cli("testnet", None);
        let env = HashMap::from([("CACHE_TTL_SECONDS", "12")]);
        let config = Config::from_sources_with_overrides(&cli, &env).unwrap();
        assert_eq!(config.cache_ttl_seconds, 12);
    }

    #[test]
    fn allowed_origins_defaults_to_localhost_3000() {
        let cli = make_cli("testnet", None);
        let config = Config::from_sources_with_overrides(&cli, &no_env()).unwrap();
        assert_eq!(config.allowed_origins, vec!["http://localhost:3000"]);
    }

    #[test]
    fn allowed_origins_parses_comma_separated_list() {
        let cli = make_cli("testnet", None);
        let env = HashMap::from([
            ("ALLOWED_ORIGINS", "http://localhost:3000,https://app.example.com"),
        ]);
        let config = Config::from_sources_with_overrides(&cli, &env).unwrap();
        assert_eq!(
            config.allowed_origins,
            vec!["http://localhost:3000", "https://app.example.com"]
        );
    }

    #[test]
    fn allowed_origins_trims_whitespace() {
        let cli = make_cli("testnet", None);
        let env = HashMap::from([
            ("ALLOWED_ORIGINS", "http://localhost:3000 , https://app.example.com "),
        ]);
        let config = Config::from_sources_with_overrides(&cli, &env).unwrap();
        assert_eq!(
            config.allowed_origins,
            vec!["http://localhost:3000", "https://app.example.com"]
        );
    }
}
