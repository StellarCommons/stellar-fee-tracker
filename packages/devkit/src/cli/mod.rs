pub mod benchmark;
pub mod completions;
pub mod config;
pub mod export;
pub mod replay;
pub mod version;

use clap::{Parser, Subcommand};

pub use completions::{CompletionsArgs, ShellType};
pub use config::ConfigArgs;
pub use version::VersionArgs;

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

// ---------------------------------------------------------------------------
// #403: Piped-input helpers for validate and inspect subcommands
// ---------------------------------------------------------------------------

/// Input source for subcommands that accept fee data (file or stdin pipe).
#[derive(Debug, Clone)]
pub enum InputSource {
    /// Read from a file at the given path.
    File(std::path::PathBuf),
    /// Read from stdin (piped input).
    Stdin,
}

impl Default for InputSource {
    fn default() -> Self {
        Self::Stdin
    }
}

impl InputSource {
    /// Detect whether data is being piped into the process.
    ///
    /// Returns `true` when stdin is not a terminal (i.e., it has been redirected
    /// or data is being piped into the process).
    pub fn is_piped() -> bool {
        // atty is not a dependency; use a simple heuristic: check if
        // stdin has data by testing the fd type via std libc metadata.
        // Safer portable approach: fall back to checking an env hint.
        // In practice callers pass `InputSource::Stdin` explicitly when piping.
        !std::io::IsTerminal::is_terminal(&std::io::stdin())
    }

    /// Load fee points using the CSV reader.
    pub fn load_csv(
        &self,
    ) -> Result<crate::utilities::csv_reader::CsvReadResult, std::io::Error> {
        match self {
            Self::File(path) => crate::utilities::csv_reader::read_csv_file(path),
            Self::Stdin => Ok(crate::utilities::csv_reader::read_csv_stdin()),
        }
    }

    /// Load fee points using the JSON reader.
    pub fn load_json(
        &self,
    ) -> Result<Vec<crate::simulation::fee_model::FeePoint>, crate::utilities::json_reader::JsonReadError>
    {
        match self {
            Self::File(path) => crate::utilities::json_reader::read_json_file(path),
            Self::Stdin => crate::utilities::json_reader::read_json_stdin(),
        }
    }
}

/// Arguments for the `validate` subcommand.
///
/// Supports piped input: `cat fees.csv | devkit validate --format csv`
pub struct ValidateArgs {
    /// Input source (file path or stdin).
    pub input: InputSource,
    /// Format of the input data: "csv" or "json".
    pub format: String,
    /// Output format for the report: "text" or "json".
    pub output_format: String,
}

impl Default for ValidateArgs {
    fn default() -> Self {
        Self {
            input: InputSource::Stdin,
            format: "csv".to_string(),
            output_format: "text".to_string(),
        }
    }
}

