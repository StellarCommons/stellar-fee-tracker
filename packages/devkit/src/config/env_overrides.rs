//! Environment-variable overrides for the TOML-loaded DevkitConfig. Env
//! vars take precedence over file values when present.
//!
//! Closes #761.

#[derive(Debug, Clone, PartialEq)]
pub struct DevkitConfig {
    pub horizon_url: String,
    pub polling_interval_secs: u64,
}

const ENV_HORIZON_URL: &str = "DEVKIT_HORIZON_URL";
const ENV_POLLING_INTERVAL: &str = "DEVKIT_POLLING_INTERVAL_SECS";

/// Applies overrides from `env`, a key/value lookup (in production this is
/// sourced from `std::env::vars()`; it's passed in here as a map so this
/// is testable without mutating the real process environment).
pub fn apply_env_overrides(
    mut config: DevkitConfig,
    env: &std::collections::HashMap<String, String>,
) -> DevkitConfig {
    if let Some(url) = env.get(ENV_HORIZON_URL) {
        config.horizon_url = url.clone();
    }
    if let Some(interval) = env.get(ENV_POLLING_INTERVAL).and_then(|v| v.parse().ok()) {
        config.polling_interval_secs = interval;
    }
    config
}
