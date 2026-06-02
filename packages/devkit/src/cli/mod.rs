pub mod benchmark;
pub mod export;
pub mod replay;

use clap::{Parser, Subcommand};

/// Arguments for the `simulate` subcommand.
pub struct SimulateArgs {
    /// Base fee floor in stroops.
    pub base_fee: u64,
    /// Probability of a fee spike on any given ledger [0.0, 1.0].
    pub spike_prob: f64,
    /// Number of ledgers to simulate.
    pub duration: u64,
}

impl Default for SimulateArgs {
    fn default() -> Self {
        Self {
            base_fee: 100,
            spike_prob: 0.05,
            duration: 1_000,
        }
    }
}

impl SimulateArgs {
    /// Run the fee simulation and stream results to stdout.
    pub fn run(&self) {
        println!(
            "Simulating {} ledgers | base_fee={} stroops | spike_prob={:.2}",
            self.duration, self.base_fee, self.spike_prob
        );
    }
}

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
}

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
