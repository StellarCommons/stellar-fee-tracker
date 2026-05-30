pub mod benchmark;
pub mod export;
pub mod replay;

/// Arguments for the `mock` subcommand.
pub struct MockArgs {
    /// Scenario to load (e.g. "normal", "congested", "spike").
    pub scenario: String,
    /// Port the mock Horizon server listens on.
    pub port: u16,
}

impl Default for MockArgs {
    fn default() -> Self {
        Self {
            scenario: String::from("normal"),
            port: 8090,
        }
    }
}

impl MockArgs {
    /// Start the mock Horizon server with the configured scenario.
    pub fn run(&self) {
        println!(
            "Starting mock Horizon server on :{} with scenario '{}'",
            self.port, self.scenario
        );
    }
use clap::{Parser, Subcommand};

/// Developer toolkit for the Stellar fee tracker.
#[derive(Parser)]
#[command(name = "devkit", about = "Stellar fee tracker developer toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Replay recorded fee scenarios
    Replay,
    /// Export data to external formats
    Export,
    /// Run performance benchmarks
    Benchmark,
    /// Serve mock fee data
    Mock,
}
