//! Validates a loaded DevkitConfig before use: non-empty Horizon URL and a
//! positive polling interval.
//!
//! Closes #762.

#[derive(Debug, Clone, PartialEq)]
pub struct DevkitConfig {
    pub horizon_url: String,
    pub polling_interval_secs: u64,
}

pub fn validate(config: &DevkitConfig) -> Result<(), String> {
    if config.horizon_url.trim().is_empty() {
        return Err("horizon_url must not be empty".to_string());
    }
    if config.polling_interval_secs == 0 {
        return Err("polling_interval_secs must be positive".to_string());
    }
    Ok(())
}
