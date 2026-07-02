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
///
/// Use `--quiet` to suppress all output except errors (useful for scripting).
#[derive(Parser)]
#[command(name = "devkit", about = "Stellar fee tracker developer toolkit")]
pub struct Cli {
    /// Suppress all output except errors and the final result.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Replay recorded fee scenarios
    Replay,
    /// Export data to external formats
    Export {
        /// Perform a dry run — show what would be written without writing it.
        #[arg(long)]
        dry_run: bool,
    },
    /// Run performance benchmarks
    Benchmark,
    /// Serve mock fee data
    Mock,
    /// Run devkit health checks
    Health {
        /// Output results as JSON.
        #[arg(long)]
        json: bool,
        /// Path to the SQLite database to check.
        #[arg(long, default_value = "stellar_fees.db")]
        db_path: String,
        /// Run only a specific named check.
        #[arg(long)]
        check: Option<String>,
    },
    /// Print the devkit metrics report
    Metrics {
        /// Output results as JSON.
        #[arg(long)]
        json: bool,
        /// Reset all counters after displaying.
        #[arg(long)]
        reset: bool,
    },
    /// Detect and repair data quality issues in fee data
    Repair {
        /// Show what would be changed without applying any repairs.
        #[arg(long)]
        dry_run: bool,
        /// Output the repair report as JSON.
        #[arg(long)]
        json: bool,
        /// Only detect and report issues; do not plan repairs.
        #[arg(long)]
        check_only: bool,
    },
}