impl ValidateArgs {
    /// Run the validate subcommand, printing a data quality report.
    pub fn run(&self) {
        let points = match self.format.as_str() {
            "csv" => {
                match self.input.load_csv() {
                    Ok(result) => {
                        if result.skipped > 0 {
                            eprintln!("Warning: {} malformed rows were skipped", result.skipped);
                        }
                        result.points
                    }
                    Err(e) => {
                        eprintln!("Error reading CSV: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "json" => match self.input.load_json() {
                Ok(pts) => pts,
                Err(e) => {
                    eprintln!("Error reading JSON: {}", e);
                    std::process::exit(1);
                }
            },
            fmt => {
                eprintln!("Unsupported format: {}. Use 'csv' or 'json'.", fmt);
                std::process::exit(1);
            }
        };

        if points.is_empty() {
            eprintln!("No fee records found in input.");
            std::process::exit(1);
        }

        let report = DataQualityReport::from_points(&points);
        if self.output_format == "json" {
            println!("{}", report.to_json());
        } else {
            println!("{}", report.display());
        }
    }
}

/// Arguments for the `inspect` subcommand.
///
/// Supports piped input: `cat fees.csv | devkit inspect --format csv`
pub struct InspectArgs {
    /// Input source (file path or stdin).
    pub input: InputSource,
    /// Format of the input data: "csv" or "json".
    pub format: String,
    /// Output format: "text" or "json".
    pub output_format: String,
}

impl Default for InspectArgs {
    fn default() -> Self {
        Self {
            input: InputSource::Stdin,
            format: "csv".to_string(),
            output_format: "text".to_string(),
        }
    }
}

impl InspectArgs {
    /// Run the inspect subcommand, printing a summary of the fee data.
    pub fn run(&self) {
        let points = match self.format.as_str() {
            "csv" => match self.input.load_csv() {
                Ok(result) => {
                    if result.skipped > 0 {
                        eprintln!("Warning: {} malformed rows were skipped", result.skipped);
                    }
                    result.points
                }
                Err(e) => {
                    eprintln!("Error reading CSV: {}", e);
                    std::process::exit(1);
                }
            },
            "json" => match self.input.load_json() {
                Ok(pts) => pts,
                Err(e) => {
                    eprintln!("Error reading JSON: {}", e);
                    std::process::exit(1);
                }
            },
            fmt => {
                eprintln!("Unsupported format: {}. Use 'csv' or 'json'.", fmt);
                std::process::exit(1);
            }
        };

        if points.is_empty() {
            eprintln!("No fee records found in input.");
            std::process::exit(1);
        }

        let summary = FeeSummary::from_points(&points);
        if self.output_format == "json" {
            println!("{}", summary.to_json());
        } else {
            println!("{}", summary.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Embedded report types (used by validate and inspect)
// ---------------------------------------------------------------------------

use crate::simulation::fee_model::FeePoint;

/// A basic data quality report for fee data.
pub struct DataQualityReport {
    pub total: usize,
    pub spike_count: usize,
    pub min_fee: u64,
    pub max_fee: u64,
    pub mean_fee: f64,
}

impl DataQualityReport {
    pub fn from_points(points: &[FeePoint]) -> Self {
        let total = points.len();
        let spike_count = points.iter().filter(|p| p.is_spike).count();
        let fees: Vec<u64> = points.iter().map(|p| p.fee).collect();
        let min_fee = *fees.iter().min().unwrap_or(&0);
        let max_fee = *fees.iter().max().unwrap_or(&0);
        let mean_fee = fees.iter().sum::<u64>() as f64 / total as f64;
        Self {
            total,
            spike_count,
            min_fee,
            max_fee,
            mean_fee,
        }
    }

    pub fn display(&self) -> String {
        format!(
            "Data Quality Report\n\
             ===================\n\
             total records: {}\n\
             spike count:   {}\n\
             min fee:       {} stroops\n\
             max fee:       {} stroops\n\
             mean fee:      {:.2} stroops",
            self.total, self.spike_count, self.min_fee, self.max_fee, self.mean_fee,
        )
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"total":{},"spike_count":{},"min_fee":{},"max_fee":{},"mean_fee":{:.2}}}"#,
            self.total, self.spike_count, self.min_fee, self.max_fee, self.mean_fee,
        )
    }
}

/// Statistical summary for a set of fee points (used by inspect).
pub struct FeeSummary {
    pub count: usize,
    pub min: u64,
    pub max: u64,
    pub mean: f64,
    pub median: u64,
    pub spike_count: usize,
    pub p25: u64,
    pub p75: u64,
    pub p95: u64,
    pub p99: u64,
}

impl FeeSummary {
    pub fn from_points(points: &[FeePoint]) -> Self {
        if points.is_empty() {
            return Self {
                count: 0,
                min: 0,
                max: 0,
                mean: 0.0,
                median: 0,
                spike_count: 0,
                p25: 0,
                p75: 0,
                p95: 0,
                p99: 0,
            };
        }
        let mut fees: Vec<u64> = points.iter().map(|p| p.fee).collect();
        fees.sort_unstable();
        let count = fees.len();
        let min = fees[0];
        let max = fees[count - 1];
        let mean = fees.iter().sum::<u64>() as f64 / count as f64;
        let median = fees[count / 2];
        let spike_count = points.iter().filter(|p| p.is_spike).count();
        let p = |pct: f64| fees[((count as f64 * pct) as usize).min(count - 1)];
        Self {
            count,
            min,
            max,
            mean,
            median,
            spike_count,
            p25: p(0.25),
            p75: p(0.75),
            p95: p(0.95),
            p99: p(0.99),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "Fee Summary\n\
             ===========\n\
             count:       {}\n\
             min:         {} stroops\n\
             max:         {} stroops\n\
             mean:        {:.2} stroops\n\
             median:      {} stroops\n\
             spike_count: {}\n\
             p25:         {} stroops\n\
             p75:         {} stroops\n\
             p95:         {} stroops\n\
             p99:         {} stroops",
            self.count,
            self.min,
            self.max,
            self.mean,
            self.median,
            self.spike_count,
            self.p25,
            self.p75,
            self.p95,
            self.p99,
        )
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"count":{},"min":{},"max":{},"mean":{:.2},"median":{},"spike_count":{},"p25":{},"p75":{},"p95":{},"p99":{}}}"#,
            self.count,
            self.min,
            self.max,
            self.mean,
            self.median,
            self.spike_count,
            self.p25,
            self.p75,
            self.p95,
            self.p99,
        )
    }
}

// ---------------------------------------------------------------------------
// Clap CLI definition
// ---------------------------------------------------------------------------

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
    /// Print devkit version and build info
    ///
    /// Use `--json` for machine-readable output.
    Version {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Check if current version is newer than the given version
        #[arg(long, value_name = "VERSION")]
        check: Option<String>,
    },
    /// Show active devkit configuration (env vars and defaults)
    Config {
        /// Path to a configuration file
        #[arg(long, value_name = "FILE")]
        config_file: Option<std::path::PathBuf>,
        /// Set a key=value configuration override (repeatable)
        #[arg(long, value_name = "KEY=VALUE")]
        set: Vec<String>,
    },
    /// Generate shell completion scripts
    ///
    /// Example: `devkit completions --shell zsh >> ~/.zshrc`
    Completions {
        /// Shell to generate completions for (bash, zsh, fish)
        #[arg(long, value_name = "SHELL", default_value = "bash")]
        shell: String,
    },
    /// Validate fee data from a file or piped stdin
    ///
    /// Example: `cat fees.csv | devkit validate --format csv`
    Validate {
        /// Path to the fee data file (omit to read from stdin)
        #[arg(long, value_name = "FILE")]
        file: Option<std::path::PathBuf>,
        /// Input format: csv or json
        #[arg(long, default_value = "csv", value_name = "FORMAT")]
        format: String,
        /// Output format for the report: text or json
        #[arg(long, default_value = "text", value_name = "FORMAT")]
        output_format: String,
    },
    /// Inspect fee data and print a statistical summary
    ///
    /// Example: `cat fees.csv | devkit inspect --format csv`
    Inspect {
        /// Path to the fee data file (omit to read from stdin)
        #[arg(long, value_name = "FILE")]
        file: Option<std::path::PathBuf>,
        /// Input format: csv or json
        #[arg(long, default_value = "csv", value_name = "FORMAT")]
        format: String,
        /// Output format: text or json
        #[arg(long, default_value = "text", value_name = "FORMAT")]
        output_format: String,
    },
}
