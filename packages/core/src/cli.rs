// CLI module placeholder
// CLI module placeholder
use clap::Parser;

/// Stellar Fee Tracker CLI arguments
#[derive(Debug, Parser)]
#[command(
    name = "stellar-fee-tracker",
    version,
    about = "Real-time insights into Stellar network transaction fees"
)]
pub struct Cli {
    /// Stellar network to use (testnet or mainnet)
    #[arg(long)]
    pub network: Option<String>,

    /// Horizon API base URL
    #[arg(long)]
    pub horizon_url: Option<String>,

    /// Fee polling interval in seconds
    #[arg(long)]
    pub poll_interval: Option<u64>,
}
